# SQLite Persistence Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist provider usage in SQLite, unify startup/settings/manual/CLI refresh flow, and make cached usage available immediately across GUI, tray, status-bar, and CLI.

**Architecture:** Rust owns SQLite persistence, provider-state reconciliation, and refresh orchestration. React remains a view cache over typed Tauri commands and renders explicit provider display states returned by Rust. The CLI and status-bar read the same cache by default, with live network fetch only behind explicit refresh commands.

**Tech Stack:** Tauri v2, Rust stable, `rusqlite`, Tokio, React 19, TanStack Query 5, Zod 4, Vitest, shell installer scripts.

---

## File Structure

Create these Rust files:

- `src-tauri/src/core/usage_state.rs`: provider display-state enum and helpers for latest-state rows.
- `src-tauri/src/core/usage_repository.rs`: repository trait, in-memory repository, SQLite repository, migrations, history pruning, app-state key/value helpers.
- `src-tauri/src/status/refresh_controller.rs`: provider refresh dedupe and refresh/reconciliation orchestration.
- `src-tauri/src/cli/usage.rs`: CLI usage command formatting, refresh spinner/progress routing, JSON/stdout behavior.

Modify these Rust files:

- `src-tauri/src/core/mod.rs`: export `usage_state` and `usage_repository`.
- `src-tauri/src/core/usage_store.rs`: replace persistence-path stub with repository-backed latest/history writes and state rows.
- `src-tauri/src/status/mod.rs`: route reads and refresh commands through explicit state/repository/controller helpers.
- `src-tauri/src/settings/commands.rs`: reconcile usage state when settings are saved.
- `src-tauri/src/lib.rs`: initialize repository/controller, use app data path, wire commands and CLI cache path.
- `src-tauri/src/cli/mod.rs`: add `--refresh` to `mochi usage`.
- `src-tauri/src/status_bar/mod.rs`: format from cached snapshots instead of static values.
- `src-tauri/src/providers/credential_probe.rs`: expose detected credential details needed by reconciliation.
- `src-tauri/Cargo.toml`: add only test/dev dependencies if a CLI integration test needs them. Do not add a second SQLite stack.

Modify these frontend files:

- `src/lib/schemas/usage.ts`: add provider display-state schema fields or wrapper schema, preserving compatibility with existing `UsageSnapshot`.
- `src/lib/tauri/commands.ts`: parse the new usage-state response shape.
- `src/lib/query/usage-snapshots.ts`: keep cached reads separate from explicit refresh.
- `src/hooks/use-cold-start-provider-refresh.ts`: remove or reduce frontend-owned refresh orchestration once Rust owns startup refresh.
- `src/hooks/use-tray-panel-refresh.ts`: call the new refresh command once instead of fanning out provider refreshes in React.
- `src/hooks/use-tray-events.ts`: settings save and tray refresh should call authoritative Tauri flows.
- `src/hooks/use-tray-panel-state.ts`: consume provider display states and keep tab selection valid after provider deletion.
- `src/components/tray/tray-panel-content.tsx`: render fetching/missing/credentials-needed/error states.
- `src/components/usage/provider-usage-section.tsx`: render state rows without fake meters when no snapshot exists.
- `src/lib/utils/is-provider-configured.ts`: treat explicit state rows as configured/visible when enabled.
- `src/lib/utils/tray-panel-tabs.ts`: build tabs from state rows, not only successful snapshots.

Modify installer files:

- `scripts/install/install-macos.sh`: create/update CLI symlink or print fallback command.
- `scripts/install/install-macos.test.sh`: test symlink target and fallback output.

## Cross-Cutting Rules

- Follow TDD for every task: write failing test, run and verify expected failure, implement, run and verify pass.
- Disabled provider usage data is deleted from latest and history immediately.
- Only successful snapshots enter `usage_history`.
- Reads do not fetch. Only explicit refresh paths fetch.
- Missing credentials are represented locally and skipped by refresh.
- Expired credentials with prior data show stale/error usage; expired credentials without prior data show a compact credentials-refresh state.
- SQLite failures fall back to in-memory mode for the session.

---

### Task 1: Provider Usage State Model

**Files:**

- Create: `src-tauri/src/core/usage_state.rs`
- Modify: `src-tauri/src/core/mod.rs`
- Test: `src-tauri/src/core/usage_state.rs`

- [ ] **Step 1: Write failing Rust tests for provider display states**

