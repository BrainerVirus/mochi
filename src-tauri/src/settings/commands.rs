use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

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

    pub fn update(&self, mut next: MochiSettings) -> Result<MochiSettings, String> {
        let mut settings = self.settings.lock().map_err(|error| error.to_string())?;
        next.selected_tab = settings.selected_tab.clone();
        next.normalize_provider_ids();
        persist_settings(&self.path, &next)?;
        *settings = next.clone();
        Ok(next)
    }

    pub fn update_selected_tab(&self, selected_tab: String) -> Result<MochiSettings, String> {
        let mut settings = self.settings.lock().map_err(|error| error.to_string())?;
        let mut next = settings.clone();
        next.selected_tab = Some(selected_tab);
        next.normalize_provider_ids();
        persist_settings(&self.path, &next)?;
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
    app: AppHandle,
    settings: MochiSettings,
    state: State<'_, SettingsState>,
    usage_store: State<'_, UsageStore>,
) -> Result<MochiSettings, String> {
    let previous = state.current()?;
    let next = state.update(settings)?;

    reconcile_usage_store_for_settings_change(&previous, &next, &usage_store)?;
    broadcast_settings_changed(&app, &next);
    spawn_refresh_for_newly_enabled(app, &previous, &next);

    Ok(next)
}

fn spawn_refresh_for_newly_enabled(app: AppHandle, previous: &MochiSettings, next: &MochiSettings) {
    let ctx = FetchContext::from_settings(next);
    let providers: Vec<ProviderId> = newly_enabled_provider_ids(previous, next)
        .into_iter()
        .filter(|provider| provider_has_credentials(*provider, &ctx))
        .collect();

    if providers.is_empty() {
        return;
    }

    tauri::async_runtime::spawn(async move {
        let Some(store) = app.try_state::<UsageStore>() else {
            return;
        };
        let Some(settings_state) = app.try_state::<SettingsState>() else {
            return;
        };
        let Ok(settings) = settings_state.current() else {
            return;
        };

        for provider in providers {
            let _ =
                crate::status::refresh_single_provider_inner(&store, &settings, provider.as_str())
                    .await;
        }

        let Ok(settings) = settings_state.current() else {
            return;
        };
        let payload = crate::status::RefreshCompletePayload {
            states: crate::status::read_cached_usage_states(&store, &settings),
        };
        let _ = app.emit("usage-refresh-complete", &payload);
    });
}

fn reconcile_usage_store_for_settings_change(
    previous: &MochiSettings,
    next: &MochiSettings,
    usage_store: &UsageStore,
) -> Result<(), String> {
    for provider in disabled_provider_ids(previous, next) {
        usage_store.delete_provider(provider)?;
    }

    let ctx = FetchContext::from_settings(next);
    for provider in newly_enabled_provider_ids(previous, next) {
        let state = if provider_has_credentials(provider, &ctx) {
            ProviderUsageState::fetching(provider, current_timestamp())
        } else {
            ProviderUsageState::missing_credentials(provider, current_timestamp())
        };
        usage_store.put_state(state)?;
    }

    Ok(())
}

fn broadcast_settings_changed(app: &AppHandle, settings: &MochiSettings) {
    if emit_settings_changed(app, settings).is_ok() {
        return;
    }

    crate::diagnostics::log_line("settings", "settings-changed emit failed; retrying once");
    if let Err(retry_error) = emit_settings_changed(app, settings) {
        crate::diagnostics::log_line(
            "settings",
            &format!("settings-changed emit retry failed: {retry_error}"),
        );
    }
}

const SETTINGS_CHANGED_EVENT: &str = "settings-changed";

