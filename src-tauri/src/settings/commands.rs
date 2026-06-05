use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::core::models::ProviderId;
use crate::core::provider::FetchContext;
use crate::core::provider_metadata::{
    provider_registry, AuthRequirement, ImplementationStatus, SettingsFieldDefinition,
    SettingsFieldKind, StrategyDefinition,
};
use crate::core::usage_state::ProviderUsageState;
use crate::core::usage_store::{current_timestamp, UsageStore};
use crate::providers::credential_probe::{
    credential_status_map, detected_provider_ids, provider_has_credentials,
};

use super::storage::{load_settings, save_settings as persist_settings, settings_file_path};
use super::MochiSettings;

pub struct SettingsState {
    pub path: PathBuf,
    pub settings: Mutex<MochiSettings>,
}

impl SettingsState {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let base_dir = app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string())?;
        let path = settings_file_path(&base_dir);
        let settings = load_or_initialize_settings(&path)?;

        Ok(Self {
            path,
            settings: Mutex::new(settings),
        })
    }

    pub fn current(&self) -> Result<MochiSettings, String> {
        self.settings
            .lock()
            .map(|settings| settings.clone())
            .map_err(|error| error.to_string())
    }

    pub fn update(&self, next: MochiSettings) -> Result<MochiSettings, String> {
        persist_settings(&self.path, &next)?;
        let mut settings = self.settings.lock().map_err(|error| error.to_string())?;
        *settings = next.clone();
        Ok(next)
    }
}

fn load_or_initialize_settings(path: &std::path::Path) -> Result<MochiSettings, String> {
    if path.exists() {
        return Ok(load_settings(path));
    }

    initialize_missing_settings(path, detected_provider_ids)
}

fn initialize_missing_settings(
    path: &std::path::Path,
    detect_enabled: impl FnOnce(&MochiSettings) -> Vec<String>,
) -> Result<MochiSettings, String> {
    let mut settings = MochiSettings::default();
    settings.enabled_providers = detect_enabled(&settings);
    settings.normalize_provider_ids();
    persist_settings(path, &settings)?;
    Ok(settings)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCatalogEntry {
    pub id: String,
    pub display_name: String,
    pub implementation_status: String,
    pub strategies: Vec<CatalogStrategy>,
    pub auth_requirements: Vec<String>,
    pub settings_fields: Vec<CatalogSettingsField>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStrategy {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSettingsField {
    pub key: String,
    pub label: String,
    pub kind: String,
}

#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Result<MochiSettings, String> {
    state.current()
}

#[tauri::command]
pub fn save_settings(
    settings: MochiSettings,
    state: State<'_, SettingsState>,
    usage_store: State<'_, UsageStore>,
) -> Result<MochiSettings, String> {
    let previous = state.current()?;
    let next = state.update(settings)?;

    for provider in disabled_provider_ids(&previous, &next) {
        usage_store.delete_provider(provider)?;
    }

    let ctx = FetchContext::from_settings(&next);
    for provider in newly_enabled_provider_ids(&previous, &next) {
        let state = if provider_has_credentials(provider, &ctx) {
            ProviderUsageState::fetching(provider, current_timestamp())
        } else {
            ProviderUsageState::missing_credentials(provider, current_timestamp())
        };
        usage_store.put_state(state)?;
    }

    Ok(next)
}

#[tauri::command]
pub fn get_provider_catalog() -> Vec<ProviderCatalogEntry> {
    provider_registry()
        .iter()
        .map(|definition| ProviderCatalogEntry {
            id: definition.id.as_str().to_string(),
            display_name: definition.display_name.to_string(),
            implementation_status: implementation_status_label(definition.implementation_status)
                .to_string(),
            strategies: definition.strategies.iter().map(catalog_strategy).collect(),
            auth_requirements: definition
                .auth_requirements
                .iter()
                .map(|requirement| auth_requirement_label(*requirement).to_string())
                .collect(),
            settings_fields: definition
                .settings_fields
                .iter()
                .map(catalog_settings_field)
                .collect(),
        })
        .collect()
}

#[tauri::command]
pub fn get_provider_credential_status(
    state: State<'_, SettingsState>,
) -> Result<
    std::collections::HashMap<String, crate::providers::credential_probe::ProviderCredentialDetail>,
    String,
> {
    let settings = state.current()?;
    Ok(credential_status_map(&settings))
}

fn catalog_strategy(strategy: &StrategyDefinition) -> CatalogStrategy {
    CatalogStrategy {
        id: strategy.id.to_string(),
        kind: fetch_kind_label(strategy.kind).to_string(),
        label: strategy.label.to_string(),
    }
}

fn catalog_settings_field(field: &SettingsFieldDefinition) -> CatalogSettingsField {
    CatalogSettingsField {
        key: field.key.to_string(),
        label: field.label.to_string(),
        kind: settings_field_kind_label(field.kind).to_string(),
    }
}

fn implementation_status_label(status: ImplementationStatus) -> &'static str {
    match status {
        ImplementationStatus::Stub => "stub",
        ImplementationStatus::Partial => "partial",
        ImplementationStatus::Done => "done",
    }
}

fn fetch_kind_label(kind: crate::core::provider::FetchKind) -> &'static str {
    use crate::core::provider::FetchKind;
    match kind {
        FetchKind::Cli => "cli",
        FetchKind::OAuth => "oauth",
        FetchKind::ApiKey => "api-key",
        FetchKind::BrowserCookies => "browser-cookies",
        FetchKind::LocalConfig => "local-config",
        FetchKind::LocalProbe => "local-probe",
    }
}

