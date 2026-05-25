//! Gecko (Firefox, Zen) cookie DB reader.
//!
//! Derived from SweetCookieKit `GeckoCookieImporter.swift` (MIT).

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use super::catalog::BrowserKind;
use super::domains::{domain_matches, CookiePair};
use super::profiles;

#[derive(Debug, Clone)]
pub struct GeckoCookieStore {
    pub browser: BrowserKind,
    pub label: String,
    pub cookies_db: PathBuf,
}

pub fn discover_gecko_stores(home: &Path, browser: BrowserKind) -> Vec<GeckoCookieStore> {
    let Some(profiles_root) = profiles::gecko_profiles_root(home, browser) else {
        return Vec::new();
    };

    let Ok(entries) = fs::read_dir(&profiles_root) else {
        return Vec::new();
    };

    let mut profiles: Vec<(u8, String, PathBuf)> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let cookies_db = entry.path().join("cookies.sqlite");
            if cookies_db.is_file() {
                Some((profile_sort_rank(&name), name, cookies_db))
            } else {
                None
            }
        })
        .collect();

    profiles.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    profiles
        .into_iter()
        .map(|(_, profile_name, cookies_db)| GeckoCookieStore {
            browser,
            label: format!("{} {profile_name}", browser.display_name()),
            cookies_db,
        })
        .collect()
}

fn profile_sort_rank(name: &str) -> u8 {
    let lower = name.to_ascii_lowercase();
    if lower.contains("default-release") {
        0
    } else if lower.contains("default") {
        1
    } else {
        2
    }
}

pub fn read_gecko_cookies(
    store: &GeckoCookieStore,
    domains: &[&str],
) -> Result<Vec<CookiePair>, String> {
    let copied = copy_locked_db(&store.cookies_db)?;
    read_gecko_cookies_from_db(&copied, domains).map_err(|error| {
        format!(
            "read {} cookies from {}: {error}",
            store.browser.display_name(),
            store.label
        )
    })
}

fn copy_locked_db(source: &Path) -> Result<PathBuf, String> {
    let temp_dir = std::env::temp_dir().join(format!(
        "mochi-gecko-cookies-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).map_err(|error| error.to_string())?;
    let copied = temp_dir.join("cookies.sqlite");
    fs::copy(source, &copied).map_err(|error| error.to_string())?;

    for suffix in ["-wal", "-shm"] {
        let src = PathBuf::from(format!("{}{suffix}", source.display()));
        if src.is_file() {
            let _ = fs::copy(&src, PathBuf::from(format!("{}{suffix}", copied.display())));
        }
    }

    Ok(copied)
}

fn read_gecko_cookies_from_db(path: &Path, domains: &[&str]) -> rusqlite::Result<Vec<CookiePair>> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare(
        "SELECT host, name, value FROM moz_cookies WHERE value IS NOT NULL AND value != ''",
    )?;

    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut cookies = Vec::new();
    for row in rows {
        let (host, name, value) = row?;
        if domain_matches(&host, domains) {
            cookies.push(CookiePair { name, value });
        }
    }

    Ok(cookies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

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
    #[cfg(target_os = "macos")]
    fn discover_zen_profile_from_application_support() {
        let temp = std::env::temp_dir().join(format!(
            "mochi-zen-discover-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let profile = temp.join("Library/Application Support/zen/Profiles/abc.Default (release)");
        fs::create_dir_all(&profile).expect("profile dir");
        write_gecko_fixture(
            &profile.join("cookies.sqlite"),
            ".cursor.com",
            "WorkosCursorSessionToken",
            "zen-token",
        );

        let stores = discover_gecko_stores(&temp, BrowserKind::Zen);
        assert_eq!(stores.len(), 1);
        assert!(stores[0].label.contains("Zen"));
        assert!(stores[0].cookies_db.is_file());

        let cookies = read_gecko_cookies(&stores[0], &["cursor.com"]).expect("cookies");
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "WorkosCursorSessionToken");

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn profile_sort_prefers_default_release() {
        assert!(profile_sort_rank("abc.default-release") < profile_sort_rank("abc.default"));
    }
}
