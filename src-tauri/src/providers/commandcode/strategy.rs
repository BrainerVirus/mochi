use std::sync::Arc;

use async_trait::async_trait;

use super::client::CommandCodeClient;
use super::credentials::resolve_session_cookie;
use super::usage_parse::{parse_credits, parse_summary, snapshot_from_commandcode};
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct WebStrategy {
    client: Arc<dyn CommandCodeClient>,
}

impl WebStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(super::client::HttpCommandCodeClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CommandCodeClient>) -> Self {
        Self { client }
    }
}

impl Default for WebStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for WebStrategy {
    fn id(&self) -> &'static str {
        "commandcode-web"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_session_cookie(ctx.config(ProviderId::CommandCode))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let cookie = resolve_session_cookie(ctx.config(ProviderId::CommandCode))?
            .ok_or(ProviderError::NotConfigured)?;
        let credits = self.client.fetch_credits(&cookie).await?;
        let summary = self.client.fetch_summary(&cookie).await?;
        let credits = parse_credits(&credits)?;
        let summary = parse_summary(&summary)?;
        snapshot_from_commandcode(&credits, &summary, &current_timestamp(), self.id())
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixtureClient;

    #[async_trait]
    impl CommandCodeClient for FixtureClient {
        async fn fetch_credits(&self, _cookie: &str) -> ProviderResult<serde_json::Value> {
            Ok(serde_json::from_str(include_str!("fixtures/credits.json")).expect("credits"))
        }

        async fn fetch_summary(&self, _cookie: &str) -> ProviderResult<serde_json::Value> {
            Ok(serde_json::from_str(include_str!("fixtures/summary.json")).expect("summary"))
        }
    }

    #[tokio::test]
    async fn fixture_client_maps_commandcode_snapshot() {
        let strategy = WebStrategy::with_client(Arc::new(FixtureClient));
        let snapshot = strategy.fetch(&FetchContext::empty()).await.expect("fetch");

        assert_eq!(snapshot.source, "commandcode-web");
        assert_eq!(snapshot.primary.label, "Monthly");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn unavailable_without_cookie() {
        let _guard = crate::core::test_env::LOCK.lock().expect("env lock");
        std::env::remove_var(super::super::credentials::ENV_COOKIE);
        let home = std::env::temp_dir().join(format!(
            "mochi-commandcode-empty-home-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&home).expect("home dir");
        std::env::set_var("HOME", &home);

        struct UnconfiguredClient;

        #[async_trait]
        impl CommandCodeClient for UnconfiguredClient {
            async fn fetch_credits(&self, _cookie: &str) -> ProviderResult<serde_json::Value> {
                Err(ProviderError::NotConfigured)
            }

            async fn fetch_summary(&self, _cookie: &str) -> ProviderResult<serde_json::Value> {
                Err(ProviderError::NotConfigured)
            }
        }

        let strategy = WebStrategy::with_client(Arc::new(UnconfiguredClient));
        let available = strategy
            .is_available(&FetchContext::empty())
            .await
            .expect("check");
        std::env::remove_var("HOME");
        let _ = std::fs::remove_dir_all(home);

        assert!(!available);
    }
}
