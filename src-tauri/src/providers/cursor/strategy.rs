use std::sync::Arc;

use async_trait::async_trait;

use super::client::{fetch_snapshot_with_client, CursorWebClient, HttpCursorWebClient};
use crate::core::models::ProviderId;
use crate::core::models::UsageSnapshot;
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};
use crate::core::usage_store::current_timestamp;

pub struct WebStrategy {
    client: Arc<dyn CursorWebClient>,
}

impl WebStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpCursorWebClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CursorWebClient>) -> Self {
        Self { client }
    }
}

impl Default for WebStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for WebStrategy {
    fn id(&self) -> &'static str {
        "cursor-web"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::BrowserCookies
    }

    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool> {
        Ok(super::credentials::resolve_cookie(ctx.config(ProviderId::Cursor))?.is_some())
    }

    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        fetch_snapshot_with_client(
            self.client.as_ref(),
            &current_timestamp(),
            ctx.config(ProviderId::Cursor),
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
    use super::super::client::CursorWebClient;
    use super::super::usage_parse::CursorUsageSummary;
    use super::*;
    use crate::core::test_env;
    use async_trait::async_trait;

    struct MockCursorWebClient {
        summary: ProviderResult<CursorUsageSummary>,
    }

    #[async_trait]
    impl CursorWebClient for MockCursorWebClient {
        async fn fetch_usage_summary(
            &self,
            _cookie_header: &str,
        ) -> ProviderResult<CursorUsageSummary> {
            match &self.summary {
                Ok(summary) => Ok(summary.clone()),
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
    async fn web_strategy_returns_snapshot_with_cookie() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        let path = std::env::temp_dir().join(format!(
            "mochi-cursor-cookie-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::write(&path, "WorkosCursorSessionToken=test").expect("write cookie");
        std::env::set_var(
            "MOCHI_CURSOR_COOKIE_FILE",
            path.to_string_lossy().to_string(),
        );
        std::env::remove_var("MOCHI_CURSOR_COOKIE");

        let summary: CursorUsageSummary =
            serde_json::from_str(include_str!("fixtures/usage_summary.json")).expect("json");
        let strategy = WebStrategy::with_client(Arc::new(MockCursorWebClient {
            summary: Ok(summary),
        }));

        let snapshot = strategy
            .fetch(&FetchContext::empty())
            .await
            .expect("cursor fetch");
        assert_eq!(snapshot.source, "cursor-web");
        assert_eq!(snapshot.primary.used_percent, 30.0);

        std::env::remove_var("MOCHI_CURSOR_COOKIE_FILE");
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn web_strategy_unavailable_without_cookie() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CURSOR_COOKIE");
        std::env::remove_var("MOCHI_CURSOR_COOKIE_FILE");

        let strategy = WebStrategy::new();
        let available = strategy
            .is_available(&FetchContext::empty())
            .await
            .expect("check");
        assert!(!available);
    }
}
