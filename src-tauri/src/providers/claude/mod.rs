//! Claude provider — OAuth API and Web API fetch strategies.
//!
//! Ported from CodexBar `docs/claude.md` (MIT). CLI PTY and Admin API are planned follow-ups.

mod oauth;
mod usage_parse;
mod web;

use async_trait::async_trait;

use oauth::OAuthStrategy;
use web::WebStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};
use crate::core::provider_metadata::definition_for;
use crate::core::statuspage;

pub struct ClaudeProvider;

impl Provider for ClaudeProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Claude,
            display_name: "Claude".to_string(),
            supports_status: true,
            supports_cost: true,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(OAuthStrategy::new()), Box::new(WebStrategy::new())]
    }
}

#[async_trait]
impl ProviderEnrichment for ClaudeProvider {
    async fn enrich_snapshot(&self, mut snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        if let Some(definition) = definition_for(ProviderId::Claude) {
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
    fn strategies_prefer_oauth_before_web() {
        let provider = ClaudeProvider;
        let strategy_ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();

        assert_eq!(strategy_ids, vec!["claude-oauth", "claude-web"]);
    }

    #[test]
    fn metadata_matches_v1_claude_expectations() {
        let metadata = ClaudeProvider.metadata();
        assert_eq!(metadata.id, ProviderId::Claude);
        assert!(metadata.supports_status);
        assert!(metadata.supports_cost);
    }
}
