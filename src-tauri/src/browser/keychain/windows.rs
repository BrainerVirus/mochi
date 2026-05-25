//! Windows credential store and Chromium Local State DPAPI for cookie decryption.

use std::path::Path;

use crate::browser::catalog::BrowserKind;
use crate::browser::profiles;
use base64::Engine;

const DPAPI_PREFIX: &[u8] = b"DPAPI";

pub fn read_safe_storage_password(home: &Path, browser: BrowserKind) -> Result<String, String> {
    for (service, account) in browser.safe_storage_labels() {
        if let Ok(password) = try_keyring_entry(service, account) {
            if !password.is_empty() {
                return Ok(password);
            }
        }
    }

    let user_data = profiles::chromium_user_data_root(home, browser).ok_or_else(|| {
        format!(
            "could not locate {} user data directory",
            browser.display_name()
        )
    })?;

    read_password_from_local_state(&user_data)
}

fn try_keyring_entry(service: &str, account: &str) -> Result<String, String> {
    let entry = keyring::Entry::new(service, account).map_err(|error| error.to_string())?;
    entry.get_password().map_err(|error| error.to_string())
}

fn read_password_from_local_state(user_data_dir: &Path) -> Result<String, String> {
    let local_state_path = user_data_dir.join("Local State");
    let contents = std::fs::read_to_string(&local_state_path)
        .map_err(|error| format!("read Local State: {error}"))?;

    let json: serde_json::Value =
        serde_json::from_str(&contents).map_err(|error| format!("parse Local State: {error}"))?;

    let encoded = json
        .get("os_crypt")
        .and_then(|value| value.get("encrypted_key"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Local State missing os_crypt.encrypted_key".to_string())?;

    let mut decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| format!("decode encrypted_key: {error}"))?;

    if decoded.len() > DPAPI_PREFIX.len() && decoded.starts_with(DPAPI_PREFIX) {
        decoded = decoded[DPAPI_PREFIX.len()..].to_vec();
    }

    let decrypted = dpapi_decrypt(&decoded)?;
    String::from_utf8(decrypted).map_err(|error| format!("encrypted_key is not UTF-8: {error}"))
}

#[cfg(windows)]
fn dpapi_decrypt(data: &[u8]) -> Result<Vec<u8>, String> {
    use std::ptr;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &mut input,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };

    if ok == 0 {
        return Err("CryptUnprotectData failed".to_string());
    }

    let slice = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let result = slice.to_vec();
    unsafe {
        let _ = LocalFree(output.pbData as *mut _);
    }
    Ok(result)
}

#[cfg(not(windows))]
fn dpapi_decrypt(_data: &[u8]) -> Result<Vec<u8>, String> {
    Err("DPAPI is only available on Windows".to_string())
}
