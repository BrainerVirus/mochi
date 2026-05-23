//! Augment usage parsing for CLI and web API responses.

use regex::Regex;
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, Date, Month, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedAugmentUsage {
    pub credits_used: f64,
    pub credits_limit: f64,
    pub credits_remaining: f64,
    pub billing_cycle_end: Option<String>,
    pub account_plan: Option<String>,
    pub account_email: Option<String>,
}

pub fn parse_auggie_cli_output(output: &str) -> ProviderResult<ParsedAugmentUsage> {
    if output.contains("Authentication failed") || output.contains("auggie login") {
        return Err(ProviderError::Auth(
            "Not authenticated. Run 'auggie login'.".into(),
        ));
    }

    let mut max_credits: Option<i64> = None;
    let mut remaining: Option<i64> = None;
    let mut used: Option<i64> = None;
    let mut total: Option<i64> = None;
    let mut billing_cycle_end: Option<String> = None;

    let remaining_re = Regex::new(r"([\d,]+)\s+remaining").expect("remaining regex");
    let used_re = Regex::new(r"([\d,]+)\s*/\s*([\d,]+)\s+credits used").expect("used regex");
    let max_re = Regex::new(r"([\d,]+)\s+credits").expect("max regex");
    let ends_re = Regex::new(r"ends\s+([\d/]+)").expect("ends regex");

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Max Plan") && trimmed.contains("credits") {
            if let Some(caps) = max_re.captures(trimmed) {
                max_credits = caps
                    .get(1)
                    .and_then(|m| m.as_str().replace(',', "").parse().ok());
            }
        }
        if trimmed.contains("remaining") && trimmed.contains("credits used") {
            if let Some(caps) = remaining_re.captures(trimmed) {
                remaining = caps
                    .get(1)
                    .and_then(|m| m.as_str().replace(',', "").parse().ok());
            }
            if let Some(caps) = used_re.captures(trimmed) {
                used = caps
                    .get(1)
                    .and_then(|m| m.as_str().replace(',', "").parse().ok());
                total = caps
                    .get(2)
                    .and_then(|m| m.as_str().replace(',', "").parse().ok());
            }
        }
        if trimmed.contains("billing cycle") && trimmed.contains("ends") {
            if let Some(caps) = ends_re.captures(trimmed) {
                if let Some(date_str) = caps.get(1).map(|m| m.as_str()) {
                    billing_cycle_end = parse_us_date(date_str);
                }
            }
        }
    }

    let final_remaining = remaining.ok_or_else(|| {
        ProviderError::Parse("Could not extract remaining credits from auggie output".into())
    })?;
    let final_used = used.ok_or_else(|| {
        ProviderError::Parse("Could not extract used credits from auggie output".into())
    })?;
    let final_total = total.ok_or_else(|| {
        ProviderError::Parse("Could not extract total credits from auggie output".into())
    })?;

    Ok(ParsedAugmentUsage {
        credits_used: final_used as f64,
        credits_limit: final_total as f64,
        credits_remaining: final_remaining as f64,
        billing_cycle_end,
        account_plan: max_credits.map(|value| format!("{value} credits/month")),
        account_email: None,
    })
}

#[derive(Debug, Deserialize)]
struct CreditsResponse {
    #[serde(default, rename = "usageUnitsRemaining")]
    usage_units_remaining: Option<f64>,
    #[serde(default, rename = "usageUnitsConsumedThisBillingCycle")]
    usage_units_consumed_this_billing_cycle: Option<f64>,
    #[serde(default, rename = "usageUnitsAvailable")]
    usage_units_available: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SubscriptionResponse {
    #[serde(default, rename = "planName")]
    plan_name: Option<String>,
    #[serde(default, rename = "billingPeriodEnd")]
    billing_period_end: Option<String>,
    #[serde(default)]
    email: Option<String>,
}

pub fn parse_web_responses(
    credits_json: &str,
    subscription_json: Option<&str>,
) -> ProviderResult<ParsedAugmentUsage> {
    let credits: CreditsResponse = serde_json::from_str(credits_json)
        .map_err(|error| ProviderError::Parse(error.to_string()))?;

    let remaining = credits.usage_units_remaining;
    let used = credits.usage_units_consumed_this_billing_cycle;
    let limit = credits.usage_units_available.or(match (remaining, used) {
        (Some(r), Some(u)) => Some(r + u),
        _ => None,
    });

    let (billing_cycle_end, account_plan, account_email) = if let Some(sub_json) = subscription_json
    {
        let subscription: SubscriptionResponse = serde_json::from_str(sub_json)
            .map_err(|error| ProviderError::Parse(error.to_string()))?;
        (
            subscription
                .billing_period_end
                .and_then(|value| OffsetDateTime::parse(&value, &Rfc3339).ok())
                .and_then(|dt| dt.format(&Rfc3339).ok()),
            subscription.plan_name,
            subscription.email,
        )
    } else {
        (None, None, None)
    };

    let credits_limit = limit
        .ok_or_else(|| ProviderError::Parse("Augment credits response missing limit".into()))?;
    let credits_used = used.unwrap_or(0.0);
    let credits_remaining = remaining.unwrap_or((credits_limit - credits_used).max(0.0));

    Ok(ParsedAugmentUsage {
        credits_used,
        credits_limit,
        credits_remaining,
        billing_cycle_end,
        account_plan,
        account_email,
    })
}

pub fn snapshot_from_parsed(
    parsed: &ParsedAugmentUsage,
    updated_at: &str,
    source: &str,
) -> UsageSnapshot {
    let used_percent = if parsed.credits_limit > 0.0 {
        ((parsed.credits_used / parsed.credits_limit) * 100.0) as f32
    } else {
        0.0
    };

    UsageSnapshot::new(
        ProviderId::Augment,
        UsageWindow::new("Credits", used_percent, parsed.billing_cycle_end.clone()),
        None,
        updated_at,
        source,
    )
}

fn parse_us_date(date_str: &str) -> Option<String> {
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let month: u8 = parts[0].parse().ok()?;
    let day: u8 = parts[1].parse().ok()?;
    let year: i32 = parts[2].parse().ok()?;
    let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;
    date.with_hms(0, 0, 0)
        .ok()?
        .assume_utc()
        .format(&Rfc3339)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_auggie_cli_fixture() {
        let parsed =
            parse_auggie_cli_output(include_str!("fixtures/auggie_status.txt")).expect("parse");
        assert_eq!(parsed.credits_remaining, 11657.0);
        assert_eq!(parsed.credits_used, 953170.0);
        assert_eq!(parsed.credits_limit, 964827.0);
        assert!(parsed.billing_cycle_end.is_some());
    }

    #[test]
    fn parses_web_api_fixtures() {
        let parsed = parse_web_responses(
            include_str!("fixtures/credits.json"),
            Some(include_str!("fixtures/subscription.json")),
        )
        .expect("parse");
        assert_eq!(parsed.credits_used, 953170.0);
        assert_eq!(parsed.credits_limit, 964827.0);
        assert_eq!(parsed.account_email.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn maps_snapshot_with_credits_label() {
        let parsed = parse_web_responses(
            include_str!("fixtures/credits.json"),
            Some(include_str!("fixtures/subscription.json")),
        )
        .expect("parse");
        let snapshot = snapshot_from_parsed(&parsed, "2026-05-23T00:00:00Z", "augment-web");
        assert_eq!(snapshot.primary.label, "Credits");
        assert!(snapshot.primary.used_percent > 98.0);
    }
}
