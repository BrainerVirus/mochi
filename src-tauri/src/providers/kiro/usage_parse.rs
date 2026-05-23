//! Kiro CLI `/usage` output parsing.
//!
//! Ported from CodexBar `KiroStatusProbe.swift` (MIT).

use regex::Regex;
use time::{format_description::well_known::Rfc3339, Date, Month, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKiroUsage {
    pub plan_name: String,
    pub credits_percent: f32,
    pub credits_used: f64,
    pub credits_total: f64,
    pub bonus_used: Option<f64>,
    pub bonus_total: Option<f64>,
    pub bonus_expiry_days: Option<i32>,
    pub resets_at: Option<String>,
}

pub fn parse_usage_output(output: &str) -> ProviderResult<ParsedKiroUsage> {
    let stripped = strip_ansi(output);
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return Err(ProviderError::Parse("Empty output from kiro-cli.".into()));
    }

    let lowered = stripped.to_lowercase();
    if lowered.contains("not logged in")
        || lowered.contains("login required")
        || lowered.contains("failed to initialize auth portal")
        || lowered.contains("kiro-cli login")
        || lowered.contains("oauth error")
    {
        return Err(ProviderError::Auth(
            "Not logged in to Kiro. Run 'kiro-cli login'.".into(),
        ));
    }

    if lowered.contains("could not retrieve usage information") {
        return Err(ProviderError::Parse(
            "Kiro CLI could not retrieve usage information.".into(),
        ));
    }

    let (plan_name, matched_new_format) = parse_plan_name(&stripped);
    let is_managed_plan =
        lowered.contains("managed by admin") || lowered.contains("managed by organization");

    let resets_at = parse_reset_date(&stripped);

    let percent_re = Regex::new(r"█+\s*(\d+)%").expect("percent regex");
    let mut matched_percent = false;
    let mut credits_percent = 0.0_f32;
    if let Some(caps) = percent_re.captures(&stripped) {
        if let Some(m) = caps.get(1) {
            credits_percent = m.as_str().parse().unwrap_or(0.0);
            matched_percent = true;
        }
    }

    let credits_re = Regex::new(r"\((\d+\.?\d*)\s+of\s+(\d+)\s+covered").expect("credits regex");
    let mut matched_credits = false;
    let mut credits_used = 0.0;
    let mut credits_total = 50.0;
    if let Some(caps) = credits_re.captures(&stripped) {
        credits_used = caps
            .get(1)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0.0);
        credits_total = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(50.0);
        matched_credits = true;
    }

    if !matched_percent && matched_credits && credits_total > 0.0 {
        credits_percent = ((credits_used / credits_total) * 100.0) as f32;
    }

    let (bonus_used, bonus_total, bonus_expiry_days) = parse_bonus_credits(&stripped);

    if matched_new_format && is_managed_plan && !matched_percent && !matched_credits {
        return Ok(ParsedKiroUsage {
            plan_name,
            credits_percent: 0.0,
            credits_used: 0.0,
            credits_total: 0.0,
            bonus_used,
            bonus_total,
            bonus_expiry_days,
            resets_at: None,
        });
    }

    if !matched_percent && !matched_credits {
        return Err(ProviderError::Parse(
            "No recognizable usage patterns found in kiro-cli output.".into(),
        ));
    }

    Ok(ParsedKiroUsage {
        plan_name,
        credits_percent,
        credits_used,
        credits_total,
        bonus_used,
        bonus_total,
        bonus_expiry_days,
        resets_at,
    })
}

pub fn snapshot_from_parsed(
    parsed: &ParsedKiroUsage,
    updated_at: &str,
    source: &str,
) -> UsageSnapshot {
    let primary = UsageWindow::new("Credits", parsed.credits_percent, parsed.resets_at.clone());

    let secondary = parsed.bonus_total.map(|total| {
        let used = parsed.bonus_used.unwrap_or(0.0);
        let percent = if total > 0.0 {
            ((used / total) * 100.0) as f32
        } else {
            0.0
        };
        let resets_at = parsed.bonus_expiry_days.and_then(|days| {
            let expiry = OffsetDateTime::now_utc() + time::Duration::days(days as i64);
            expiry.format(&Rfc3339).ok()
        });
        UsageWindow::new("Bonus", percent, resets_at)
    });

    UsageSnapshot::new(ProviderId::Kiro, primary, secondary, updated_at, source)
}

