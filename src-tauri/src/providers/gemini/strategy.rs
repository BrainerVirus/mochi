use std::sync::Arc;

use async_trait::async_trait;

use super::client::{current_timestamp, GeminiQuotaClient, HttpGeminiQuotaClient};
use super::credentials::{current_auth_type, load_credentials, validate_auth_type};
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};

pub struct OAuthQuotaStrategy {
    client: Arc<dyn GeminiQuotaClient>,
}

impl OAuthQuotaStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpGeminiQuotaClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn GeminiQuotaClient>) -> Self {
        Self { client }
    }
}

impl Default for OAuthQuotaStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for OAuthQuotaStrategy {
    fn id(&self) -> &'static str {
        "gemini-oauth-quota"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        validate_auth_type(current_auth_type())?;
        match load_credentials() {
            Ok(_) => Ok(true),
            Err(ProviderError::NotConfigured) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        validate_auth_type(current_auth_type())?;
        self.client
            .fetch_snapshot(&current_timestamp(), "gemini-oauth-quota")
            .await
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::usage_parse::{parse_quota_response, snapshot_from_quotas};
    use super::*;
    use async_trait::async_trait;

    struct MockGeminiQuotaClient;

    #[async_trait]
    impl GeminiQuotaClient for MockGeminiQuotaClient {
        async fn fetch_snapshot(
            &self,
            updated_at: &str,
            source: &str,
        ) -> ProviderResult<UsageSnapshot> {
            let quotas = parse_quota_response(include_str!("fixtures/quota_response.json"))?;
            snapshot_from_quotas(&quotas, updated_at, source)
        }
    }

    #[tokio::test]
    async fn oauth_quota_strategy_returns_snapshot_from_client() {
        let strategy = OAuthQuotaStrategy::with_client(Arc::new(MockGeminiQuotaClient));
        let snapshot = strategy.fetch(&FetchContext::empty()).await.expect("fetch");

        assert_eq!(snapshot.source, "gemini-oauth-quota");
        assert_eq!(snapshot.primary.label, "Pro");
        assert_eq!(snapshot.primary.used_percent, 40.0);
    }

    #[test]
    fn strategy_id_matches_metadata_registry() {
        let strategy = OAuthQuotaStrategy::new();
        assert_eq!(strategy.id(), "gemini-oauth-quota");
    }
}
