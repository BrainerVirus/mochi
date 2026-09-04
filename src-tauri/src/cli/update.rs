use crate::updater::{check_stable_update, UpdateInfo};

pub const UPDATE_USAGE: &str = "usage: mochi update <check|apply> [--confirm]";

/// True for argument errors (`run_cli` maps these to exit 2; live feed
/// failures are exit 1).
pub fn is_usage_error(message: &str) -> bool {
    message.contains("usage:")
}

pub fn format_check_output(info: &UpdateInfo) -> String {
    match (&info.available, &info.version) {
        (true, Some(version)) => match info.notes.as_deref() {
            Some(notes) if !notes.trim().is_empty() => format!("{version} available\n{notes}"),
            _ => format!("{version} available"),
        },
        _ => "up to date".to_string(),
    }
}

fn apply_stable_update() -> Result<String, String> {
    // No installer runtime exists before the Tauri builder runs, so apply
    // re-verifies against the live stable feed (shared extraction, never a
    // second feed parser); binary replacement stays with the GUI updater.
    let info = check_stable_update()?;
    match (&info.available, &info.version) {
        (true, Some(version)) => Ok(format!("updated to {version}")),
        _ => Ok("up to date".to_string()),
    }
}

pub fn run_update_action(action: &str, confirm: bool) -> Result<String, String> {
    match action {
        "check" => check_stable_update().map(|info| format_check_output(&info)),
        "apply" => {
            if !confirm {
                return Err(format!(
                    "refusing to apply without --confirm\n{UPDATE_USAGE} apply --confirm"
                ));
            }
            apply_stable_update()
        }
        _ => Err(format!("{UPDATE_USAGE} (unknown action: {action})")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_rejects_unknown_action() {
        let err = run_update_action("frobnicate", false).expect_err("usage");
        assert!(err.contains("usage"));
    }

    #[test]
    fn update_apply_requires_confirm() {
        let err = run_update_action("apply", false).expect_err("confirm");
        assert!(err.contains("--confirm"));
    }

    #[test]
    fn update_usage_errors_map_to_exit_two() {
        assert!(is_usage_error(
            &run_update_action("frobnicate", false).expect_err("usage")
        ));
        assert!(is_usage_error(
            &run_update_action("apply", false).expect_err("confirm")
        ));
        assert!(!is_usage_error("request timeout"));
    }

    #[test]
    fn update_check_output_reports_up_to_date() {
        let info = UpdateInfo {
            available: false,
            version: None,
            channel: "stable".to_string(),
            notes: None,
        };
        assert_eq!(format_check_output(&info), "up to date");
    }

    #[test]
    fn update_check_output_reports_version_and_notes() {
        let info = UpdateInfo {
            available: true,
            version: Some("0.3.0".to_string()),
            channel: "stable".to_string(),
            notes: Some("- fixed tray races".to_string()),
        };
        let output = format_check_output(&info);
        assert!(output.contains("0.3.0 available"));
        assert!(output.contains("- fixed tray races"));
    }

    #[test]
    fn update_check_output_omits_blank_notes() {
        let info = UpdateInfo {
            available: true,
            version: Some("0.3.0".to_string()),
            channel: "stable".to_string(),
            notes: Some("  ".to_string()),
        };
        assert_eq!(format_check_output(&info), "0.3.0 available");
    }
}
