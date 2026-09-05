use tauri::WebviewUrl;

/// Static React app shell shipped in `dist`.
pub const APP_SHELL_ASSET: &str = "index.html";

pub fn app_shell_url() -> WebviewUrl {
    WebviewUrl::App(APP_SHELL_ASSET.into())
}

/// First-paint URL for a dedicated app window opened at `path` (`/about`,
/// `/update`, `/settings`, `/`). The route rides the fragment, which never
/// leaves the webview, so packaged asset serving still resolves
/// `index.html` while the router boots synchronously at the requested path
/// instead of `/` (tray provider list) + an async pending-route handoff.
pub fn initial_app_url_for_path(path: &str) -> WebviewUrl {
    if path.is_empty() || path == "/" {
        return app_shell_url();
    }
    WebviewUrl::App(format!("{APP_SHELL_ASSET}#{path}").into())
}

#[cfg(test)]
mod tests {
    use super::{app_shell_url, initial_app_url_for_path, APP_SHELL_ASSET};

    #[test]
    fn app_shell_asset_is_index_html() {
        assert_eq!(APP_SHELL_ASSET, "index.html");
    }

    #[test]
    fn initial_app_url_boots_fresh_windows_at_the_requested_path() {
        // Regression: a fresh window opened for /about must not boot at the
        // generic shell URL (which first-paints the tray provider list).
        let about = initial_app_url_for_path("/about").to_string();
        assert_ne!(about, app_shell_url().to_string());
        assert_eq!(about, format!("{APP_SHELL_ASSET}#/about"));

        assert_eq!(
            initial_app_url_for_path("/update").to_string(),
            format!("{APP_SHELL_ASSET}#/update")
        );
        assert_eq!(
            initial_app_url_for_path("/settings").to_string(),
            format!("{APP_SHELL_ASSET}#/settings")
        );
    }

    #[test]
    fn initial_app_url_for_root_is_the_plain_shell_url() {
        assert_eq!(
            initial_app_url_for_path("/").to_string(),
            app_shell_url().to_string()
        );
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
            !conf.contains(r#""label": "widget""#),
            "the widget must be created by the Rust builder, not precreated from tauri.conf.json"
        );
    }

    #[test]
    fn app_window_resize_is_never_gated_by_platform_policy() {
        // Regression (Linux): per-path sizes are dead code when open_app_window
        // skips set_size behind should_mutate_size_before_first_show.
        assert!(
            !include_str!("tray/panel.rs").contains("should_mutate_size_before_first_show"),
            "open_app_window must resize on every open"
        );
    }
}
