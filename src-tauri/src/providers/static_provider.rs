use async_trait::async_trait;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, Provider, ProviderError, ProviderMetadata,
    ProviderResult,
};

pub struct StaticProvider {
    id: ProviderId,
    name: &'static str,
}

impl StaticProvider {
    pub const fn new(id: ProviderId, name: &'static str) -> Self {
        Self { id, name }
    }
}

impl Provider for StaticProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: self.id,
            display_name: self.name.to_string(),
            supports_status: true,
            supports_cost: matches!(self.id, ProviderId::Codex | ProviderId::Claude),
        }
    }

    fn strategies(&self) -> Vec<Box<dyn FetchStrategy>> {
        vec![Box::new(StaticStrategy {
            id: self.id,
            name: self.name,
        })]
    }
}

struct StaticStrategy {
    id: ProviderId,
    name: &'static str,
}

#[async_trait]
impl FetchStrategy for StaticStrategy {
    fn id(&self) -> &'static str {
        "static-snapshot"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::LocalConfig
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(true)
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        Ok(UsageSnapshot {
            provider: self.id,
            primary: UsageWindow::new("Session", 0.0, None),
            secondary: Some(UsageWindow::new("Weekly", 0.0, None)),
            updated_at: "1970-01-01T00:00:00Z".to_string(),
            source: self.name.to_string(),
        })
    }

    fn should_fallback(&self, _error: &ProviderError) -> bool {
        false
    }
}
