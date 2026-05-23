//! HTTP client for Claude OAuth usage API.
//!
//! Derived from CodexBar `ClaudeOAuthUsageFetcher` (MIT).

use std::time::Duration;

use async_trait::async_trait;
use time::OffsetDateTime;

use super::credentials::{
    claude_credentials_path, load_credentials, load_credentials_from_path, ClaudeOAuthCredentials,
};
use crate::core::provider::{ProviderError, ProviderResult};
use crate::providers::claude::usage_parse::ClaudeUsageResponse;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const REFRESH_URL: &str = "https://platform.claude.com/v1/oauth/token";
const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20";
const DEFAULT_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait ClaudeOAuthClient: Send + Sync {
    async fn load_credentials(&self) -> ProviderResult<ClaudeOAuthCredentials>;
    async fn save_credentials(&self, credentials: &ClaudeOAuthCredentials) -> ProviderResult<()>;
    async fn refresh_credentials(
        &self,
        credentials: &ClaudeOAuthCredentials,
    ) -> ProviderResult<ClaudeOAuthCredentials>;
    async fn fetch_usage(&self, access_token: &str) -> ProviderResult<ClaudeUsageResponse>;
}

pub struct HttpClaudeOAuthClient {
    credentials_path: std::path::PathBuf,
    http: reqwest::Client,
}

impl HttpClaudeOAuthClient {
    pub fn new() -> Self {
        Self::with_credentials_path(claude_credentials_path())
    }

    pub fn with_credentials_path(credentials_path: std::path::PathBuf) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            credentials_path,
            http,
        }
    }
}

impl Default for HttpClaudeOAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

fn oauth_client_id() -> String {
    std::env::var("MOCHI_CLAUDE_OAUTH_CLIENT_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string())
}

#[async_trait]
impl ClaudeOAuthClient for HttpClaudeOAuthClient {
    async fn load_credentials(&self) -> ProviderResult<ClaudeOAuthCredentials> {
        load_credentials_from_path(&self.credentials_path).or_else(|error| {
            if matches!(error, ProviderError::NotConfigured) {
                load_credentials()
            } else {
                Err(error)
            }
        })
    }

    async fn save_credentials(&self, credentials: &ClaudeOAuthCredentials) -> ProviderResult<()> {
        let mut root: serde_json::Value = if self.credentials_path.is_file() {
            let data = std::fs::read_to_string(&self.credentials_path)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?;
            serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let expires_at_ms = credentials.expires_at.map(|value| {
            (value.unix_timestamp() as f64) * 1000.0 + (value.nanosecond() as f64) / 1_000_000.0
        });

        root["claudeAiOauth"] = serde_json::json!({
            "accessToken": credentials.access_token,
            "refreshToken": credentials.refresh_token,
            "expiresAt": expires_at_ms,
            "scopes": credentials.scopes,
        });

        if let Some(parent) = self.credentials_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        }

        std::fs::write(
            &self.credentials_path,
            serde_json::to_string_pretty(&root)
                .map_err(|error| ProviderError::Parse(error.to_string()))?,
        )
        .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        Ok(())
    }

    async fn refresh_credentials(
        &self,
        credentials: &ClaudeOAuthCredentials,
    ) -> ProviderResult<ClaudeOAuthCredentials> {
        let refresh_token = credentials
            .refresh_token
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ProviderError::Auth("claude oauth missing refresh token".into()))?;

        let client_id = oauth_client_id();
        let body = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id.as_str()),
        ];

        let response = self
            .http
            .post(REFRESH_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&body)
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let data = response
            .bytes()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if !status.is_success() {
            return Err(ProviderError::Auth(format!(
                "claude oauth refresh failed: HTTP {status}"
            )));
        }

        let json: serde_json::Value = serde_json::from_slice(&data)
            .map_err(|error| ProviderError::Parse(error.to_string()))?;

        let access_token = json
            .get("access_token")
            .and_then(|value| value.as_str())
            .ok_or_else(|| ProviderError::Parse("claude refresh missing access_token".into()))?;

        let refresh_token = json
            .get("refresh_token")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| credentials.refresh_token.clone());

        let expires_in = json.get("expires_in").and_then(|value| value.as_u64());
        let expires_at = expires_in
            .map(|seconds| OffsetDateTime::now_utc() + time::Duration::seconds(seconds as i64));

        Ok(ClaudeOAuthCredentials {
            access_token: access_token.to_string(),
            refresh_token,
            expires_at,
            scopes: credentials.scopes.clone(),
        })
    }

    async fn fetch_usage(&self, access_token: &str) -> ProviderResult<ClaudeUsageResponse> {
        let response = self
            .http
            .get(USAGE_URL)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("anthropic-beta", OAUTH_BETA_HEADER)
            .header("User-Agent", "claude-code/2.1.0")
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
                "claude oauth unauthorized; run `claude` to re-authenticate".into(),
            )),
            code => Err(ProviderError::Fetch(format!(
                "claude oauth usage failed: HTTP {code}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::credentials::parse_credentials;
    use super::*;

    struct MockClaudeOAuthClient {
        credentials: ProviderResult<ClaudeOAuthCredentials>,
        usage: ProviderResult<ClaudeUsageResponse>,
    }

    #[async_trait]
    impl ClaudeOAuthClient for MockClaudeOAuthClient {
        async fn load_credentials(&self) -> ProviderResult<ClaudeOAuthCredentials> {
            match &self.credentials {
                Ok(credentials) => Ok(credentials.clone()),
                Err(ProviderError::NotConfigured) => Err(ProviderError::NotConfigured),
                Err(ProviderError::Auth(message)) => Err(ProviderError::Auth(message.clone())),
                Err(ProviderError::Timeout) => Err(ProviderError::Timeout),
                Err(ProviderError::Parse(message)) => Err(ProviderError::Parse(message.clone())),
                Err(ProviderError::Fetch(message)) => Err(ProviderError::Fetch(message.clone())),
            }
        }

        async fn save_credentials(
            &self,
            _credentials: &ClaudeOAuthCredentials,
        ) -> ProviderResult<()> {
            Ok(())
        }

        async fn refresh_credentials(
            &self,
            credentials: &ClaudeOAuthCredentials,
        ) -> ProviderResult<ClaudeOAuthCredentials> {
            Ok(credentials.clone())
        }

        async fn fetch_usage(&self, _access_token: &str) -> ProviderResult<ClaudeUsageResponse> {
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

    fn fixture_credentials() -> ClaudeOAuthCredentials {
        parse_credentials(include_str!("../fixtures/credentials.json")).expect("credentials")
    }

    fn fixture_usage() -> ClaudeUsageResponse {
        serde_json::from_str(include_str!("../fixtures/oauth_usage.json")).expect("usage")
    }

    #[tokio::test]
    async fn mock_client_returns_usage_fixture() {
        let client = MockClaudeOAuthClient {
            credentials: Ok(fixture_credentials()),
            usage: Ok(fixture_usage()),
        };

        let creds = client.load_credentials().await.expect("credentials");
        let usage = client
            .fetch_usage(&creds.access_token)
            .await
            .expect("usage");
        assert!(usage.five_hour.is_some());
    }
}
