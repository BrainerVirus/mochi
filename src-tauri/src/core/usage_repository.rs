use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};

use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot};
use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};

pub type UsageRepositoryResult<T> = Result<T, String>;

pub trait UsageRepository: Send + Sync {
    fn latest_for_enabled(
        &self,
        enabled_providers: &[String],
    ) -> UsageRepositoryResult<Vec<ProviderUsageState>>;
    fn put_latest(&self, state: &ProviderUsageState) -> UsageRepositoryResult<()>;
    fn append_success_history(&self, snapshot: &UsageSnapshot) -> UsageRepositoryResult<()>;
    fn delete_provider(&self, provider: ProviderId) -> UsageRepositoryResult<()>;
    fn prune_history_before(&self, cutoff_rfc3339: &str) -> UsageRepositoryResult<()>;
    fn initial_provider_detection_completed(&self) -> UsageRepositoryResult<bool>;
    fn set_initial_provider_detection_completed(
        &self,
        completed: bool,
    ) -> UsageRepositoryResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteUsageRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteUsageRepository {
    pub fn open(path: &Path) -> UsageRepositoryResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        Self::from_connection(connection)
    }

    pub fn from_connection(connection: Connection) -> UsageRepositoryResult<Self> {
        connection
            .execute_batch(
                "
                PRAGMA journal_mode = WAL;
                CREATE TABLE IF NOT EXISTS usage_latest (
                    provider TEXT PRIMARY KEY NOT NULL,
                    state_kind TEXT NOT NULL,
                    snapshot_json TEXT,
                    health TEXT NOT NULL,
                    message TEXT,
                    updated_at TEXT NOT NULL,
                    last_success_at TEXT,
                    last_error TEXT,
                    last_fetch_attempt_json TEXT
                );
                CREATE TABLE IF NOT EXISTS usage_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    provider TEXT NOT NULL,
                    snapshot_json TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_usage_history_provider_updated
                    ON usage_history(provider, updated_at);
                CREATE TABLE IF NOT EXISTS app_state (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );
                ",
            )
            .map_err(|error| error.to_string())?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    #[cfg(test)]
    pub fn connection(&self) -> MutexGuard<'_, Connection> {
        self.connection.lock().expect("sqlite lock")
    }

    fn lock(&self) -> UsageRepositoryResult<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|error| error.to_string())
    }
}

impl UsageRepository for SqliteUsageRepository {
    fn latest_for_enabled(
        &self,
        enabled_providers: &[String],
    ) -> UsageRepositoryResult<Vec<ProviderUsageState>> {
        let connection = self.lock()?;
        let mut states = Vec::new();

        for provider in enabled_providers {
            let Some(state) = connection
                .query_row(
                    "
                    SELECT provider, state_kind, snapshot_json, health, message, updated_at
                    FROM usage_latest
                    WHERE provider = ?1
                    ",
                    params![provider],
                    row_to_usage_state,
                )
                .optional()
                .map_err(|error| error.to_string())?
            else {
                continue;
            };

            states.push(state);
        }

        Ok(states)
    }

    fn put_latest(&self, state: &ProviderUsageState) -> UsageRepositoryResult<()> {
        let connection = self.lock()?;
        let snapshot_json = state
            .snapshot
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;
        let last_success_at = state
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.updated_at.as_str());
        let last_error = state
            .message
            .as_deref()
            .filter(|_| state.health == ProviderHealth::Error);
        let last_fetch_attempt_json = state
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.last_fetch_attempt.as_ref())
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| error.to_string())?;

        connection
            .execute(
                "
                INSERT INTO usage_latest (
                    provider,
                    state_kind,
                    snapshot_json,
                    health,
                    message,
                    updated_at,
                    last_success_at,
                    last_error,
                    last_fetch_attempt_json
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(provider) DO UPDATE SET
                    state_kind = excluded.state_kind,
                    snapshot_json = excluded.snapshot_json,
                    health = excluded.health,
                    message = excluded.message,
                    updated_at = excluded.updated_at,
                    last_success_at = excluded.last_success_at,
                    last_error = excluded.last_error,
                    last_fetch_attempt_json = excluded.last_fetch_attempt_json
                ",
                params![
                    state.provider.as_str(),
                    state_kind_as_str(state.kind),
                    snapshot_json,
                    provider_health_as_str(state.health),
                    state.message,
                    state.updated_at,
                    last_success_at,
                    last_error,
                    last_fetch_attempt_json,
                ],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn append_success_history(&self, snapshot: &UsageSnapshot) -> UsageRepositoryResult<()> {
        let connection = self.lock()?;
        let snapshot_json = serde_json::to_string(snapshot).map_err(|error| error.to_string())?;
        connection
            .execute(
                "
                INSERT INTO usage_history (provider, snapshot_json, updated_at)
                VALUES (?1, ?2, ?3)
                ",
                params![
                    snapshot.provider.as_str(),
                    snapshot_json,
                    snapshot.updated_at
                ],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn delete_provider(&self, provider: ProviderId) -> UsageRepositoryResult<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "DELETE FROM usage_latest WHERE provider = ?1",
                params![provider.as_str()],
            )
            .map_err(|error| error.to_string())?;
        connection
            .execute(
                "DELETE FROM usage_history WHERE provider = ?1",
                params![provider.as_str()],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn prune_history_before(&self, cutoff_rfc3339: &str) -> UsageRepositoryResult<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "DELETE FROM usage_history WHERE updated_at < ?1",
                params![cutoff_rfc3339],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn initial_provider_detection_completed(&self) -> UsageRepositoryResult<bool> {
        let connection = self.lock()?;
        let value = connection
            .query_row(
                "SELECT value FROM app_state WHERE key = ?1",
                params!["initial_provider_detection_completed"],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;

        Ok(value.as_deref() == Some("true"))
    }

    fn set_initial_provider_detection_completed(
        &self,
        completed: bool,
    ) -> UsageRepositoryResult<()> {
        let connection = self.lock()?;
        connection
            .execute(
                "
                INSERT INTO app_state (key, value)
                VALUES (?1, ?2)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value
                ",
                params![
                    "initial_provider_detection_completed",
                    if completed { "true" } else { "false" }
                ],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }
}

fn row_to_usage_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderUsageState> {
    let provider: String = row.get(0)?;
    let state_kind: String = row.get(1)?;
    let snapshot_json: Option<String> = row.get(2)?;
    let health: String = row.get(3)?;
    let message: Option<String> = row.get(4)?;
    let updated_at: String = row.get(5)?;

    let provider = ProviderId::parse(&provider).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown provider id {provider}").into(),
        )
    })?;
    let kind = parse_state_kind(&state_kind).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, error.into())
    })?;
    let snapshot = snapshot_json
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, error.into())
        })?;
    let health = parse_provider_health(&health).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, error.into())
    })?;

    Ok(ProviderUsageState {
        provider,
        kind,
        snapshot,
        health,
        message,
        updated_at,
    })
}

