use crate::core::models::UsageSnapshot;
use crate::core::provider::{FetchContext, ProviderError};
use crate::providers::built_in_providers;

#[tauri::command]
pub async fn get_usage_snapshots() -> Result<Vec<UsageSnapshot>, String> {
    collect_usage_snapshots()
        .await
        .map_err(|error| error.to_string())
}

async fn collect_usage_snapshots() -> Result<Vec<UsageSnapshot>, ProviderError> {
    let ctx = FetchContext;
    let mut snapshots = Vec::new();

    for provider in built_in_providers() {
        let mut fetched = None;

        for strategy in provider.strategies() {
            if !strategy.is_available(&ctx).await.unwrap_or(false) {
                continue;
            }

            match strategy.fetch(&ctx).await {
                Ok(snapshot) => {
                    fetched = Some(snapshot);
                    break;
                }
                Err(error) if strategy.should_fallback(&error) => continue,
                Err(error) => return Err(error),
            }
        }

        if let Some(snapshot) = fetched {
            snapshots.push(snapshot);
        }
    }

    Ok(snapshots)
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
}
