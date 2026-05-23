//! GitHub OAuth token resolution for Copilot usage fetch.
//!
//! Derived from CodexBar `docs/copilot.md` (MIT). Device-flow login is out of scope;
//! tokens are supplied via env or credentials file until UI auth lands.

use std::path::Path;

use crate::core::provider::{ProviderError, ProviderResult};
use crate::settings::ProviderConfig;

const ENV_TOKEN: &str = "MOCHI_COPILOT_TOKEN";
const ENV_TOKEN_FILE: &str = "MOCHI_COPILOT_TOKEN_FILE";
const ENV_ENTERPRISE_HOST: &str = "MOCHI_COPILOT_ENTERPRISE_HOST";

pub fn resolve_token(config: Option<&ProviderConfig>) -> ProviderResult<Option<String>> {
    if let Some(token) = config.and_then(ProviderConfig::token_account_value) {
        return Ok(Some(token.to_string()));
    }

    resolve_token_from_env()
}

pub fn resolve_token_from_env() -> ProviderResult<Option<String>> {
    if let Ok(value) = std::env::var(ENV_TOKEN) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }

    if let Ok(path) = std::env::var(ENV_TOKEN_FILE) {
        return read_token_file(Path::new(&path));
    }

    Ok(None)
}

pub fn resolve_enterprise_host() -> Option<String> {
    std::env::var(ENV_ENTERPRISE_HOST)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn api_host(enterprise_host: Option<&str>) -> String {
    let host = normalize_host(enterprise_host);
    if host == "github.com" {
        return "api.github.com".to_string();
    }
    if host.starts_with("api.") {
        return host;
    }
    format!("api.{host}")
}

pub fn usage_url(enterprise_host: Option<&str>) -> String {
    format!(
        "https://{}/copilot_internal/user",
        api_host(enterprise_host)
    )
}

fn normalize_host(raw: Option<&str>) -> String {
    let Some(raw) = raw else {
        return "github.com".to_string();
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "github.com".to_string();
    }

    let without_scheme = trimmed
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let without_login = without_scheme
        .strip_prefix("login/")
        .unwrap_or(without_scheme);
    without_login
        .split('/')
        .next()
        .unwrap_or("github.com")
        .to_string()
}

fn read_token_file(path: &Path) -> ProviderResult<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read copilot token file: {error}")))?;
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(trimmed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_host_defaults_to_github() {
        assert_eq!(api_host(None), "api.github.com");
    }

    #[test]
    fn api_host_supports_enterprise_host() {
        assert_eq!(api_host(Some("octocorp.ghe.com")), "api.octocorp.ghe.com");
        assert_eq!(
            api_host(Some("https://octocorp.ghe.com/login")),
            "api.octocorp.ghe.com"
        );
    }

    #[test]
    fn usage_url_uses_copilot_internal_path() {
        assert_eq!(
            usage_url(None),
            "https://api.github.com/copilot_internal/user"
        );
    }
}
