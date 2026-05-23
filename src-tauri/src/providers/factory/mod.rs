//! Factory/Droid provider — web cookies and CodexBar session import.
//!
//! Ported from CodexBar `docs/factory.md` (MIT).

mod client;
mod credentials;
mod strategy;
mod usage_parse;

pub(crate) use credentials::has_credentials;

use async_trait::async_trait;

use strategy::WebCookiesStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};
use crate::core::provider_metadata::definition_for;
use crate::core::statuspage;

pub struct FactoryProvider;

impl Provider for FactoryProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Factory,
            display_name: "Factory/Droid".to_string(),
            supports_status: true,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(WebCookiesStrategy::new())]
    }
}

#[async_trait]
impl ProviderEnrichment for FactoryProvider {
    async fn enrich_snapshot(&self, mut snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        if let Some(definition) = definition_for(ProviderId::Factory) {
            if let Some(status_url) = definition.status_url {
                snapshot.provider_status = statuspage::fetch_status(status_url).await.ok();
            }
        }
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_web_cookies() {
        let provider = FactoryProvider;
        let ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();
        assert_eq!(ids, vec!["factory-web-cookies"]);
    }
}
