use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::MochiSettings;

pub fn settings_file_path(base_dir: &Path) -> PathBuf {
    base_dir.join("settings.json")
}

pub fn load_settings(path: &Path) -> MochiSettings {
    if !path.exists() {
        return MochiSettings::default();
    }

    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return MochiSettings::default(),
    };

    let mut settings: MochiSettings =
        serde_json::from_str(&contents).unwrap_or_else(|_| MochiSettings::default());
    settings.normalize_provider_ids();
    settings
}

pub fn save_settings(path: &Path, settings: &MochiSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    // Atomic persist: write to a temp file in the same directory, fsync it,
    // then rename over the live path so readers never observe a torn file.
    let tmp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp_path).map_err(|error| error.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    fs::rename(&tmp_path, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::UpdateChannel;

    #[test]
    fn load_settings_returns_defaults_when_file_missing() {
        let dir = std::env::temp_dir().join(format!("mochi-settings-{}", uuid_like()));
        let path = settings_file_path(&dir);

        let settings = load_settings(&path);
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
        assert_eq!(settings.refresh_interval_seconds, 300);
        assert!(settings.enabled_providers.is_empty());
        assert_eq!(settings.usage_warn_percent, 80);
    }

    #[test]
    fn save_and_load_settings_round_trips() {
        let dir = std::env::temp_dir().join(format!("mochi-settings-{}", uuid_like()));
        let path = settings_file_path(&dir);
        let settings = MochiSettings {
            update_channel: UpdateChannel::Stable,
            refresh_interval_seconds: 120,
            enabled_providers: vec!["claude".into(), "cursor".into()],
            show_notifications: false,
            usage_warn_percent: 80,
            provider_configs: Default::default(),
            selected_tab: None,
        };

        save_settings(&path, &settings).expect("settings should save");
        let loaded = load_settings(&path);

        assert_eq!(loaded, settings);
        let _ = fs::remove_dir_all(dir);
    }

    fn uuid_like() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    }
}
