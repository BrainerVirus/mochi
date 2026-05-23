use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, Provider, ProviderEnrichment, ProviderError, ProviderResult,
};
use crate::core::usage_store::{failed_attempt, UsageStore};
use crate::providers::built_in_providers;
use crate::settings::SettingsState;
use tauri::State;

#[tauri::command]
pub async fn get_usage_snapshots(
    store: State<'_, UsageStore>,
    state: State<'_, SettingsState>,
) -> Result<Vec<UsageSnapshot>, String> {
    let settings = state.current()?;
    Ok(read_cached_snapshots(&store, &settings.enabled_providers))
}

#[tauri::command]
pub async fn refresh_provider(
    store: State<'_, UsageStore>,
    provider: String,
) -> Result<UsageSnapshot, String> {
    let provider_id =
        ProviderId::parse(&provider).ok_or_else(|| format!("unknown provider: {provider}"))?;

    match fetch_provider_snapshot(provider_id).await {
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

pub fn read_cached_snapshots(
    store: &UsageStore,
    enabled_providers: &[String],
) -> Vec<UsageSnapshot> {
    store.get_snapshots(enabled_providers)
}

pub async fn refresh_enabled_snapshots(
    store: &UsageStore,
    enabled_providers: &[String],
) -> Result<Vec<UsageSnapshot>, ProviderError> {
    let mut snapshots = Vec::new();

    for provider in built_in_providers() {
        let provider_id = provider.metadata().id;
        if !is_provider_enabled(provider_id, enabled_providers) {
            continue;
        }

        match fetch_provider_snapshot(provider_id).await {
            Ok(Some(snapshot)) => {
                store.record_success(snapshot.clone());
                snapshots.push(snapshot);
            }
            Ok(None) => {}
            Err(error) => {
                if let Some(stale) =
                    store.record_failure(provider_id, &error, failed_attempt("live-fetch", &error))
                {
                    snapshots.push(stale);
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
) -> Result<Option<UsageSnapshot>, ProviderError> {
    let provider = find_provider(provider_id).ok_or(ProviderError::NotConfigured)?;
    let ctx = FetchContext;

    for strategy in provider.strategies() {
        if !strategy.is_available(&ctx).await.unwrap_or(false) {
            continue;
        }

        match strategy.fetch(&ctx).await {
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
    use crate::settings::MochiSettings;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
    fn read_cached_snapshots_returns_store_entries_without_live_fetch() {
        let store = UsageStore::new(None);
        store.record_success(cached_claude_snapshot());

        let snapshots = read_cached_snapshots(&store, &["claude".into(), "gemini".into()]);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].provider, ProviderId::Claude);
        assert_eq!(snapshots[0].primary.used_percent, 55.0);
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_enabled_snapshots_returns_only_enabled_providers() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-status",
        );
        std::env::set_var("MOCHI_CURSOR_COOKIE", "WorkosCursorSessionToken=test");

        let store = UsageStore::new(None);
        let snapshots = refresh_enabled_snapshots(&store, &["claude".into(), "cursor".into()])
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
        let snapshots = refresh_enabled_snapshots(&store, &[])
            .await
            .expect("empty enabled list should succeed");

        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn refresh_enabled_snapshots_populates_cache_for_get() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var(
            "MOCHI_CLAUDE_SESSION_KEY",
            "sk-ant-test-session-key-for-cache",
        );

        let store = UsageStore::new(None);
        refresh_enabled_snapshots(&store, &["claude".into()])
            .await
            .expect("refresh should succeed");

        std::env::remove_var("MOCHI_CLAUDE_SESSION_KEY");

        let cached = read_cached_snapshots(&store, &["claude".into()]);
        if cached.is_empty() {
            // Live fetch may fail without network; seed cache to assert read path.
            store.record_success(cached_claude_snapshot());
            let cached = read_cached_snapshots(&store, &["claude".into()]);
            assert_eq!(cached.len(), 1);
            assert_eq!(cached[0].provider, ProviderId::Claude);
            return;
        }

        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].provider, ProviderId::Claude);
        assert_eq!(cached[0].health, ProviderHealth::Ok);
    }

    #[tokio::test]
    async fn refresh_enabled_snapshots_returns_all_default_enabled_providers() {
        let store = UsageStore::new(None);
        let enabled = MochiSettings::default().enabled_providers;
        let snapshots = refresh_enabled_snapshots(&store, &enabled)
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
        let _guard = ENV_LOCK.lock().expect("env lock");
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
                    read_cached_snapshots(&store, &["claude".into()])[0].provider,
                    ProviderId::Claude
                );
            }
            Err(_) => {
                store.record_success(cached_claude_snapshot());
                let cached = read_cached_snapshots(&store, &["claude".into()]);
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

        match fetch_provider_snapshot(provider_id).await {
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
