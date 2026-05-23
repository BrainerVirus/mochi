//! Claude OAuth credential loading from Claude Code `~/.claude/.credentials.json`.
//!
//! Derived from CodexBar `ClaudeOAuthCredentialModels` (MIT).

use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::core::provider::{ProviderError, ProviderResult};

const ENV_ACCESS_TOKEN: &str = "MOCHI_CLAUDE_OAUTH_TOKEN";
const ENV_CREDENTIALS_FILE: &str = "MOCHI_CLAUDE_CREDENTIALS_FILE";

#[derive(Debug, Clone, PartialEq)]
pub struct ClaudeOAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<OffsetDateTime>,
    pub scopes: Vec<String>,
}

impl ClaudeOAuthCredentials {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => OffsetDateTime::now_utc() >= expires,
            None => true,
        }
    }

    pub fn has_user_profile_scope(&self) -> bool {
        self.scopes.iter().any(|scope| scope == "user:profile")
    }
}

pub fn claude_credentials_path() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_CREDENTIALS_FILE) {
        return PathBuf::from(path);
    }

    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| {
            PathBuf::from(home)
                .join(".claude")
                .join(".credentials.json")
        })
        .unwrap_or_else(|| PathBuf::from(".claude/.credentials.json"))
}

pub fn load_credentials() -> ProviderResult<ClaudeOAuthCredentials> {
    if let Ok(token) = std::env::var(ENV_ACCESS_TOKEN) {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            return Ok(ClaudeOAuthCredentials {
                access_token: trimmed.to_string(),
                refresh_token: None,
                expires_at: None,
                scopes: vec!["user:profile".into()],
            });
        }
    }

    load_credentials_from_path(&claude_credentials_path())
}

pub fn load_credentials_from_path(path: &Path) -> ProviderResult<ClaudeOAuthCredentials> {
    if !path.is_file() {
        return Err(ProviderError::NotConfigured);
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read claude credentials: {error}")))?;
    parse_credentials(&data)
}

pub fn parse_credentials(data: &str) -> ProviderResult<ClaudeOAuthCredentials> {
    let json: serde_json::Value =
        serde_json::from_str(data).map_err(|error| ProviderError::Parse(error.to_string()))?;

    let oauth = json
        .get("claudeAiOauth")
        .ok_or_else(|| ProviderError::Auth("claude credentials missing claudeAiOauth".into()))?;

    let access_token = oauth
        .get("accessToken")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ProviderError::Auth("claude credentials missing access token".into()))?;

    let refresh_token = oauth
        .get("refreshToken")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let expires_at = oauth
        .get("expiresAt")
        .and_then(|value| value.as_f64())
        .and_then(|millis| OffsetDateTime::from_unix_timestamp((millis / 1000.0) as i64).ok());

    let scopes = oauth
        .get("scopes")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(ClaudeOAuthCredentials {
        access_token: access_token.to_string(),
        refresh_token,
        expires_at,
        scopes,
    })
}

pub fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_credentials_fixture() {
        let creds = parse_credentials(include_str!("../fixtures/credentials.json")).expect("parse");
        assert_eq!(creds.access_token, "test-access-token");
        assert_eq!(creds.refresh_token.as_deref(), Some("test-refresh-token"));
        assert!(creds.has_user_profile_scope());
        assert!(!creds.is_expired());
    }

    #[test]
    fn missing_oauth_block_is_auth_error() {
        let error = parse_credentials(r#"{ "other": {} }"#).expect_err("missing oauth");
        assert!(matches!(error, ProviderError::Auth(_)));
    }
}