fn parse_plan_name(text: &str) -> (String, bool) {
    let mut plan_name = "Kiro".to_string();
    let mut matched_new_format = false;

    if let Some(caps) = Regex::new(r"\|\s*(KIRO\s+\w+)")
        .expect("plan regex")
        .captures(text)
    {
        if let Some(m) = caps.get(1) {
            plan_name = m.as_str().trim().to_string();
        }
    }

    if let Some(caps) = Regex::new(r"Estimated Usage\s*\|[^\n|]*\|\s*([A-Z][A-Z0-9 ]+)")
        .expect("estimated regex")
        .captures(text)
    {
        if let Some(m) = caps.get(1) {
            plan_name = m.as_str().trim().to_string();
        }
    }

    if let Some(caps) = Regex::new(r"Plan:\s*(.+)")
        .expect("new plan regex")
        .captures(text)
    {
        if let Some(m) = caps.get(1) {
            let line = m.as_str().lines().next().unwrap_or("").trim();
            if !line.is_empty() {
                plan_name = line.to_string();
                matched_new_format = true;
            }
        }
    }

    (plan_name, matched_new_format)
}

fn parse_bonus_credits(text: &str) -> (Option<f64>, Option<f64>, Option<i32>) {
    let bonus_re = Regex::new(r"Bonus credits:\s*(\d+\.?\d*)/(\d+)").expect("bonus regex");
    let mut used = None;
    let mut total = None;
    if let Some(caps) = bonus_re.captures(text) {
        used = caps.get(1).and_then(|m| m.as_str().parse().ok());
        total = caps.get(2).and_then(|m| m.as_str().parse().ok());
    }

    let expiry_re = Regex::new(r"expires in (\d+) days?").expect("expiry regex");
    let expiry_days = expiry_re
        .captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());

    (used, total, expiry_days)
}

fn parse_reset_date(text: &str) -> Option<String> {
    let reset_re = Regex::new(r"resets on (\d{4}-\d{2}-\d{2}|\d{2}/\d{2})").expect("reset regex");
    let date_str = reset_re.captures(text)?.get(1)?.as_str();

    parse_reset_date_value(date_str).and_then(|date| date.format(&Rfc3339).ok())
}

fn parse_reset_date_value(date_str: &str) -> Option<OffsetDateTime> {
    if date_str.contains('-') {
        let date = Date::parse(
            date_str,
            &time::format_description::parse("[year]-[month]-[day]").ok()?,
        )
        .ok()?;
        return date.with_hms(0, 0, 0).ok().map(|dt| dt.assume_utc());
    }

    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    let month: u8 = parts[0].parse().ok()?;
    let day: u8 = parts[1].parse().ok()?;
    let month = Month::try_from(month).ok()?;
    let now = OffsetDateTime::now_utc();
    let year = now.year();

    let mut date = Date::from_calendar_date(year, month, day).ok()?;
    let candidate = date.with_hms(0, 0, 0).ok()?.assume_utc();
    if candidate > now {
        return Some(candidate);
    }

    date = Date::from_calendar_date(year + 1, month, day).ok()?;
    date.with_hms(0, 0, 0).ok().map(|dt| dt.assume_utc())
}

pub fn strip_ansi(text: &str) -> String {
    let re = Regex::new(r"\x1B\[[0-9;?]*[A-Za-z]|\x1B\].*?\x07").expect("ansi regex");
    re.replace_all(text, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_usage_fixture() {
        let parsed = parse_usage_output(include_str!("fixtures/usage_basic.txt")).expect("parse");
        assert_eq!(parsed.plan_name, "KIRO FREE");
        assert_eq!(parsed.credits_percent, 25.0);
        assert_eq!(parsed.credits_used, 12.5);
        assert_eq!(parsed.credits_total, 50.0);
        assert!(parsed.resets_at.is_some());
        assert!(parsed.bonus_total.is_none());
    }

    #[test]
    fn parses_bonus_credits_fixture() {
        let parsed = parse_usage_output(include_str!("fixtures/usage_bonus.txt")).expect("parse");
        assert_eq!(parsed.plan_name, "KIRO PRO");
        assert_eq!(parsed.credits_percent, 80.0);
        assert_eq!(parsed.bonus_used, Some(5.0));
        assert_eq!(parsed.bonus_total, Some(10.0));
        assert_eq!(parsed.bonus_expiry_days, Some(7));
    }

    #[test]
    fn maps_snapshot_windows_with_codexbar_labels() {
        let parsed = parse_usage_output(include_str!("fixtures/usage_bonus.txt")).expect("parse");
        let snapshot = snapshot_from_parsed(&parsed, "2026-05-23T00:00:00Z", "kiro-cli-usage");
        assert_eq!(snapshot.primary.label, "Credits");
        assert_eq!(snapshot.secondary.as_ref().expect("bonus").label, "Bonus");
        assert_eq!(snapshot.primary.used_percent, 80.0);
    }

    #[test]
    fn rejects_unrecognized_output() {
        let error = parse_usage_output("| KIRO FREE |").expect_err("parse");
        assert!(matches!(error, ProviderError::Parse(_)));
    }
}
