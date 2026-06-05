use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::core::models::{FetchAttempt, ProviderHealth, ProviderId, UsageSnapshot};
use crate::core::provider::ProviderError;
use crate::core::usage_repository::UsageRepository;
use crate::core::usage_state::ProviderUsageState;

#[derive(Debug, Clone)]
struct CacheEntry {
    snapshot: UsageSnapshot,
}

/// In-memory usage cache with optional repository persistence and last-known-good fallback.
#[derive(Default)]
pub struct UsageStore {
    entries: RwLock<HashMap<ProviderId, CacheEntry>>,
    repository: Option<Arc<dyn UsageRepository>>,
}

impl UsageStore {
    pub fn new(_persistence_path: Option<PathBuf>) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            repository: None,
        }
    }

    pub fn with_repository(repository: Arc<dyn UsageRepository>) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            repository: Some(repository),
        }
    }

    pub fn get_snapshots(&self, enabled_providers: &[String]) -> Vec<UsageSnapshot> {
        let entries = self
            .entries
            .read()
            .unwrap_or_else(|error| error.into_inner());

        enabled_providers
            .iter()
            .filter_map(|id| ProviderId::parse(id))
            .filter_map(|provider_id| {
                entries
                    .get(&provider_id)
                    .map(|entry| entry.snapshot.clone())
            })
            .collect()
    }

    pub fn record_success(&self, snapshot: UsageSnapshot) {
        let provider_id = snapshot.provider;
        let mut entries = self
            .entries
            .write()
            .unwrap_or_else(|error| error.into_inner());
        entries.insert(
            provider_id,
            CacheEntry {
                snapshot: snapshot.clone(),
            },
        );
        drop(entries);

        if let Some(repository) = &self.repository {
            let state = ProviderUsageState::fresh(snapshot.clone());
            let _ = repository.put_latest(&state);
            let _ = repository.append_success_history(&snapshot);
        }
    }

    pub fn record_failure(
        &self,
        provider_id: ProviderId,
        error: &ProviderError,
        attempt: FetchAttempt,
    ) -> Option<UsageSnapshot> {
        let mut entries = self
            .entries
            .write()
            .unwrap_or_else(|error| error.into_inner());

        let message = error.to_string();
        if let Some(entry) = entries.get_mut(&provider_id) {
            let mut snapshot = entry.snapshot.clone();
            snapshot.health = ProviderHealth::Stale;
            snapshot.is_stale = true;
            snapshot.error = Some(message.clone());
            snapshot.last_fetch_attempt = Some(attempt);
            entry.snapshot = snapshot.clone();
            drop(entries);

            if let Some(repository) = &self.repository {
                let _ = repository.put_latest(&ProviderUsageState::stale_error(snapshot.clone(), message));
            }

            return Some(snapshot);
        }

        None
    }

    pub fn record_error(
        &self,
        provider_id: ProviderId,
        message: impl Into<String>,
        attempt: FetchAttempt,
    ) -> UsageSnapshot {
        let snapshot = UsageSnapshot::new(
            provider_id,
            crate::core::models::UsageWindow::new("Session", 0.0, None),
            None,
            current_timestamp(),
            "error",
        )
        .mark_error(message);

        let mut snapshot = snapshot;
        snapshot.last_fetch_attempt = Some(attempt);

        let mut entries = self
            .entries
            .write()
            .unwrap_or_else(|error| error.into_inner());
        entries.insert(
            provider_id,
            CacheEntry {
                snapshot: snapshot.clone(),
            },
        );
        drop(entries);

        if let Some(repository) = &self.repository {
            let _ = repository.put_latest(&ProviderUsageState::error(
                provider_id,
                snapshot.error.clone().unwrap_or_default(),
                snapshot.updated_at.clone(),
            ));
        }

        snapshot
    }

    pub fn delete_provider(&self, provider: ProviderId) -> Result<(), String> {
        let mut entries = self
            .entries
            .write()
            .unwrap_or_else(|error| error.into_inner());
        entries.remove(&provider);
        drop(entries);

        if let Some(repository) = &self.repository {
            repository.delete_provider(provider)?;
        }

        Ok(())
    }

    pub fn load_latest_states(
        &self,
        enabled_providers: &[String],
    ) -> Result<Vec<ProviderUsageState>, String> {
        let Some(repository) = &self.repository else {
            return Ok(Vec::new());
        };
        let states = repository.latest_for_enabled(enabled_providers)?;
        let mut entries = self
            .entries
            .write()
            .unwrap_or_else(|error| error.into_inner());

        for state in &states {
            if let Some(snapshot) = &state.snapshot {
                entries.insert(
                    state.provider,
                    CacheEntry {
                        snapshot: snapshot.clone(),
                    },
                );
            }
        }

        Ok(states)
    }

    pub fn put_state(&self, state: ProviderUsageState) -> Result<(), String> {
        if let Some(snapshot) = &state.snapshot {
            let mut entries = self
                .entries
                .write()
                .unwrap_or_else(|error| error.into_inner());
            entries.insert(
                state.provider,
                CacheEntry {
                    snapshot: snapshot.clone(),
                },
            );
        }

        if let Some(repository) = &self.repository {
            repository.put_latest(&state)?;
        }

        Ok(())
    }
}

