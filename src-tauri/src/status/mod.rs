use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{FetchContext, Provider, ProviderError};
use crate::providers::built_in_providers;

#[tauri::command]
pub async fn get_usage_snapshots() -> Result<Vec<UsageSnapshot>, String> {
    collect_usage_snapshots()
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

async fn collect_usage_snapshots() -> Result<Vec<UsageSnapshot>, ProviderError> {
    let mut snapshots = Vec::new();

    for provider in built_in_providers() {
        if let Some(snapshot) = fetch_provider_snapshot(provider.metadata().id).await? {
            snapshots.push(snapshot);
        }
    }

    Ok(snapshots)
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

    #[tokio::test]
    async fn collect_usage_snapshots_returns_all_built_in_providers() {
        let snapshots = collect_usage_snapshots()
            .await
            .expect("static providers should fetch");

        assert_eq!(snapshots.len(), 10);
        assert!(snapshots.iter().all(|snapshot| !snapshot.source.is_empty()));
    }

    #[tokio::test]
    async fn refresh_provider_returns_snapshot_for_known_provider() {
        let snapshot = refresh_provider("claude".into())
            .await
            .expect("claude should refresh");

        assert_eq!(snapshot.provider, ProviderId::Claude);
    }

    #[tokio::test]
    async fn refresh_provider_rejects_unknown_provider() {
        let error = refresh_provider("not-a-provider".into())
            .await
            .expect_err("unknown provider should fail");

        assert!(error.contains("unknown provider"));
    }
}
