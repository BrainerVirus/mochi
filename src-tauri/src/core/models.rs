use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    Codex,
    Claude,
    Cursor,
    Gemini,
    Copilot,
    Antigravity,
    Factory,
    Zai,
    Kiro,
    Augment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f32,
    pub remaining_percent: f32,
    pub resets_at: Option<String>,
}

impl UsageWindow {
    pub fn new(label: impl Into<String>, used_percent: f32, resets_at: Option<String>) -> Self {
        let used_percent = used_percent.clamp(0.0, 100.0);
        Self {
            label: label.into(),
            used_percent,
            remaining_percent: 100.0 - used_percent,
            resets_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub provider: ProviderId,
    pub primary: UsageWindow,
    pub secondary: Option<UsageWindow>,
    pub updated_at: String,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_window_clamps_overflowing_values() {
        let window = UsageWindow::new("Session", 140.0, None);
        assert_eq!(window.used_percent, 100.0);
        assert_eq!(window.remaining_percent, 0.0);
    }

    #[test]
    fn usage_window_clamps_negative_values() {
        let window = UsageWindow::new("Weekly", -8.0, None);
        assert_eq!(window.used_percent, 0.0);
        assert_eq!(window.remaining_percent, 100.0);
    }
}
