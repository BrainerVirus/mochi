use std::time::Duration;

use async_trait::async_trait;

use super::credentials::resolve_manual_cookie;
use super::parse::snapshot_from_dashboard_html;
use crate::core::models::UsageSnapshot;
use crate::core::provider::{ProviderError, ProviderResult};

const DASHBOARD_URL: &str = "https://chatgpt.com/codex/settings/usage";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait CodexWebDashboardClient: Send + Sync {
    async fn fetch_dashboard_html(&self, cookie_header: &str) -> ProviderResult<String>;
}

pub struct HttpCodexWebDashboardClient {
    http: reqwest::Client,
}

impl HttpCodexWebDashboardClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpCodexWebDashboardClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CodexWebDashboardClient for HttpCodexWebDashboardClient {
    async fn fetch_dashboard_html(&self, cookie_header: &str) -> ProviderResult<String> {
        let response = self
            .http
            .get(DASHBOARD_URL)
            .header("Cookie", cookie_header)
            .header("Accept", "text/html")
            .send()
            .await
            .map_err(|error| {
                ProviderError::Fetch(format!("codex web dashboard request: {error}"))
            })?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(format!("codex web dashboard read: {error}")))?;

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ProviderError::Auth(
                "openai web dashboard cookie rejected".into(),
            ));
        }

        if !status.is_success() {
            return Err(ProviderError::Fetch(format!(
                "codex web dashboard HTTP {status}"
            )));
        }

        Ok(body)
    }
}

pub async fn fetch_snapshot_with_client(
    client: &dyn CodexWebDashboardClient,
    updated_at: &str,
) -> ProviderResult<UsageSnapshot> {
    let cookie = resolve_manual_cookie()?.ok_or(ProviderError::NotConfigured)?;

    let html = client.fetch_dashboard_html(&cookie).await?;
    snapshot_from_dashboard_html(&html, updated_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    use crate::core::usage_store::current_timestamp;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
    async fn fetch_snapshot_parses_dashboard_fixture() {
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

        let client = MockDashboardClient {
            html: Ok(include_str!("../fixtures/dashboard_usage.html").to_string()),
        };

        let snapshot = fetch_snapshot_with_client(&client, &current_timestamp())
            .await
            .expect("snapshot");

        assert_eq!(snapshot.source, "codex-browser-cookies");
        assert_eq!(snapshot.primary.used_percent, 78.0);

        std::env::remove_var("MOCHI_CODEX_COOKIE_FILE");
        let _ = std::fs::remove_file(path);
    }
}
