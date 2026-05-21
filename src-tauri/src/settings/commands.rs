use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

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
        let settings = load_settings(&path);

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

#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Result<MochiSettings, String> {
    state.current()
}

#[tauri::command]
pub fn save_settings(
    settings: MochiSettings,
    state: State<'_, SettingsState>,
) -> Result<MochiSettings, String> {
    state.update(settings)
}