fn auth_requirement_label(requirement: AuthRequirement) -> &'static str {
    match requirement {
        AuthRequirement::OAuth => "oauth",
        AuthRequirement::ApiKey => "api-key",
        AuthRequirement::BrowserCookies => "browser-cookies",
        AuthRequirement::CliSession => "cli-session",
        AuthRequirement::LocalProbe => "local-probe",
        AuthRequirement::AdminApiKey => "admin-api-key",
    }
}

fn settings_field_kind_label(kind: SettingsFieldKind) -> &'static str {
    match kind {
        SettingsFieldKind::ApiKey => "api-key",
        SettingsFieldKind::CookieSource => "cookie-source",
        SettingsFieldKind::ManualCookie => "manual-cookie",
        SettingsFieldKind::TokenAccount => "token-account",
        SettingsFieldKind::HistoryWindow => "history-window",
        SettingsFieldKind::RegionHost => "region-host",
    }
}

fn disabled_provider_ids(previous: &MochiSettings, next: &MochiSettings) -> Vec<ProviderId> {
    let next: HashSet<_> = next
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .collect();

    previous
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .filter(|provider| !next.contains(provider))
        .collect()
}

fn newly_enabled_provider_ids(previous: &MochiSettings, next: &MochiSettings) -> Vec<ProviderId> {
    let previous: HashSet<_> = previous
        .enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .collect();

    next.enabled_providers
        .iter()
        .filter_map(|id| ProviderId::parse(id))
        .filter(|provider| !previous.contains(provider))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_missing_settings_with_detected_providers_only() {
        let dir = std::env::temp_dir().join(format!(
            "mochi-settings-init-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let path = super::settings_file_path(&dir);

        let settings = initialize_missing_settings(&path, |_| {
            vec!["codex".to_string(), "opencodego".to_string()]
        })
        .expect("settings should initialize");

        assert_eq!(
            settings.enabled_providers,
            vec!["codex".to_string(), "opencode-go".to_string()]
        );
        assert!(path.is_file());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn disabled_providers_are_detected_from_settings_change() {
        let previous = MochiSettings {
            enabled_providers: vec!["claude".into(), "cursor".into()],
            ..MochiSettings::default()
        };
        let next = MochiSettings {
            enabled_providers: vec!["claude".into()],
            ..MochiSettings::default()
        };

        let disabled = disabled_provider_ids(&previous, &next);

        assert_eq!(disabled, vec![crate::core::models::ProviderId::Cursor]);
    }
}
