use tauri::WebviewWindow;

/// Logical px; keep in sync with macOS `--radius-tray-panel` (`0.875rem` @ 16px) in `src/styles/index.css`.
#[allow(dead_code)]
pub const TRAY_PANEL_CORNER_RADIUS_MACOS: f64 = 14.0;

/// Logical px; keep in sync with Windows `--radius-tray-panel` (`0.6875rem` @ 16px).
#[allow(dead_code)]
pub const TRAY_PANEL_CORNER_RADIUS_WINDOWS: f64 = 11.0;

/// Applies platform-native translucent backdrop to the tray popover window.
pub fn apply_tray_panel_vibrancy(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use tauri::window::{Effect, EffectState, EffectsBuilder};

        window
            .set_effects(
                EffectsBuilder::new()
                    .effect(Effect::Popover)
                    .state(EffectState::Active)
                    .radius(TRAY_PANEL_CORNER_RADIUS_MACOS)
                    .build(),
            )
            .map_err(|error| error.to_string())?;

        let _ = window.set_shadow(true);

        window
            .with_webview(|webview| {
                super::macos_window_shape::clip_window_content_view(
                    webview.ns_window(),
                    TRAY_PANEL_CORNER_RADIUS_MACOS,
                );
            })
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic, apply_mica};

        apply_windows_rounded_corners(window)?;

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

/// Applies platform-native translucent backdrop to settings/about app windows.
pub fn apply_app_window_vibrancy(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use tauri::window::{Effect, EffectState, EffectsBuilder};

        window
            .set_effects(
                EffectsBuilder::new()
                    .effect(Effect::Popover)
                    .state(EffectState::Active)
                    .radius(TRAY_PANEL_CORNER_RADIUS_MACOS)
                    .build(),
            )
            .map_err(|error| error.to_string())?;

        let _ = window.set_shadow(true);

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic, apply_mica};

        apply_windows_rounded_corners(window)?;

        if apply_mica(window, None).is_ok() {
            return Ok(());
        }

        apply_acrylic(window, Some((243, 243, 243, 200))).map_err(|error| error.to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn apply_windows_rounded_corners(window: &WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };

    let hwnd: HWND = window.hwnd().map_err(|error| error.to_string())?.0;
    let preference = DWMWCP_ROUND as u32;

    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &preference as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        )
    };

    if result != 0 {
        return Err(format!(
            "DwmSetWindowAttribute corner preference failed: {result}"
        ));
    }

    Ok(())
}