fn state_kind_as_str(kind: ProviderUsageStateKind) -> &'static str {
    match kind {
        ProviderUsageStateKind::Fresh => "fresh",
        ProviderUsageStateKind::Fetching => "fetching",
        ProviderUsageStateKind::StaleError => "stale_error",
        ProviderUsageStateKind::MissingCredentials => "missing_credentials",
        ProviderUsageStateKind::CredentialsNeedRefresh => "credentials_need_refresh",
        ProviderUsageStateKind::Error => "error",
    }
}

fn parse_state_kind(value: &str) -> Result<ProviderUsageStateKind, String> {
    match value {
        "fresh" => Ok(ProviderUsageStateKind::Fresh),
        "fetching" => Ok(ProviderUsageStateKind::Fetching),
        "stale_error" => Ok(ProviderUsageStateKind::StaleError),
        "missing_credentials" => Ok(ProviderUsageStateKind::MissingCredentials),
        "credentials_need_refresh" => Ok(ProviderUsageStateKind::CredentialsNeedRefresh),
        "error" => Ok(ProviderUsageStateKind::Error),
        _ => Err(format!("unknown usage state kind {value}")),
    }
}

fn provider_health_as_str(health: ProviderHealth) -> &'static str {
    match health {
        ProviderHealth::Ok => "ok",
        ProviderHealth::Stale => "stale",
        ProviderHealth::Error => "error",
    }
}

fn parse_provider_health(value: &str) -> Result<ProviderHealth, String> {
    match value {
        "ok" => Ok(ProviderHealth::Ok),
        "stale" => Ok(ProviderHealth::Stale),
        "error" => Ok(ProviderHealth::Error),
        _ => Err(format!("unknown provider health {value}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};
    use rusqlite::Connection;

    fn snapshot(provider: ProviderId, updated_at: &str) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", 31.0, None),
            None,
            updated_at,
            "test",
        )
    }

    fn repo() -> SqliteUsageRepository {
        let connection = Connection::open_in_memory().expect("open sqlite");
        SqliteUsageRepository::from_connection(connection).expect("migrate")
    }

    #[test]
    fn stores_latest_and_successful_history() {
        let repo = repo();
        let state = ProviderUsageState::fresh(snapshot(ProviderId::Claude, "2026-06-04T12:00:00Z"));

        repo.put_latest(&state).expect("put latest");
        repo.append_success_history(state.snapshot.as_ref().expect("snapshot"))
            .expect("append history");

        let latest = repo.latest_for_enabled(&["claude".into()]).expect("latest");
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].provider, ProviderId::Claude);
        assert_eq!(latest[0].kind, ProviderUsageStateKind::Fresh);

        let history_count: i64 = repo
            .connection()
            .query_row("SELECT COUNT(*) FROM usage_history", [], |row| row.get(0))
            .expect("history count");
        assert_eq!(history_count, 1);
    }

    #[test]
    fn non_success_state_does_not_enter_history() {
        let repo = repo();
        let state = ProviderUsageState::missing_credentials(
            ProviderId::Cursor,
            "2026-06-04T12:00:00Z".into(),
        );

        repo.put_latest(&state).expect("put latest");

        let history_count: i64 = repo
            .connection()
            .query_row("SELECT COUNT(*) FROM usage_history", [], |row| row.get(0))
            .expect("history count");
        assert_eq!(history_count, 0);
    }

    #[test]
    fn delete_provider_removes_latest_and_history() {
        let repo = repo();
        let state = ProviderUsageState::fresh(snapshot(ProviderId::Claude, "2026-06-04T12:00:00Z"));
        repo.put_latest(&state).expect("put latest");
        repo.append_success_history(state.snapshot.as_ref().expect("snapshot"))
            .expect("history");

        repo.delete_provider(ProviderId::Claude).expect("delete");

        assert!(repo
            .latest_for_enabled(&["claude".into()])
            .expect("latest")
            .is_empty());
        let history_count: i64 = repo
            .connection()
            .query_row("SELECT COUNT(*) FROM usage_history", [], |row| row.get(0))
            .expect("history count");
        assert_eq!(history_count, 0);
    }

    #[test]
    fn app_state_flag_round_trips() {
        let repo = repo();
        assert!(!repo.initial_provider_detection_completed().expect("read"));

        repo.set_initial_provider_detection_completed(true)
            .expect("write");

        assert!(repo.initial_provider_detection_completed().expect("read"));
    }
}
