//! Gemini CLI OAuth credential loading from `~/.gemini/`.
//!
//! Derived from CodexBar `docs/gemini.md` and `GeminiStatusProbe.swift` (MIT).

use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::core::provider::{ProviderError, ProviderResult};

const ENV_HOME: &str = "MOCHI_GEMINI_HOME";
const ENV_CREDENTIALS_FILE: &str = "MOCHI_GEMINI_CREDENTIALS_FILE";
const ENV_SETTINGS_FILE: &str = "MOCHI_GEMINI_SETTINGS_FILE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminiAuthType {
    OAuthPersonal,
    ApiKey,
    VertexAi,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeminiOAuthCredentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expiry_date: Option<OffsetDateTime>,
}

impl GeminiOAuthCredentials {
    pub fn needs_refresh(&self) -> bool {
        self.access_token
            .as_ref()
            .is_none_or(|token| token.trim().is_empty())
            || self
                .expiry_date
                .is_some_and(|expiry| OffsetDateTime::now_utc() >= expiry)
    }
}

pub fn gemini_home_dir() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_HOME) {
        return PathBuf::from(path);
    }

    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn credentials_path() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_CREDENTIALS_FILE) {
        return PathBuf::from(path);
    }

    gemini_home_dir().join(".gemini").join("oauth_creds.json")
}

pub fn settings_path() -> PathBuf {
    if let Ok(path) = std::env::var(ENV_SETTINGS_FILE) {
        return PathBuf::from(path);
    }

    gemini_home_dir().join(".gemini").join("settings.json")
}

pub fn current_auth_type() -> GeminiAuthType {
    let path = settings_path();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return GeminiAuthType::Unknown;
    };

    parse_auth_type(&data).unwrap_or(GeminiAuthType::Unknown)
}

pub fn parse_auth_type(data: &str) -> Option<GeminiAuthType> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    let selected = json
        .get("security")
        .and_then(|security| security.get("auth"))
        .and_then(|auth| auth.get("selectedType"))
        .and_then(|value| value.as_str())?;

    Some(match selected {
        "oauth-personal" => GeminiAuthType::OAuthPersonal,
        "api-key" => GeminiAuthType::ApiKey,
        "vertex-ai" => GeminiAuthType::VertexAi,
        _ => GeminiAuthType::Unknown,
    })
}

pub fn load_credentials() -> ProviderResult<GeminiOAuthCredentials> {
    load_credentials_from_path(&credentials_path())
}

pub fn load_credentials_from_path(path: &Path) -> ProviderResult<GeminiOAuthCredentials> {
    if !path.is_file() {
        return Err(ProviderError::NotConfigured);
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read gemini credentials: {error}")))?;
    parse_credentials(&data)
}

pub fn parse_credentials(data: &str) -> ProviderResult<GeminiOAuthCredentials> {
    let json: serde_json::Value =
        serde_json::from_str(data).map_err(|error| ProviderError::Parse(error.to_string()))?;

    let access_token = json
        .get("access_token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let refresh_token = json
        .get("refresh_token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let id_token = json
        .get("id_token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let expiry_date = json
        .get("expiry_date")
        .and_then(|value| value.as_f64())
        .and_then(|millis| OffsetDateTime::from_unix_timestamp((millis / 1000.0) as i64).ok());

    Ok(GeminiOAuthCredentials {
        access_token,
        refresh_token,
        id_token,
        expiry_date,
    })
}

pub fn update_stored_credentials(
    path: &Path,
    refresh_response: &serde_json::Value,
) -> ProviderResult<()> {
    if !path.is_file() {
        return Ok(());
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read gemini credentials: {error}")))?;
    let mut json: serde_json::Value =
        serde_json::from_str(&data).map_err(|error| ProviderError::Parse(error.to_string()))?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| ProviderError::Parse("gemini credentials must be an object".into()))?;

    if let Some(access_token) = refresh_response.get("access_token") {
        object.insert("access_token".into(), access_token.clone());
    }
    if let Some(expires_in) = refresh_response
        .get("expires_in")
        .and_then(|value| value.as_f64())
    {
        let expiry_ms = (OffsetDateTime::now_utc().unix_timestamp() as f64 + expires_in) * 1000.0;
        object.insert(
            "expiry_date".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(expiry_ms)
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
    }
    if let Some(id_token) = refresh_response.get("id_token") {
        object.insert("id_token".into(), id_token.clone());
    }

    let updated = serde_json::to_string_pretty(&json)
        .map_err(|error| ProviderError::Parse(error.to_string()))?;
    std::fs::write(path, updated)
        .map_err(|error| ProviderError::Fetch(format!("write gemini credentials: {error}")))?;

    Ok(())
}

pub fn validate_auth_type(auth_type: GeminiAuthType) -> ProviderResult<()> {
    match auth_type {
        GeminiAuthType::ApiKey => Err(ProviderError::Auth(
            "gemini api-key auth not supported; use Google account (OAuth)".into(),
        )),
        GeminiAuthType::VertexAi => Err(ProviderError::Auth(
            "gemini vertex-ai auth not supported; use Google account (OAuth)".into(),
        )),
        GeminiAuthType::OAuthPersonal | GeminiAuthType::Unknown => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_oauth_credentials_fixture() {
        let creds = parse_credentials(include_str!("fixtures/oauth_creds.json")).expect("parse");
        assert_eq!(creds.access_token.as_deref(), Some("test-access-token"));
        assert_eq!(creds.refresh_token.as_deref(), Some("test-refresh-token"));
        assert!(!creds.needs_refresh());
    }

    #[test]
    fn parses_settings_auth_type_fixture() {
        let auth_type =
            parse_auth_type(include_str!("fixtures/settings_oauth.json")).expect("auth type");
        assert_eq!(auth_type, GeminiAuthType::OAuthPersonal);
    }

    #[test]
    fn rejects_api_key_auth_type() {
        let data = r#"{"security":{"auth":{"selectedType":"api-key"}}}"#;
        let auth_type = parse_auth_type(data).expect("auth type");
        let error = validate_auth_type(auth_type).expect_err("unsupported");
        assert!(matches!(error, ProviderError::Auth(_)));
    }

    #[test]
    fn missing_credentials_file_is_not_configured() {
        let error = load_credentials_from_path(Path::new("/tmp/mochi-missing-gemini-creds.json"))
            .expect_err("missing");
        assert!(matches!(error, ProviderError::NotConfigured));
    }
}
