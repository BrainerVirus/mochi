use async_trait::async_trait;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{FetchContext, FetchKind, FetchStrategy, ProviderResult};
use crate::providers::opencode::strategy::WebStrategy as OpenCodeWebStrategy;

pub struct WebStrategy {
    inner: OpenCodeWebStrategy,
}

impl WebStrategy {
    pub fn new() -> Self {
        Self {
            inner: OpenCodeWebStrategy::new(ProviderId::OpenCodeGo, "opencode-go-web"),
        }
    }
}

#[async_trait]
impl FetchStrategy for WebStrategy {
    fn id(&self) -> &'static str {
        "opencode-go-web"
    }

    fn kind(&self) -> FetchKind {
        self.inner.kind()
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        self.inner.is_available(ctx).await
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        self.inner.fetch(ctx).await
    }

    fn should_fallback(&self, error: &crate::core::provider::ProviderError) -> bool {
        self.inner.should_fallback(error)
    }
}
