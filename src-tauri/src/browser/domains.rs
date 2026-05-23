//! Cookie domain matching and header assembly.

#[derive(Debug, Clone)]
pub struct CookiePair {
    pub name: String,
    pub value: String,
}

pub fn domain_matches(host: &str, patterns: &[&str]) -> bool {
    let normalized = normalize_domain(host);
    patterns.iter().any(|pattern| {
        let pattern = normalize_domain(pattern);
        normalized == pattern || normalized.ends_with(&format!(".{pattern}"))
    })
}

pub fn normalize_domain(raw: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('.');
    trimmed.to_ascii_lowercase()
}

pub fn build_cookie_header(pairs: &[CookiePair]) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut parts = Vec::new();
    for pair in pairs {
        if seen.insert(pair.name.clone()) {
            parts.push(format!("{}={}", pair.name, pair.value));
        }
    }
    parts.join("; ")
}

pub fn has_session_cookie(names: &[&str], session_names: &[&str]) -> bool {
    names.iter().any(|name| session_names.contains(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_matches_cursor_hosts() {
        assert!(domain_matches("cursor.com", &["cursor.com"]));
        assert!(domain_matches(".cursor.com", &["cursor.com"]));
        assert!(domain_matches("www.cursor.com", &["cursor.com"]));
        assert!(domain_matches("authenticator.cursor.sh", &["cursor.sh"]));
        assert!(!domain_matches("evil.com", &["cursor.com"]));
    }

    #[test]
    fn build_cookie_header_deduplicates_names() {
        let header = build_cookie_header(&[
            CookiePair {
                name: "a".into(),
                value: "1".into(),
            },
            CookiePair {
                name: "a".into(),
                value: "2".into(),
            },
            CookiePair {
                name: "b".into(),
                value: "3".into(),
            },
        ]);
        assert_eq!(header, "a=1; b=3");
    }
}
