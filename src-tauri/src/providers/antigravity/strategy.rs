use async_trait::async_trait;

use super::probe::{fetch_local_usage, is_probe_available};
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};

pub struct LocalProbeStrategy;

impl LocalProbeStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalProbeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for LocalProbeStrategy {
    fn id(&self) -> &'static str {
        "antigravity-local-probe"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::LocalProbe
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(is_probe_available())
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        fetch_local_usage().await
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}
