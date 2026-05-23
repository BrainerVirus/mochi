use std::collections::HashMap;

use serde::Serialize;

use crate::core::models::ProviderId;
use crate::core::provider::FetchContext;
use crate::settings::{MochiSettings, ProviderConfig};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredentialDetail {
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub fn credential_status_map(
    settings: &MochiSettings,
) -> HashMap<String, ProviderCredentialDetail> {
    let ctx = FetchContext::from_settings(settings);
    ProviderId::all()
        .iter()
        .map(|provider| {
            let detail = provider_credential_detail(*provider, &ctx);
            (provider.as_str().to_string(), detail)
        })
        .collect()
}

pub fn provider_has_credentials(provider: ProviderId, ctx: &FetchContext) -> bool {
    provider_credential_detail(provider, ctx).configured
}

fn provider_credential_detail(
    provider: ProviderId,
    ctx: &FetchContext,
) -> ProviderCredentialDetail {
    let config = ctx.config(provider);

    if config.is_some_and(ProviderConfig::cookie_source_is_off) {
        let configured = provider_has_non_cookie_credentials(provider, config);
        return ProviderCredentialDetail {
            configured,
            source: configured.then_some("Configured".into()),
        };
    }

    match provider {
        ProviderId::Cursor => cursor_credential_detail(config),
        ProviderId::Codex => simple_credential_detail(super::codex::has_credentials(config), None),
        ProviderId::Claude => {
            simple_credential_detail(super::claude::has_credentials(config), None)
        }
        ProviderId::Gemini => {
            simple_credential_detail(super::gemini::has_credentials(config), None)
        }
        ProviderId::Copilot => {
            simple_credential_detail(super::copilot::has_credentials(config), None)
        }
        ProviderId::OpenCode => opencode_credential_detail(config),
        ProviderId::OpenCodeGo => {
            let configured = super::opencodego::has_credentials(config);
            ProviderCredentialDetail {
                configured,
                source: configured.then_some("Browser cookies".into()),
            }
        }
        ProviderId::Zai => {
            let configured = config.and_then(ProviderConfig::api_key_value).is_some()
                || std::env::var("Z_AI_API_KEY")
                    .ok()
                    .is_some_and(|value| !value.trim().is_empty());
            simple_credential_detail(configured, configured.then_some("API key".into()))
        }
        ProviderId::Kiro => {
            let configured = kiro_cli_available();
            simple_credential_detail(configured, configured.then_some("CLI".into()))
        }
        ProviderId::Antigravity | ProviderId::Factory | ProviderId::Augment => {
            let configured = config
                .and_then(ProviderConfig::manual_cookie_value)
                .is_some();
            simple_credential_detail(configured, configured.then_some("Manual cookie".into()))
        }
    }
}

fn opencode_credential_detail(config: Option<&ProviderConfig>) -> ProviderCredentialDetail {
    let configured = super::opencode::has_credentials(config);
    ProviderCredentialDetail {
        configured,
        source: configured.then_some("Browser cookies".into()),
    }
}

fn cursor_credential_detail(config: Option<&ProviderConfig>) -> ProviderCredentialDetail {
    let configured = super::cursor::has_credentials(config);
    ProviderCredentialDetail {
        configured,
        source: if configured {
            super::cursor::credential_source_label(config)
        } else {
            None
        },
    }
}

fn simple_credential_detail(configured: bool, source: Option<String>) -> ProviderCredentialDetail {
    ProviderCredentialDetail {
        configured,
        source: if configured { source } else { None },
    }
}

fn provider_has_non_cookie_credentials(
    provider: ProviderId,
    config: Option<&ProviderConfig>,
) -> bool {
    match provider {
        ProviderId::Claude => super::claude::has_credentials(config),
        ProviderId::Gemini => super::gemini::has_credentials(config),
        ProviderId::Copilot => super::copilot::has_credentials(config),
        ProviderId::Zai => config.and_then(ProviderConfig::api_key_value).is_some(),
        _ => false,
    }
}

fn kiro_cli_available() -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let candidate = dir.join(if cfg!(windows) {
                "kiro-cli.exe"
            } else {
                "kiro-cli"
            });
            candidate.is_file()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_env;
    use rusqlite::Connection;
    use std::fs;

    #[test]
    fn zai_configured_when_api_key_in_settings() {
        let mut settings = MochiSettings::default();
        settings.provider_configs.insert(
            "zai".into(),
            ProviderConfig {
                api_key: Some("test-key".into()),
                ..ProviderConfig::default()
            },
        );

        let ctx = FetchContext::from_settings(&settings);
        let detail = provider_credential_detail(ProviderId::Zai, &ctx);
        assert!(detail.configured);
        assert_eq!(detail.source.as_deref(), Some("API key"));
    }

    #[test]
    fn cursor_configured_from_settings_cookie() {
        let mut settings = MochiSettings::default();
        settings.provider_configs.insert(
            "cursor".into(),
            ProviderConfig {
                manual_cookie: Some("WorkosCursorSessionToken=test".into()),
                ..ProviderConfig::default()
            },
        );

        let ctx = FetchContext::from_settings(&settings);
        let detail = provider_credential_detail(ProviderId::Cursor, &ctx);
        assert!(detail.configured);
        assert_eq!(detail.source.as_deref(), Some("Manual"));
    }

    #[test]
    #[allow(clippy::await_holding_lock)]
    fn cursor_configured_from_zen_browser_fixture() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CURSOR_COOKIE");
        std::env::remove_var("MOCHI_CURSOR_COOKIE_FILE");
        std::env::remove_var("HOME");

        let temp = std::env::temp_dir().join(format!(
            "mochi-cursor-probe-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let profile = temp.join("Library/Application Support/zen/Profiles/abc.default-release");
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

        std::env::set_var("HOME", &temp);
        let detail = provider_credential_detail(ProviderId::Cursor, &FetchContext::empty());
        std::env::remove_var("HOME");
        let _ = fs::remove_dir_all(temp);

        assert!(detail.configured);
        assert!(detail.source.unwrap_or_default().starts_with("Browser:"));
    }
}
