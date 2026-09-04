use std::path::{Path, PathBuf};

use crate::browser::{default_import_order, import_cookies, CookieImportQuery};
use crate::core::provider::{ProviderError, ProviderResult};
use crate::settings::ProviderConfig;

pub const ENV_COOKIE: &str = "MOCHI_COMMANDCODE_COOKIE";
pub const DOMAINS: &[&str] = &["commandcode.ai"];
pub const SESSION_COOKIE_NAMES: &[&str] = &["__Secure-commandcode_prod_.session_token"];

pub fn resolve_session_cookie(config: Option<&ProviderConfig>) -> ProviderResult<Option<String>> {
    if config.is_some_and(ProviderConfig::cookie_source_is_off) {
        return Ok(None);
    }

    if let Some(manual) = config.and_then(ProviderConfig::manual_cookie_value) {
        return session_cookie_from_cookie_header(manual);
    }

    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }

    if config.is_none_or(|cfg| !cfg.cookie_source_is_manual()) {
        if let Some(home) = user_home_dir() {
            if let Some(imported) = import_browser_cookies(&home) {
                return Ok(Some(imported.cookie_header));
            }
        }
    }

    Ok(None)
}

fn import_browser_cookies(home: &Path) -> Option<crate::browser::ImportedCookies> {
    import_cookies(&CookieImportQuery {
        home,
        browsers: &default_import_order(),
        domains: DOMAINS,
        session_cookie_names: SESSION_COOKIE_NAMES,
        require_session_name: true,
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

pub(crate) fn has_credentials(config: Option<&ProviderConfig>) -> ProviderResult<bool> {
    Ok(resolve_session_cookie(config)?.is_some())
}

pub fn session_cookie_from_cookie_header(raw: &str) -> ProviderResult<Option<String>> {
    let trimmed = raw.trim().trim_start_matches("Cookie:").trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    for part in trimmed.split(';') {
        let part = part.trim();
        for name in SESSION_COOKIE_NAMES {
            if let Some(stripped) = part.strip_prefix(name) {
                let Some(value) = stripped.strip_prefix('=') else {
                    continue;
                };
                let value = value.trim();
                if value.is_empty() {
                    return Err(ProviderError::Auth(
                        "commandcode session cookie value is empty".into(),
                    ));
                }
                return Ok(Some(format!("{name}={value}")));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_env;

    #[test]
    fn extracts_session_cookie_from_cookie_header() {
        let cookie = session_cookie_from_cookie_header(
            "__Secure-commandcode_prod_.session_token=abc123; path=/; Secure",
        )
        .expect("parse")
        .expect("cookie");
        assert_eq!(cookie, "__Secure-commandcode_prod_.session_token=abc123");
    }

    #[test]
    fn extracts_session_cookie_from_prefixed_header() {
        let cookie = session_cookie_from_cookie_header(
            "Cookie: other=value; __Secure-commandcode_prod_.session_token=xyz",
        )
        .expect("parse")
        .expect("cookie");
        assert_eq!(cookie, "__Secure-commandcode_prod_.session_token=xyz");
    }

    #[test]
    fn returns_none_without_session_cookie() {
        let cookie = session_cookie_from_cookie_header("other=value; another=1").expect("parse");
        assert!(cookie.is_none());
    }

    #[test]
    fn manual_cookie_takes_precedence() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var(ENV_COOKIE);
        let config = ProviderConfig {
            manual_cookie: Some("__Secure-commandcode_prod_.session_token=manual".into()),
            ..Default::default()
        };
        let resolved = resolve_session_cookie(Some(&config)).expect("resolve");
        assert_eq!(
            resolved.as_deref(),
            Some("__Secure-commandcode_prod_.session_token=manual")
        );
    }

    #[test]
    fn cookie_source_off_disables_resolution() {
        let config = ProviderConfig {
            cookie_source: Some("off".into()),
            ..Default::default()
        };
        let resolved = resolve_session_cookie(Some(&config)).expect("resolve");
        assert!(resolved.is_none());
    }

    #[test]
    fn manual_cookie_without_session_name_is_rejected() {
        let config = ProviderConfig {
            manual_cookie: Some("session=not-commandcode".into()),
            ..Default::default()
        };
        let resolved = resolve_session_cookie(Some(&config)).expect("resolve");
        assert!(resolved.is_none());
    }
}