Add tests to the new file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot, UsageWindow};

    fn snapshot(provider: ProviderId) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", 42.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        )
    }

    #[test]
    fn fresh_state_wraps_successful_snapshot() {
        let state = ProviderUsageState::fresh(snapshot(ProviderId::Claude));

        assert_eq!(state.provider, ProviderId::Claude);
        assert_eq!(state.kind, ProviderUsageStateKind::Fresh);
        assert!(state.snapshot.is_some());
        assert_eq!(state.health, ProviderHealth::Ok);
        assert_eq!(state.message, None);
    }

    #[test]
    fn credentials_need_refresh_without_snapshot_has_no_meters() {
        let state = ProviderUsageState::credentials_need_refresh(ProviderId::Codex);

        assert_eq!(state.provider, ProviderId::Codex);
        assert_eq!(state.kind, ProviderUsageStateKind::CredentialsNeedRefresh);
        assert!(state.snapshot.is_none());
        assert_eq!(state.health, ProviderHealth::Error);
        assert_eq!(state.message.as_deref(), Some("credentials need refresh"));
    }

    #[test]
    fn stale_error_preserves_last_successful_snapshot() {
        let state = ProviderUsageState::stale_error(
            snapshot(ProviderId::Cursor),
            "provider fetch failed: network",
        );

        assert_eq!(state.kind, ProviderUsageStateKind::StaleError);
        assert_eq!(state.health, ProviderHealth::Stale);
        assert_eq!(state.message.as_deref(), Some("provider fetch failed: network"));
        assert!(state.snapshot.as_ref().is_some_and(|snapshot| snapshot.is_stale));
    }
}
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_state -- --nocapture
```

Expected: FAIL because `usage_state` and `ProviderUsageState` do not exist.

- [ ] **Step 3: Implement the state model**

Create `src-tauri/src/core/usage_state.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderUsageStateKind {
    Fresh,
    Fetching,
    StaleError,
    MissingCredentials,
    CredentialsNeedRefresh,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderUsageState {
    pub provider: ProviderId,
    pub kind: ProviderUsageStateKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<UsageSnapshot>,
    pub health: ProviderHealth,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub updated_at: String,
}

impl ProviderUsageState {
    pub fn fresh(snapshot: UsageSnapshot) -> Self {
        Self {
            provider: snapshot.provider,
            updated_at: snapshot.updated_at.clone(),
            snapshot: Some(snapshot),
            kind: ProviderUsageStateKind::Fresh,
            health: ProviderHealth::Ok,
            message: None,
        }
    }

    pub fn fetching(provider: ProviderId, updated_at: String) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::Fetching,
            snapshot: None,
            health: ProviderHealth::Stale,
            message: Some("fetching usage".into()),
            updated_at,
        }
    }

    pub fn missing_credentials(provider: ProviderId, updated_at: String) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::MissingCredentials,
            snapshot: None,
            health: ProviderHealth::Error,
            message: Some("credentials missing".into()),
            updated_at,
        }
    }

    pub fn credentials_need_refresh(provider: ProviderId) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::CredentialsNeedRefresh,
            snapshot: None,
            health: ProviderHealth::Error,
            message: Some("credentials need refresh".into()),
            updated_at: crate::core::usage_store::current_timestamp(),
        }
    }

    pub fn stale_error(mut snapshot: UsageSnapshot, message: impl Into<String>) -> Self {
        snapshot.health = ProviderHealth::Stale;
        snapshot.is_stale = true;
        let message = message.into();
        snapshot.error = Some(message.clone());

        Self {
            provider: snapshot.provider,
            updated_at: snapshot.updated_at.clone(),
            snapshot: Some(snapshot),
            kind: ProviderUsageStateKind::StaleError,
            health: ProviderHealth::Stale,
            message: Some(message),
        }
    }

    pub fn error(provider: ProviderId, message: impl Into<String>, updated_at: String) -> Self {
        Self {
            provider,
            kind: ProviderUsageStateKind::Error,
            snapshot: None,
            health: ProviderHealth::Error,
            message: Some(message.into()),
            updated_at,
        }
    }
}
```

Modify `src-tauri/src/core/mod.rs`:

```rust
pub mod usage_state;
```

- [ ] **Step 4: Run the focused test and verify it passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_state -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/mod.rs src-tauri/src/core/usage_state.rs
git commit -m "feat(usage): add provider state model"
```

---

### Task 2: SQLite Usage Repository

**Files:**

- Create: `src-tauri/src/core/usage_repository.rs`
- Modify: `src-tauri/src/core/mod.rs`
- Test: `src-tauri/src/core/usage_repository.rs`

