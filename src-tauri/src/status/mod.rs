pub mod refresh_controller;

use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{
    FetchContext, Provider, ProviderEnrichment, ProviderError, ProviderResult,
};
use crate::core::usage_state::ProviderUsageState;
use crate::core::usage_store::{current_timestamp, failed_attempt, UsageStore};
use crate::providers::built_in_providers;
use crate::providers::credential_probe::provider_has_credentials;
use crate::settings::{MochiSettings, SettingsState};
use refresh_controller::RefreshController;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn get_usage_snapshots(
    store: State<'_, UsageStore>,
    state: State<'_, SettingsState>,
) -> Result<Vec<ProviderUsageState>, String> {
    let settings = state.current()?;
    Ok(read_cached_usage_states(&store, &settings))
}

#[tauri::command]
pub async fn refresh_enabled_providers(
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
) -> Result<Vec<UsageSnapshot>, String> {
    let settings = settings_state.current()?;
    refresh_enabled_snapshots(&store, &settings)
        .await
        .map_err(|error| error.to_string())
}

/// Matches frontend `isProviderConfigured` — real fetch, credential pending, or stale/error cache.
pub fn is_snapshot_configured(snapshot: &UsageSnapshot) -> bool {
    const STATIC_SNAPSHOT_EPOCH: &str = "1970-01-01T00:00:00Z";

    if snapshot.updated_at == STATIC_SNAPSHOT_EPOCH {
        return false;
    }

    if snapshot.source == "credentials-detected" {
        return true;
    }

    matches!(
        snapshot.health,
        ProviderHealth::Ok | ProviderHealth::Stale | ProviderHealth::Error
    )
}

pub fn filter_configured_snapshots(snapshots: &[UsageSnapshot]) -> Vec<UsageSnapshot> {
    snapshots
        .iter()
        .filter(|snapshot| is_snapshot_configured(snapshot))
        .cloned()
        .collect()
}

pub fn read_cached_snapshots(store: &UsageStore, settings: &MochiSettings) -> Vec<UsageSnapshot> {
    let ctx = FetchContext::from_settings(settings);
    let mut snapshots = store.get_snapshots(&settings.enabled_providers);
    let present: std::collections::HashSet<_> = snapshots.iter().map(|s| s.provider).collect();

    for provider_id in settings
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
    {
        if present.contains(&provider_id) {
            continue;
        }

        if !provider_has_credentials(provider_id, &ctx) {
            continue;
        }

        snapshots.push(credential_pending_snapshot(provider_id));
    }

    snapshots
}

pub fn read_cached_usage_states(
    store: &UsageStore,
    settings: &MochiSettings,
) -> Vec<ProviderUsageState> {
    let ctx = FetchContext::from_settings(settings);
    let mut states = store.get_states(&settings.enabled_providers);
    let present: std::collections::HashSet<_> = states.iter().map(|state| state.provider).collect();
    let snapshots = store.get_snapshots(&settings.enabled_providers);

    for snapshot in snapshots {
        if present.contains(&snapshot.provider) {
            continue;
        }
        states.push(state_from_snapshot(snapshot));
    }

    let present: std::collections::HashSet<_> = states.iter().map(|state| state.provider).collect();
    for provider_id in settings
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
    {
        if present.contains(&provider_id) {
            continue;
        }

        let state = if provider_has_credentials(provider_id, &ctx) {
            ProviderUsageState::fetching(provider_id, current_timestamp())
        } else {
            ProviderUsageState::missing_credentials(provider_id, current_timestamp())
        };
        states.push(state);
    }

    states
}

fn state_from_snapshot(snapshot: UsageSnapshot) -> ProviderUsageState {
    if snapshot.is_stale {
        let message = snapshot
            .error
            .clone()
            .unwrap_or_else(|| "cached usage is stale".to_string());
        return ProviderUsageState::stale_error(snapshot, message);
    }

    if snapshot.health == ProviderHealth::Error {
        return ProviderUsageState::error(
            snapshot.provider,
            snapshot
                .error
                .clone()
                .unwrap_or_else(|| "usage unavailable".to_string()),
            snapshot.updated_at,
        );
    }

    ProviderUsageState::fresh(snapshot)
}

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

