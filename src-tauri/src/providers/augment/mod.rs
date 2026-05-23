//! Augment provider — `auggie` CLI with web cookie fallback.
//!
//! Ported from CodexBar `docs/augment.md` (MIT).

mod client;
mod credentials;
mod strategy;
mod usage_parse;

pub(crate) use credentials::has_credentials;

use async_trait::async_trait;

use strategy::{CliStrategy, WebStrategy};

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderMetadata, ProviderResult};

pub struct AugmentProvider;

impl Provider for AugmentProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Augment,
            display_name: "Augment".to_string(),
            supports_status: false,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(CliStrategy::new()), Box::new(WebStrategy::new())]
    }
}

#[async_trait]
impl crate::core::provider::ProviderEnrichment for AugmentProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_cli_then_web() {
        let provider = AugmentProvider;
        let ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();
        assert_eq!(ids, vec!["augment-cli", "augment-web"]);
    }
}