- [ ] **Step 1: Write failing repository tests**

Create `src-tauri/src/core/usage_repository.rs` with tests first:

```rust
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

        assert!(repo.latest_for_enabled(&["claude".into()]).expect("latest").is_empty());
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
```

- [ ] **Step 2: Run focused tests and verify expected failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_repository -- --nocapture
```

Expected: FAIL because repository types and methods are not implemented.

- [ ] **Step 3: Implement repository interface, migrations, and SQLite methods**

Implement in `src-tauri/src/core/usage_repository.rs`:

```rust
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};

pub type UsageRepositoryResult<T> = Result<T, String>;

pub trait UsageRepository: Send + Sync {
    fn latest_for_enabled(&self, enabled_providers: &[String]) -> UsageRepositoryResult<Vec<ProviderUsageState>>;
    fn put_latest(&self, state: &ProviderUsageState) -> UsageRepositoryResult<()>;
    fn append_success_history(&self, snapshot: &UsageSnapshot) -> UsageRepositoryResult<()>;
    fn delete_provider(&self, provider: ProviderId) -> UsageRepositoryResult<()>;
    fn prune_history_before(&self, cutoff_rfc3339: &str) -> UsageRepositoryResult<()>;
    fn initial_provider_detection_completed(&self) -> UsageRepositoryResult<bool>;
    fn set_initial_provider_detection_completed(&self, completed: bool) -> UsageRepositoryResult<()>;
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
    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection.lock().expect("sqlite lock")
    }
}
```

Then implement `UsageRepository for SqliteUsageRepository` with:

- `put_latest`: `INSERT ... ON CONFLICT(provider) DO UPDATE`.
- `append_success_history`: insert only when called explicitly with a successful snapshot.
- `latest_for_enabled`: select provider rows matching enabled provider IDs, deserialize JSON into `ProviderUsageState`.
- `delete_provider`: delete from `usage_latest` and `usage_history`.
- `prune_history_before`: `DELETE FROM usage_history WHERE updated_at < ?1`.
- `initial_provider_detection_completed`: read key from `app_state`, default false.
- `set_initial_provider_detection_completed`: upsert `"true"` or `"false"`.

Export from `src-tauri/src/core/mod.rs`:

```rust
pub mod usage_repository;
```

- [ ] **Step 4: Run focused tests and verify pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_repository -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/mod.rs src-tauri/src/core/usage_repository.rs
git commit -m "feat(usage): add sqlite repository"
```

---

### Task 3: Repository-Backed UsageStore

**Files:**

- Modify: `src-tauri/src/core/usage_store.rs`
- Test: `src-tauri/src/core/usage_store.rs`

- [ ] **Step 1: Write failing UsageStore persistence tests**

Add tests:

```rust
#[test]
fn record_success_persists_latest_and_history() {
    let repository = crate::core::usage_repository::SqliteUsageRepository::from_connection(
        rusqlite::Connection::open_in_memory().expect("sqlite"),
    )
    .expect("repo");
    let store = UsageStore::with_repository(std::sync::Arc::new(repository.clone()));

    store.record_success(sample_snapshot(ProviderId::Claude, 10.0));

    let latest = repository.latest_for_enabled(&["claude".into()]).expect("latest");
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

    store.delete_provider(ProviderId::Claude).expect("delete provider");

    assert!(store.get_snapshots(&["claude".into()]).is_empty());
    assert!(repository.latest_for_enabled(&["claude".into()]).expect("latest").is_empty());
}
```

- [ ] **Step 2: Run focused tests and verify expected failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_store -- --nocapture
```

Expected: FAIL because `with_repository`, `delete_provider`, and repository writes do not exist.

- [ ] **Step 3: Implement repository-backed store methods**

Modify `UsageStore`:

```rust
pub struct UsageStore {
    entries: RwLock<HashMap<ProviderId, CacheEntry>>,
    repository: Option<Arc<dyn UsageRepository>>,
}
```

Add constructors:

```rust
pub fn new(_persistence_path: Option<PathBuf>) -> Self {
    Self {
        entries: RwLock::new(HashMap::new()),
        repository: None,
    }
}