fn emit_settings_changed(app: &AppHandle, settings: &MochiSettings) -> Result<(), String> {
    app.emit(SETTINGS_CHANGED_EVENT, settings)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_selected_tab(
    selected_tab: String,
    state: State<'_, SettingsState>,
) -> Result<MochiSettings, String> {
    state.update_selected_tab(selected_tab)
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
    use crate::core::usage_state::ProviderUsageStateKind;
    use crate::status::refresh_single_provider_inner;

    #[tokio::test]
    async fn refresh_single_provider_gemini_invalid_token_records_error_without_panicking() {
        let dir = std::env::temp_dir().join(format!(
            "mochi-gemini-invalid-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let creds_path = dir.join("oauth_creds.json");
        std::fs::write(
            &creds_path,
            r#"{"refresh_token":"invalid-token","access_token":""}"#,
        )
        .expect("write creds");

        std::env::set_var("MOCHI_GEMINI_CREDENTIALS_FILE", &creds_path);
        std::env::set_var("MOCHI_GEMINI_OAUTH_CLIENT_ID", "test-client-id");
        std::env::set_var("MOCHI_GEMINI_OAUTH_CLIENT_SECRET", "test-client-secret");

        let store = UsageStore::new(None);
        let settings = MochiSettings {
            enabled_providers: vec!["gemini".into()],
            ..MochiSettings::default()
        };

        let payload = refresh_single_provider_inner(&store, &settings, "gemini")
            .await
            .expect("invalid gemini credentials should return usage payload, not crash");

        let gemini_state = payload
            .states
            .iter()
            .find(|state| state.provider == ProviderId::Gemini)
            .expect("gemini state");
        assert!(
            matches!(
                gemini_state.kind,
                ProviderUsageStateKind::Error
                    | ProviderUsageStateKind::StaleError
                    | ProviderUsageStateKind::CredentialsNeedRefresh
            ),
            "expected auth failure state, got {:?}",
            gemini_state.kind
        );

        std::env::remove_var("MOCHI_GEMINI_CREDENTIALS_FILE");
        std::env::remove_var("MOCHI_GEMINI_OAUTH_CLIENT_ID");
        std::env::remove_var("MOCHI_GEMINI_OAUTH_CLIENT_SECRET");
        let _ = std::fs::remove_dir_all(dir);
    }

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

    #[test]
    fn settings_changed_event_name_matches_frontend_listener() {
        assert_eq!(SETTINGS_CHANGED_EVENT, "settings-changed");
    }

    #[test]
    fn reconcile_usage_store_removes_disabled_and_seeds_newly_enabled() {
        let store = UsageStore::new(None);
        store
            .put_state(ProviderUsageState::fetching(
                ProviderId::Cursor,
                current_timestamp(),
            ))
            .expect("cursor state");
        store
            .put_state(ProviderUsageState::fetching(
                ProviderId::Gemini,
                current_timestamp(),
            ))
            .expect("gemini state");

        let previous = MochiSettings {
            enabled_providers: vec!["cursor".into(), "gemini".into()],
            ..MochiSettings::default()
        };
        let next = MochiSettings {
            enabled_providers: vec!["cursor".into(), "codex".into()],
            ..MochiSettings::default()
        };

        reconcile_usage_store_for_settings_change(&previous, &next, &store)
            .expect("usage store should reconcile");

        assert!(store.get_states(&["gemini".into()]).is_empty());
        assert_eq!(store.get_states(&["cursor".into()]).len(), 1);

        let codex_state = store
            .get_states(&["codex".into()])
            .into_iter()
            .next()
            .expect("codex state");
        assert_eq!(codex_state.provider, ProviderId::Codex);
        assert!(matches!(
            codex_state.kind,
            crate::core::usage_state::ProviderUsageStateKind::MissingCredentials
                | crate::core::usage_state::ProviderUsageStateKind::Fetching
        ));
    }

    #[test]
    fn reconcile_usage_store_clears_all_disabled_providers() {
        let store = UsageStore::new(None);
        store
            .put_state(ProviderUsageState::fetching(
                ProviderId::Cursor,
                current_timestamp(),
            ))
            .expect("cursor state");
        store
            .put_state(ProviderUsageState::fetching(
                ProviderId::Gemini,
                current_timestamp(),
            ))
            .expect("gemini state");

        let previous = MochiSettings {
            enabled_providers: vec!["cursor".into(), "gemini".into()],
            ..MochiSettings::default()
        };
        let next = MochiSettings {
            enabled_providers: vec![],
            ..MochiSettings::default()
        };

        reconcile_usage_store_for_settings_change(&previous, &next, &store)
            .expect("usage store should reconcile");

        assert!(store
            .get_states(&["cursor".into(), "gemini".into()])
            .is_empty());
    }

    #[test]
    fn settings_update_preserves_current_selected_tab() {
        let dir = std::env::temp_dir().join(format!(
            "mochi-settings-update-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let state = SettingsState {
            path: settings_file_path(&dir),
            settings: Mutex::new(MochiSettings {
                selected_tab: Some("codex".into()),
                ..MochiSettings::default()
            }),
        };

        let updated = state
            .update(MochiSettings {
                show_notifications: false,
                selected_tab: Some("cursor".into()),
                ..MochiSettings::default()
            })
            .expect("settings should update");

        assert_eq!(
            updated,
            MochiSettings {
                show_notifications: false,
                selected_tab: Some("codex".into()),
                ..MochiSettings::default()
            }
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn selected_tab_update_preserves_unrelated_settings() {
        let dir = std::env::temp_dir().join(format!(
            "mochi-selected-tab-update-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let state = SettingsState {
            path: settings_file_path(&dir),
            settings: Mutex::new(MochiSettings {
                show_notifications: false,
                ..MochiSettings::default()
            }),
        };

        let updated = state
            .update_selected_tab("opencodego".into())
            .expect("selected tab should update");

        assert_eq!(
            updated,
            MochiSettings {
                show_notifications: false,
                selected_tab: Some("opencode-go".into()),
                ..MochiSettings::default()
            }
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
