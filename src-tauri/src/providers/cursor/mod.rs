//! Cursor provider — web cookie session + usage-summary API.
//!
//! Ported from CodexBar `docs/cursor.md` (MIT). Browser cookie import and WebKit
//! session storage are planned follow-ups.

mod client;
mod credentials;
mod strategy;
mod usage_parse;

use async_trait::async_trait;

use strategy::WebStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};
use crate::core::provider_metadata::definition_for;
use crate::core::statuspage;

pub struct CursorProvider;

impl Provider for CursorProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Cursor,
            display_name: "Cursor".to_string(),
            supports_status: true,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(WebStrategy::new())]
    }
}

#[async_trait]
impl ProviderEnrichment for CursorProvider {
    async fn enrich_snapshot(&self, mut snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        if let Some(definition) = definition_for(ProviderId::Cursor) {
            if let Some(status_url) = definition.status_url {
                snapshot.provider_status = statuspage::fetch_status(status_url).await.ok();
            }
        }

        Ok(snapshot)
    }
}

pub(crate) fn has_credentials(config: Option<&crate::settings::ProviderConfig>) -> bool {
    credentials::resolve_manual_cookie(config)
        .ok()
        .flatten()
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_web_api() {
        let provider = CursorProvider;
        let strategy_ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();

        assert_eq!(strategy_ids, vec!["cursor-web"]);
    }

    #[test]
    fn metadata_matches_v1_cursor_expectations() {
        let metadata = CursorProvider.metadata();
        assert_eq!(metadata.id, ProviderId::Cursor);
        assert!(metadata.supports_status);
        assert!(!metadata.supports_cost);
    }
}
