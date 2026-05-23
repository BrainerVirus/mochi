//! Chromium cookie decryption (macOS v10 format).
//!
//! Derived from SweetCookieKit `ChromeCookieImporter.swift` (MIT).

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;

type Aes128CbcDec = Decryptor<aes::Aes128>;

pub fn derive_chromium_key(password: &str) -> [u8; 16] {
    let mut key = [0u8; 16];
    pbkdf2_hmac::<Sha1>(password.as_bytes(), b"saltysalt", 1003, &mut key);
    key
}

pub fn decrypt_chromium_value(encrypted: &[u8], key: &[u8; 16]) -> Option<String> {
    if encrypted.len() <= 3 {
        return None;
    }
    if &encrypted[..3] != b"v10" {
        return None;
    }

    let payload = &encrypted[3..];
    let iv = [0x20u8; 16];
    let mut buffer = payload.to_vec();
    let decrypted = Aes128CbcDec::new(key.into(), (&iv).into())
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .ok()?;

    let cleaned = if decrypted.len() > 32 {
        &decrypted[32..]
    } else {
        decrypted
    };

    String::from_utf8(cleaned.to_vec())
        .ok()
        .map(|value| {
            value
                .trim_start_matches(|ch: char| (ch as u32) < 0x20)
                .to_string()
        })
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_chromium_key_is_stable() {
        let key_a = derive_chromium_key("test-password");
        let key_b = derive_chromium_key("test-password");
        assert_eq!(key_a, key_b);
        assert_ne!(key_a, derive_chromium_key("other"));
    }
}