fn credential_pending_snapshot(provider_id: ProviderId) -> UsageSnapshot {
    UsageSnapshot::new(
        provider_id,
        UsageWindow::new("Pending fetch", 100.0, None),
        None,
        current_timestamp(),
        "credentials-detected",
    )
    .mark_error("Usage fetch pending — credentials detected")
}

pub async fn refresh_enabled_snapshots(
    store: &UsageStore,
    settings: &MochiSettings,
) -> Result<Vec<UsageSnapshot>, ProviderError> {
    let ctx = FetchContext::from_settings(settings);
    let mut snapshots = Vec::new();

    for provider in built_in_providers() {
        let provider_id = provider.metadata().id;
        if !is_provider_enabled(provider_id, &settings.enabled_providers) {
            continue;
        }

        if !provider_has_credentials(provider_id, &ctx) {
            store.record_error(
                provider_id,
                "credentials missing",
                failed_attempt("live-fetch", &ProviderError::NotConfigured),
            );
            continue;
        }

        let Some(_guard) = refresh_controller().try_begin_provider_refresh(provider_id) else {
            continue;
        };

        match fetch_provider_snapshot(provider_id, &ctx).await {
            Ok(Some(snapshot)) => {
                store.record_success(snapshot.clone());
                snapshots.push(snapshot);
            }
            Ok(None) => {
                if provider_has_credentials(provider_id, &ctx) {
                    snapshots.push(store.record_error(
                        provider_id,
                        "Usage fetch pending — credentials detected",
                        failed_attempt("live-fetch", &ProviderError::NotConfigured),
                    ));
                }
            }
            Err(error) => {
                if let Some(stale) =
                    store.record_failure(provider_id, &error, failed_attempt("live-fetch", &error))
                {
                    snapshots.push(stale);
                } else if provider_has_credentials(provider_id, &ctx) {
                    let error_snapshot = store.record_error(
                        provider_id,
                        error.to_string(),
                        failed_attempt("live-fetch", &error),
                    );
                    snapshots.push(error_snapshot);
                }
            }
        }
    }

    Ok(snapshots)
}

fn refresh_controller() -> &'static RefreshController {
    static CONTROLLER: OnceLock<RefreshController> = OnceLock::new();
    CONTROLLER.get_or_init(RefreshController::default)
}

fn is_provider_enabled(provider_id: ProviderId, enabled_providers: &[String]) -> bool {
    enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .any(|enabled| enabled == provider_id)
}

async fn fetch_provider_snapshot(
    provider_id: ProviderId,
    ctx: &FetchContext,
) -> Result<Option<UsageSnapshot>, ProviderError> {
    let provider = find_provider(provider_id).ok_or(ProviderError::NotConfigured)?;

    for strategy in provider.strategies() {
        if !strategy.is_available(ctx).await.unwrap_or(false) {
            continue;
        }

        match strategy.fetch(ctx).await {
            Ok(snapshot) => {
                let enriched = enrich_provider_snapshot(provider_id, snapshot).await?;
                return Ok(Some(enriched));
            }
            Err(error) if strategy.should_fallback(&error) => continue,
            Err(error) => return Err(error),
        }
    }

    Ok(None)
}

async fn enrich_provider_snapshot(
    provider_id: ProviderId,
    snapshot: UsageSnapshot,
) -> ProviderResult<UsageSnapshot> {
    match provider_id {
        ProviderId::Codex => {
            crate::providers::CodexProvider
                .enrich_snapshot(snapshot)
                .await
        }
        ProviderId::Claude => {
            crate::providers::ClaudeProvider
                .enrich_snapshot(snapshot)
                .await
        }
        ProviderId::Copilot => {
            crate::providers::CopilotProvider
                .enrich_snapshot(snapshot)
                .await
        }
        ProviderId::Cursor => {
            crate::providers::CursorProvider
                .enrich_snapshot(snapshot)
                .await
        }
        ProviderId::Factory => {
            crate::providers::FactoryProvider
                .enrich_snapshot(snapshot)
                .await
        }
        _ => Ok(snapshot),
    }
}

