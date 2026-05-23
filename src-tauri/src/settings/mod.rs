use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::models::ProviderId;

mod commands;
mod storage;

pub use commands::{
    get_provider_catalog, get_provider_credential_status, get_settings, save_settings,
    SettingsState,
};
pub use storage::{load_settings, save_settings as persist_settings, settings_file_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateChannel {
    Stable,
    Unstable,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_cookie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_window_days: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_account: Option<String>,
}

impl ProviderConfig {
    pub fn manual_cookie_value(&self) -> Option<&str> {
        self.manual_cookie
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn api_key_value(&self) -> Option<&str> {
        self.api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn admin_api_key_value(&self) -> Option<&str> {
        self.admin_api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn token_account_value(&self) -> Option<&str> {
        self.token_account
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn cookie_source_is_off(&self) -> bool {
        self.cookie_source
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("off"))
    }

    pub fn cookie_source_is_manual(&self) -> bool {
        self.cookie_source
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("manual"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MochiSettings {
    pub update_channel: UpdateChannel,
    pub refresh_interval_seconds: u64,
    pub enabled_providers: Vec<String>,
    pub show_notifications: bool,
    #[serde(default)]
    pub provider_configs: HashMap<String, ProviderConfig>,
}

impl Default for MochiSettings {
    fn default() -> Self {
        Self {
            update_channel: UpdateChannel::Stable,
            refresh_interval_seconds: 300,
            enabled_providers: vec![
                "codex".into(),
                "claude".into(),
                "cursor".into(),
                "gemini".into(),
                "copilot".into(),
                "antigravity".into(),
                "factory".into(),
                "zai".into(),
                "kiro".into(),
                "augment".into(),
            ],
            show_notifications: true,
            provider_configs: HashMap::new(),
        }
    }
}

impl MochiSettings {
    pub fn provider_config(&self, provider: ProviderId) -> Option<&ProviderConfig> {
        self.provider_configs.get(provider.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_stable_channel() {
        let settings = MochiSettings::default();
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
    }

    #[test]
    fn serializes_update_channel_as_kebab_case() {
        let settings = MochiSettings {
            update_channel: UpdateChannel::Unstable,
            ..MochiSettings::default()
        };
        let json = serde_json::to_string(&settings).expect("settings should serialize");
        assert!(json.contains("\"update_channel\":\"unstable\""));
    }
}
