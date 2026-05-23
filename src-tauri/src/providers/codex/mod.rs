mod cli_rpc;
mod cookies;
mod cost;
mod oauth;
mod parse;

use async_trait::async_trait;

use cli_rpc::CliRpcStrategy;
use cookies::BrowserCookiesStrategy;
use cost::{default_window_days, scan_session_cost};
use oauth::OAuthStrategy;

use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{Provider, ProviderEnrichment, ProviderMetadata, ProviderResult};
use crate::core::provider_metadata::definition_for;
use crate::core::statuspage;

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

#[async_trait]
impl ProviderEnrichment for CodexProvider {
    async fn enrich_snapshot(&self, mut snapshot: UsageSnapshot) -> ProviderResult<UsageSnapshot> {
        snapshot.session_cost = scan_session_cost(default_window_days()).ok();

        if let Some(definition) = definition_for(ProviderId::Codex) {
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
    use std::sync::{Arc, Mutex};

    use crate::core::models::UsageWindow;
    use crate::core::provider::{FetchContext, FetchStrategy};
    use crate::providers::codex::cookies::BrowserCookiesStrategy;
    use crate::providers::codex::cookies::CodexWebDashboardClient;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        assert!(metadata.supports_status);
    }

    #[tokio::test]
    async fn enrich_snapshot_attaches_session_cost_when_sessions_exist() {
        let snapshot = UsageSnapshot::new(
            ProviderId::Codex,
            UsageWindow::new("Session", 10.0, None),
            None,
            "2026-05-20T12:00:00Z",
            "codex-oauth",
        );

        let enriched = CodexProvider
            .enrich_snapshot(snapshot)
            .await
            .expect("enrich");

        assert!(enriched.session_cost.is_some());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn browser_cookie_strategy_parses_dashboard_fixture() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let path = std::env::temp_dir().join(format!(
            "mochi-codex-cookie-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&path, "session=test").expect("write cookie");
        std::env::set_var(
            "MOCHI_CODEX_COOKIE_FILE",
            path.to_string_lossy().to_string(),
        );
        std::env::remove_var("MOCHI_CODEX_COOKIE");

        let strategy = BrowserCookiesStrategy::with_client(Arc::new(FixtureDashboardClient));

        let snapshot = strategy.fetch(&FetchContext).await.expect("cookie fetch");

        assert_eq!(snapshot.source, "codex-browser-cookies");
        assert_eq!(snapshot.primary.used_percent, 78.0);

        std::env::remove_var("MOCHI_CODEX_COOKIE_FILE");
        let _ = std::fs::remove_file(path);
    }

    struct FixtureDashboardClient;

    #[async_trait]
    impl CodexWebDashboardClient for FixtureDashboardClient {
        async fn fetch_dashboard_html(
            &self,
            _cookie_header: &str,
        ) -> crate::core::provider::ProviderResult<String> {
            Ok(include_str!("fixtures/dashboard_usage.html").to_string())
        }
    }
}
