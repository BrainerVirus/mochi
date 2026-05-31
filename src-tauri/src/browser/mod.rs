//! Browser cookie auto-import for provider sessions.
//!
//! Ported from CodexBar / SweetCookieKit (MIT). Reads Safari (macOS), Chromium,
//! and Gecko (Firefox, Zen) profiles on macOS, Windows, and Linux.

mod catalog;
mod domains;
mod gecko;
mod import;
pub(crate) mod profiles;

mod keychain;
#[cfg(target_os = "macos")]
mod safari;

mod chromium;

pub use catalog::{cursor_import_order, default_import_order, BrowserKind};
pub use import::{import_cookies, CookieImportQuery, ImportedCookies};

pub mod opencode {
    use super::{import, CookieImportQuery, ImportedCookies};

    pub const DOMAINS: &[&str] = &["opencode.ai", "app.opencode.ai"];

    pub const AUTH_COOKIE_NAMES: &[&str] = &["auth", "__Host-auth"];

    /// Prefer likely OpenCode browsers first, but keep scanning the full catalog.
    pub fn import_order() -> Vec<super::BrowserKind> {
        #[cfg(all(unix, not(target_os = "macos")))]
        let preferred = [
            super::BrowserKind::Zen,
            super::BrowserKind::Firefox,
            super::BrowserKind::Chrome,
            super::BrowserKind::Edge,
            super::BrowserKind::Brave,
            super::BrowserKind::Chromium,
        ];

        #[cfg(not(all(unix, not(target_os = "macos"))))]
        let preferred = [
            super::BrowserKind::Chrome,
            super::BrowserKind::Edge,
            super::BrowserKind::Brave,
            super::BrowserKind::Chromium,
            super::BrowserKind::Zen,
            super::BrowserKind::Firefox,
        ];

        let mut order = preferred.to_vec();
        let remaining: Vec<_> = super::default_import_order()
            .into_iter()
            .filter(|browser| !order.contains(browser))
            .collect();
        order.extend(remaining);
        order
    }

    pub fn import_from_browsers(home: &std::path::Path) -> Option<ImportedCookies> {
        import::import_cookies(&CookieImportQuery {
            home,
            browsers: &import_order(),
            domains: DOMAINS,
            session_cookie_names: AUTH_COOKIE_NAMES,
            require_session_name: true,
        })
    }
}

pub mod cursor {
    use super::{catalog, import, CookieImportQuery, ImportedCookies};

    pub const DOMAINS: &[&str] = &[
        "cursor.com",
        "www.cursor.com",
        "cursor.sh",
        "authenticator.cursor.sh",
    ];

    pub const SESSION_COOKIE_NAMES: &[&str] = &[
        "WorkosCursorSessionToken",
        "__Secure-next-auth.session-token",
        "next-auth.session-token",
        "wos-session",
        "__Secure-wos-session",
        "authjs.session-token",
        "__Secure-authjs.session-token",
    ];

    pub fn import_from_browsers(home: &std::path::Path) -> Option<ImportedCookies> {
        if let Some(imported) = import::import_cookies(&CookieImportQuery {
            home,
            browsers: &catalog::cursor_import_order(),
            domains: DOMAINS,
            session_cookie_names: SESSION_COOKIE_NAMES,
            require_session_name: true,
        }) {
            return Some(imported);
        }

        import::import_cookies(&CookieImportQuery {
            home,
            browsers: &catalog::cursor_import_order(),
            domains: DOMAINS,
            session_cookie_names: SESSION_COOKIE_NAMES,
            require_session_name: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::cursor;

    #[test]
    fn cursor_domains_include_cursor_com_and_sh() {
        assert!(cursor::DOMAINS.contains(&"cursor.com"));
        assert!(cursor::DOMAINS.contains(&"cursor.sh"));
    }

    #[test]
    fn opencode_import_order_scans_full_modern_browser_catalog() {
        let order = super::opencode::import_order();

        assert!(order.contains(&super::BrowserKind::Chrome));
        assert!(order.contains(&super::BrowserKind::Edge));
        assert!(order.contains(&super::BrowserKind::Brave));
        assert!(order.contains(&super::BrowserKind::Chromium));
        assert!(order.contains(&super::BrowserKind::Vivaldi));
        assert!(order.contains(&super::BrowserKind::Firefox));
        assert!(order.contains(&super::BrowserKind::Zen));
    }
}
