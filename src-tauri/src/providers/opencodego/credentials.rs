//! OpenCode Go session cookie resolution.
//!
//! Derived from CodexBar `OpenCodeGoSettingsStore.swift` and
//! `OpenCodeWebCookieSupport.swift` (MIT).

use crate::browser::opencode;
use crate::core::provider::ProviderResult;
use crate::providers::opencode::credentials::{
    request_cookie_header, user_home_dir, ResolvedOpenCodeSession,
};
use crate::settings::{codexbar_import, ProviderConfig};

const ENV_COOKIE: &str = "MOCHI_OPENCODE_GO_COOKIE";
const ENV_COOKIE_LEGACY: &str = "MOCHI_OPENCODEGO_COOKIE";
const ENV_WORKSPACE_ID: &str = "MOCHI_OPENCODE_GO_WORKSPACE_ID";
const ENV_WORKSPACE_ID_LEGACY: &str = "MOCHI_OPENCODE_WORKSPACE_ID";

pub fn resolve_session(
    config: Option<&ProviderConfig>,
) -> ProviderResult<Option<ResolvedOpenCodeSession>> {
    let config = codexbar_import::merge_codexbar_token_accounts(config, "opencodego");

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

    for key in [ENV_COOKIE, ENV_COOKIE_LEGACY] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if let Some(cookie_header) = request_cookie_header(trimmed) {
                return Ok(Some(ResolvedOpenCodeSession {
                    cookie_header,
                    source_label: "Environment".into(),
                    workspace_id,
                }));
            }
        }
    }

    if config
        .as_ref()
        .is_none_or(|cfg| !cfg.cookie_source_is_manual())
    {
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
    for key in [ENV_WORKSPACE_ID, ENV_WORKSPACE_ID_LEGACY] {
        if let Ok(value) = std::env::var(key) {
            if let Some(normalized) =
                crate::providers::opencode::credentials::normalize_workspace_id(&value)
            {
                return Some(normalized);
            }
        }
    }

    config
        .and_then(ProviderConfig::workspace_id_value)
        .and_then(crate::providers::opencode::credentials::normalize_workspace_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_env;
    use crate::settings::{TokenAccount, TokenAccountData};
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
                 VALUES ('.opencode.ai', 'auth', '/', 'zen-opencode-go', 0, 1, 1)",
                [],
            )
            .expect("insert");
    }

    #[test]
    fn resolves_active_token_account_cookie() {
        let config = ProviderConfig {
            cookie_source: Some("manual".into()),
            token_accounts: Some(TokenAccountData {
                version: 1,
                accounts: vec![TokenAccount {
                    id: "acc-1".into(),
                    label: "zen".into(),
                    token: "oc_locale=en; auth=Fe26.test".into(),
                }],
                active_index: 0,
            }),
            ..ProviderConfig::default()
        };

        let session = resolve_session(Some(&config))
            .expect("resolve")
            .expect("session");
        assert!(session.cookie_header.contains("auth=Fe26.test"));
        assert_eq!(session.source_label, "Token account");
    }

    #[test]
    fn normalizes_manual_cookie_header() {
        let config = ProviderConfig {
            manual_cookie: Some("Cookie: auth=manual-token".into()),
            ..ProviderConfig::default()
        };

        let session = resolve_session(Some(&config))
            .expect("resolve")
            .expect("session");
        assert_eq!(session.cookie_header, "auth=manual-token");
    }

    #[test]
    #[allow(clippy::await_holding_lock)]
    fn resolve_session_imports_from_zen_browser_fixture() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        for key in [
            ENV_COOKIE,
            ENV_COOKIE_LEGACY,
            ENV_WORKSPACE_ID,
            ENV_WORKSPACE_ID_LEGACY,
        ] {
            std::env::remove_var(key);
        }
        std::env::remove_var("HOME");

        let temp = std::env::temp_dir().join(format!(
            "mochi-opencode-go-zen-{}",
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

        assert_eq!(session.cookie_header, "auth=zen-opencode-go");
        assert!(session.source_label.contains("Zen"));
    }
}
