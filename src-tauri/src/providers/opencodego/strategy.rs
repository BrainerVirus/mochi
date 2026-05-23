use std::sync::Arc;

use async_trait::async_trait;

use super::credentials::resolve_session;
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;
use crate::providers::opencode::client::{HttpOpenCodeWebClient, OpenCodeWebClient};

pub struct WebStrategy {
    client: Arc<dyn OpenCodeWebClient>,
}

impl WebStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpOpenCodeWebClient::new()),
        }
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
        "opencode-go-web"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_session(ctx.config(ProviderId::OpenCodeGo))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let session = resolve_session(ctx.config(ProviderId::OpenCodeGo))?
            .ok_or(ProviderError::NotConfigured)?;

        self.client
            .fetch_usage(
                &session,
                ProviderId::OpenCodeGo,
                &current_timestamp(),
                self.id(),
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
