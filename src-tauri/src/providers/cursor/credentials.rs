//! Cursor session cookie resolution for web API fetch.
//!
//! Derived from CodexBar `docs/cursor.md` and `CursorStatusProbe.swift` (MIT).

use std::path::Path;

use crate::browser::cursor;
use crate::core::provider::{ProviderError, ProviderResult};
use crate::settings::ProviderConfig;

const ENV_COOKIE: &str = "MOCHI_CURSOR_COOKIE";
const ENV_COOKIE_FILE: &str = "MOCHI_CURSOR_COOKIE_FILE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieCredentialSource {
    ManualSettings,
    ManualEnv,
    Browser(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCookie {
    pub header: String,
    pub source: CookieCredentialSource,
}

impl CookieCredentialSource {
    pub fn label(&self) -> String {
        match self {
            Self::ManualSettings => "Manual".into(),
            Self::ManualEnv => "Environment".into(),
            Self::Browser(label) => format!("Browser: {label}"),
        }
    }
}

pub fn resolve_cookie(config: Option<&ProviderConfig>) -> ProviderResult<Option<ResolvedCookie>> {
    if config.is_some_and(ProviderConfig::cookie_source_is_off) {
        return Ok(None);
    }

    if let Some(manual) = config.and_then(ProviderConfig::manual_cookie_value) {
        return Ok(Some(ResolvedCookie {
            header: normalize_cookie_header(manual),
            source: CookieCredentialSource::ManualSettings,
        }));
    }

    if let Some(env_cookie) = resolve_manual_cookie_from_env()? {
        return Ok(Some(ResolvedCookie {
            header: env_cookie,
            source: CookieCredentialSource::ManualEnv,
        }));
    }

    if config.is_none_or(|cfg| !cfg.cookie_source_is_manual()) {
        if let Some(imported) = import_browser_cookies() {
            return Ok(Some(ResolvedCookie {
                header: imported.cookie_header,
                source: CookieCredentialSource::Browser(imported.source_label),
            }));
        }
    }

    Ok(None)
}

pub fn resolve_manual_cookie_from_env() -> ProviderResult<Option<String>> {
    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(normalize_cookie_header(trimmed)));
        }
    }

    if let Ok(path) = std::env::var(ENV_COOKIE_FILE) {
        return read_cookie_file(Path::new(&path));
    }

    Ok(None)
}

fn import_browser_cookies() -> Option<crate::browser::ImportedCookies> {
    let home = user_home_dir()?;
    cursor::import_from_browsers(&home)
}

fn user_home_dir() -> Option<std::path::PathBuf> {
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

fn read_cookie_file(path: &Path) -> ProviderResult<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }

    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read cursor cookie file: {error}")))?;
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(normalize_cookie_header(trimmed)))
}

pub fn normalize_cookie_header(raw: &str) -> String {
    raw.trim().trim_start_matches("Cookie:").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_env;
    use rusqlite::Connection;
    use std::fs;

    fn write_zen_fixture(home: &Path) {
        let profile = home.join("Library/Application Support/zen/Profiles/abc.default-release");
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
                 VALUES ('.cursor.com', 'WorkosCursorSessionToken', '/', 'zen-session', 0, 1, 1)",
                [],
            )
            .expect("insert");
    }

    #[test]
    fn normalize_cookie_header_strips_prefix() {
        assert_eq!(
            normalize_cookie_header("Cookie: WorkosCursorSessionToken=abc"),
            "WorkosCursorSessionToken=abc"
        );
    }

    #[test]
    #[allow(clippy::await_holding_lock)]
    fn resolve_cookie_prefers_manual_settings_over_browser() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CURSOR_COOKIE");
        std::env::remove_var("MOCHI_CURSOR_COOKIE_FILE");

        let resolved = resolve_cookie(Some(&ProviderConfig {
            manual_cookie: Some("WorkosCursorSessionToken=manual".into()),
            ..Default::default()
        }))
        .expect("resolve")
        .expect("cookie");

        assert_eq!(resolved.source, CookieCredentialSource::ManualSettings);
        assert!(resolved.header.contains("manual"));
    }

    #[test]
    #[allow(clippy::await_holding_lock)]
    fn resolve_cookie_imports_from_zen_browser_fixture() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CURSOR_COOKIE");
        std::env::remove_var("MOCHI_CURSOR_COOKIE_FILE");
        std::env::remove_var("HOME");

        let temp = std::env::temp_dir().join(format!(
            "mochi-cursor-resolve-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        write_zen_fixture(&temp);
        std::env::set_var("HOME", &temp);

        let resolved = resolve_cookie(None).expect("resolve").expect("cookie");

        std::env::remove_var("HOME");
        let _ = fs::remove_dir_all(temp);

        assert!(matches!(
            resolved.source,
            CookieCredentialSource::Browser(_)
        ));
        assert!(resolved
            .header
            .contains("WorkosCursorSessionToken=zen-session"));
    }
}
