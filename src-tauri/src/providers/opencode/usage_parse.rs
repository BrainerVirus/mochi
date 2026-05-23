//! OpenCode `_server` subscription payload parsing.
//!
//! Logic derived from CodexBar `OpenCodeUsageFetcher.swift` (MIT).

use regex::Regex;
use serde_json::Value;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, PartialEq)]
pub struct OpenCodeUsageData {
    pub rolling_usage_percent: f32,
    pub weekly_usage_percent: f32,
    pub rolling_reset_in_sec: i64,
    pub weekly_reset_in_sec: i64,
    pub monthly_usage_percent: Option<f32>,
    pub monthly_reset_in_sec: Option<i64>,
}

pub fn parse_workspace_ids(text: &str) -> Vec<String> {
    let Ok(regex) = Regex::new(r#"id\s*:\s*"(wrk_[^"]+)""#) else {
        return Vec::new();
    };
    regex
        .captures_iter(text)
        .filter_map(|capture| capture.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

pub fn parse_subscription(text: &str, now: &str) -> ProviderResult<OpenCodeUsageData> {
    let now = parse_timestamp(now).unwrap_or_else(OffsetDateTime::now_utc);

    if let Ok(value) = serde_json::from_str::<Value>(text) {
        if let Some(data) = parse_usage_json(&value, now) {
            return Ok(data);
        }
    }

    let rolling_percent = extract_f64(
        r"rollingUsage[^}]*?usagePercent\s*:\s*([0-9]+(?:\.[0-9]+)?)",
        text,
    )
    .ok_or_else(|| ProviderError::Parse("opencode missing rolling usagePercent".into()))?;
    let rolling_reset = extract_i64(r"rollingUsage[^}]*?resetInSec\s*:\s*([0-9]+)", text)
        .ok_or_else(|| ProviderError::Parse("opencode missing rolling resetInSec".into()))?;
    let weekly_percent = extract_f64(
        r"weeklyUsage[^}]*?usagePercent\s*:\s*([0-9]+(?:\.[0-9]+)?)",
        text,
    )
    .ok_or_else(|| ProviderError::Parse("opencode missing weekly usagePercent".into()))?;
    let weekly_reset = extract_i64(r"weeklyUsage[^}]*?resetInSec\s*:\s*([0-9]+)", text)
        .ok_or_else(|| ProviderError::Parse("opencode missing weekly resetInSec".into()))?;

    Ok(OpenCodeUsageData {
        rolling_usage_percent: rolling_percent as f32,
        weekly_usage_percent: weekly_percent as f32,
        rolling_reset_in_sec: rolling_reset,
        weekly_reset_in_sec: weekly_reset,
        monthly_usage_percent: None,
        monthly_reset_in_sec: None,
    })
}

pub fn snapshot_from_usage(
    data: &OpenCodeUsageData,
    provider: ProviderId,
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let now = parse_timestamp(updated_at).unwrap_or_else(OffsetDateTime::now_utc);
    let primary_reset = reset_at(now, data.rolling_reset_in_sec);
    let weekly_reset = reset_at(now, data.weekly_reset_in_sec);

    let mut snapshot = UsageSnapshot::new(
        provider,
        UsageWindow::new("5-hour", data.rolling_usage_percent, primary_reset),
        Some(UsageWindow::new(
            "Weekly",
            data.weekly_usage_percent,
            weekly_reset,
        )),
        updated_at,
        source,
    );

    if let (Some(percent), Some(reset_in_sec)) =
        (data.monthly_usage_percent, data.monthly_reset_in_sec)
    {
        snapshot = snapshot.with_tertiary(UsageWindow::new(
            "Monthly",
            percent,
            reset_at(now, reset_in_sec),
        ));
    }

    Ok(snapshot)
}

fn parse_usage_json(value: &Value, now: OffsetDateTime) -> Option<OpenCodeUsageData> {
    parse_usage_dictionary(value, now).or_else(|| {
        for key in ["data", "result", "usage", "billing", "payload"] {
            if let Some(nested) = value.get(key) {
                if let Some(data) = parse_usage_dictionary(nested, now) {
                    return Some(data);
                }
            }
        }
        None
    })
}

fn parse_usage_dictionary(value: &Value, now: OffsetDateTime) -> Option<OpenCodeUsageData> {
    if let Some(usage) = value.get("usage") {
        if let Some(data) = parse_usage_dictionary(usage, now) {
            return Some(data);
        }
    }

    let rolling = first_object(
        value,
        &[
            "rollingUsage",
            "rolling",
            "rolling_usage",
            "rollingWindow",
            "rolling_window",
        ],
    )?;
    let weekly = first_object(
        value,
        &[
            "weeklyUsage",
            "weekly",
            "weekly_usage",
            "weeklyWindow",
            "weekly_window",
        ],
    )?;
    let monthly = first_object(
        value,
        &[
            "monthlyUsage",
            "monthly",
            "monthly_usage",
            "monthlyWindow",
            "monthly_window",
        ],
    );

    build_snapshot(rolling, weekly, monthly, now)
}

fn build_snapshot(
    rolling: &Value,
    weekly: &Value,
    monthly: Option<&Value>,
    now: OffsetDateTime,
) -> Option<OpenCodeUsageData> {
    let rolling_percent = window_percent(rolling)?;
    let weekly_percent = window_percent(weekly)?;
    let rolling_reset = window_reset_in_sec(rolling, now)?;
    let weekly_reset = window_reset_in_sec(weekly, now)?;
    let (monthly_percent, monthly_reset) = monthly
        .and_then(|value| Some((window_percent(value)?, window_reset_in_sec(value, now)?)))
        .map(|(percent, reset)| (Some(percent), Some(reset)))
        .unwrap_or((None, None));

    Some(OpenCodeUsageData {
        rolling_usage_percent: rolling_percent,
        weekly_usage_percent: weekly_percent,
        rolling_reset_in_sec: rolling_reset,
        weekly_reset_in_sec: weekly_reset,
        monthly_usage_percent: monthly_percent,
        monthly_reset_in_sec: monthly_reset,
    })
}

fn first_object<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .find_map(|key| value.get(*key))
        .filter(|value| value.is_object())
}

fn window_percent(value: &Value) -> Option<f32> {
    for key in [
        "usagePercent",
        "usedPercent",
        "percentUsed",
        "percent",
        "usage_percent",
        "used_percent",
        "utilization",
        "utilizationPercent",
        "utilization_percent",
        "usage",
    ] {
        if let Some(number) = json_number(value.get(key)?) {
            let percent = if number <= 1.0 {
                number * 100.0
            } else {
                number
            };
            return Some(percent.clamp(0.0, 100.0) as f32);
        }
    }

    let used = json_number(value.get("used")?)?;
    let limit = json_number(value.get("limit")?)?;
    if limit <= 0.0 {
        return None;
    }
    Some(((used / limit) * 100.0).clamp(0.0, 100.0) as f32)
}

fn window_reset_in_sec(value: &Value, now: OffsetDateTime) -> Option<i64> {
    for key in [
        "resetInSec",
        "resetInSeconds",
        "resetSeconds",
        "reset_sec",
        "reset_in_sec",
        "resetsInSec",
        "resetsInSeconds",
        "resetIn",
        "resetSec",
    ] {
        if let Some(seconds) = value.get(key).and_then(json_i64) {
            return Some(seconds);
        }
    }

    for key in [
        "resetAt",
        "resetsAt",
        "reset_at",
        "resets_at",
        "nextReset",
        "next_reset",
    ] {
        if let Some(raw) = value.get(key).and_then(|value| value.as_str()) {
            if let Ok(reset_at) = OffsetDateTime::parse(raw, &Rfc3339) {
                return Some((reset_at - now).whole_seconds().max(0));
            }
        }
    }

    None
}

fn json_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn json_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn extract_f64(pattern: &str, text: &str) -> Option<f64> {
    let regex = Regex::new(pattern).ok()?;
    regex
        .captures(text)
        .and_then(|capture| capture.get(1))
        .and_then(|value| value.as_str().parse().ok())
}

fn extract_i64(pattern: &str, text: &str) -> Option<i64> {
    extract_f64(pattern, text).map(|value| value as i64)
}

fn reset_at(now: OffsetDateTime, reset_in_sec: i64) -> Option<String> {
    (now + Duration::seconds(reset_in_sec))
        .format(&Rfc3339)
        .ok()
}

fn parse_timestamp(raw: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(raw, &Rfc3339).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_workspace_ids_from_serialized_payload() {
        let text = r#";0x00000089;((self.$R=self.$R||{})["codexbar"]]=[],($R=>$R[0]=[$R[1]={id:"wrk_01K6AR1ZET89H8NB691FQ2C2VB",name:"Default",slug:null}])"#;
        assert_eq!(
            parse_workspace_ids(text),
            vec!["wrk_01K6AR1ZET89H8NB691FQ2C2VB".to_string()]
        );
    }

    #[test]
    fn parses_subscription_from_regex_payload() {
        let text = r#"$R[16]($R[30],$R[41]={rollingUsage:$R[42]={status:"ok",resetInSec:5944,usagePercent:17},weeklyUsage:$R[43]={status:"ok",resetInSec:278201,usagePercent:75}});"#;
        let data = parse_subscription(text, "1970-01-01T00:00:00Z").expect("parse");
        assert_eq!(data.rolling_usage_percent, 17.0);
        assert_eq!(data.weekly_usage_percent, 75.0);
        assert_eq!(data.rolling_reset_in_sec, 5944);
        assert_eq!(data.weekly_reset_in_sec, 278_201);
    }

    #[test]
    fn maps_open_code_snapshot_with_five_hour_and_weekly_labels() {
        let data = OpenCodeUsageData {
            rolling_usage_percent: 17.0,
            weekly_usage_percent: 75.0,
            rolling_reset_in_sec: 5944,
            weekly_reset_in_sec: 278_201,
            monthly_usage_percent: None,
            monthly_reset_in_sec: None,
        };
        let snapshot = snapshot_from_usage(
            &data,
            ProviderId::OpenCode,
            "2026-05-22T12:00:00Z",
            "opencode-web",
        )
        .expect("snapshot");

        assert_eq!(snapshot.primary.label, "5-hour");
        assert_eq!(snapshot.secondary.as_ref().expect("weekly").label, "Weekly");
        assert!(snapshot.tertiary.is_none());
    }
}
