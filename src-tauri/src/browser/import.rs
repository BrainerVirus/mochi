//! Browser cookie import orchestration.

use std::path::Path;

use crate::browser::catalog::{BrowserEngine, BrowserKind};
use crate::browser::chromium::{
    chromium_decryption_key, discover_chromium_stores, read_chromium_cookies,
};
use crate::browser::domains::{build_cookie_header, has_session_cookie, CookiePair};
use crate::browser::gecko::{discover_gecko_stores, read_gecko_cookies};

#[cfg(target_os = "macos")]
use crate::browser::safari::{discover_safari_stores, read_safari_cookies};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedCookies {
    pub cookie_header: String,
    pub source_label: String,
}

pub struct CookieImportQuery<'a> {
    pub home: &'a Path,
    pub browsers: &'a [BrowserKind],
    pub domains: &'a [&'static str],
    pub session_cookie_names: &'a [&'static str],
    pub require_session_name: bool,
}

pub fn import_cookies(query: &CookieImportQuery<'_>) -> Option<ImportedCookies> {
    for browser in query.browsers {
        if let Some(imported) = import_from_browser(*browser, query) {
            return Some(imported);
        }
    }
    None
}

fn import_from_browser(
    browser: BrowserKind,
    query: &CookieImportQuery<'_>,
) -> Option<ImportedCookies> {
    match browser.engine() {
        BrowserEngine::Gecko => import_from_gecko(browser, query),
        BrowserEngine::Chromium => import_from_chromium(browser, query),
        BrowserEngine::WebKit => import_from_safari(query),
    }
}

fn import_from_gecko(
    browser: BrowserKind,
    query: &CookieImportQuery<'_>,
) -> Option<ImportedCookies> {
    for store in discover_gecko_stores(query.home, browser) {
        let cookies = read_gecko_cookies(&store, query.domains).ok()?;
        if let Some(imported) = finalize_import(&store.label, &cookies, query) {
            return Some(imported);
        }
    }
    None
}

fn import_from_chromium(
    browser: BrowserKind,
    query: &CookieImportQuery<'_>,
) -> Option<ImportedCookies> {
    let key = chromium_decryption_key(browser).ok()?;
    for store in discover_chromium_stores(query.home, browser) {
        let cookies = read_chromium_cookies(&store, query.domains, &key).ok()?;
        if let Some(imported) = finalize_import(&store.label, &cookies, query) {
            return Some(imported);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn import_from_safari(query: &CookieImportQuery<'_>) -> Option<ImportedCookies> {
    for store in discover_safari_stores(query.home) {
        let cookies = read_safari_cookies(&store, query.domains).ok()?;
        if let Some(imported) = finalize_import(&store.label, &cookies, query) {
            return Some(imported);
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn import_from_safari(_query: &CookieImportQuery<'_>) -> Option<ImportedCookies> {
    None
}

fn finalize_import(
    source_label: &str,
    cookies: &[CookiePair],
    query: &CookieImportQuery<'_>,
) -> Option<ImportedCookies> {
    if cookies.is_empty() {
        return None;
    }

    let names: Vec<&str> = cookies.iter().map(|cookie| cookie.name.as_str()).collect();
    let has_named_session = has_session_cookie(&names, query.session_cookie_names);

    if query.require_session_name && !has_named_session {
        return None;
    }

    let header = build_cookie_header(cookies);
    if header.is_empty() {
        return None;
    }

    Some(ImportedCookies {
        cookie_header: header,
        source_label: source_label.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::catalog::BrowserKind;
    use rusqlite::Connection;
    use std::fs;

    fn write_gecko_fixture(path: &Path, host: &str, name: &str, value: &str) {
        let connection = Connection::open(path).expect("open fixture db");
        connection
            .execute_batch(
                "CREATE TABLE moz_cookies (
                    host TEXT NOT NULL,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    value TEXT,
                    expiry INTEGER,
                    isSecure INTEGER,
                    isHttpOnly INTEGER
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO moz_cookies (host, name, path, value, expiry, isSecure, isHttpOnly)
                 VALUES (?1, ?2, '/', ?3, 0, 1, 1)",
                [host, name, value],
            )
            .expect("insert");
    }

    #[test]
    fn import_cookies_finds_cursor_session_in_zen() {
        let temp = std::env::temp_dir().join(format!(
            "mochi-import-zen-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let profile = temp.join("Library/Application Support/zen/Profiles/abc.default-release");
        fs::create_dir_all(&profile).expect("profile dir");
        write_gecko_fixture(
            &profile.join("cookies.sqlite"),
            ".cursor.com",
            "WorkosCursorSessionToken",
            "zen-session",
        );

        let imported = import_cookies(&CookieImportQuery {
            home: &temp,
            browsers: &[BrowserKind::Zen],
            domains: &["cursor.com", "cursor.sh"],
            session_cookie_names: &["WorkosCursorSessionToken"],
            require_session_name: true,
        })
        .expect("import");

        assert!(imported.source_label.contains("Zen"));
        assert!(imported
            .cookie_header
            .contains("WorkosCursorSessionToken=zen-session"));

        let _ = fs::remove_dir_all(temp);
    }
}
