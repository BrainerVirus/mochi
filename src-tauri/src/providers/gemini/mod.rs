//! Gemini provider — OAuth-backed quota API via Gemini CLI credentials.
//!
//! Ported from CodexBar `docs/gemini.md` (MIT).

mod client;
mod credentials;
mod oauth_client;
mod strategy;
mod usage_parse;

use async_trait::async_trait;

use strategy::OAuthQuotaStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};

pub struct GeminiProvider;

impl Provider for GeminiProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Gemini,
            display_name: "Gemini".to_string(),
            supports_status: false,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(OAuthQuotaStrategy::new())]
    }
}

#[async_trait]
impl ProviderEnrichment for GeminiProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        // Google Workspace incidents page is not Statuspage.io; skip live polling for now.
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_oauth_quota_api() {
        let provider = GeminiProvider;
        let strategy_ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();

        assert_eq!(strategy_ids, vec!["gemini-oauth-quota"]);
    }

    #[test]
    fn metadata_matches_v1_gemini_expectations() {
        let metadata = GeminiProvider.metadata();
        assert_eq!(metadata.id, ProviderId::Gemini);
        assert!(!metadata.supports_status);
        assert!(!metadata.supports_cost);
    }
}