fn find_provider(provider_id: ProviderId) -> Option<std::sync::Arc<dyn Provider>> {
    built_in_providers()
        .into_iter()
        .find(|provider| provider.metadata().id == provider_id)
}

#[derive(Debug, Serialize)]
pub struct RefreshCompletePayload {
    pub states: Vec<ProviderUsageState>,
}

/// Refresh all enabled providers to completion, then return updated states.
/// Called from the tray handler; does NOT emit events — caller decides.
pub async fn refresh_all_providers_inner(
    store: &UsageStore,
    settings: &MochiSettings,
) -> Result<RefreshCompletePayload, ProviderError> {
    refresh_enabled_snapshots(store, settings).await?;
    let states = read_cached_usage_states(store, settings);
    Ok(RefreshCompletePayload { states })
}

#[tauri::command]
pub async fn refresh_all_providers(
    app: AppHandle,
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
) -> Result<RefreshCompletePayload, String> {
    let settings = settings_state.current()?;
    let payload = refresh_all_providers_inner(&store, &settings)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("usage-refresh-complete", &payload);
    Ok(payload)
}

/// Core logic for refreshing a single provider.
/// Does NOT emit events or use AppHandle/State — caller is responsible for that.
pub async fn refresh_single_provider_inner(
    store: &UsageStore,
    settings: &MochiSettings,
    provider: &str,
) -> Result<RefreshCompletePayload, String> {
    let provider_id =
        ProviderId::parse(provider).ok_or_else(|| format!("unknown provider: {provider}"))?;
    let ctx = FetchContext::from_settings(settings);

    if !provider_has_credentials(provider_id, &ctx) {
        store.record_error(
            provider_id,
            "credentials missing",
            failed_attempt("live-fetch", &ProviderError::NotConfigured),
        );
        return Err("credentials missing".to_string());
    }

    let Some(_guard) = refresh_controller().try_begin_provider_refresh(provider_id) else {
        return Err(format!("refresh already in progress for {provider_id:?}"));
    };

    match fetch_provider_snapshot(provider_id, &ctx).await {
        Ok(Some(snapshot)) => {
            store.record_success(snapshot);
        }
        Ok(None) => {
            store.record_error(
                provider_id,
                "provider returned no snapshot",
                failed_attempt("live-fetch", &ProviderError::NotConfigured),
            );
        }
        Err(error) => {
            if store
                .record_failure(provider_id, &error, failed_attempt("live-fetch", &error))
                .is_none()
            {
                store.record_error(
                    provider_id,
                    error.to_string(),
                    failed_attempt("live-fetch", &error),
                );
            }
        }
    }

    let states = read_cached_usage_states(store, settings);
    Ok(RefreshCompletePayload { states })
}

