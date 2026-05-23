use std::sync::Arc;

use async_trait::async_trait;

use super::client::{
    fetch_snapshot_with_client, CodexWebDashboardClient, HttpCodexWebDashboardClient,
};
use super::credentials::resolve_manual_cookie;
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct BrowserCookiesStrategy {
    client: Arc<dyn CodexWebDashboardClient>,
}

impl BrowserCookiesStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpCodexWebDashboardClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CodexWebDashboardClient>) -> Self {
        Self { client }
    }
}

impl Default for BrowserCookiesStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for BrowserCookiesStrategy {
    fn id(&self) -> &'static str {
        "codex-browser-cookies"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(resolve_manual_cookie(ctx.config(ProviderId::Codex))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        fetch_snapshot_with_client(
            self.client.as_ref(),
            &current_timestamp(),
            ctx.config(ProviderId::Codex),
        )
        .await
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(
            error,
            ProviderError::NotConfigured
                | ProviderError::Auth(_)
                | ProviderError::Timeout
                | ProviderError::Fetch(_)
                | ProviderError::Parse(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::client::CodexWebDashboardClient;
    use super::*;
    use crate::core::test_env;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockDashboardClient {
        html: ProviderResult<String>,
    }

    #[async_trait]
    impl CodexWebDashboardClient for MockDashboardClient {
        async fn fetch_dashboard_html(&self, _cookie_header: &str) -> ProviderResult<String> {
            match &self.html {
                Ok(html) => Ok(html.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn is_available_when_manual_cookie_configured() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        let path = std::env::temp_dir().join(format!(
            "mochi-codex-cookie-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&path, "session=abc").expect("write cookie");
        std::env::set_var(
            "MOCHI_CODEX_COOKIE_FILE",
            path.to_string_lossy().to_string(),
        );
        std::env::remove_var("MOCHI_CODEX_COOKIE");

        let strategy = BrowserCookiesStrategy::new();
        assert!(strategy
            .is_available(&FetchContext::empty())
            .await
            .expect("availability"));

        std::env::remove_var("MOCHI_CODEX_COOKIE_FILE");
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn fetch_reports_not_configured_without_cookie() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CODEX_COOKIE");
        std::env::remove_var("MOCHI_CODEX_COOKIE_FILE");
        let strategy = BrowserCookiesStrategy::new();

        let error = strategy
            .fetch(&FetchContext::empty())
            .await
            .expect_err("missing cookie should fail");

        assert!(matches!(error, ProviderError::NotConfigured));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn browser_cookie_fetch_can_fallback_on_auth_errors() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        let path = std::env::temp_dir().join(format!(
            "mochi-codex-cookie-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&path, "session=abc").expect("write cookie");
        std::env::set_var(
            "MOCHI_CODEX_COOKIE_FILE",
            path.to_string_lossy().to_string(),
        );
        std::env::remove_var("MOCHI_CODEX_COOKIE");

        let strategy = BrowserCookiesStrategy::with_client(Arc::new(MockDashboardClient {
            html: Err(ProviderError::Auth("expired".into())),
        }));

        let error = strategy
            .fetch(&FetchContext::empty())
            .await
            .expect_err("auth should fail");

        assert!(strategy.should_fallback(&error));
        std::env::remove_var("MOCHI_CODEX_COOKIE_FILE");
        let _ = std::fs::remove_file(path);
    }
}
