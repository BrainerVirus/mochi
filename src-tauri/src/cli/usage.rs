use crate::core::models::ProviderId;
use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};
use crate::tray::provider_display_name;

pub fn format_usage_text(states: &[ProviderUsageState]) -> String {
    if states.is_empty() {
        return "No usage data cached. Enable providers in settings.".to_string();
    }

    states
        .iter()
        .map(|state| match (&state.kind, state.snapshot.as_ref()) {
            (_, Some(snapshot)) => format!(
                "{} {}%{}",
                provider_label(snapshot.provider),
                snapshot.primary.used_percent.round() as u8,
                state
                    .message
                    .as_ref()
                    .map(|message| format!(" ({message})"))
                    .unwrap_or_default()
            ),
            (ProviderUsageStateKind::MissingCredentials, None) => {
                format!("{} credentials missing", provider_label(state.provider))
            }
            (ProviderUsageStateKind::CredentialsNeedRefresh, None) => {
                format!(
                    "{} credentials need refresh",
                    provider_label(state.provider)
                )
            }
            (_, None) => format!(
                "{} {}",
                provider_label(state.provider),
                state.message.as_deref().unwrap_or("no usage data")
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_usage_json(states: &[ProviderUsageState]) -> Result<String, serde_json::Error> {
    serde_json::to_string(states)
}

fn provider_label(provider: ProviderId) -> &'static str {
    provider_display_name(provider)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::core::usage_state::ProviderUsageState;

    fn state() -> ProviderUsageState {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 64.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        );
        ProviderUsageState::fresh(snapshot)
    }

    #[test]
    fn formats_cached_usage_without_refresh() {
        let output = format_usage_text(&[state()]);

        assert!(output.contains("Claude"));
        assert!(output.contains("64%"));
    }

    #[test]
    fn formats_json_from_usage_states() {
        let output = format_usage_json(&[state()]).expect("json");

        assert!(output.contains("\"provider\":\"claude\""));
        assert!(output.contains("\"kind\":\"fresh\""));
    }
}
