//! Non-macOS stub for Chromium cookie import.

use std::path::Path;

use crate::browser::catalog::BrowserKind;
use crate::browser::domains::CookiePair;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChromiumCookieStore {
    pub browser: BrowserKind,
    pub label: String,
    pub cookies_db: std::path::PathBuf,
}

pub fn discover_chromium_stores(_home: &Path, _browser: BrowserKind) -> Vec<ChromiumCookieStore> {
    Vec::new()
}

pub fn read_chromium_cookies(
    _store: &ChromiumCookieStore,
    _domains: &[&str],
    _decryption_key: &[u8; 16],
) -> Result<Vec<CookiePair>, String> {
    Err("Chromium cookie import is only supported on macOS".into())
}

pub fn chromium_decryption_key(_browser: BrowserKind) -> Result<[u8; 16], String> {
    Err("Chromium cookie import is only supported on macOS".into())
}
