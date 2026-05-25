use tauri::WebviewUrl;

/// Static SPA shell shipped in `.output/public` (TanStack Start `spa.prerender.outputPath`).
pub const APP_SHELL_ASSET: &str = "index.html";

pub fn app_shell_url() -> WebviewUrl {
    WebviewUrl::App(APP_SHELL_ASSET.into())
}

#[cfg(test)]
mod tests {
    use super::APP_SHELL_ASSET;

    #[test]
    fn app_shell_asset_is_index_html() {
        assert_eq!(APP_SHELL_ASSET, "index.html");
    }

    #[test]
    fn tauri_windows_boot_from_shell_not_deep_routes() {
        let panel = include_str!("tray/panel.rs");
        let widget = include_str!("widget/commands.rs");
        let conf = include_str!("../tauri.conf.json");

        for source in [panel, widget] {
            assert!(
                !source.contains(r#"WebviewUrl::App("/settings"#),
                "settings must not load as a static asset path"
            );
            assert!(
                !source.contains(r#"WebviewUrl::App("/widget"#),
                "widget must not load as a static asset path"
            );
            assert!(
                source.contains("app_shell_url"),
                "window builders must use the shared SPA shell URL"
            );
        }

        assert!(
            !conf.contains(r#""url": "/widget""#),
            "tauri.conf.json must not boot the widget from a deep route"
        );
        assert!(
            conf.contains(r#""url": "index.html""#),
            "tauri.conf.json widget window must boot from the SPA shell"
        );
    }
}
