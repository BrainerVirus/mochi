use tauri::WebviewWindow;

/// Applies platform-native translucent backdrop to the tray popover window.
pub fn apply_tray_panel_vibrancy(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

        apply_vibrancy(
            window,
            NSVisualEffectMaterial::Popover,
            Some(NSVisualEffectState::Active),
            None,
        )
        .map_err(|error| error.to_string())?;

        let _ = window.set_shadow(true);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic, apply_mica};

        if apply_mica(window, None).is_ok() {
            return Ok(());
        }

        apply_acrylic(window, Some((32, 32, 32, 180))).map_err(|error| error.to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        Ok(())
    }
}