pub fn with_repository(repository: Arc<dyn UsageRepository>) -> Self {
    let store = Self {
        entries: RwLock::new(HashMap::new()),
        repository: Some(repository),
    };
    store
}
```

Update methods:

- `record_success`: update memory, write `ProviderUsageState::fresh(snapshot.clone())` to latest, append history.
- `record_failure`: update memory, write `ProviderUsageState::stale_error(...)` to latest when previous snapshot exists.
- `record_error`: write `ProviderUsageState::error(...)` latest but do not append history.
- `delete_provider`: remove memory and repository rows.
- `load_latest_states`: read repository rows and hydrate memory snapshots where `snapshot.is_some()`.
- `put_state`: persist non-snapshot states such as fetching/missing credentials.

- [ ] **Step 4: Run focused tests and verify pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml usage_store -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/usage_store.rs
git commit -m "feat(usage): persist usage store"
```

---

### Task 4: Startup Reconciliation

**Files:**

- Modify: `src-tauri/src/status/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/status/mod.rs`

- [ ] **Step 1: Write failing startup reconciliation tests**

Add tests in `src-tauri/src/status/mod.rs`:

```rust
#[test]
fn first_start_auto_enables_detected_providers_once() {
    let mut settings = MochiSettings::default();
    let detected = vec!["claude".to_string(), "cursor".to_string()];

    let next = reconcile_first_start_enabled_providers(&mut settings, false, detected);

    assert!(next.initial_detection_completed);
    assert_eq!(settings.enabled_providers, vec!["claude", "cursor"]);
}

#[test]
fn later_start_keeps_user_provider_preferences() {
    let mut settings = settings_with_enabled(&["claude"]);
    let detected = vec!["claude".to_string(), "cursor".to_string(), "gemini".to_string()];

    let next = reconcile_first_start_enabled_providers(&mut settings, true, detected);

    assert!(next.initial_detection_completed);
    assert_eq!(settings.enabled_providers, vec!["claude"]);
}
```

- [ ] **Step 2: Run focused tests and verify expected failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml first_start later_start -- --nocapture
```

Expected: FAIL because reconciliation helper does not exist.

- [ ] **Step 3: Implement startup reconciliation helper**

Add in `src-tauri/src/status/mod.rs`:

```rust
pub struct StartupReconciliation {
    pub initial_detection_completed: bool,
}

pub fn reconcile_first_start_enabled_providers(
    settings: &mut MochiSettings,
    initial_detection_completed: bool,
    detected_provider_ids: Vec<String>,
) -> StartupReconciliation {
    if !initial_detection_completed {
        settings.enabled_providers = detected_provider_ids;
        settings.normalize_provider_ids();
    }

    StartupReconciliation {
        initial_detection_completed: true,
    }
}
```

In `src-tauri/src/lib.rs` setup:

- Resolve database path under `app.path().app_data_dir()` or `app.path().app_config_dir()` consistently.
- Open `SqliteUsageRepository::open(&db_path)`.
- If open succeeds, call `store.load_latest_states()`, prune 90-day history, and manage store with repository.
- If open fails, manage in-memory `UsageStore::new(None)` and record a warning state for frontend diagnostics.
- Run first-start reconciliation using `detected_provider_ids(&settings)` and persist settings if changed.
- Set `initial_provider_detection_completed` in `app_state`.

- [ ] **Step 4: Run focused tests and verify pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml first_start later_start -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/status/mod.rs src-tauri/src/lib.rs
git commit -m "feat(usage): reconcile startup providers"
```

---

### Task 5: Refresh Controller And Cache-Only Reads

**Files:**

- Create: `src-tauri/src/status/refresh_controller.rs`
- Modify: `src-tauri/src/status/mod.rs`
- Test: `src-tauri/src/status/refresh_controller.rs`, `src-tauri/src/status/mod.rs`

- [ ] **Step 1: Write failing refresh controller tests**

Create tests for dedupe and missing credentials:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ProviderId;

    #[tokio::test]
    async fn provider_lock_allows_one_active_refresh() {
        let controller = RefreshController::default();

        let first = controller.try_begin_provider_refresh(ProviderId::Claude);
        let second = controller.try_begin_provider_refresh(ProviderId::Claude);

        assert!(first.is_some());
        assert!(second.is_none());
        drop(first);
        assert!(controller.try_begin_provider_refresh(ProviderId::Claude).is_some());
    }
}
```

Add a status test:

```rust
#[tokio::test]
async fn refresh_enabled_snapshots_skips_missing_credentials() {
    let store = UsageStore::new(None);
    let settings = settings_with_enabled(&["claude"]);

    let snapshots = refresh_enabled_snapshots(&store, &settings)
        .await
        .expect("refresh");

    assert!(snapshots.is_empty());
    assert!(read_cached_snapshots(&store, &settings).iter().any(|snapshot| {
        snapshot.provider == ProviderId::Claude && snapshot.error.as_deref() == Some("credentials missing")
    }));
}
```

- [ ] **Step 2: Run focused tests and verify expected failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml refresh_enabled_snapshots_skips_missing_credentials provider_lock -- --nocapture
```

