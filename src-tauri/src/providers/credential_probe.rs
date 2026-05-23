use std::collections::HashMap;

use serde::Serialize;

use crate::core::models::ProviderId;
use crate::core::provider::FetchContext;
use crate::settings::{MochiSettings, ProviderConfig};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredentialStatus {
    pub provider: String,
    pub configured: bool,
}

pub fn credential_status_map(settings: &MochiSettings) -> HashMap<String, bool> {
    let ctx = FetchContext::from_settings(settings);
    ProviderId::all()
        .iter()
        .map(|provider| {
            (
                provider.as_str().to_string(),
                provider_has_credentials(*provider, &ctx),
            )
        })
        .collect()
}

pub fn provider_has_credentials(provider: ProviderId, ctx: &FetchContext) -> bool {
    let config = ctx.config(provider);

    if config.is_some_and(ProviderConfig::cookie_source_is_off) {
        return provider_has_non_cookie_credentials(provider, config);
    }

    match provider {
        ProviderId::Codex => super::codex::has_credentials(config),
        ProviderId::Claude => super::claude::has_credentials(config),
        ProviderId::Cursor => super::cursor::has_credentials(config),
        ProviderId::Gemini => super::gemini::has_credentials(config),
        ProviderId::Copilot => super::copilot::has_credentials(config),
        ProviderId::Zai => {
            config.and_then(ProviderConfig::api_key_value).is_some()
                || std::env::var("Z_AI_API_KEY")
                    .ok()
                    .is_some_and(|value| !value.trim().is_empty())
        }
        ProviderId::Kiro => kiro_cli_available(),
        ProviderId::Antigravity | ProviderId::Factory | ProviderId::Augment => config
            .and_then(ProviderConfig::manual_cookie_value)
            .is_some(),
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
        assert!(provider_has_credentials(ProviderId::Zai, &ctx));
    }

    #[test]
    #[allow(clippy::await_holding_lock)]
    fn cursor_configured_from_settings_cookie() {
        let _guard = test_env::LOCK.lock().expect("env lock");
        std::env::remove_var("MOCHI_CURSOR_COOKIE");
        std::env::remove_var("MOCHI_CURSOR_COOKIE_FILE");

        let mut settings = MochiSettings::default();
        settings.provider_configs.insert(
            "cursor".into(),
            ProviderConfig {
                manual_cookie: Some("WorkosCursorSessionToken=test".into()),
                ..ProviderConfig::default()
            },
        );

        let ctx = FetchContext::from_settings(&settings);
        assert!(provider_has_credentials(ProviderId::Cursor, &ctx));
    }
}
