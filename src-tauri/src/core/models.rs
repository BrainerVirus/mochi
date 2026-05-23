use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    Codex,
    Claude,
    Cursor,
    Gemini,
    Copilot,
    OpenCode,
    OpenCodeGo,
    Antigravity,
    Factory,
    Zai,
    Kiro,
    Augment,
}

impl ProviderId {
    pub const ALL: [Self; 12] = [
        Self::Codex,
        Self::Claude,
        Self::Cursor,
        Self::Gemini,
        Self::Copilot,
        Self::OpenCode,
        Self::OpenCodeGo,
        Self::Antigravity,
        Self::Factory,
        Self::Zai,
        Self::Kiro,
        Self::Augment,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            "gemini" => Some(Self::Gemini),
            "copilot" => Some(Self::Copilot),
            "opencode" => Some(Self::OpenCode),
            "opencode-go" => Some(Self::OpenCodeGo),
            "antigravity" => Some(Self::Antigravity),
            "factory" => Some(Self::Factory),
            "zai" => Some(Self::Zai),
            "kiro" => Some(Self::Kiro),
            "augment" => Some(Self::Augment),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Gemini => "gemini",
            Self::Copilot => "copilot",
            Self::OpenCode => "opencode",
            Self::OpenCodeGo => "opencode-go",
            Self::Antigravity => "antigravity",
            Self::Factory => "factory",
            Self::Zai => "zai",
            Self::Kiro => "kiro",
            Self::Augment => "augment",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderHealth {
    #[default]
    Ok,
    Stale,
    Error,
}

/// Statuspage.io indicator mapped for provider incident overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StatusIndicator {
    #[default]
    None,
    Minor,
    Major,
    Critical,
    Maintenance,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub indicator: StatusIndicator,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl ProviderStatus {
    pub fn has_incident(&self) -> bool {
        !matches!(
            self.indicator,
            StatusIndicator::None | StatusIndicator::Unknown
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionCostSummary {
    pub window_days: u32,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub session_files_scanned: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchAttempt {
    pub strategy_id: String,
    pub succeeded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub attempted_at: String,
}

/// Provider-specific spend/budget (e.g. Cursor on-demand, OpenCode Go Zen balance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderCostSnapshot {
    pub used: f64,
    pub limit: f64,
    pub currency_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<String>,
}

impl ProviderCostSnapshot {
    pub fn new(
        used: f64,
        limit: f64,
        currency_code: impl Into<String>,
        period: Option<String>,
        resets_at: Option<String>,
    ) -> Self {
        Self {
            used,
            limit,
            currency_code: currency_code.into(),
            period,
            resets_at,
        }
    }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tertiary: Option<UsageWindow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_windows: Vec<UsageWindow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_cost: Option<ProviderCostSnapshot>,
    pub updated_at: String,
    pub source: String,
    #[serde(default)]
    pub health: ProviderHealth,
    #[serde(default)]
    pub is_stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_fetch_attempt: Option<FetchAttempt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_status: Option<ProviderStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_cost: Option<SessionCostSummary>,
}

impl UsageSnapshot {
    pub fn new(
        provider: ProviderId,
        primary: UsageWindow,
        secondary: Option<UsageWindow>,
        updated_at: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            primary,
            secondary,
            tertiary: None,
            extra_windows: Vec::new(),
            provider_cost: None,
            updated_at: updated_at.into(),
            source: source.into(),
            health: ProviderHealth::Ok,
            is_stale: false,
            error: None,
            last_fetch_attempt: None,
            provider_status: None,
            session_cost: None,
        }
    }

    pub fn with_tertiary(mut self, window: UsageWindow) -> Self {
        self.tertiary = Some(window);
        self
    }

    pub fn with_extra_windows(mut self, windows: Vec<UsageWindow>) -> Self {
        self.extra_windows = windows;
        self
    }

    pub fn with_provider_cost(mut self, cost: ProviderCostSnapshot) -> Self {
        self.provider_cost = Some(cost);
        self
    }

    /// Ordered rate-limit windows for UI rendering (primary → secondary → tertiary → extras).
    pub fn rate_windows(&self) -> Vec<&UsageWindow> {
        let mut windows = vec![&self.primary];
        if let Some(secondary) = &self.secondary {
            windows.push(secondary);
        }
        if let Some(tertiary) = &self.tertiary {
            windows.push(tertiary);
        }
        for window in &self.extra_windows {
            windows.push(window);
        }
        windows
    }

    pub fn with_health(mut self, health: ProviderHealth) -> Self {
        self.health = health;
        self
    }

    pub fn mark_stale(mut self, message: impl Into<String>) -> Self {
        self.health = ProviderHealth::Stale;
        self.is_stale = true;
        self.error = Some(message.into());
        self
    }

    pub fn mark_error(mut self, message: impl Into<String>) -> Self {
        self.health = ProviderHealth::Error;
        self.error = Some(message.into());
        self
    }
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

    #[test]
    fn provider_id_parses_kebab_case_values() {
        assert_eq!(ProviderId::parse("factory"), Some(ProviderId::Factory));
        assert_eq!(ProviderId::parse("unknown"), None);
    }

    #[test]
    fn usage_snapshot_defaults_health_to_ok() {
        let json = r#"{"provider":"claude","primary":{"label":"Session","used_percent":0.0,"remaining_percent":100.0,"resets_at":null},"secondary":null,"updated_at":"2026-05-20T00:00:00Z","source":"test"}"#;
        let snapshot: UsageSnapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.health, ProviderHealth::Ok);
        assert!(!snapshot.is_stale);
        assert!(snapshot.error.is_none());
        assert!(snapshot.last_fetch_attempt.is_none());
    }

    #[test]
    fn mark_stale_sets_health_and_error() {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 12.0, None),
            None,
            "2026-05-20T00:00:00Z",
            "claude",
        )
        .mark_stale("network timeout");

        assert_eq!(snapshot.health, ProviderHealth::Stale);
        assert!(snapshot.is_stale);
        assert_eq!(snapshot.error.as_deref(), Some("network timeout"));
    }
}
