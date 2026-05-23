use std::sync::Arc;

use async_trait::async_trait;

use super::client::{ClaudeWebClient, HttpClaudeWebClient};
use super::credentials::resolve_session_key;
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::providers::claude::oauth::current_timestamp;
use crate::providers::claude::usage_parse::snapshot_from_usage_response;

pub struct WebStrategy {
    client: Arc<dyn ClaudeWebClient>,
}

impl WebStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpClaudeWebClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn ClaudeWebClient>) -> Self {
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
        "claude-web"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_session_key(ctx.config(ProviderId::Claude))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let session_key = resolve_session_key(ctx.config(ProviderId::Claude))?
            .ok_or(ProviderError::NotConfigured)?;
        let usage = self.client.fetch_usage(&session_key).await?;
        snapshot_from_usage_response(&usage, &current_timestamp(), "claude-web")
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::ClaudeWebClient;
    use super::*;
    use crate::providers::claude::usage_parse::ClaudeUsageResponse;

    struct MockClaudeWebClient {
        usage: ProviderResult<ClaudeUsageResponse>,
    }

    #[async_trait]
    impl ClaudeWebClient for MockClaudeWebClient {
        async fn fetch_usage(&self, _session_key: &str) -> ProviderResult<ClaudeUsageResponse> {
            match &self.usage {
                Ok(response) => Ok(response.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }
    }

    fn fixture_usage() -> ClaudeUsageResponse {
        serde_json::from_str(include_str!("../fixtures/web_usage.json")).expect("usage")
    }

    #[tokio::test]
    async fn web_strategy_returns_snapshot_with_session_key() {
        let strategy = WebStrategy::with_client(Arc::new(MockClaudeWebClient {
            usage: Ok(fixture_usage()),
        }));

        let mut ctx = FetchContext::empty();
        ctx.provider_configs.insert(
            "claude".into(),
            crate::settings::ProviderConfig {
                manual_cookie: Some("sessionKey=sk-ant-test".into()),
                ..Default::default()
            },
        );

        let snapshot = strategy.fetch(&ctx).await.expect("web fetch");

        assert_eq!(snapshot.source, "claude-web");
        assert_eq!(snapshot.primary.used_percent, 9.0);
    }

    #[tokio::test]
    async fn web_strategy_unavailable_without_session_key() {
        struct UnconfiguredClient;

        #[async_trait]
        impl ClaudeWebClient for UnconfiguredClient {
            async fn fetch_usage(&self, _session_key: &str) -> ProviderResult<ClaudeUsageResponse> {
                Err(ProviderError::NotConfigured)
            }
        }

        let strategy = WebStrategy::with_client(Arc::new(UnconfiguredClient));
        let available = strategy
            .is_available(&FetchContext::empty())
            .await
            .expect("check");
        assert!(!available);
    }
}
