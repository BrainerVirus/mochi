//! HTML parsing for OpenAI Codex usage dashboard (CodexBar-derived, MIT).
//! Reference: CodexBar `OpenAIDashboardParser.swift`.

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

pub fn snapshot_from_dashboard_html(html: &str, updated_at: &str) -> ProviderResult<UsageSnapshot> {
    if matches!(parse_auth_status(html), Some(status) if status == "logged_out") {
        return Err(ProviderError::Auth(
            "openai web dashboard requires login".into(),
        ));
    }

    let body_text = visible_text_from_html(html);
    let (primary, secondary) = parse_rate_limits(&body_text);

    let primary = primary.ok_or_else(|| {
        ProviderError::Parse("codex web dashboard missing primary rate limit".into())
    })?;

    Ok(UsageSnapshot::new(
        ProviderId::Codex,
        primary,
        secondary,
        updated_at,
        "codex-browser-cookies",
    ))
}

pub fn parse_auth_status(html: &str) -> Option<String> {
    let json = client_bootstrap_json(html)?;
    json.get("authStatus")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn parse_rate_limits(body_text: &str) -> (Option<UsageWindow>, Option<UsageWindow>) {
    let cleaned = body_text.replace('\r', "\n");
    let lines: Vec<String> = cleaned
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();

    let primary = parse_rate_window(&lines, is_five_hour_limit_line, "Session");
    let secondary = parse_rate_window(&lines, is_weekly_limit_line, "Weekly");
    (primary, secondary)
}

fn parse_rate_window(
    lines: &[String],
    matcher: fn(&str) -> bool,
    label: &'static str,
) -> Option<UsageWindow> {
    for (idx, line) in lines.iter().enumerate() {
        if !matcher(line) {
            continue;
        }

        let end = lines.len().min(idx + 6);
        let window_lines = &lines[idx..end];
        let used_percent = window_lines
            .iter()
            .find_map(|candidate| parse_percent_used(candidate))?;

        return Some(UsageWindow::new(label, used_percent, None));
    }

    None
}

fn parse_percent_used(line: &str) -> Option<f32> {
    if !line.contains('%') {
        return None;
    }

    let lower = line.to_lowercase();
    let digits: String = line
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    let value = digits.parse::<f32>().ok()?;

    if lower.contains("used") || lower.contains("spent") || lower.contains("consumed") {
        return Some(value.clamp(0.0, 100.0));
    }

    Some((100.0 - value).clamp(0.0, 100.0))
}

fn is_five_hour_limit_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("5h")
        || lower.contains("5-hour")
        || lower.contains("5 hour")
        || lower.contains("5 h")
}

fn is_weekly_limit_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("weekly")
        || lower.contains("7-day")
        || lower.contains("7 day")
        || lower.contains("7d")
        || lower.contains("7 d")
}

fn client_bootstrap_json(html: &str) -> Option<serde_json::Value> {
    let marker = "id=\"client-bootstrap\"";
    let start = html.find(marker)?;
    let after_marker = &html[start + marker.len()..];
    let open = after_marker.find('>')? + 1;
    let content = &after_marker[open..];
    let close = content.find("</script>")?;
    let json_text = content[..close].trim();
    serde_json::from_str(json_text).ok()
}

fn visible_text_from_html(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut skip_until_close = false;

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        if skip_until_close {
            if ch == '>' {
                skip_until_close = false;
            }
            continue;
        }

        match ch {
            '<' => {
                let mut tag = String::new();
                for next in chars.by_ref() {
                    if next == '>' {
                        break;
                    }
                    tag.push(next);
                }
                let lower = tag.to_lowercase();
                if lower.starts_with("script") || lower.starts_with("style") {
                    skip_until_close = true;
                } else if lower.starts_with("br")
                    || lower.starts_with("div")
                    || lower.starts_with("p")
                {
                    text.push('\n');
                }
                in_tag = false;
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_dashboard_fixture_to_snapshot() {
        let html = include_str!("../fixtures/dashboard_usage.html");
        let snapshot =
            snapshot_from_dashboard_html(html, "2026-05-20T12:00:00Z").expect("snapshot");

        assert_eq!(snapshot.source, "codex-browser-cookies");
        assert_eq!(snapshot.primary.label, "Session");
        assert_eq!(snapshot.primary.used_percent, 78.0);
        let secondary = snapshot.secondary.expect("weekly window");
        assert_eq!(secondary.label, "Weekly");
        assert_eq!(secondary.used_percent, 43.0);
    }

    #[test]
    fn rejects_logged_out_dashboard_html() {
        let html = r#"<script type="application/json" id="client-bootstrap">{"authStatus":"logged_out"}</script>"#;
        let error = snapshot_from_dashboard_html(html, "2026-05-20T12:00:00Z")
            .expect_err("logged out should fail");

        assert!(matches!(error, ProviderError::Auth(_)));
    }

    #[test]
    fn parse_auth_status_reads_client_bootstrap() {
        let html = include_str!("../fixtures/dashboard_usage.html");
        assert_eq!(parse_auth_status(html).as_deref(), Some("logged_in"));
    }
}