Expected: FAIL because controller and missing-credentials state do not exist.

- [ ] **Step 3: Implement refresh controller**

Create `src-tauri/src/status/refresh_controller.rs`:

```rust
use std::collections::HashSet;
use std::sync::Mutex;

use crate::core::models::ProviderId;

#[derive(Default)]
pub struct RefreshController {
    active: Mutex<HashSet<ProviderId>>,
}

pub struct ProviderRefreshGuard<'a> {
    controller: &'a RefreshController,
    provider: ProviderId,
}

impl RefreshController {
    pub fn try_begin_provider_refresh(
        &self,
        provider: ProviderId,
    ) -> Option<ProviderRefreshGuard<'_>> {
        let mut active = self.active.lock().ok()?;
        if !active.insert(provider) {
            return None;
        }
        Some(ProviderRefreshGuard {
            controller: self,
            provider,
        })
    }
}

impl Drop for ProviderRefreshGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.controller.active.lock() {
            active.remove(&self.provider);
        }
    }
}
```

Modify `status/mod.rs`:

- export `mod refresh_controller;`.
- make refresh paths acquire a provider guard before fetching.
- before fetching, call `provider_has_credentials`; when false, write missing-credentials state and skip network.
- keep `get_usage_snapshots` as cache-only.

- [ ] **Step 4: Run focused tests and verify pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml refresh_enabled_snapshots_skips_missing_credentials provider_lock -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/status/mod.rs src-tauri/src/status/refresh_controller.rs
git commit -m "feat(usage): add refresh controller"
```

---

### Task 6: Settings Save Reconciliation

**Files:**

- Modify: `src-tauri/src/settings/commands.rs`
- Modify: `src-tauri/src/status/mod.rs`
- Modify: `src/hooks/use-tray-events.ts`
- Test: `src-tauri/src/settings/commands.rs`, `src/hooks/use-tray-events.test.ts`

- [ ] **Step 1: Write failing Rust tests for disabled-provider deletion**

Add a pure helper test in `settings/commands.rs`:

```rust
#[test]
fn disabled_providers_are_detected_from_settings_change() {
    let previous = MochiSettings {
        enabled_providers: vec!["claude".into(), "cursor".into()],
        ..MochiSettings::default()
    };
    let next = MochiSettings {
        enabled_providers: vec!["claude".into()],
        ..MochiSettings::default()
    };

    let disabled = disabled_provider_ids(&previous, &next);

    assert_eq!(disabled, vec![crate::core::models::ProviderId::Cursor]);
}
```

- [ ] **Step 2: Run focused test and verify expected failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml disabled_providers_are_detected -- --nocapture
```

Expected: FAIL because `disabled_provider_ids` does not exist.

- [ ] **Step 3: Implement settings reconciliation**

Add helper:

```rust
fn disabled_provider_ids(previous: &MochiSettings, next: &MochiSettings) -> Vec<ProviderId> {
    let next: std::collections::HashSet<_> = next
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .collect();

    previous
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .filter(|provider| !next.contains(provider))
        .collect()
}
```

Change Tauri command signature:

```rust
pub fn save_settings(
    settings: MochiSettings,
    state: State<'_, SettingsState>,
    usage_store: State<'_, UsageStore>,
) -> Result<MochiSettings, String>
```

Implementation:

- read previous settings.
- persist next settings.
- delete disabled providers via `usage_store.delete_provider(provider)`.
- create latest state rows for newly enabled providers:
  - `fetching` if credentials detected.
  - `missing_credentials` if not detected.
- do not start network fetch directly in this command unless the refresh controller command is available as a follow-up async path.

Update `src/hooks/use-tray-events.ts`:

- on settings save success, set settings query data.
- invalidate usage snapshots.
- call `syncTrayUsage`.
- start authoritative refresh command for newly enabled detected providers if Rust exposes it; otherwise call `refreshEnabledProviders` once.

- [ ] **Step 4: Run focused Rust test and existing frontend tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml disabled_providers_are_detected -- --nocapture
pnpm test src/hooks/use-tray-events.test.ts
```

Expected: Rust PASS. If the frontend test file does not exist yet, create it before this step with a focused test for `shouldRunProviderRefreshForTrayEvent` or extracted settings-save helper.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/settings/commands.rs src-tauri/src/status/mod.rs src/hooks/use-tray-events.ts src/hooks/use-tray-events.test.ts
git commit -m "feat(settings): reconcile usage state"
```

