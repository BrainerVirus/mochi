//! z.ai provider — API token quota endpoint.
//!
//! Ported from CodexBar `docs/zai.md` (MIT).

mod client;
mod credentials;
mod strategy;
mod usage_parse;

pub(crate) use credentials::resolve_api_key;

use async_trait::async_trait;

use strategy::ApiQuotaStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderMetadata, ProviderResult};

pub struct ZaiProvider;

impl Provider for ZaiProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Zai,
            display_name: "z.ai".to_string(),
            supports_status: false,
            supports_cost: false,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![Box::new(ApiQuotaStrategy::new())]
    }
}

#[async_trait]
impl crate::core::provider::ProviderEnrichment for ZaiProvider {
    async fn enrich_snapshot(&self, snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        Ok(snapshot)
    }
}

pub(crate) fn has_credentials(config: Option<&crate::settings::ProviderConfig>) -> bool {
    resolve_api_key(config).ok().flatten().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_expose_api_quota() {
        let provider = ZaiProvider;
        let ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();
        assert_eq!(ids, vec!["zai-api-quota"]);
    }
}
