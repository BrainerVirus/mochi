//! Factory/Droid session cookie resolution.

use std::path::{Path, PathBuf};

use crate::browser::{default_import_order, import_cookies, CookieImportQuery, ImportedCookies};
use crate::core::provider::ProviderResult;
use crate::settings::{codexbar_import, ProviderConfig};

const ENV_COOKIE: &str = "MOCHI_FACTORY_COOKIE";
const SESSION_COOKIE_NAMES: &[&str] = &[
    "wos-session",
    "__Secure-next-auth.session-token",
    "next-auth.session-token",
    "__Secure-authjs.session-token",
    "__Host-authjs.csrf-token",
    "authjs.session-token",
    "session",
    "access-token",
];
const DOMAINS: &[&str] = &["factory.ai", "app.factory.ai", "auth.factory.ai"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFactorySession {
    pub cookie_header: String,
    pub bearer_token: Option<String>,
    pub source_label: String,
}

pub fn resolve_session(
    config: Option<&ProviderConfig>,
) -> ProviderResult<Option<ResolvedFactorySession>> {
    let config = codexbar_import::merge_codexbar_token_accounts(config, "factory");

    if config
        .as_ref()
        .is_some_and(ProviderConfig::cookie_source_is_off)
    {
        return Ok(None);
    }

    if let Some(manual) = config
        .as_ref()
        .and_then(ProviderConfig::manual_cookie_value)
    {
        return Ok(Some(ResolvedFactorySession {
            cookie_header: manual.trim().to_string(),
            bearer_token: extract_bearer_token(manual),
            source_label: "Manual".into(),
        }));
    }

    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(ResolvedFactorySession {
                cookie_header: trimmed.to_string(),
                bearer_token: extract_bearer_token(trimmed),
                source_label: "Environment".into(),
            }));
        }
    }

    if let Some(session) = load_codexbar_session_file() {
        return Ok(Some(session));
    }

    if config
        .as_ref()
        .is_none_or(|cfg| !cfg.cookie_source_is_manual())
    {
        if let Some(home) = user_home_dir() {
            if let Some(imported) = import_browser_cookies(&home) {
                return Ok(Some(ResolvedFactorySession {
                    cookie_header: imported.cookie_header,
                    bearer_token: None,
                    source_label: imported.source_label,
                }));
            }
        }
    }

    Ok(None)
}

fn import_browser_cookies(home: &Path) -> Option<ImportedCookies> {
    import_cookies(&CookieImportQuery {
        home,
        browsers: &default_import_order(),
        domains: DOMAINS,
        session_cookie_names: SESSION_COOKIE_NAMES,
        require_session_name: true,
    })
}

fn load_codexbar_session_file() -> Option<ResolvedFactorySession> {
    let path = codexbar_session_path()?;
    let contents = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let cookie_header = value
        .get("cookieHeader")
        .or_else(|| value.get("cookie_header"))
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())?
        .to_string();
    let bearer_token = value
        .get("bearerToken")
        .or_else(|| value.get("bearer_token"))
        .and_then(|entry| entry.as_str())
        .map(str::to_string);
    Some(ResolvedFactorySession {
        cookie_header,
        bearer_token,
        source_label: "CodexBar session".into(),
    })
}

fn codexbar_session_path() -> Option<PathBuf> {
    let home = user_home_dir()?;
    #[cfg(target_os = "macos")]
    {
        Some(home.join("Library/Application Support/CodexBar/factory-session.json"))
    }
    #[cfg(target_os = "windows")]
    {
        Some(home.join("AppData/Roaming/CodexBar/factory-session.json"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Some(home.join(".config/codexbar/factory-session.json"))
    }
}

fn extract_bearer_token(cookie_header: &str) -> Option<String> {
    cookie_header.split(';').find_map(|pair| {
        let (name, value) = pair.trim().split_once('=')?;
        if name.trim() == "access-token" {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

fn user_home_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    #[cfg(windows)]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            if !home.trim().is_empty() {
                return Some(PathBuf::from(home));
            }
        }
    }
    None
}

pub fn has_credentials(config: Option<&ProviderConfig>) -> ProviderResult<bool> {
    Ok(resolve_session(config)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_manual_cookie_from_settings() {
        let config = ProviderConfig {
            manual_cookie: Some("session=factory-test".into()),
            ..Default::default()
        };
        let resolved = resolve_session(Some(&config)).expect("resolve");
        assert_eq!(
            resolved.expect("session").source_label,
            "Manual".to_string()
        );
    }
}
