//! OpenCode session cookie resolution.
//!
//! Derived from CodexBar `OpenCodeCookieImporter.swift` (MIT).

use crate::browser::opencode;
use crate::core::provider::ProviderResult;
use crate::settings::{codexbar_import, ProviderConfig};

const ENV_COOKIE: &str = "MOCHI_OPENCODE_COOKIE";
const ENV_WORKSPACE_ID: &str = "MOCHI_OPENCODE_WORKSPACE_ID";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOpenCodeSession {
    pub cookie_header: String,
    pub source_label: String,
    pub workspace_id: Option<String>,
}

const REQUEST_COOKIE_NAMES: &[&str] = &["auth", "__Host-auth"];

pub fn request_cookie_header(raw: &str) -> Option<String> {
    let normalized = normalize_cookie_header(raw);
    if normalized.is_empty() {
        return None;
    }

    let pairs: Vec<&str> = normalized
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if pairs.is_empty() {
        return None;
    }

    let filtered: Vec<String> = pairs
        .into_iter()
        .filter_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            if REQUEST_COOKIE_NAMES.contains(&name.trim()) {
                Some(format!("{}={}", name.trim(), value.trim()))
            } else {
                None
            }
        })
        .collect();

    if !filtered.is_empty() {
        return Some(filtered.join("; "));
    }

    Some(normalized)
}

pub fn resolve_session(
    config: Option<&ProviderConfig>,
) -> ProviderResult<Option<ResolvedOpenCodeSession>> {
    let config = codexbar_import::merge_codexbar_token_accounts(config, "opencode");

    if config
        .as_ref()
        .is_some_and(ProviderConfig::cookie_source_is_off)
    {
        return Ok(None);
    }

    let workspace_id = resolve_workspace_id(config.as_ref());

    if let Some(token) = config
        .as_ref()
        .and_then(ProviderConfig::active_token_account)
        .map(|account| account.token.as_str())
    {
        if let Some(cookie_header) = request_cookie_header(token) {
            return Ok(Some(ResolvedOpenCodeSession {
                cookie_header,
                source_label: "Token account".into(),
                workspace_id,
            }));
        }
    }

    if let Some(manual) = config
        .as_ref()
        .and_then(ProviderConfig::manual_cookie_value)
    {
        if let Some(cookie_header) = request_cookie_header(manual) {
            return Ok(Some(ResolvedOpenCodeSession {
                cookie_header,
                source_label: "Manual".into(),
                workspace_id,
            }));
        }
    }

    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if let Some(cookie_header) = request_cookie_header(trimmed) {
            return Ok(Some(ResolvedOpenCodeSession {
                cookie_header,
                source_label: "Environment".into(),
                workspace_id,
            }));
        }
    }

    if config.is_none_or(|cfg| !cfg.cookie_source_is_manual()) {
        if let Some(home) = user_home_dir() {
            if let Some(imported) = opencode::import_from_browsers(&home) {
                if let Some(cookie_header) = request_cookie_header(&imported.cookie_header) {
                    return Ok(Some(ResolvedOpenCodeSession {
                        cookie_header,
                        source_label: imported.source_label,
                        workspace_id,
                    }));
                }
            }
        }
    }

    Ok(None)
}

pub fn resolve_workspace_id(config: Option<&ProviderConfig>) -> Option<String> {
    if let Ok(value) = std::env::var(ENV_WORKSPACE_ID) {
        if let Some(normalized) = normalize_workspace_id(&value) {
            return Some(normalized);
        }
    }

    config
        .and_then(|cfg| cfg.workspace_id_value())
        .and_then(normalize_workspace_id)
}

pub fn normalize_workspace_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with("wrk_") && trimmed.len() > 4 {
        return Some(trimmed.to_string());
    }

    if let Some(index) = trimmed.find("wrk_") {
        let candidate = &trimmed[index..];
        if let Some(end) = candidate.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
            return Some(candidate[..end].to_string());
        }
        return Some(candidate.to_string());
    }

    None
}

fn normalize_cookie_header(raw: &str) -> String {
    raw.trim().trim_start_matches("Cookie:").trim().to_string()
}

pub(crate) fn user_home_dir() -> Option<std::path::PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return Some(std::path::PathBuf::from(home));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            if !home.trim().is_empty() {
                return Some(std::path::PathBuf::from(home));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_env;
    use rusqlite::Connection;
    use std::fs;
    use std::path::Path;

    fn write_zen_fixture(home: &Path) {
        let profile = crate::browser::profiles::gecko_test_profile_dir(
            home,
            crate::browser::BrowserKind::Zen,
            "abc.default-release",
        );
        fs::create_dir_all(&profile).expect("profile dir");
        let db_path = profile.join("cookies.sqlite");
        let connection = Connection::open(&db_path).expect("open fixture db");
        connection
            .execute_batch(
                "CREATE TABLE moz_cookies (
                    host TEXT NOT NULL,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    value TEXT,
                    expiry INTEGER,
                    isSecure INTEGER,
                    isHttpOnly INTEGER
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO moz_cookies (host, name, path, value, expiry, isSecure, isHttpOnly)
                 VALUES ('.opencode.ai', 'auth', '/', 'zen-opencode', 0, 1, 1)",
                [],
            )
            .expect("insert");
    }

    #[test]
    fn normalizes_workspace_id_from_url() {
        assert_eq!(
            normalize_workspace_id("https://opencode.ai/workspace/wrk_abc123/billing"),
            Some("wrk_abc123".into())
        );
    }

    #[test]
    fn request_cookie_header_filters_auth_cookies() {
        assert_eq!(
            request_cookie_header("oc_locale=en; auth=Fe26.test"),
            Some("auth=Fe26.test".into())
        );
    }

    #[test]
    fn request_cookie_header_keeps_full_manual_header() {
        assert_eq!(
            request_cookie_header("auth=only-token"),
            Some("auth=only-token".into())
        );
    }

    #[test]
    #[allow(clippy::await_holding_lock)]
    fn resolve_session_imports_from_zen_browser_fixture() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var(ENV_COOKIE);
        std::env::remove_var(ENV_WORKSPACE_ID);
        std::env::remove_var("HOME");

        let temp = std::env::temp_dir().join(format!(
            "mochi-opencode-zen-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        write_zen_fixture(&temp);
        std::env::set_var("HOME", &temp);

        let session = resolve_session(None).expect("resolve").expect("session");

        std::env::remove_var("HOME");
        let _ = fs::remove_dir_all(temp);

        assert_eq!(session.cookie_header, "auth=zen-opencode");
        assert!(session.source_label.contains("Zen"));
    }
}