---

### Task 7: Frontend Usage State Rendering

**Files:**

- Modify: `src/lib/schemas/usage.ts`
- Modify: `src/lib/tauri/commands.ts`
- Modify: `src/components/tray/tray-panel-content.tsx`
- Modify: `src/components/usage/provider-usage-section.tsx`
- Modify: `src/lib/utils/tray-panel-tabs.ts`
- Test: `src/lib/schemas/usage.test.ts`, `src/lib/utils/tray-panel-tabs.test.ts`, `src/components/usage/provider-usage-section.test.tsx` if component tests already support TSX rendering.

- [ ] **Step 1: Write failing Zod schema tests**

Add to `src/lib/schemas/usage.test.ts`:

```typescript
import { ProviderUsageStateSchema } from "./usage";

test("parses missing credentials provider state without snapshot", () => {
  const state = ProviderUsageStateSchema.parse({
    provider: "claude",
    kind: "missing_credentials",
    snapshot: null,
    health: "error",
    message: "credentials missing",
    updated_at: "2026-06-04T12:00:00Z",
  });

  expect(state.provider).toBe("claude");
  expect(state.kind).toBe("missing_credentials");
  expect(state.snapshot).toBeNull();
});

test("parses fresh provider state with snapshot", () => {
  const state = ProviderUsageStateSchema.parse({
    provider: "cursor",
    kind: "fresh",
    health: "ok",
    message: null,
    updated_at: "2026-06-04T12:00:00Z",
    snapshot: {
      provider: "cursor",
      primary: {
        label: "Session",
        used_percent: 50,
        remaining_percent: 50,
        resets_at: null,
      },
      secondary: null,
      extra_windows: [],
      updated_at: "2026-06-04T12:00:00Z",
      source: "test",
      health: "ok",
      is_stale: false,
    },
  });

  expect(state.snapshot?.provider).toBe("cursor");
});
```

- [ ] **Step 2: Run focused frontend tests and verify expected failure**

Run:

```bash
pnpm test src/lib/schemas/usage.test.ts
```

Expected: FAIL because `ProviderUsageStateSchema` does not exist.

- [ ] **Step 3: Implement frontend schemas and parser**

In `src/lib/schemas/usage.ts`, add:

```typescript
export const ProviderUsageStateKindSchema = z.enum([
  "fresh",
  "fetching",
  "stale_error",
  "missing_credentials",
  "credentials_need_refresh",
  "error",
]);

export type ProviderUsageStateKind = z.infer<typeof ProviderUsageStateKindSchema>;

export const ProviderUsageStateSchema = z.object({
  provider: ProviderIdSchema,
  kind: ProviderUsageStateKindSchema,
  snapshot: UsageSnapshotSchema.nullable().optional(),
  health: ProviderHealthSchema,
  message: z.string().nullable().optional(),
  updated_at: z.string(),
});

export type ProviderUsageState = z.infer<typeof ProviderUsageStateSchema>;

export const ProviderUsageStatesSchema = z.array(ProviderUsageStateSchema);
export type ProviderUsageStates = z.infer<typeof ProviderUsageStatesSchema>;
```

Update `commands.ts` so `getUsageSnapshots` either becomes `getUsageStates` or parses the new state shape. Prefer adding a new command wrapper:

```typescript
export async function getUsageStates(): Promise<ProviderUsageStates> {
  const result = await invoke<unknown>("get_usage_snapshots");
  return ProviderUsageStatesSchema.parse(result);
}
```

Then update query options and consumers to use state arrays.

- [ ] **Step 4: Add rendering tests or utility tests for tabs**

In `src/lib/utils/tray-panel-tabs.test.ts`, add:

```typescript
test("builds tabs for missing credential state rows", () => {
  const tabs = buildTrayPanelTabsFromStates(
    [
      {
        provider: "claude",
        kind: "missing_credentials",
        snapshot: null,
        health: "error",
        message: "credentials missing",
        updated_at: "2026-06-04T12:00:00Z",
      },
    ],
    ["claude"],
  );

  expect(tabs).toEqual([
    { id: "overview", label: "Overview" },
    { id: "claude", label: "Claude" },
  ]);
});
```

Run:

```bash
pnpm test src/lib/schemas/usage.test.ts src/lib/utils/tray-panel-tabs.test.ts
```

