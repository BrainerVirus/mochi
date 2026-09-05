//! First-paint background colors for window/webview creation.
//!
//! Linux has no native vibrancy/Mica; windows are opaque. Setting the native
//! window + WebKit background color at creation removes the white flash and
//! the "wrong color then correct color" sequence before the webview CSS
//! loads. macOS/Windows keep their native glass flow and never call this.

use tauri::webview::Color;
#[cfg(target_os = "linux")]
use tauri::{Manager, Runtime, WebviewWindowBuilder};

/// CSS light shell background (`--background` on `[data-platform="linux"]`).
pub const SHELL_BG_LIGHT: (u8, u8, u8, u8) = (0xFA, 0xFA, 0xFA, 0xFF);
/// CSS dark shell background.
pub const SHELL_BG_DARK: (u8, u8, u8, u8) = (0x24, 0x24, 0x24, 0xFF);

/// Case-insensitive theme-name fragments that imply a dark theme without
/// containing "dark" (e.g. Adapta-Nokto — "nokto" is night in Esperanto).
/// Custom setups can still diverge; see docs/qa/linux-ubuntu-evidence.md.
const DARK_THEME_HINTS: &[&str] = &["dark", "nokto", "night"];

/// Decides the shell background from the GTK theme name. GTK is unavailable
/// outside Linux builds, so the theme is injected as a plain string.
pub fn shell_background_from_theme(theme_name: Option<&str>) -> Color {
    let is_dark = theme_name.is_some_and(|name| {
        let lower = name.to_lowercase();
        DARK_THEME_HINTS.iter().any(|hint| lower.contains(hint))
    });
    let (red, green, blue, alpha) = if is_dark {
        SHELL_BG_DARK
    } else {
        SHELL_BG_LIGHT
    };
    Color(red, green, blue, alpha)
}

/// Applies the opaque shell background to a decorated/panel window builder.
/// Callers on macOS/Windows must not invoke this (native transparency there).
#[cfg(target_os = "linux")]
pub fn apply_shell_background<'a, R, M>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M>
where
    R: Runtime,
    M: Manager<R>,
{
    builder.background_color(theme_background_color())
}

#[cfg(target_os = "linux")]
fn theme_background_color() -> Color {
    use gtk::prelude::*;

    let settings = gtk::Settings::default();
    // An explicit desktop-wide dark preference wins over the theme-name
    // heuristic (covers dark modes whose theme name looks light).
    let prefer_dark = settings
        .as_ref()
        .map(|settings| settings.property::<bool>("gtk-application-prefer-dark-theme"))
        .unwrap_or(false);
    if prefer_dark {
        let (red, green, blue, alpha) = SHELL_BG_DARK;
        return Color(red, green, blue, alpha);
    }
    let theme_name = settings
        .and_then(|settings| settings.gtk_theme_name())
        .map(|name| name.to_string());
    shell_background_from_theme(theme_name.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_theme_resolves_light_background() {
        assert_eq!(
            shell_background_from_theme(Some("Yaru")),
            Color(0xFA, 0xFA, 0xFA, 0xFF)
        );
        assert_eq!(
            shell_background_from_theme(None),
            Color(0xFA, 0xFA, 0xFA, 0xFF)
        );
    }

    #[test]
    fn dark_theme_name_resolves_dark_background() {
        assert_eq!(
            shell_background_from_theme(Some("Yaru-dark")),
            Color(0x24, 0x24, 0x24, 0xFF)
        );
        assert_eq!(
            shell_background_from_theme(Some("Adwaita-dark")),
            Color(0x24, 0x24, 0x24, 0xFF)
        );
    }

    #[test]
    fn dark_detection_is_case_insensitive() {
        assert_eq!(
            shell_background_from_theme(Some("YARU-DARK")),
            Color(0x24, 0x24, 0x24, 0xFF)
        );
    }

    #[test]
    fn dark_theme_names_without_dark_resolve_dark_background() {
        assert_eq!(
            shell_background_from_theme(Some("Adapta-Nokto")),
            Color(0x24, 0x24, 0x24, 0xFF)
        );
    }
}
