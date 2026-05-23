//! Kiro provider — `kiro-cli chat --no-interactive "/usage"`.
//!
//! Ported from CodexBar `docs/kiro.md` (MIT).

mod strategy;
mod usage_parse;

use async_trait::async_trait;

use strategy::CliUsageStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderMetadata, ProviderResult};

pub struct KiroProvider;

impl Provider for KiroProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Kiro,
            display_name: "Kiro".to_string(),
            supports_status: true,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(CliUsageStrategy::new())]
    }
}

#[async_trait]
impl crate::core::provider::ProviderEnrichment for KiroProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

pub(crate) fn has_credentials() -> bool {
    strategy::resolve_kiro_binary().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_cli_usage() {
        let provider = KiroProvider;
        let ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();
        assert_eq!(ids, vec!["kiro-cli-usage"]);
    }
}
