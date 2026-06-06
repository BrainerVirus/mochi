use tauri::WebviewWindow;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LinuxWindowControlDiagnostics {
    pub label: String,
    pub platform: String,
    pub creation_source: String,
    pub decorations_action: String,
    pub decorations_error: Option<String>,
    pub decorations_ok: bool,
    pub resizable_action: String,
    pub resizable_error: Option<String>,
    pub resizable_ok: bool,
}

pub fn prepare_decorated_window(
    window: &WebviewWindow,
    label: &str,
    creation_source: &str,
) -> LinuxWindowControlDiagnostics {
    #[cfg(target_os = "linux")]
    {
        let decorations_result = window.set_decorations(true);
        crate::diagnostics::log_window_action_result(
            label,
            "linux_set_decorations",
            decorations_result.as_ref().map(|_| ()),
        );

        let resizable_result = window.set_resizable(true);
        crate::diagnostics::log_window_action_result(
            label,
            "linux_set_resizable",
            resizable_result.as_ref().map(|_| ()),
        );

        return LinuxWindowControlDiagnostics {
            label: label.to_string(),
            platform: "linux".to_string(),
            creation_source: creation_source.to_string(),
            decorations_action: "linux_set_decorations".to_string(),
            decorations_error: decorations_result.as_ref().err().map(ToString::to_string),
            decorations_ok: decorations_result.is_ok(),
            resizable_action: "linux_set_resizable".to_string(),
            resizable_error: resizable_result.as_ref().err().map(ToString::to_string),
            resizable_ok: resizable_result.is_ok(),
        };
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        LinuxWindowControlDiagnostics {
            label: label.to_string(),
            platform: std::env::consts::OS.to_string(),
            creation_source: creation_source.to_string(),
            decorations_action: "native_decorations_default".to_string(),
            decorations_error: None,
            decorations_ok: true,
            resizable_action: "native_resizable_default".to_string(),
            resizable_error: None,
            resizable_ok: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LinuxWindowControlDiagnostics;

    #[test]
    fn linux_window_control_diagnostics_serializes_actions_and_errors() {
        let diagnostics = LinuxWindowControlDiagnostics {
            label: "widget".to_string(),
            platform: "linux".to_string(),
            creation_source: "rust-builder".to_string(),
            decorations_action: "linux_set_decorations".to_string(),
            decorations_error: Some("decorations failed".to_string()),
            decorations_ok: false,
            resizable_action: "linux_set_resizable".to_string(),
            resizable_error: None,
            resizable_ok: true,
        };

        let value = serde_json::to_value(diagnostics).expect("serialize diagnostics");
        assert_eq!(value["label"], "widget");
        assert_eq!(value["creation_source"], "rust-builder");
        assert_eq!(value["decorations_action"], "linux_set_decorations");
        assert_eq!(value["decorations_error"], "decorations failed");
        assert_eq!(value["resizable_action"], "linux_set_resizable");
        assert_eq!(value["resizable_error"], serde_json::Value::Null);
    }
}
