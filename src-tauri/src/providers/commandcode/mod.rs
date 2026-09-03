//! Command Code provider — `api.commandcode.ai` web-session API.

mod client;
mod credentials;
mod strategy;
mod usage_parse;

pub(crate) use credentials::has_credentials;

use async_trait::async_trait;

use strategy::WebStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderMetadata, ProviderResult};

pub struct CommandCodeProvider;

impl Provider for CommandCodeProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::CommandCode,
            display_name: "Command Code".to_string(),
            supports_status: false,
            supports_cost: true,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(WebStrategy::new())]
    }
}

#[async_trait]
impl crate::core::provider::ProviderEnrichment for CommandCodeProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_web_quota() {
        let provider = CommandCodeProvider;
        let ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();
        assert_eq!(ids, vec!["commandcode-web"]);
    }

    #[test]
    fn metadata_matches_v1_commandcode_expectations() {
        let metadata = CommandCodeProvider.metadata();
        assert_eq!(metadata.id, ProviderId::CommandCode);
        assert_eq!(metadata.display_name, "Command Code");
        assert!(!metadata.supports_status);
        assert!(metadata.supports_cost);
    }
}
