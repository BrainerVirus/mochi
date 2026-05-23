//! HTTP client for Claude Web API (`claude.ai/api`).
//!
//! Derived from CodexBar `ClaudeWebAPIFetcher` (MIT).

use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

use super::credentials::resolve_session_key;
use crate::core::provider::{ProviderError, ProviderResult};
use crate::providers::claude::usage_parse::ClaudeUsageResponse;

const BASE_URL: &str = "https://claude.ai/api";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Deserialize)]
struct ClaudeOrganization {
    uuid: String,
    #[allow(dead_code)]
    name: Option<String>,
    capabilities: Option<Vec<String>>,
    #[serde(rename = "is_api_only")]
    is_api_only: Option<bool>,
}

#[async_trait]
pub trait ClaudeWebClient: Send + Sync {
    async fn resolve_session_key(&self) -> ProviderResult<String>;
    async fn fetch_usage(&self, session_key: &str) -> ProviderResult<ClaudeUsageResponse>;
}

pub struct HttpClaudeWebClient {
    http: reqwest::Client,
}

impl HttpClaudeWebClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { http }
    }
}

impl Default for HttpClaudeWebClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClaudeWebClient for HttpClaudeWebClient {
    async fn resolve_session_key(&self) -> ProviderResult<String> {
        resolve_session_key()?.ok_or(ProviderError::NotConfigured)
    }

    async fn fetch_usage(&self, session_key: &str) -> ProviderResult<ClaudeUsageResponse> {
        let org_id = self.fetch_organization_id(session_key).await?;
        self.fetch_org_usage(session_key, &org_id).await
    }
}

impl HttpClaudeWebClient {
    async fn fetch_organization_id(&self, session_key: &str) -> ProviderResult<String> {
        let url = format!("{BASE_URL}/organizations");
        let response = self
            .get_json(&url, session_key)
            .await
            .map_err(|error| map_web_error(error, "organizations"))?;

        let organizations: Vec<ClaudeOrganization> = serde_json::from_value(response)
            .map_err(|error| ProviderError::Parse(error.to_string()))?;

        let selected = organizations
            .iter()
            .find(|org| {
                org.capabilities
                    .as_ref()
                    .is_some_and(|caps| caps.iter().any(|cap| cap == "chat"))
            })
            .or_else(|| {
                organizations
                    .iter()
                    .find(|org| org.is_api_only != Some(true))
            })
            .or_else(|| organizations.first())
            .ok_or_else(|| ProviderError::Parse("claude web: no organization".into()))?;

        Ok(selected.uuid.clone())
    }

    async fn fetch_org_usage(
        &self,
        session_key: &str,
        org_id: &str,
    ) -> ProviderResult<ClaudeUsageResponse> {
        let url = format!("{BASE_URL}/organizations/{org_id}/usage");
        let response = self.get_json(&url, session_key).await?;
        serde_json::from_value(response).map_err(|error| ProviderError::Parse(error.to_string()))
    }

    async fn get_json(&self, url: &str, session_key: &str) -> ProviderResult<serde_json::Value> {
        let response = self
            .http
            .get(url)
            .header("Cookie", format!("sessionKey={session_key}"))
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
                "claude web session unauthorized or expired".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "claude web request failed: HTTP {code}"
            ))),
        }
    }
}

fn map_web_error(error: ProviderError, step: &str) -> ProviderError {
    match error {
        ProviderError::Fetch(message) => {
            ProviderError::Fetch(format!("claude web {step}: {message}"))
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockClaudeWebClient {
        usage: ProviderResult<ClaudeUsageResponse>,
    }

    #[async_trait]
    impl ClaudeWebClient for MockClaudeWebClient {
        async fn resolve_session_key(&self) -> ProviderResult<String> {
            Ok("sk-ant-test".into())
        }

        async fn fetch_usage(&self, _session_key: &str) -> ProviderResult<ClaudeUsageResponse> {
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

    fn fixture_usage() -> ClaudeUsageResponse {
        serde_json::from_str(include_str!("../fixtures/web_usage.json")).expect("usage")
    }

    #[test]
    fn parses_organizations_fixture() {
        let orgs: Vec<ClaudeOrganization> =
            serde_json::from_str(include_str!("../fixtures/organizations.json")).expect("orgs");
        assert_eq!(orgs[0].uuid, "org-11111111-2222-3333-4444-555555555555");
    }

    #[tokio::test]
    async fn mock_web_client_returns_usage() {
        let client = MockClaudeWebClient {
            usage: Ok(fixture_usage()),
        };
        let key = client.resolve_session_key().await.expect("key");
        let usage = client.fetch_usage(&key).await.expect("usage");
        assert!(usage.five_hour.is_some());
    }
}
