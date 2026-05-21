use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{FetchContext, Provider, ProviderError};
use crate::providers::built_in_providers;
use crate::settings::SettingsState;
use tauri::State;

#[tauri::command]
pub async fn get_usage_snapshots(
    state: State<'_, SettingsState>,
) -> Result<Vec<UsageSnapshot>, String> {
    let settings = state.current()?;
    collect_usage_snapshots(&settings.enabled_providers)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn refresh_provider(provider: String) -> Result<UsageSnapshot, String> {
    let provider_id = ProviderId::parse(&provider)
        .ok_or_else(|| format!("unknown provider: {provider}"))?;

    match fetch_provider_snapshot(provider_id).await {
        Ok(Some(snapshot)) => Ok(snapshot),
        Ok(None) => Err("provider returned no snapshot".to_string()),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) async fn collect_usage_snapshots(
    enabled_providers: &[String],
) -> Result<Vec<UsageSnapshot>, ProviderError> {
    let mut snapshots = Vec::new();

    for provider in built_in_providers() {
        let provider_id = provider.metadata().id;
        if !is_provider_enabled(provider_id, enabled_providers) {
            continue;
        }

        if let Some(snapshot) = fetch_provider_snapshot(provider_id).await? {
            snapshots.push(snapshot);
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
            Ok(snapshot) => return Ok(Some(snapshot)),
            Err(error) if strategy.should_fallback(&error) => continue,
            Err(error) => return Err(error),
        }
    }

    Ok(None)
}

fn find_provider(provider_id: ProviderId) -> Option<std::sync::Arc<dyn Provider>> {
    built_in_providers()
        .into_iter()
        .find(|provider| provider.metadata().id == provider_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::MochiSettings;

    #[tokio::test]
    async fn collect_usage_snapshots_returns_only_enabled_providers() {
        let snapshots = collect_usage_snapshots(&["claude".into(), "cursor".into()])
            .await
            .expect("static providers should fetch");

        assert_eq!(snapshots.len(), 2);
        assert!(snapshots
            .iter()
            .all(|snapshot| matches!(snapshot.provider, ProviderId::Claude | ProviderId::Cursor)));
        assert!(snapshots.iter().all(|snapshot| !snapshot.source.is_empty()));
    }

    #[tokio::test]
    async fn collect_usage_snapshots_returns_empty_when_none_enabled() {
        let snapshots = collect_usage_snapshots(&[])
            .await
            .expect("empty enabled list should succeed");

        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    async fn collect_usage_snapshots_returns_all_default_enabled_providers() {
        let enabled = MochiSettings::default().enabled_providers;
        let snapshots = collect_usage_snapshots(&enabled)
            .await
            .expect("static providers should fetch");

        assert_eq!(snapshots.len(), 10);
    }

    #[tokio::test]
    async fn refresh_provider_returns_snapshot_for_known_provider() {
        let snapshot = refresh_provider("claude".into())
            .await
            .expect("claude should refresh");

        assert_eq!(snapshot.provider, ProviderId::Claude);
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
        let error = refresh_provider("not-a-provider".into())
            .await
            .expect_err("unknown provider should fail");

        assert!(error.contains("unknown provider"));
    }
}
