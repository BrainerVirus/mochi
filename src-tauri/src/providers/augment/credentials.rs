//! Augment session cookie resolution.

use std::path::Path;

use crate::browser::{
    default_import_order, import_cookies, BrowserKind, CookieImportQuery, ImportedCookies,
};
use crate::core::provider::ProviderResult;
use crate::settings::{codexbar_import, ProviderConfig};

const ENV_COOKIE: &str = "MOCHI_AUGMENT_COOKIE";
const SESSION_COOKIE_NAMES: &[&str] = &[
    "_session",
    "auth0",
    "auth0.is.authenticated",
    "a0.spajs.txs",
    "__Secure-next-auth.session-token",
    "next-auth.session-token",
    "__Host-authjs.csrf-token",
    "authjs.session-token",
    "session",
    "web_rpc_proxy_session",
];
const DOMAINS: &[&str] = &["augmentcode.com", "app.augmentcode.com"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAugmentSession {
    pub cookie_header: String,
    pub source_label: String,
}

pub fn resolve_session(
    config: Option<&ProviderConfig>,
) -> ProviderResult<Option<ResolvedAugmentSession>> {
    let config = codexbar_import::merge_codexbar_token_accounts(config, "augment");

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
        return Ok(Some(ResolvedAugmentSession {
            cookie_header: manual.trim().to_string(),
            source_label: "Manual".into(),
        }));
    }

    if let Ok(value) = std::env::var(ENV_COOKIE) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(Some(ResolvedAugmentSession {
                cookie_header: trimmed.to_string(),
                source_label: "Environment".into(),
            }));
        }
    }

    if config
        .as_ref()
        .is_none_or(|cfg| !cfg.cookie_source_is_manual())
    {
        if let Some(home) = user_home_dir() {
            if let Some(imported) = import_browser_cookies(&home) {
                return Ok(Some(ResolvedAugmentSession {
                    cookie_header: imported.cookie_header,
                    source_label: imported.source_label,
                }));
            }
        }
    }

    Ok(None)
}

fn import_browser_cookies(home: &Path) -> Option<ImportedCookies> {
    let browsers = augment_browser_order();
    import_cookies(&CookieImportQuery {
        home,
        browsers: &browsers,
        domains: DOMAINS,
        session_cookie_names: SESSION_COOKIE_NAMES,
        require_session_name: true,
    })
}

fn augment_browser_order() -> Vec<BrowserKind> {
    let preferred = [
        BrowserKind::ChromeBeta,
        BrowserKind::Chrome,
        BrowserKind::ChromeCanary,
        BrowserKind::Arc,
        BrowserKind::Safari,
    ];
    let mut order = preferred.to_vec();
    for browser in default_import_order() {
        if !order.contains(&browser) {
            order.push(browser);
        }
    }
    order
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

pub fn has_credentials(config: Option<&ProviderConfig>) -> ProviderResult<bool> {
    Ok(resolve_session(config)?.is_some() || auggie_cli_available())
}

fn auggie_cli_available() -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let candidate = dir.join(if cfg!(windows) {
                "auggie.exe"
            } else {
                "auggie"
            });
            candidate.is_file()
        })
    })
}

pub(crate) fn resolve_auggie_binary() -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(if cfg!(windows) {
                "auggie.exe"
            } else {
                "auggie"
            });
            candidate.is_file().then_some(candidate)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_manual_cookie_from_settings() {
        let config = ProviderConfig {
            manual_cookie: Some("session=test".into()),
            ..Default::default()
        };
        let resolved = resolve_session(Some(&config)).expect("resolve");
        assert_eq!(
            resolved.expect("session").source_label,
            "Manual".to_string()
        );
    }
}
