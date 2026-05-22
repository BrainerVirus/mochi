mod cli_rpc;
mod cookies;
mod oauth;
mod parse;

use cli_rpc::CliRpcStrategy;
use cookies::BrowserCookiesStrategy;
use oauth::OAuthStrategy;

use crate::core::models::ProviderId;
use crate::core::provider::{Provider, ProviderMetadata};

pub struct CodexProvider;

impl Provider for CodexProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderId::Codex,
            display_name: "Codex".to_string(),
            supports_status: true,
            supports_cost: true,
        }
    }

    fn strategies(&self) -> Vec<Box<dyn crate::core::provider::FetchStrategy>> {
        vec![
            Box::new(OAuthStrategy::new()),
            Box::new(CliRpcStrategy::new()),
            Box::new(BrowserCookiesStrategy::new()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategies_prefer_oauth_before_cli_rpc_and_cookies() {
        let provider = CodexProvider;
        let strategy_ids: Vec<_> = provider
            .strategies()
            .iter()
            .map(|strategy| strategy.id())
            .collect();

        assert_eq!(
            strategy_ids,
            vec!["codex-oauth", "codex-cli-rpc", "codex-browser-cookies"]
        );
    }

    #[test]
    fn metadata_matches_v1_codex_expectations() {
        let metadata = CodexProvider.metadata();
        assert_eq!(metadata.id, ProviderId::Codex);
        assert!(metadata.supports_cost);
    }
}
