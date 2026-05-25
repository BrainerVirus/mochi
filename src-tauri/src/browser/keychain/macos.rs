//! macOS Keychain access for Chromium Safe Storage passwords.
//!
//! Derived from SweetCookieKit `ChromeCookieImporter.swift` (MIT).

use std::path::Path;

use crate::browser::catalog::BrowserKind;

pub fn read_safe_storage_password(_home: &Path, browser: BrowserKind) -> Result<String, String> {
    use security_framework::passwords::get_generic_password;

    for (service, account) in browser.safe_storage_labels() {
        if let Ok(password) = get_generic_password(service, account) {
            if let Ok(value) = String::from_utf8(password) {
                if !value.is_empty() {
                    return Ok(value);
                }
            }
        }
    }

    Err(format!(
        "Keychain denied access to {} Safe Storage",
        browser.display_name()
    ))
}
