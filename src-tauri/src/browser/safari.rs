//! Safari `Cookies.binarycookies` reader (macOS).
//!
//! Derived from SweetCookieKit `SafariCookieImporter.swift` (MIT).

use std::fs;
use std::path::{Path, PathBuf};

use crate::browser::domains::{domain_matches, normalize_domain, CookiePair};

#[derive(Debug, Clone)]
pub struct SafariCookieStore {
    pub label: String,
    pub cookie_file: PathBuf,
}

pub fn discover_safari_stores(home: &Path) -> Vec<SafariCookieStore> {
    let mut paths = vec![
        home.join("Library/Cookies/Cookies.binarycookies"),
        home.join("Library/Containers/com.apple.Safari/Data/Library/Cookies/Cookies.binarycookies"),
    ];

    for root in [
        home.join("Library/WebKit/WebsiteDataStore"),
        home.join("Library/Containers/com.apple.Safari/Data/Library/WebKit/WebsiteDataStore"),
    ] {
        paths.extend(find_website_data_cookies(&root));
    }

    let mut seen = std::collections::HashSet::new();
    paths
        .into_iter()
        .filter(|path| path.is_file())
        .filter(|path| seen.insert(path.clone()))
        .map(|path| SafariCookieStore {
            label: safari_store_label(&path),
            cookie_file: path,
        })
        .collect()
}

fn find_website_data_cookies(root: &Path) -> Vec<PathBuf> {
    let Ok(walker) = fs::read_dir(root) else {
        return Vec::new();
    };

    walker
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path().join("Cookies/Cookies.binarycookies"))
        .filter(|path| path.is_file())
        .collect()
}

fn safari_store_label(path: &Path) -> String {
    if path.to_string_lossy().contains("WebsiteDataStore") {
        if let Some(token) = path
            .components()
            .map(|component| component.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .windows(2)
            .find(|parts| parts[0] == "WebsiteDataStore")
            .map(|parts| parts[1].clone())
        {
            return format!("Safari ({token})");
        }
    }
    "Safari".to_string()
}

pub fn read_safari_cookies(
    store: &SafariCookieStore,
    domains: &[&str],
) -> Result<Vec<CookiePair>, String> {
    let data = fs::read(&store.cookie_file).map_err(|error| {
        format!(
            "read Safari cookies from {}: {error}",
            store.cookie_file.display()
        )
    })?;
    let records = parse_binary_cookies(&data)
        .map_err(|error| format!("parse Safari cookies from {}: {error}", store.label))?;

    Ok(records
        .into_iter()
        .filter(|record| domain_matches(&record.domain, domains))
        .map(|record| CookiePair {
            name: record.name,
            value: record.value,
        })
        .collect())
}

struct SafariRecord {
    domain: String,
    name: String,
    value: String,
}

fn parse_binary_cookies(data: &[u8]) -> Result<Vec<SafariRecord>, String> {
    if data.len() < 8 || &data[..4] != b"cook" {
        return Err("invalid binarycookies header".into());
    }

    let page_count = u32::from_be_bytes(data[4..8].try_into().map_err(|_| "invalid page count")?);
    let mut offset = 8;
    let mut page_sizes = Vec::with_capacity(page_count as usize);
    for _ in 0..page_count {
        if offset + 4 > data.len() {
            return Err("truncated page size table".into());
        }
        page_sizes.push(u32::from_be_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|_| "invalid page size")?,
        ) as usize);
        offset += 4;
    }

    let mut records = Vec::new();
    for page_size in page_sizes {
        if offset + page_size > data.len() {
            break;
        }
        records.extend(parse_cookie_page(&data[offset..offset + page_size])?);
        offset += page_size;
    }

    Ok(records)
}

fn parse_cookie_page(page: &[u8]) -> Result<Vec<SafariRecord>, String> {
    if page.len() < 8 {
        return Ok(Vec::new());
    }

    let cookie_count =
        u32::from_le_bytes(page[4..8].try_into().map_err(|_| "invalid cookie count")?);
    let mut offset = 8;
    let mut cookie_offsets = Vec::with_capacity(cookie_count as usize);
    for _ in 0..cookie_count {
        if offset + 4 > page.len() {
            break;
        }
        cookie_offsets.push(u32::from_le_bytes(
            page[offset..offset + 4]
                .try_into()
                .map_err(|_| "invalid cookie offset")?,
        ) as usize);
        offset += 4;
    }

    cookie_offsets
        .into_iter()
        .filter_map(|cookie_offset| parse_cookie_record(page, cookie_offset))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_cookie_record(page: &[u8], offset: usize) -> Option<Result<SafariRecord, String>> {
    if offset + 56 > page.len() {
        return None;
    }

    let size = u32::from_le_bytes(page[offset..offset + 4].try_into().ok()?) as usize;
    if size == 0 || offset + size > page.len() {
        return None;
    }

    let url_offset = u32::from_le_bytes(page[offset + 16..offset + 20].try_into().ok()?) as usize;
    let name_offset = u32::from_le_bytes(page[offset + 20..offset + 24].try_into().ok()?) as usize;
    let value_offset = u32::from_le_bytes(page[offset + 28..offset + 32].try_into().ok()?) as usize;

    let domain = read_c_string(page, offset, url_offset)?;
    let name = read_c_string(page, offset, name_offset)?;
    let value = read_c_string(page, offset, value_offset)?;

    if domain.is_empty() || name.is_empty() {
        return None;
    }

    Some(Ok(SafariRecord {
        domain: normalize_domain(&domain),
        name,
        value,
    }))
}

fn read_c_string(page: &[u8], base: usize, relative: usize) -> Option<String> {
    let start = base.checked_add(relative)?;
    if start >= page.len() {
        return None;
    }
    let end = page[start..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|index| start + index)
        .unwrap_or(page.len());
    if end <= start {
        return Some(String::new());
    }
    Some(String::from_utf8_lossy(&page[start..end]).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_safari_legacy_cookie_path() {
        let temp = std::env::temp_dir().join(format!(
            "mochi-safari-discover-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let cookie_file = temp.join("Library/Cookies/Cookies.binarycookies");
        fs::create_dir_all(cookie_file.parent().expect("parent")).expect("dir");
        fs::write(&cookie_file, b"cook").expect("write");

        let stores = discover_safari_stores(&temp);
        assert_eq!(stores.len(), 1);
        assert_eq!(stores[0].label, "Safari");

        let _ = fs::remove_dir_all(temp);
    }
}
