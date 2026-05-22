use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use time::OffsetDateTime;

use super::credentials::{
    codex_auth_path, current_timestamp, load_credentials_from_path, CodexOAuthCredentials,
};
use super::parse::CodexUsageResponse;
use crate::core::provider::{ProviderError, ProviderResult};

const REFRESH_ENDPOINT: &str = "https://auth.openai.com/oauth/token";
const DEFAULT_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait CodexOAuthClient: Send + Sync {
    async fn load_credentials(&self) -> ProviderResult<CodexOAuthCredentials>;
    async fn save_credentials(&self, credentials: &CodexOAuthCredentials) -> ProviderResult<()>;
    async fn refresh_credentials(
        &self,
        credentials: &CodexOAuthCredentials,
    ) -> ProviderResult<CodexOAuthCredentials>;
    async fn fetch_usage(
        &self,
        access_token: &str,
        account_id: Option<&str>,
    ) -> ProviderResult<CodexUsageResponse>;
}

pub struct HttpCodexOAuthClient {
    auth_path: PathBuf,
    http: reqwest::Client,
}

impl HttpCodexOAuthClient {
    pub fn new() -> Self {
        Self::with_auth_path(codex_auth_path())
    }

    pub fn with_auth_path(auth_path: PathBuf) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { auth_path, http }
    }
}

impl Default for HttpCodexOAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CodexOAuthClient for HttpCodexOAuthClient {
    async fn load_credentials(&self) -> ProviderResult<CodexOAuthCredentials> {
        load_credentials_from_path(&self.auth_path)
    }

    async fn save_credentials(&self, credentials: &CodexOAuthCredentials) -> ProviderResult<()> {
        let mut json: serde_json::Value = if self.auth_path.is_file() {
            let data = std::fs::read_to_string(&self.auth_path)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?;
            serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let mut tokens = serde_json::Map::new();
        tokens.insert(
            "access_token".into(),
            credentials.access_token.clone().into(),
        );
        tokens.insert(
            "refresh_token".into(),
            credentials.refresh_token.clone().into(),
        );
        if let Some(id_token) = &credentials.id_token {
            tokens.insert("id_token".into(), id_token.clone().into());
        }
        if let Some(account_id) = &credentials.account_id {
            tokens.insert("account_id".into(), account_id.clone().into());
        }

        json["tokens"] = serde_json::Value::Object(tokens);
        json["last_refresh"] = current_timestamp().into();

        if let Some(parent) = self.auth_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        }

        let serialized = serde_json::to_string_pretty(&json)
            .map_err(|error| ProviderError::Parse(error.to_string()))?;
        std::fs::write(&self.auth_path, serialized)
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;
        Ok(())
    }

    async fn refresh_credentials(
        &self,
        credentials: &CodexOAuthCredentials,
    ) -> ProviderResult<CodexOAuthCredentials> {
        if credentials.refresh_token.is_empty() {
            return Ok(credentials.clone());
        }

        let response = self
            .http
            .post(REFRESH_ENDPOINT)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "client_id": OAUTH_CLIENT_ID,
                "grant_type": "refresh_token",
                "refresh_token": credentials.refresh_token,
                "scope": "openid profile email",
            }))
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Auth(
                "codex refresh token expired; run `codex` to log in again".into(),
            ));
        }

        if !status.is_success() {
            return Err(ProviderError::Fetch(format!(
                "codex token refresh failed ({status}): {body}"
            )));
        }

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|error| ProviderError::Parse(error.to_string()))?;

        Ok(CodexOAuthCredentials {
            access_token: json
                .get("access_token")
                .and_then(|value| value.as_str())
                .unwrap_or(&credentials.access_token)
                .to_string(),
            refresh_token: json
                .get("refresh_token")
                .and_then(|value| value.as_str())
                .unwrap_or(&credentials.refresh_token)
                .to_string(),
            id_token: json
                .get("id_token")
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .or_else(|| credentials.id_token.clone()),
            account_id: credentials.account_id.clone(),
            last_refresh: Some(OffsetDateTime::now_utc()),
        })
    }

    async fn fetch_usage(
        &self,
        access_token: &str,
        account_id: Option<&str>,
    ) -> ProviderResult<CodexUsageResponse> {
        let mut request = self
            .http
            .get(DEFAULT_USAGE_URL)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("User-Agent", "mochi")
            .header("Accept", "application/json");

        if let Some(account_id) = account_id.filter(|value| !value.is_empty()) {
            request = request.header("ChatGPT-Account-Id", account_id);
        }

        let response = request
            .send()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| ProviderError::Fetch(error.to_string()))?;

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ProviderError::Auth(
                "codex oauth token expired or invalid".into(),
            ));
        }

        if !status.is_success() {
            return Err(ProviderError::Fetch(format!(
                "codex usage API error ({status}): {body}"
            )));
        }

        serde_json::from_str(&body).map_err(|error| ProviderError::Parse(error.to_string()))
    }
}