Expected before implementation: FAIL because `buildTrayPanelTabsFromStates` does not exist. After implementation: PASS.

- [ ] **Step 5: Implement state rendering**

Implementation requirements:

- Fetching state renders skeleton shaped like provider card plus text `Fetching usage...`.
- Missing credentials renders provider header and `credentials missing`.
- Credentials need refresh renders provider header and `credentials need refresh`.
- Fresh/stale states render `ProviderUsageSection` with the snapshot.
- Provider tabs are built from enabled state rows.
- Existing snapshot rendering remains intact.

- [ ] **Step 6: Run focused frontend tests and verify pass**

Run:

```bash
pnpm test src/lib/schemas/usage.test.ts src/lib/utils/tray-panel-tabs.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/lib/schemas/usage.ts src/lib/schemas/usage.test.ts src/lib/tauri/commands.ts src/lib/query/usage-snapshots.ts src/components/tray/tray-panel-content.tsx src/components/usage/provider-usage-section.tsx src/lib/utils/tray-panel-tabs.ts src/lib/utils/tray-panel-tabs.test.ts
git commit -m "feat(ui): render usage states"
```

---

### Task 8: CLI Usage And Status-Bar Cache Reads

**Files:**

- Modify: `src-tauri/src/cli/mod.rs`
- Create: `src-tauri/src/cli/usage.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/status_bar/mod.rs`
- Test: `src-tauri/src/cli/usage.rs`, `src-tauri/src/status_bar/mod.rs`

- [ ] **Step 1: Write failing CLI unit tests**

Create `src-tauri/src/cli/usage.rs` tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::core::usage_state::ProviderUsageState;

    fn state() -> ProviderUsageState {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 64.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        );
        ProviderUsageState::fresh(snapshot)
    }

    #[test]
    fn formats_cached_usage_without_refresh() {
        let output = format_usage_text(&[state()]);

        assert!(output.contains("Claude"));
        assert!(output.contains("64%"));
    }

    #[test]
    fn formats_json_from_usage_states() {
        let output = format_usage_json(&[state()]).expect("json");

        assert!(output.contains("\"provider\":\"claude\""));
        assert!(output.contains("\"kind\":\"fresh\""));
    }
}
```

Update `cli/mod.rs` test compile expectation by adding:

```rust
Usage {
    provider: Option<String>,
    #[arg(long)]
    refresh: bool,
    #[arg(long)]
    json: bool,
},
```

- [ ] **Step 2: Run focused tests and verify expected failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml cli::usage -- --nocapture
```

Expected: FAIL because `cli::usage` and formatters do not exist.

- [ ] **Step 3: Implement CLI usage formatting and refresh flag**

Create `src-tauri/src/cli/usage.rs`:

```rust
use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};

pub fn format_usage_text(states: &[ProviderUsageState]) -> String {
    if states.is_empty() {
        return "No usage data cached. Enable providers in settings.".to_string();
    }

    states
        .iter()
        .map(|state| match (&state.kind, state.snapshot.as_ref()) {
            (_, Some(snapshot)) => format!(
                "{} {}%{}",
                snapshot.provider.display_name(),
                snapshot.primary.used_percent.round() as u8,
                state
                    .message
                    .as_ref()
                    .map(|message| format!(" ({message})"))
                    .unwrap_or_default()
            ),
            (ProviderUsageStateKind::MissingCredentials, None) => {
                format!("{} credentials missing", state.provider.display_name())
            }
            (ProviderUsageStateKind::CredentialsNeedRefresh, None) => {
                format!("{} credentials need refresh", state.provider.display_name())
            }
            (_, None) => format!(
                "{} {}",
                state.provider.display_name(),
                state.message.as_deref().unwrap_or("no usage data")
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_usage_json(states: &[ProviderUsageState]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(states)
}
```

Add a small helper in this file for labels:

```rust
fn provider_label(provider: ProviderId) -> &'static str
```

Wire `run_cli(Command::Usage { provider, refresh, json })`:

- open settings and SQLite repository using the same app path resolution helper that does not require a Tauri app handle.
- read cached states by default.
- when `refresh` is true, print progress to `stderr`, run controller refresh, then read states.
- when `json` is true, write only final JSON to `stdout`.

- [ ] **Step 4: Update status-bar formatting tests**

Add test:

