//! GitHub Copilot provider — OAuth token + `copilot_internal` usage API.
//!
//! Ported from CodexBar `docs/copilot.md` (MIT). Device-flow login UI is planned;
//! tokens are supplied via `MOCHI_COPILOT_TOKEN` or `MOCHI_COPILOT_TOKEN_FILE`.

mod client;
mod credentials;
mod strategy;
mod usage_parse;

use async_trait::async_trait;

use strategy::OAuthInternalStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};
use crate::core::provider_metadata::definition_for;
use crate::core::statuspage;

pub struct CopilotProvider;

impl Provider for CopilotProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Copilot,
            display_name: "Copilot".to_string(),
            supports_status: true,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(OAuthInternalStrategy::new())]
    }
}

#[async_trait]
impl ProviderEnrichment for CopilotProvider {
    async fn enrich_snapshot(&self, mut snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        if let Some(definition) = definition_for(ProviderId::Copilot) {
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
    fn strategies_expose_oauth_internal_api() {
        let provider = CopilotProvider;
        let strategy_ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();

        assert_eq!(strategy_ids, vec!["copilot-oauth-internal"]);
    }

    #[test]
    fn metadata_matches_v1_copilot_expectations() {
        let metadata = CopilotProvider.metadata();
        assert_eq!(metadata.id, ProviderId::Copilot);
        assert!(metadata.supports_status);
        assert!(!metadata.supports_cost);
    }
}
