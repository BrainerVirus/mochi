//! Codex OAuth fetch strategy.
//!
//! Token read/refresh and usage API logic derived from
//! [CodexBar](https://github.com/steipete/CodexBar) (`docs/codex-oauth.md`, MIT license).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{
    FetchContext, FetchKind, FetchStrategy, ProviderError, ProviderResult,
};

const REFRESH_ENDPOINT: &str = "https://auth.openai.com/oauth/token";
const DEFAULT_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(8 * 24 * 60 * 60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq)]
pub struct CodexOAuthCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>,
    pub account_id: Option<String>,
    pub last_refresh: Option<OffsetDateTime>,
}

impl CodexOAuthCredentials {
    pub fn needs_refresh(&self) -> bool {
        match self.last_refresh {
            Some(last) => {
                OffsetDateTime::now_utc()
                    .unix_timestamp()
                    .saturating_sub(last.unix_timestamp()) as u64
                    > TOKEN_REFRESH_INTERVAL.as_secs()
            }
            None => !self.refresh_token.is_empty(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodexUsageResponse {
    #[serde(default, rename = "plan_type")]
    pub _plan_type: Option<String>,
    pub rate_limit: Option<RateLimitDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitDetails {
    pub primary_window: Option<WindowSnapshot>,
    pub secondary_window: Option<WindowSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowSnapshot {
    pub used_percent: i32,
    pub reset_at: i64,
    pub limit_window_seconds: i32,
}

pub fn codex_auth_path() -> PathBuf {
    if let Some(codex_home) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(codex_home).join("auth.json");
    }

    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| PathBuf::from(home).join(".codex").join("auth.json"))
        .unwrap_or_else(|| PathBuf::from(".codex/auth.json"))
}

pub fn parse_credentials(data: &str) -> ProviderResult<CodexOAuthCredentials> {
    let json: serde_json::Value =
        serde_json::from_str(data).map_err(|error| ProviderError::Parse(error.to_string()))?;

    if let Some(api_key) = json.get("OPENAI_API_KEY").and_then(|value| value.as_str()) {
        let trimmed = api_key.trim();
        if !trimmed.is_empty() {
            return Ok(CodexOAuthCredentials {
                access_token: trimmed.to_string(),
                refresh_token: String::new(),
                id_token: None,
                account_id: None,
                last_refresh: None,
            });
        }
    }

    let tokens = json
        .get("tokens")
        .ok_or_else(|| ProviderError::Auth("codex auth.json missing tokens".into()))?;

    let access_token = token_field(tokens, "access_token")
        .ok_or_else(|| ProviderError::Auth("codex auth.json missing access token".into()))?;
    let refresh_token = token_field(tokens, "refresh_token").unwrap_or_default();

    Ok(CodexOAuthCredentials {
        access_token,
        refresh_token,
        id_token: token_field(tokens, "id_token"),
        account_id: token_field(tokens, "account_id"),
        last_refresh: json
            .get("last_refresh")
            .and_then(|value| value.as_str())
            .and_then(parse_timestamp),
    })
}

pub fn load_credentials_from_path(path: &Path) -> ProviderResult<CodexOAuthCredentials> {
    if !path.is_file() {
        return Err(ProviderError::NotConfigured);
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read codex auth.json: {error}")))?;
    parse_credentials(&data)
}

pub fn snapshot_from_oauth_usage(
    response: &CodexUsageResponse,
    updated_at: &str,
) -> ProviderResult<UsageSnapshot> {
    let rate_limit = response
        .rate_limit
        .as_ref()
        .ok_or_else(|| ProviderError::Parse("codex oauth usage missing rate_limit".into()))?;

    let primary = rate_limit
        .primary_window
        .as_ref()
        .map(window_to_usage)
        .transpose()?;
    let secondary = rate_limit
        .secondary_window
        .as_ref()
        .map(window_to_usage)
        .transpose()?;

    let (primary, secondary) = match (primary, secondary) {
        (Some(primary), secondary) => (primary, secondary),
        (None, Some(secondary)) => (secondary, None),
        (None, None) => {
            return Err(ProviderError::Parse(
                "codex oauth usage missing rate windows".into(),
            ));
        }
    };

    Ok(UsageSnapshot::new(
        ProviderId::Codex,
        primary,
        secondary,
        updated_at,
        "codex-oauth",
    ))
}

fn window_to_usage(window: &WindowSnapshot) -> ProviderResult<UsageWindow> {
    Ok(UsageWindow::new(
        window_label(window.limit_window_seconds),
        window.used_percent as f32,
        format_unix_timestamp(window.reset_at),
    ))
}

fn window_label(limit_window_seconds: i32) -> &'static str {
    let minutes = limit_window_seconds / 60;
    match minutes {
        m if m <= 300 => "Session",
        m if m <= 2_880 => "Daily",
        _ => "Weekly",
    }
}

fn format_unix_timestamp(secs: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()?
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}

fn token_field(tokens: &serde_json::Value, key: &str) -> Option<String> {
    tokens
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_timestamp(value: &str) -> Option<OffsetDateTime> {
    if let Ok(parsed) = OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
    {
        return Some(parsed);
    }

    let format = time::format_description::parse(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]Z",
    )
    .ok()?;
    OffsetDateTime::parse(value, &format).ok()
}

fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

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

pub struct OAuthStrategy {
    client: Arc<dyn CodexOAuthClient>,
}

impl OAuthStrategy {
    pub fn new() -> Self {
        Self {
            client: Arc::new(HttpCodexOAuthClient::new()),
        }
    }

    #[cfg(test)]
    pub fn with_client(client: Arc<dyn CodexOAuthClient>) -> Self {
        Self { client }
    }
}

impl Default for OAuthStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FetchStrategy for OAuthStrategy {
    fn id(&self) -> &'static str {
        "codex-oauth"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::OAuth
    }

    async fn is_available(&self, _ctx: &FetchContext) -> ProviderResult<bool> {
        match self.client.load_credentials().await {
            Ok(_) => Ok(true),
            Err(ProviderError::NotConfigured) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn fetch(&self, _ctx: &FetchContext) -> ProviderResult<UsageSnapshot> {
        let mut credentials = self.client.load_credentials().await?;

        if credentials.needs_refresh() && !credentials.refresh_token.is_empty() {
            credentials = self.client.refresh_credentials(&credentials).await?;
            self.client.save_credentials(&credentials).await?;
        }

        let usage = self
            .client
            .fetch_usage(&credentials.access_token, credentials.account_id.as_deref())
            .await?;

        snapshot_from_oauth_usage(&usage, &current_timestamp())
    }

    fn should_fallback(&self, error: &ProviderError) -> bool {
        matches!(error, ProviderError::NotConfigured | ProviderError::Auth(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCodexOAuthClient {
        credentials: ProviderResult<CodexOAuthCredentials>,
        usage: ProviderResult<CodexUsageResponse>,
    }

    #[async_trait]
    impl CodexOAuthClient for MockCodexOAuthClient {
        async fn load_credentials(&self) -> ProviderResult<CodexOAuthCredentials> {
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
            _credentials: &CodexOAuthCredentials,
        ) -> ProviderResult<()> {
            Ok(())
        }

        async fn refresh_credentials(
            &self,
            credentials: &CodexOAuthCredentials,
        ) -> ProviderResult<CodexOAuthCredentials> {
            Ok(credentials.clone())
        }

        async fn fetch_usage(
            &self,
            _access_token: &str,
            _account_id: Option<&str>,
        ) -> ProviderResult<CodexUsageResponse> {
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

    fn fixture_credentials() -> CodexOAuthCredentials {
        CodexOAuthCredentials {
            access_token: "access-token".into(),
            refresh_token: "refresh-token".into(),
            id_token: None,
            account_id: Some("account-123".into()),
            last_refresh: Some(OffsetDateTime::now_utc()),
        }
    }

    fn fixture_usage_response() -> CodexUsageResponse {
        serde_json::from_str(include_str!("fixtures/oauth_usage.json")).expect("fixture json")
    }

    #[test]
    fn parse_credentials_reads_oauth_tokens_fixture() {
        let data = include_str!("fixtures/auth.json");
        let credentials = parse_credentials(data).expect("credentials");

        assert_eq!(credentials.access_token, "eyJ-test-access");
        assert_eq!(credentials.refresh_token, "refresh-token");
        assert_eq!(credentials.account_id.as_deref(), Some("account-123"));
        assert!(credentials.last_refresh.is_some());
    }

    #[test]
    fn maps_oauth_usage_fixture_to_snapshot() {
        let response = fixture_usage_response();
        let snapshot =
            snapshot_from_oauth_usage(&response, "2026-05-20T12:00:00Z").expect("snapshot");

        assert_eq!(snapshot.provider, ProviderId::Codex);
        assert_eq!(snapshot.source, "codex-oauth");
        assert_eq!(snapshot.primary.label, "Session");
        assert_eq!(snapshot.primary.used_percent, 22.0);
        let secondary = snapshot.secondary.expect("weekly window");
        assert_eq!(secondary.label, "Weekly");
        assert_eq!(secondary.used_percent, 43.0);
    }

    #[tokio::test]
    async fn oauth_strategy_returns_snapshot_when_token_valid() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockCodexOAuthClient {
            credentials: Ok(fixture_credentials()),
            usage: Ok(fixture_usage_response()),
        }));

        let snapshot = strategy
            .fetch(&FetchContext)
            .await
            .expect("oauth fetch should succeed");

        assert_eq!(snapshot.source, "codex-oauth");
        assert_eq!(snapshot.primary.used_percent, 22.0);
        assert_eq!(snapshot.secondary.expect("secondary").used_percent, 43.0);
    }

    #[tokio::test]
    async fn oauth_strategy_unavailable_without_credentials() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockCodexOAuthClient {
            credentials: Err(ProviderError::NotConfigured),
            usage: Ok(fixture_usage_response()),
        }));

        let available = strategy
            .is_available(&FetchContext)
            .await
            .expect("availability check");

        assert!(!available);
    }

    #[tokio::test]
    async fn oauth_auth_errors_fallback_to_cli() {
        let strategy = OAuthStrategy::with_client(Arc::new(MockCodexOAuthClient {
            credentials: Ok(fixture_credentials()),
            usage: Err(ProviderError::Auth("expired".into())),
        }));

        let error = strategy
            .fetch(&FetchContext)
            .await
            .expect_err("expired token should fail");

        assert!(strategy.should_fallback(&error));
    }
}