```rust
#[test]
fn status_bar_formats_from_cached_state() {
    let snapshot = crate::core::models::UsageSnapshot::new(
        crate::core::models::ProviderId::Claude,
        crate::core::models::UsageWindow::new("Session", 72.0, None),
        None,
        "2026-06-04T12:00:00Z",
        "test",
    );
    let output = format_output_from_states(
        "text",
        &[crate::core::usage_state::ProviderUsageState::fresh(snapshot)],
    );

    assert_eq!(output, "Mochi 72% (Claude)");
}
```

- [ ] **Step 5: Run focused tests and verify pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml cli::usage status_bar -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/cli/mod.rs src-tauri/src/cli/usage.rs src-tauri/src/lib.rs src-tauri/src/status_bar/mod.rs
git commit -m "feat(cli): read cached usage"
```

---

### Task 9: macOS CLI Installer Link

**Files:**

- Modify: `scripts/install/install-macos.sh`
- Create: `scripts/install/install-macos.test.sh` or `scripts/install/install-macos.test.mjs`
- Test: installer test file

- [ ] **Step 1: Write failing installer test**

Prefer a shell test if it can avoid privileged writes by testing helper functions with temp directories. Create `scripts/install/install-macos.test.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if ! grep -q "mochi_install_cli_link" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected macOS installer to define mochi_install_cli_link" >&2
  exit 1
fi

if ! grep -q "/usr/local/bin/mochi" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected default CLI link path" >&2
  exit 1
fi

if ! grep -q "Contents/MacOS/mochi" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected app bundle binary target" >&2
  exit 1
fi

if ! grep -q "sudo ln -sf" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected sudo fallback command" >&2
  exit 1
fi
```

- [ ] **Step 2: Run installer test and verify expected failure**

Run:

```bash
bash scripts/install/install-macos.test.sh
```

Expected: FAIL because helper and symlink behavior are not present.

- [ ] **Step 3: Implement installer CLI link helper**

In `scripts/install/install-macos.sh`, after `ditto`:

```bash
mochi_install_cli_link() {
  local app_path="$1"
  local link_path="${MOCHI_CLI_LINK:-/usr/local/bin/mochi}"
  local target="${app_path}/Contents/MacOS/mochi"
  local link_dir
  link_dir="$(dirname "${link_path}")"

  if [[ ! -x "${target}" ]]; then
    echo "Skipping CLI link: ${target} is not executable"
    return 0
  fi

  if mkdir -p "${link_dir}" 2>/dev/null && ln -sf "${target}" "${link_path}" 2>/dev/null; then
    echo "Installed CLI command to ${link_path}"
    return 0
  fi

  echo "Could not write ${link_path}. To enable the CLI, run:"
  echo "sudo ln -sf ${target} ${link_path}"
}

mochi_install_cli_link "${DEST}"
```

- [ ] **Step 4: Run installer test and verify pass**

Run:

```bash
bash scripts/install/install-macos.test.sh
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add scripts/install/install-macos.sh scripts/install/install-macos.test.sh
git commit -m "fix(install): link macos cli"
```

---

### Task 10: End-To-End Verification And Cleanup

**Files:**

- Modify: only the files named by the failing verification output from Tasks 1-9.

- [ ] **Step 1: Run full frontend verification**

Run:

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
```

Expected: all commands exit 0. If a command fails, fix the smallest relevant issue using TDD when behavior changes.

- [ ] **Step 2: Run full Rust verification**

Run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all commands exit 0. If a command fails, fix the smallest relevant issue using TDD when behavior changes.

- [ ] **Step 3: Run installer verification**

Run:

```bash
bash scripts/install/install-macos.test.sh
bash scripts/install/lib/linux-deps.test.sh
```

Expected: both scripts exit 0.

- [ ] **Step 4: Manual smoke checks**

Run:

```bash
pnpm tauri dev
```

Manual checks:

- Launch shows cached provider state immediately when SQLite has rows.
- Newly enabled provider with credentials shows fetching skeleton.
- Newly enabled provider without credentials shows missing-credentials state.
- Disabled provider disappears and does not come back after restart.
- Manual refresh does not fetch missing-credential providers.
- Repeated refresh clicks do not launch overlapping per-provider fetches.
- `cargo run --manifest-path src-tauri/Cargo.toml -- usage` reads cache.
- `cargo run --manifest-path src-tauri/Cargo.toml -- usage --refresh --json` writes progress to `stderr` and final JSON to `stdout`.

- [ ] **Step 5: Final commit if verification fixes were needed**

```bash
git status --short
git add <changed-files>
git commit -m "test(usage): verify persistence flow"
```

Only create this commit if verification changes were required after Task 9.