pub fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn failed_attempt(strategy_id: impl Into<String>, error: &ProviderError) -> FetchAttempt {
    FetchAttempt {
        strategy_id: strategy_id.into(),
        succeeded: false,
        error: Some(error.to_string()),
        attempted_at: current_timestamp(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::UsageWindow;
    use crate::core::usage_repository::UsageRepository;

    fn sample_snapshot(provider: ProviderId, used_percent: f32) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", used_percent, None),
            None,
            "2026-05-20T12:00:00Z",
            "test",
        )
    }

    fn attempt() -> FetchAttempt {
        FetchAttempt {
            strategy_id: "static-snapshot".to_string(),
            succeeded: false,
            error: Some("network".to_string()),
            attempted_at: "2026-05-20T12:01:00Z".to_string(),
        }
    }

    #[test]
    fn record_failure_returns_stale_last_known_good() {
        let store = UsageStore::new(None);
        let good = sample_snapshot(ProviderId::Claude, 42.0);
        store.record_success(good.clone());

        let stale = store
            .record_failure(
                ProviderId::Claude,
                &ProviderError::Fetch("network".into()),
                attempt(),
            )
            .expect("lkg");

        assert!(stale.is_stale);
        assert_eq!(stale.health, ProviderHealth::Stale);
        assert_eq!(stale.primary.used_percent, good.primary.used_percent);
        assert_eq!(
            stale.error.as_deref(),
            Some("provider fetch failed: network")
        );
    }

    #[test]
    fn record_failure_without_cache_returns_none() {
        let store = UsageStore::new(None);
        assert!(store
            .record_failure(ProviderId::Cursor, &ProviderError::NotConfigured, attempt(),)
            .is_none());
    }

    #[test]
    fn get_snapshots_returns_enabled_cached_providers_only() {
        let store = UsageStore::new(None);
        store.record_success(sample_snapshot(ProviderId::Claude, 10.0));
        store.record_success(sample_snapshot(ProviderId::Cursor, 20.0));

        let snapshots = store.get_snapshots(&["claude".into(), "gemini".into()]);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].provider, ProviderId::Claude);
    }

    #[test]
    fn record_error_stores_error_snapshot() {
        let store = UsageStore::new(None);
        let snapshot = store.record_error(ProviderId::Codex, "auth failed", attempt());

        assert_eq!(snapshot.health, ProviderHealth::Error);
        assert_eq!(snapshot.error.as_deref(), Some("auth failed"));

        let cached = store.get_snapshots(&["codex".into()]);
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].health, ProviderHealth::Error);
    }

    #[test]
    fn record_success_persists_latest_and_history() {
        let repository = crate::core::usage_repository::SqliteUsageRepository::from_connection(
            rusqlite::Connection::open_in_memory().expect("sqlite"),
        )
        .expect("repo");
        let store = UsageStore::with_repository(std::sync::Arc::new(repository.clone()));

        store.record_success(sample_snapshot(ProviderId::Claude, 10.0));

        let latest = repository
            .latest_for_enabled(&["claude".into()])
            .expect("latest");
        assert_eq!(latest.len(), 1);
        assert!(latest[0].snapshot.is_some());

        let history_count: i64 = repository
            .connection()
            .query_row("SELECT COUNT(*) FROM usage_history", [], |row| row.get(0))
            .expect("history count");
        assert_eq!(history_count, 1);
    }

    #[test]
    fn delete_provider_removes_memory_and_repository_state() {
        let repository = crate::core::usage_repository::SqliteUsageRepository::from_connection(
            rusqlite::Connection::open_in_memory().expect("sqlite"),
        )
        .expect("repo");
        let store = UsageStore::with_repository(std::sync::Arc::new(repository.clone()));
        store.record_success(sample_snapshot(ProviderId::Claude, 10.0));

        store
            .delete_provider(ProviderId::Claude)
            .expect("delete provider");

        assert!(store.get_snapshots(&["claude".into()]).is_empty());
        assert!(repository
            .latest_for_enabled(&["claude".into()])
            .expect("latest")
            .is_empty());
    }
}
