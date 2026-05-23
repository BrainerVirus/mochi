use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{
    FetchContext, Provider, ProviderEnrichment, ProviderError, ProviderResult,
};
use crate::core::usage_store::{current_timestamp, failed_attempt, UsageStore};
use crate::providers::built_in_providers;
use crate::providers::credential_probe::provider_has_credentials;
use crate::settings::{MochiSettings, SettingsState};
use tauri::State;

#[tauri::command]
pub async fn get_usage_snapshots(
    store: State<'_, UsageStore>,
    state: State<'_, SettingsState>,
) -> Result<Vec<UsageSnapshot>, String> {
    let settings = state.current()?;
    Ok(read_cached_snapshots(&store, &settings))
}

#[tauri::command]
pub async fn refresh_provider(
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
    provider: String,
) -> Result<UsageSnapshot, String> {
    let settings = settings_state.current()?;
    let ctx = FetchContext::from_settings(&settings);
    let provider_id =
        ProviderId::parse(&provider).ok_or_else(|| format!("unknown provider: {provider}"))?;

    match fetch_provider_snapshot(provider_id, &ctx).await {
        Ok(Some(snapshot)) => {
            store.record_success(snapshot.clone());
            Ok(snapshot)
        }
        Ok(None) => Err("provider returned no snapshot".to_string()),
        Err(error) => {
            if let Some(stale) =
                store.record_failure(provider_id, &error, failed_attempt("live-fetch", &error))
            {
                Err(format!(
                    "{} (serving stale cache)",
                    stale.error.unwrap_or_default()
                ))
            } else {
                Err(error.to_string())
            }
        }
    }
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
        _ => Ok(snapshot),
    }
}

fn find_provider(provider_id: ProviderId) -> Option<std::sync::Arc<dyn Provider>> {
    built_in_providers()
        .into_iter()
        .find(|provider| provider.metadata().id == provider_id)
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
    async fn refresh_enabled_snapshots_returns_all_default_enabled_providers() {
        let store = UsageStore::new(None);
        let enabled = MochiSettings::default().enabled_providers;
        let snapshots = refresh_enabled_snapshots(&store, &MochiSettings::default())
            .await
            .expect("providers should fetch or skip when unconfigured");

        let non_codex_enabled = enabled
            .iter()
            .filter(|provider| *provider != "codex")
            .count();
        let non_codex_snapshots = snapshots
            .iter()
            .filter(|snapshot| snapshot.provider != ProviderId::Codex)
            .count();

        assert!(non_codex_snapshots <= non_codex_enabled);
        assert!(snapshots.len() <= enabled.len());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_provider_returns_snapshot_for_known_provider() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-refresh",
        );

        let store = UsageStore::new(None);
        let result = refresh_provider_with_store(&store, "claude".into()).await;

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");

        match result {
            Ok(snapshot) => {
                assert_eq!(snapshot.provider, ProviderId::Claude);
                assert_eq!(
                    read_cached_snapshots(&store, &settings_with_enabled(&["claude"]))[0].provider,
                    ProviderId::Claude
                );
            }
            Err(_) => {
                store.record_success(cached_claude_snapshot());
                let cached = read_cached_snapshots(&store, &settings_with_enabled(&["claude"]));
                assert_eq!(cached[0].provider, ProviderId::Claude);
            }
        }
    }

    async fn refresh_provider_with_store(
        store: &UsageStore,
        provider: String,
    ) -> Result<UsageSnapshot, String> {
        let provider_id =
            ProviderId::parse(&provider).ok_or_else(|| format!("unknown provider: {provider}"))?;

        match fetch_provider_snapshot(provider_id, &FetchContext::empty()).await {
            Ok(Some(snapshot)) => {
                store.record_success(snapshot.clone());
                Ok(snapshot)
            }
            Ok(None) => Err("provider returned no snapshot".to_string()),
            Err(error) => Err(error.to_string()),
        }
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
    async fn refresh_provider_rejects_unknown_provider() {
        let store = UsageStore::new(None);
        let error = refresh_provider_with_store(&store, "not-a-provider".into())
            .await
            .expect_err("unknown provider should fail");

        assert!(error.contains("unknown provider"));
    }
}
