use std::sync::Arc;

use async_trait::async_trait;

use super::client::{HttpOpenCodeWebClient, OpenCodeWebClient};
use super::credentials::resolve_session;
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct WebStrategy {
    provider: ProviderId,
    strategy_id: &'static str,
    client: Arc<dyn OpenCodeWebClient>,
}

impl WebStrategy {
    pub fn new(provider: ProviderId, strategy_id: &'static str) -> Self {
        Self {
            provider,
            strategy_id,
            client: Arc::new(HttpOpenCodeWebClient::new()),
        }
    }
}

#[async_trait]
impl FetchStrategy for WebStrategy {
    fn id(&self) -> &'static str {
        self.strategy_id
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_session(ctx.config(self.provider))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let session = resolve_session(ctx.config(self.provider))?
            .ok_or(ProviderError::NotConfigured)?;

        self.client
            .fetch_usage(
                &session,
                self.provider,
                &current_timestamp(),
                self.strategy_id,
            )
            .await
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::NotConfigured
                | ProviderError::Auth(_)
                | ProviderError::Timeout
                | ProviderError::Fetch(_)
                | ProviderError::Parse(_)
        )
    }
}
