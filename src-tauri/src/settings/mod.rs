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
            enabled_providers: vec!["codex".into(), "claude".into()],
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
}
