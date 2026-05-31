use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::models::ProviderId;

pub(crate) mod codexbar_import;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenAccount {
    pub id: String,
    pub label: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountData {
    pub version: u32,
    pub accounts: Vec<TokenAccount>,
    pub active_index: u32,
}

impl TokenAccountData {
    pub fn active_account(&self) -> Option<&TokenAccount> {
        if self.accounts.is_empty() {
            return None;
        }
        let index = self
            .active_index
            .min(self.accounts.len().saturating_sub(1) as u32) as usize;
        Some(&self.accounts[index])
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_accounts: Option<TokenAccountData>,
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

    pub fn workspace_id_value(&self) -> Option<&str> {
        self.workspace_id
            .as_deref()
            .or(self.token_account.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn active_token_account(&self) -> Option<&TokenAccount> {
        self.token_accounts.as_ref()?.active_account()
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
            enabled_providers: Vec::new(),
            show_notifications: true,
            provider_configs: HashMap::new(),
        }
    }
}

impl MochiSettings {
    pub fn provider_config(&self, provider: ProviderId) -> Option<&ProviderConfig> {
        if let Some(config) = self.provider_configs.get(provider.as_str()) {
            return Some(config);
        }

        for alias in provider.config_key_aliases() {
            if let Some(config) = self.provider_configs.get(*alias) {
                return Some(config);
            }
        }

        None
    }

    pub fn normalize_provider_ids(&mut self) {
        self.enabled_providers = self
            .enabled_providers
            .iter()
            .filter_map(|id| ProviderId::parse(id).map(|provider| provider.as_str().to_string()))
            .collect();

        let mut normalized_configs = HashMap::new();
        for (key, config) in &self.provider_configs {
            if let Some(provider) = ProviderId::parse(key) {
                normalized_configs
                    .entry(provider.as_str().to_string())
                    .or_insert_with(|| config.clone());
            }
        }
        self.provider_configs = normalized_configs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_stable_channel() {
        let settings = MochiSettings::default();
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
        assert!(settings.enabled_providers.is_empty());
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

    #[test]
    fn normalize_provider_ids_maps_codexbar_aliases() {
        let mut settings = MochiSettings {
            enabled_providers: vec!["opencodego".into(), "open-code".into()],
            provider_configs: HashMap::from([(
                "opencodego".into(),
                ProviderConfig {
                    manual_cookie: Some("auth=test".into()),
                    ..ProviderConfig::default()
                },
            )]),
            ..MochiSettings::default()
        };

        settings.normalize_provider_ids();

        assert_eq!(
            settings.enabled_providers,
            vec!["opencode-go".to_string(), "opencode".to_string()]
        );
        assert!(settings.provider_config(ProviderId::OpenCodeGo).is_some());
    }

    #[test]
    fn token_account_data_returns_active_account() {
        let data = TokenAccountData {
            version: 1,
            accounts: vec![TokenAccount {
                id: "a".into(),
                label: "zen".into(),
                token: "auth=test".into(),
            }],
            active_index: 0,
        };

        assert_eq!(
            data.active_account().map(|account| account.label.as_str()),
            Some("zen")
        );
    }
}
