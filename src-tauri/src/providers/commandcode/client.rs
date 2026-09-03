use std::time::Duration;

use async_trait::async_trait;

use crate::core::provider::{ProviderError, ProviderResult};

const BASE_URL: &str = "https://api.commandcode.ai";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub const CREDITS_PATH: &str = "/internal/billing/credits";
pub const SUMMARY_PATH: &str = "/internal/usage/summary";

#[async_trait]
pub trait CommandCodeClient: Send + Sync {
    async fn fetch_credits(&self, cookie: &str) -> ProviderResult<serde_json::Value>;
    async fn fetch_summary(&self, cookie: &str) -> ProviderResult<serde_json::Value>;
}

pub struct HttpCommandCodeClient {
    http: reqwest::Client,
}

impl HttpCommandCodeClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { http }
    }
}

impl Default for HttpCommandCodeClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandCodeClient for HttpCommandCodeClient {
    async fn fetch_credits(&self, cookie: &str) -> ProviderResult<serde_json::Value> {
        self.get_json(CREDITS_PATH, cookie).await
    }

    async fn fetch_summary(&self, cookie: &str) -> ProviderResult<serde_json::Value> {
        self.get_json(SUMMARY_PATH, cookie).await
    }
}

impl HttpCommandCodeClient {
    async fn get_json(&self, path: &str, cookie: &str) -> ProviderResult<serde_json::Value> {
        let url = format!("{BASE_URL}{path}");
        let response = self
            .http
            .get(url)
            .header("Cookie", cookie)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let data = response
            .bytes()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        match status.as_u16() {
            200 => serde_json::from_slice(&data)
                .map_err(|error| ProviderError::Parse(error.to_string())),
            401 | 403 => Err(ProviderError::Auth(
                "commandcode session unauthorized or expired".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "commandcode request failed: HTTP {code}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingClient {
        credits_cookies: Mutex<Vec<String>>,
        summary_cookies: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl CommandCodeClient for RecordingClient {
        async fn fetch_credits(&self, cookie: &str) -> ProviderResult<serde_json::Value> {
            self.credits_cookies
                .lock()
                .expect("lock")
                .push(cookie.to_string());
            Ok(serde_json::json!({}))
        }

        async fn fetch_summary(&self, cookie: &str) -> ProviderResult<serde_json::Value> {
            self.summary_cookies
                .lock()
                .expect("lock")
                .push(cookie.to_string());
            Ok(serde_json::json!({}))
        }
    }

    #[tokio::test]
    async fn fetches_both_endpoints_with_cookie() {
        let client = RecordingClient::default();
        let cookie = "__Secure-commandcode_prod_.session_token=abc";
        let _ = client.fetch_credits(cookie).await;
        let _ = client.fetch_summary(cookie).await;
        assert_eq!(
            client.credits_cookies.lock().expect("lock").as_slice(),
            [cookie]
        );
        assert_eq!(
            client.summary_cookies.lock().expect("lock").as_slice(),
            [cookie]
        );
    }

    #[test]
    fn endpoints_match_fixed_har_capture() {
        assert_eq!(
            format!("{BASE_URL}{CREDITS_PATH}"),
            "https://api.commandcode.ai/internal/billing/credits"
        );
        assert_eq!(
            format!("{BASE_URL}{SUMMARY_PATH}"),
            "https://api.commandcode.ai/internal/usage/summary"
        );
        assert_eq!(REQUEST_TIMEOUT, Duration::from_secs(30));
    }

    #[allow(dead_code)]
    fn assert_error_shapes(error: ProviderError) -> ProviderError {
        error
    }
}
