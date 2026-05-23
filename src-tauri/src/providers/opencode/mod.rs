mod client;
mod credentials;
pub(crate) mod strategy;
mod usage_parse;
mod zen_balance;

use async_trait::async_trait;

use strategy::WebStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};

pub struct OpenCodeProvider;

impl Provider for OpenCodeProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::OpenCode,
            display_name: "OpenCode".to_string(),
            supports_status: false,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(WebStrategy::new(ProviderId::OpenCode, "opencode-web"))]
    }
}

#[async_trait]
impl ProviderEnrichment for OpenCodeProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

pub(crate) fn has_credentials(config: Option<&crate::settings::ProviderConfig>) -> bool {
    credentials::resolve_session(config)
        .ok()
        .flatten()
        .is_some()
}
