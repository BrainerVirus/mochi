//! Chromium cookie DB reader with macOS Keychain decryption.
//!
//! Derived from SweetCookieKit `ChromeCookieImporter.swift` (MIT).

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use super::decrypt::{decrypt_chromium_value, derive_chromium_key};
use crate::browser::catalog::BrowserKind;
use crate::browser::domains::{domain_matches, CookiePair};

#[derive(Debug, Clone)]
pub struct ChromiumCookieStore {
    pub browser: BrowserKind,
    pub label: String,
    pub cookies_db: PathBuf,
}

pub fn discover_chromium_stores(home: &Path, browser: BrowserKind) -> Vec<ChromiumCookieStore> {
    let Some(support_path) = browser.chromium_support_path() else {
        return Vec::new();
    };

    let root = home.join("Library/Application Support").join(support_path);
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };

    let mut profiles: Vec<(String, PathBuf)> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == "Default" || name.starts_with("Profile ") || name.starts_with("user-") {
                Some((name, entry.path()))
            } else {
                None
            }
        })
        .collect();

    profiles.sort_by(|left, right| left.0.cmp(&right.0));

    profiles
        .into_iter()
        .flat_map(|(profile_name, profile_dir)| {
            let label_base = format!("{} {profile_name}", browser.display_name());
            [
                ChromiumCookieStore {
                    browser,
                    label: format!("{label_base} (Network)"),
                    cookies_db: profile_dir.join("Network/Cookies"),
                },
                ChromiumCookieStore {
                    browser,
                    label: label_base,
                    cookies_db: profile_dir.join("Cookies"),
                },
            ]
        })
        .filter(|store| store.cookies_db.is_file())
        .collect()
}

pub fn read_chromium_cookies(
    store: &ChromiumCookieStore,
    domains: &[&str],
    decryption_key: &[u8; 16],
) -> Result<Vec<CookiePair>, String> {
    let copied = copy_locked_db(&store.cookies_db)?;
    read_chromium_cookies_from_db(&copied, domains, decryption_key).map_err(|error| {
        format!(
            "read {} cookies from {}: {error}",
            store.browser.display_name(),
            store.label
        )
    })
}

fn copy_locked_db(source: &Path) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir().join(format!(
        "mochi-chromium-cookies-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).map_err(|error| error.to_string())?;
    let copied = temp_dir.join("Cookies");
    fs::copy(source, &copied).map_err(|error| error.to_string())?;

    for suffix in ["-wal", "-shm"] {
        let src = PathBuf::from(format!("{}{suffix}", source.display()));
        if src.is_file() {
            let _ = fs::copy(&src, PathBuf::from(format!("{}{suffix}", copied.display())));
        }
    }

    Ok(copied)
}

fn read_chromium_cookies_from_db(
    path: &Path,
    domains: &[&str],
    decryption_key: &[u8; 16],
) -> rusqlite::Result<Vec<CookiePair>> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT host_key, name, value, encrypted_value FROM cookies
         WHERE (value IS NOT NULL AND value != '') OR (encrypted_value IS NOT NULL AND length(encrypted_value) > 0)",
    )?;

    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<Vec<u8>>>(3)?,
        ))
    })?;

    let mut cookies = Vec::new();
    for row in rows {
        let (host, name, plain, encrypted) = row?;
        if !domain_matches(&host, domains) {
            continue;
        }

        let value = if let Some(plain) = plain.filter(|value| !value.is_empty()) {
            plain
        } else if let Some(encrypted) = encrypted {
            match decrypt_chromium_value(&encrypted, decryption_key) {
                Some(value) => value,
                None => continue,
            }
        } else {
            continue;
        };

        cookies.push(CookiePair { name, value });
    }

    Ok(cookies)
}

pub fn chromium_decryption_key(browser: BrowserKind) -> Result<[u8; 16], String> {
    let password = crate::browser::keychain::read_safe_storage_password(browser)?;
    Ok(derive_chromium_key(&password))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn write_chromium_fixture(path: &Path, host: &str, name: &str, value: &str) {
        let connection = Connection::open(path).expect("open fixture db");
        connection
            .execute_batch(
                "CREATE TABLE cookies (
                    host_key TEXT NOT NULL,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    value TEXT,
                    encrypted_value BLOB,
                    expires_utc INTEGER,
                    is_secure INTEGER,
                    is_httponly INTEGER
                );",
            )
            .expect("schema");
        connection
            .execute(
                "INSERT INTO cookies (host_key, name, path, value, encrypted_value, expires_utc, is_secure, is_httponly)
                 VALUES (?1, ?2, '/', ?3, NULL, 0, 1, 1)",
                [host, name, value],
            )
            .expect("insert");
    }

    #[test]
    fn discover_chrome_network_cookies_db() {
        let temp = std::env::temp_dir().join(format!(
            "mochi-chrome-discover-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let profile = temp.join("Library/Application Support/Google/Chrome/Default/Network");
        fs::create_dir_all(&profile).expect("profile dir");
        write_chromium_fixture(
            &profile.join("Cookies"),
            ".cursor.com",
            "WorkosCursorSessionToken",
            "chrome-token",
        );

        let stores = discover_chromium_stores(&temp, BrowserKind::Chrome);
        assert_eq!(stores.len(), 1);
        assert!(stores[0].label.contains("Network"));

        let key = derive_chromium_key("unused-for-plaintext");
        let cookies = read_chromium_cookies(&stores[0], &["cursor.com"], &key).expect("cookies");
        assert_eq!(cookies[0].value, "chrome-token");

        let _ = fs::remove_dir_all(temp);
    }
}
