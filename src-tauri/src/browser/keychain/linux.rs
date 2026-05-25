//! Linux Secret Service (GNOME Keyring / KWallet) for Chromium Safe Storage passwords.

use std::path::Path;

use crate::browser::catalog::BrowserKind;

pub fn read_safe_storage_password(_home: &Path, browser: BrowserKind) -> Result<String, String> {
    for (service, account) in browser.safe_storage_labels() {
        if let Ok(password) = try_keyring_entry(service, account) {
            if !password.is_empty() {
                return Ok(password);
            }
        }
    }

    Err(format!(
        "Secret Service denied access to {} Safe Storage (install libsecret and unlock your keyring)",
        browser.display_name()
    ))
}

fn try_keyring_entry(service: &str, account: &str) -> Result<String, String> {
    let entry = keyring::Entry::new(service, account).map_err(|error| error.to_string())?;
    entry.get_password().map_err(|error| error.to_string())
}
