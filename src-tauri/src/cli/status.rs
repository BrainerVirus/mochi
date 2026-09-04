use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};
use crate::tray::provider_display_name;

pub fn format_status_text(states: &[ProviderUsageState]) -> String {
    if states.is_empty() {
        return "No providers configured.".to_string();
    }
    states
        .iter()
        .map(|state| match (&state.kind, state.snapshot.as_ref()) {
            (_, Some(snapshot)) => format!(
                "{} {}%{}",
                provider_display_name(snapshot.provider),
                snapshot.primary.used_percent.round() as u8,
                state
                    .message
                    .as_ref()
                    .map(|message| format!(" ({message})"))
                    .unwrap_or_default()
            ),
            (ProviderUsageStateKind::MissingCredentials, None) => {
                format!(
                    "{} credentials missing",
                    provider_display_name(state.provider)
                )
            }
            (_, None) => format!(
                "{} {}",
                provider_display_name(state.provider),
                state.message.as_deref().unwrap_or("no data")
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderHealth, ProviderId, UsageSnapshot, UsageWindow};
    use crate::core::usage_state::{ProviderUsageState, ProviderUsageStateKind};

    fn fresh_claude() -> ProviderUsageState {
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
    fn status_line_shows_label_and_percent() {
        let output = format_status_text(&[fresh_claude()]);
        assert!(output.contains("Claude"));
        assert!(output.contains("64%"));
    }

    #[test]
    fn status_line_marks_missing_credentials() {
        let state = ProviderUsageState {
            provider: ProviderId::Zai,
            kind: ProviderUsageStateKind::MissingCredentials,
            snapshot: None,
            health: ProviderHealth::Error,
            message: None,
            updated_at: "2026-06-04T12:00:00Z".to_string(),
        };
        let output = format_status_text(&[state]);
        assert!(output.contains("credentials missing"));
    }
}
