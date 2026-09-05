use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::models::ProviderId;

pub(crate) mod codexbar_import;
mod commands;
mod storage;
#[cfg(test)]
mod tests;

pub use commands::{
    get_provider_catalog, get_provider_credential_status, get_settings, save_selected_tab,
    save_settings, SettingsState,
};
pub use storage::{load_settings, save_settings as persist_settings, settings_file_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateChannel {
    #[default]
    Stable,
}

impl UpdateChannel {
    fn deserialize_value<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = Option::<String>::deserialize(deserializer)?;
        Ok(match raw.as_deref() {
            None | Some("stable") | Some("Stable") | Some("unstable") | Some("Unstable") => {
                UpdateChannel::Stable
            }
            Some(other) => {
                return Err(serde::de::Error::custom(format!(
                    "unknown update channel: {other}"
                )))
            }
        })
    }
}

impl<'de> Deserialize<'de> for UpdateChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::deserialize_value(deserializer)
    }
}

impl Serialize for UpdateChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("stable")
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warn_percent: Option<u8>,
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
    /// Kebab-case on disk ("stable"); legacy "unstable" values map to stable.
    #[serde(default, deserialize_with = "UpdateChannel::deserialize_value")]
    pub update_channel: UpdateChannel,
    pub refresh_interval_seconds: u64,
    pub enabled_providers: Vec<String>,
    pub show_notifications: bool,
    #[serde(default = "default_usage_warn_percent")]
    pub usage_warn_percent: u8,
    #[serde(default)]
    pub provider_configs: HashMap<String, ProviderConfig>,
    /// Tray panel / widget selected tab, persisted across windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_tab: Option<String>,
}

impl Default for MochiSettings {
    fn default() -> Self {
        Self {
            update_channel: UpdateChannel::default(),
            refresh_interval_seconds: 300,
            enabled_providers: Vec::new(),
            show_notifications: true,
            usage_warn_percent: default_usage_warn_percent(),
            provider_configs: HashMap::new(),
            selected_tab: None,
        }
    }
}

fn default_usage_warn_percent() -> u8 {
    80
}

pub(crate) fn clamp_warn_percent(value: u8) -> u8 {
    value.clamp(1, 100)
}

pub(crate) fn should_notify_threshold(usage: f64, threshold: u8, armed: bool) -> bool {
    armed && usage >= threshold as f64
}

pub(crate) fn rearmed_below_threshold(usage: f64, threshold: u8) -> bool {
    usage < threshold as f64
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

    /// Provider override else the global default; clamped at read time so
    /// stored out-of-range values degrade gracefully instead of rejecting.
    pub(crate) fn effective_warn_percent(&self, provider: ProviderId) -> u8 {
        let override_value = self
            .provider_config(provider)
            .and_then(|config| config.warn_percent);
        clamp_warn_percent(override_value.unwrap_or(self.usage_warn_percent))
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

        self.selected_tab = self.selected_tab.as_deref().and_then(|tab| {
            if tab == "overview" {
                return Some(tab.to_string());
            }
            ProviderId::parse(tab).map(|provider| provider.as_str().to_string())
        });
    }
}
