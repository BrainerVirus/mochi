//! Antigravity provider — local LSP HTTPS probe.
//!
//! Ported from CodexBar `docs/antigravity.md` (MIT).

mod probe;
mod strategy;
mod usage_parse;

pub(crate) use probe::is_probe_available;

use async_trait::async_trait;

use strategy::LocalProbeStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderMetadata, ProviderResult};

pub struct AntigravityProvider;

impl Provider for AntigravityProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Antigravity,
            display_name: "Antigravity".to_string(),
            supports_status: true,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(LocalProbeStrategy::new())]
    }
}

#[async_trait]
impl crate::core::provider::ProviderEnrichment for AntigravityProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_local_probe() {
        let provider = AntigravityProvider;
        let ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();
        assert_eq!(ids, vec!["antigravity-local-probe"]);
    }
}
