use tauri::WebviewWindow;

pub fn prepare_decorated_window(window: &WebviewWindow, label: &str) {
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
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        let _ = label;
    }
}
