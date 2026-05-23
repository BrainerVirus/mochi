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
    use crate::settings::{TokenAccount, TokenAccountData};

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
}
