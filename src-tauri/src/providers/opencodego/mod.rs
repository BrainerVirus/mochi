//! OpenCode Go provider — OpenCode web usage plus optional Zen pay-as-you-go balance.
//!
//! Zen here is the OpenCode Go pay-as-you-go balance, not the Zen browser cookie source.

mod strategy;

use async_trait::async_trait;

use credentials::resolve_session;
use strategy::WebStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};

pub mod credentials;

pub struct OpenCodeGoProvider;

impl Provider for OpenCodeGoProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::OpenCodeGo,
            display_name: "OpenCode Go".to_string(),
            supports_status: false,
            supports_cost: true,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(WebStrategy::new())]
    }
}

#[async_trait]
impl ProviderEnrichment for OpenCodeGoProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

pub(crate) fn has_credentials(config: Option<&crate::settings::ProviderConfig>) -> bool {
    resolve_session(config).ok().flatten().is_some()
}