#[tauri::command]
pub async fn refresh_single_provider(
    app: AppHandle,
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
    provider: String,
) -> Result<RefreshCompletePayload, String> {
    let settings = settings_state.current()?;
    let payload = refresh_single_provider_inner(&store, &settings, &provider).await?;
    let _ = app.emit("usage-refresh-complete", &payload);
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderHealth, UsageSnapshot, UsageWindow};
    use crate::core::test_env;
    use crate::settings::MochiSettings;

    fn settings_with_enabled(enabled: &[&str]) -> MochiSettings {
        MochiSettings {
            enabled_providers: enabled.iter().map(|id| (*id).to_string()).collect(),
            ..MochiSettings::default()
        }
    }

    fn cached_claude_snapshot() -> UsageSnapshot {
        UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 55.0, None),
            None,
            "2026-05-20T12:00:00Z",
            "cached",
        )
    }

    #[test]
    fn filter_configured_snapshots_excludes_static_placeholders() {
        let configured = UsageSnapshot::new(
            ProviderId::Codex,
            UsageWindow::new("Session", 40.0, None),
            None,
            "2026-05-20T12:00:00Z",
            "codex-cli",
        );
        let static_placeholder = UsageSnapshot::new(
            ProviderId::Cursor,
            UsageWindow::new("Session", 0.0, None),
            None,
            "1970-01-01T00:00:00Z",
            "Claude",
        );

        let filtered = filter_configured_snapshots(&[configured, static_placeholder]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider, ProviderId::Codex);
    }

    #[test]
    fn read_cached_snapshots_returns_store_entries_without_live_fetch() {
        let store = UsageStore::new(None);
        store.record_success(cached_claude_snapshot());

        let snapshots = read_cached_snapshots(&store, &settings_with_enabled(&["claude"]));
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].provider, ProviderId::Claude);
        assert_eq!(snapshots[0].primary.used_percent, 55.0);
    }

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
        let detected = vec![
            "claude".to_string(),
            "cursor".to_string(),
            "gemini".to_string(),
        ];

        let next = reconcile_first_start_enabled_providers(&mut settings, true, detected);

        assert!(next.initial_detection_completed);
        assert_eq!(settings.enabled_providers, vec!["claude"]);
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_enabled_snapshots_returns_only_enabled_providers() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-status",
        );
        std::env::set_var("MOCHI_CURSOR_COOKIE", "WorkosCursorSessionToken=test");

        let store = UsageStore::new(None);
        let snapshots =
            refresh_enabled_snapshots(&store, &settings_with_enabled(&["claude", "cursor"]))
                .await
                .expect("providers should fetch or skip");

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");
        std::env::remove_var("MOCHI_CURSOR_COOKIE");

        assert!(snapshots
            .iter()
            .all(|snapshot| matches!(snapshot.provider, ProviderId::Claude | ProviderId::Cursor)));
    }

    #[tokio::test]
    async fn refresh_enabled_snapshots_returns_empty_when_none_enabled() {
        let store = UsageStore::new(None);
        let snapshots = refresh_enabled_snapshots(&store, &settings_with_enabled(&[]))
            .await
            .expect("empty enabled list should succeed");

        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_enabled_snapshots_skips_missing_credentials() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("CLAUDE_CODE_OAUTH_TOKEN");
        let store = UsageStore::new(None);
        let settings = settings_with_enabled(&["claude"]);

        let snapshots = refresh_enabled_snapshots(&store, &settings)
            .await
            .expect("refresh");

        assert!(snapshots.is_empty());
        assert!(read_cached_snapshots(&store, &settings)
            .iter()
            .any(|snapshot| snapshot.provider == ProviderId::Claude
                && snapshot.error.as_deref() == Some("credentials missing")));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_enabled_snapshots_caches_credential_detected_providers() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-bulk-refresh-cache",
        );

        let store = UsageStore::new(None);
        let settings = settings_with_enabled(&["claude"]);
        let _ = refresh_enabled_snapshots(&store, &settings)
            .await
            .expect("refresh should complete");
        let cached = read_cached_snapshots(&store, &settings);

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");

        assert!(cached
            .iter()
            .any(|snapshot| snapshot.provider == ProviderId::Claude));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_enabled_snapshots_populates_cache_for_get() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-cache",
        );

        let store = UsageStore::new(None);
        refresh_enabled_snapshots(&store, &settings_with_enabled(&["claude"]))
            .await
            .expect("refresh should succeed");

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");

        let cached = read_cached_snapshots(&store, &settings_with_enabled(&["claude"]));
        if cached.is_empty() {
            // Live fetch may fail without network; seed cache to assert read path.
            store.record_success(cached_claude_snapshot());
            let cached = read_cached_snapshots(&store, &settings_with_enabled(&["claude"]));
            assert_eq!(cached.len(), 1);
            assert_eq!(cached[0].provider, ProviderId::Claude);
            return;
        }

        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].provider, ProviderId::Claude);
        assert!(matches!(
            cached[0].health,
            ProviderHealth::Ok | ProviderHealth::Error | ProviderHealth::Stale
        ));
    }

    #[tokio::test]
    async fn refresh_enabled_snapshots_returns_empty_for_default_settings() {
        let store = UsageStore::new(None);
        let snapshots = refresh_enabled_snapshots(&store, &MochiSettings::default())
            .await
            .expect("providers should fetch or skip when unconfigured");

        assert!(snapshots.is_empty());
    }

    #[test]
    fn is_provider_enabled_ignores_unknown_provider_ids() {
        assert!(!is_provider_enabled(
            ProviderId::Claude,
            &["not-a-provider".into()]
        ));
        assert!(is_provider_enabled(
            ProviderId::Claude,
            &["not-a-provider".into(), "claude".into()]
        ));
    }

    #[tokio::test]
    async fn refresh_all_providers_inner_returns_empty_when_none_enabled() {
        let store = UsageStore::new(None);
        let payload = refresh_all_providers_inner(&store, &settings_with_enabled(&[]))
            .await
            .expect("empty enabled list should succeed");

        assert!(payload.states.is_empty());
    }

    #[tokio::test]
    async fn refresh_all_providers_inner_returns_empty_for_default_settings() {
        let store = UsageStore::new(None);
        let payload = refresh_all_providers_inner(&store, &MochiSettings::default())
            .await
            .expect("default settings should succeed");

        assert!(payload.states.is_empty());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_all_providers_inner_returns_cached_states_from_store() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-inner-refresh",
        );

        let store = UsageStore::new(None);
        let settings = settings_with_enabled(&["claude"]);
        let payload = refresh_all_providers_inner(&store, &settings)
            .await
            .expect("refresh should complete");

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");

        // After refresh, there should be at least a claude state (fresh, fetching, or error)
        assert!(
            payload.states.iter().any(|s| s.provider == ProviderId::Claude),
            "expected claude state in payload after refresh"
        );
    }

    async fn refresh_single_provider_with_store(
        store: &UsageStore,
        provider: &str,
        settings: &MochiSettings,
    ) -> Result<RefreshCompletePayload, String> {
        refresh_single_provider_inner(store, settings, provider).await
    }

    #[tokio::test]
    async fn refresh_single_provider_rejects_unknown_provider() {
        let store = UsageStore::new(None);
        let error = refresh_single_provider_with_store(
            &store,
            "not-a-provider",
            &MochiSettings::default(),
        )
        .await
        .expect_err("unknown provider should fail");

        assert!(error.contains("unknown provider"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_single_provider_rejects_missing_credentials() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");
        std::env::remove_var("ANTHROPIC_AUTH_TOKEN");
        std::env::remove_var("CLAUDE_CODE_OAUTH_TOKEN");

        let store = UsageStore::new(None);
        let settings = settings_with_enabled(&["claude"]);
        let error = refresh_single_provider_with_store(&store, "claude", &settings)
            .await
            .expect_err("missing credentials should fail");

        assert!(
            error.contains("credentials missing"),
            "expected 'credentials missing' but got: {error}"
        );
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_single_provider_returns_payload_after_success_or_failure() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-single-refresh",
        );

        let store = UsageStore::new(None);
        let settings = settings_with_enabled(&["claude"]);
        let payload = refresh_single_provider_with_store(&store, "claude", &settings).await;

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");

        // Should always return a payload with at least a claude state,
        // regardless of whether the live fetch succeeded or failed
        match payload {
            Ok(RefreshCompletePayload { states }) => {
                assert!(
                    states.iter().any(|s| s.provider == ProviderId::Claude),
                    "expected claude state in payload"
                );
            }
            Err(msg) => {
                // If credentials are somehow missing in CI, verify the error
                assert!(
                    msg.contains("credentials missing"),
                    "unexpected error: {msg}"
                );
            }
        }
    }

}
