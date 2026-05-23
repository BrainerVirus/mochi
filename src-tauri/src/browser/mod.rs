//! Browser cookie auto-import for provider sessions.
//!
//! Ported from CodexBar / SweetCookieKit (MIT). macOS reads Safari, Chromium,
//! and Gecko (Firefox, Zen) profiles; other platforms return `None` until ported.

mod catalog;
mod domains;
mod gecko;
mod import;

#[cfg(target_os = "macos")]
mod keychain;
#[cfg(target_os = "macos")]
mod safari;

mod chromium;

pub use catalog::{cursor_import_order, BrowserKind};
pub use import::{import_cookies, CookieImportQuery, ImportedCookies};

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
}
