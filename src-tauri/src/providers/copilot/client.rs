//! HTTP client for GitHub Copilot internal usage API.
//!
//! Derived from CodexBar `CopilotUsageFetcher.swift` (MIT).

use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use super::credentials::{resolve_enterprise_host, usage_url};
use super::usage_parse::{parse_usage_response, CopilotUsageResponse};
use crate::core::provider::{ProviderError, ProviderResult};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait CopilotUsageClient: Send + Sync {
    async fn fetch_usage(&self, token: &str) -> ProviderResult<CopilotUsageResponse>;
}

pub struct HttpCopilotUsageClient {
    http: reqwest::Client,
    enterprise_host: Option<String>,
}

impl HttpCopilotUsageClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http,
            enterprise_host: resolve_enterprise_host(),
        }
    }
}

impl Default for HttpCopilotUsageClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CopilotUsageClient for HttpCopilotUsageClient {
    async fn fetch_usage(&self, token: &str) -> ProviderResult<CopilotUsageResponse> {
        let url = usage_url(self.enterprise_host.as_deref());
        let response = self
            .http
            .get(url)
            .header("Authorization", format!("token {token}"))
            .header("Accept", "application/json")
            .header("Editor-Version", "vscode/1.96.2")
            .header("Editor-Plugin-Version", "copilot-chat/0.26.7")
            .header("User-Agent", "GitHubCopilotChat/0.26.7")
            .header("X-Github-Api-Version", "2025-04-01")
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let data = response
            .bytes()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        match status.as_u16() {
            200 => {
                let value: Value = serde_json::from_slice(&data)
                    .map_err(|error| ProviderError::Parse(error.to_string()))?;
                parse_usage_response(&value)
            }
            401 | 403 => Err(ProviderError::Auth(
                "copilot token unauthorized or expired".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "copilot usage request failed: HTTP {code}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockCopilotUsageClient {
        usage: ProviderResult<CopilotUsageResponse>,
    }

    #[async_trait]
    impl CopilotUsageClient for MockCopilotUsageClient {
        async fn fetch_usage(&self, _token: &str) -> ProviderResult<CopilotUsageResponse> {
            match &self.usage {
                Ok(response) => Ok(response.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }
    }

    #[tokio::test]
    async fn mock_client_returns_fixture_usage() {
        let value: Value =
            serde_json::from_str(include_str!("fixtures/usage_premium_chat.json")).expect("json");
        let usage = parse_usage_response(&value).expect("parse");

        let client = MockCopilotUsageClient {
            usage: Ok(usage.clone()),
        };
        let fetched = client.fetch_usage("gho_test").await.expect("usage");
        assert_eq!(
            fetched.quota_snapshots.premium_interactions.is_some(),
            usage.quota_snapshots.premium_interactions.is_some()
        );
    }
}
