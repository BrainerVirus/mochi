use std::sync::Arc;

use async_trait::async_trait;

use super::client::{HttpZaiQuotaClient, ZaiQuotaClient};
use super::credentials::{quota_url, resolve_api_key, resolve_region};
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct ApiQuotaStrategy {
    client: Arc<dyn ZaiQuotaClient>,
}

impl ApiQuotaStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpZaiQuotaClient::new()),
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn with_client(client: Arc<dyn ZaiQuotaClient>) -> Self {
        Self { client }
    }
}

impl Default for ApiQuotaStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for ApiQuotaStrategy {
    fn id(&self) -> &'static str {
        "zai-api-quota"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::ApiKey
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_api_key(ctx.config(ProviderId::Zai))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let api_key =
            resolve_api_key(ctx.config(ProviderId::Zai))?.ok_or(ProviderError::NotConfigured)?;
        let region = resolve_region(ctx.config(ProviderId::Zai));
        let url = quota_url(region);
        self.client
            .fetch_usage(&api_key, &url, &current_timestamp(), self.id())
            .await
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::usage_parse::{parse_quota_response, snapshot_from_limits};
    use super::*;
    use async_trait::async_trait;

    struct FixtureClient;

    #[async_trait]
    impl ZaiQuotaClient for FixtureClient {
        async fn fetch_usage(
            &self,
            _api_key: &str,
            _quota_url: &str,
            updated_at: &str,
            source: &str,
        ) -> ProviderResult<UsageSnapshot> {
            let (limits, _) =
                parse_quota_response(include_str!("fixtures/quota_response.json")).expect("parse");
            snapshot_from_limits(&limits, updated_at, source)
        }
    }

    #[tokio::test]
    async fn fixture_client_maps_quota_snapshot() {
        let snapshot = FixtureClient
            .fetch_usage("token", "https://api.z.ai", "2026-05-22T12:00:00Z", "zai-api-quota")
            .await
            .expect("fetch");

        assert_eq!(snapshot.source, "zai-api-quota");
        assert_eq!(snapshot.primary.label, "5 hours");
    }
}
