mod commands;
mod storage;

pub use commands::{get_settings, save_settings, SettingsState};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateChannel {
    Stable,
    Unstable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MochiSettings {
    pub update_channel: UpdateChannel,
    pub refresh_interval_seconds: u64,
    pub enabled_providers: Vec<String>,
    pub show_notifications: bool,
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
        }
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
