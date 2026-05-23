use std::sync::Arc;

use async_trait::async_trait;

use super::client::{CopilotUsageClient, HttpCopilotUsageClient};
use super::usage_parse::snapshot_from_usage_response;
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct OAuthInternalStrategy {
    client: Arc<dyn CopilotUsageClient>,
}

impl OAuthInternalStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpCopilotUsageClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CopilotUsageClient>) -> Self {
        Self { client }
    }
}

impl Default for OAuthInternalStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for OAuthInternalStrategy {
    fn id(&self) -> &'static str {
        "copilot-oauth-internal"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        match self.client.resolve_token().await {
            Ok(_) => Ok(true),
            Err(ProviderError::NotConfigured) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let token = self.client.resolve_token().await?;
        let usage = self.client.fetch_usage(&token).await?;
        snapshot_from_usage_response(&usage, &current_timestamp(), "copilot-oauth-internal")
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::CopilotUsageClient;
    use super::super::usage_parse::parse_usage_response;
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;

    struct MockCopilotUsageClient {
        usage: ProviderResult<super::super::usage_parse::CopilotUsageResponse>,
    }

    #[async_trait]
    impl CopilotUsageClient for MockCopilotUsageClient {
        async fn resolve_token(&self) -> ProviderResult<String> {
            Ok("gho_test".into())
        }

        async fn fetch_usage(
            &self,
            _token: &str,
        ) -> ProviderResult<super::super::usage_parse::CopilotUsageResponse> {
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

    fn fixture_usage() -> super::super::usage_parse::CopilotUsageResponse {
        let value: Value =
            serde_json::from_str(include_str!("fixtures/usage_premium_chat.json")).expect("json");
        parse_usage_response(&value).expect("parse")
    }

    #[tokio::test]
    async fn oauth_internal_strategy_returns_snapshot_when_token_valid() {
        let strategy = OAuthInternalStrategy::with_client(Arc::new(MockCopilotUsageClient {
            usage: Ok(fixture_usage()),
        }));

        let snapshot = strategy.fetch(&FetchContext).await.expect("copilot fetch");

        assert_eq!(snapshot.source, "copilot-oauth-internal");
        assert_eq!(snapshot.primary.label, "Premium");
        assert_eq!(snapshot.primary.used_percent, 10.0);
    }

    #[tokio::test]
    async fn oauth_internal_strategy_unavailable_without_token() {
        struct UnconfiguredClient;

        #[async_trait]
        impl CopilotUsageClient for UnconfiguredClient {
            async fn resolve_token(&self) -> ProviderResult<String> {
                Err(ProviderError::NotConfigured)
            }

            async fn fetch_usage(
                &self,
                _token: &str,
            ) -> ProviderResult<super::super::usage_parse::CopilotUsageResponse> {
                Err(ProviderError::NotConfigured)
            }
        }

        let strategy = OAuthInternalStrategy::with_client(Arc::new(UnconfiguredClient));
        let available = strategy.is_available(&FetchContext).await.expect("check");
        assert!(!available);
    }
}
