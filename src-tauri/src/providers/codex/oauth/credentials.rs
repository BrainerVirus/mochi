use std::path::{Path, PathBuf};
use std::time::Duration;

use time::OffsetDateTime;

use crate::core::provider::{ProviderError, ProviderResult};

const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(8 * 24 * 60 * 60);

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

pub fn current_timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_credentials_reads_oauth_tokens_fixture() {
        let data = include_str!("../fixtures/auth.json");
        let credentials = parse_credentials(data).expect("credentials");

        assert_eq!(credentials.access_token, "eyJ-test-access");
        assert_eq!(credentials.refresh_token, "refresh-token");
        assert_eq!(credentials.account_id.as_deref(), Some("account-123"));
        assert!(credentials.last_refresh.is_some());
    }
}
