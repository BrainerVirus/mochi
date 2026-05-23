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
    pub renews_at: Option<String>,
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
    let monthly_percent = extract_f64(
        r"monthlyUsage[^}]*?usagePercent\s*:\s*([0-9]+(?:\.[0-9]+)?)",
        text,
    );
    let monthly_reset = extract_i64(r"monthlyUsage[^}]*?resetInSec\s*:\s*([0-9]+)", text);

    Ok(OpenCodeUsageData {
        rolling_usage_percent: rolling_percent as f32,
        weekly_usage_percent: weekly_percent as f32,
        rolling_reset_in_sec: rolling_reset,
        weekly_reset_in_sec: weekly_reset,
        monthly_usage_percent: monthly_percent.map(|value| value as f32),
        monthly_reset_in_sec: monthly_reset,
        renews_at: None,
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

    if let Some(renews_at) = &data.renews_at {
        snapshot = snapshot.with_extra_windows(vec![UsageWindow::new(
            "Renews",
            0.0,
            Some(renews_at.clone()),
        )]);
    }

    Ok(snapshot)
}

fn parse_usage_json(value: &Value, now: OffsetDateTime) -> Option<OpenCodeUsageData> {
    let renews_at = renew_at_value(value, now);

    if let Some(mut data) = parse_usage_dictionary(value, now) {
        if data.renews_at.is_none() {
            data.renews_at = renews_at.clone();
        }
        return Some(data);
    }

    for key in ["data", "result", "usage", "billing", "payload"] {
        if let Some(nested) = value.get(key) {
            if let Some(mut data) = parse_usage_dictionary(nested, now) {
                if data.renews_at.is_none() {
                    data.renews_at = renews_at.clone();
                }
                return Some(data);
            }
        }
    }

    parse_usage_nested(value, now, 0).or_else(|| parse_usage_from_candidates(value, now))
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
        .map(|mut data| {
            if data.renews_at.is_none() {
                data.renews_at = renew_at_value(value, now);
            }
            data
        })
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
        renews_at: None,
    })
}

fn parse_usage_nested(value: &Value, now: OffsetDateTime, depth: usize) -> Option<OpenCodeUsageData> {
    let object = value.as_object()?;
    if depth > 3 {
        return None;
    }

    let mut rolling = None;
    let mut weekly = None;
    let mut monthly = None;

    for (key, nested) in object {
        let Some(nested_object) = nested.as_object() else {
            continue;
        };
        let lower = key.to_lowercase();
        if lower.contains("rolling") || lower.contains("hour") || lower.contains("5h") {
            rolling = Some(Value::Object(nested_object.clone()));
        } else if lower.contains("weekly") || lower.contains("week") {
            weekly = Some(Value::Object(nested_object.clone()));
        } else if lower.contains("monthly") || lower.contains("month") {
            monthly = Some(Value::Object(nested_object.clone()));
        }
    }

    if let (Some(rolling), Some(weekly)) = (&rolling, &weekly) {
        if let Some(data) = build_snapshot(rolling, weekly, monthly.as_ref(), now) {
            return Some(data);
        }
    }

    for nested in object.values() {
        if nested.is_object() {
            if let Some(data) = parse_usage_nested(nested, now, depth + 1) {
                return Some(data);
            }
        }
    }

    None
}

#[derive(Clone)]
struct WindowCandidate {
    percent: f32,
    reset_in_sec: i64,
    path_lower: String,
}

fn parse_usage_from_candidates(value: &Value, now: OffsetDateTime) -> Option<OpenCodeUsageData> {
    let mut candidates = Vec::new();
    collect_window_candidates(value, now, &[], &mut candidates);
    if candidates.is_empty() {
        return None;
    }

    let rolling_candidates: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            candidate.path_lower.contains("rolling")
                || candidate.path_lower.contains("hour")
                || candidate.path_lower.contains("5h")
                || candidate.path_lower.contains("5-hour")
        })
        .cloned()
        .collect();
    let weekly_candidates: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            candidate.path_lower.contains("weekly") || candidate.path_lower.contains("week")
        })
        .cloned()
        .collect();
    let monthly_candidates: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            candidate.path_lower.contains("monthly") || candidate.path_lower.contains("month")
        })
        .cloned()
        .collect();

    let rolling = pick_candidate(&rolling_candidates, &candidates, true)?;
    let weekly = pick_candidate(
        &weekly_candidates
            .iter()
            .filter(|candidate| candidate.path_lower != rolling.path_lower)
            .cloned()
            .collect::<Vec<_>>(),
        &candidates,
        false,
    )?;
    let monthly = pick_candidate(
        &monthly_candidates
            .iter()
            .filter(|candidate| {
                candidate.path_lower != rolling.path_lower && candidate.path_lower != weekly.path_lower
            })
            .cloned()
            .collect::<Vec<_>>(),
        &candidates,
        false,
    );

    Some(OpenCodeUsageData {
        rolling_usage_percent: rolling.percent,
        weekly_usage_percent: weekly.percent,
        rolling_reset_in_sec: rolling.reset_in_sec,
        weekly_reset_in_sec: weekly.reset_in_sec,
        monthly_usage_percent: monthly.as_ref().map(|candidate| candidate.percent),
        monthly_reset_in_sec: monthly.map(|candidate| candidate.reset_in_sec),
        renews_at: renew_at_value(value, now),
    })
}

fn collect_window_candidates(
    value: &Value,
    now: OffsetDateTime,
    path: &[String],
    out: &mut Vec<WindowCandidate>,
) {
    if let Some(object) = value.as_object() {
        if let (Some(percent), Some(reset_in_sec)) =
            (window_percent(value), window_reset_in_sec(value, now))
        {
            out.push(WindowCandidate {
                percent,
                reset_in_sec,
                path_lower: path.join(".").to_lowercase(),
            });
        }
        for (key, nested) in object {
            let mut next_path = path.to_vec();
            next_path.push(key.clone());
            collect_window_candidates(nested, now, &next_path, out);
        }
        return;
    }

    if let Some(array) = value.as_array() {
        for (index, nested) in array.iter().enumerate() {
            let mut next_path = path.to_vec();
            next_path.push(format!("[{index}]"));
            collect_window_candidates(nested, now, &next_path, out);
        }
    }
}

fn pick_candidate(
    preferred: &[WindowCandidate],
    fallback: &[WindowCandidate],
    pick_shorter: bool,
) -> Option<WindowCandidate> {
    pick_from_candidates(preferred, pick_shorter)
        .or_else(|| pick_from_candidates(fallback, pick_shorter))
}

fn pick_from_candidates(candidates: &[WindowCandidate], pick_shorter: bool) -> Option<WindowCandidate> {
    if candidates.is_empty() {
        return None;
    }

    candidates
        .iter()
        .cloned()
        .min_by(|lhs, rhs| {
            if pick_shorter {
                if lhs.reset_in_sec == rhs.reset_in_sec {
                    rhs.percent
                        .partial_cmp(&lhs.percent)
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    lhs.reset_in_sec.cmp(&rhs.reset_in_sec)
                }
            } else if lhs.reset_in_sec == rhs.reset_in_sec {
                rhs.percent
                    .partial_cmp(&lhs.percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                rhs.reset_in_sec.cmp(&lhs.reset_in_sec)
            }
        })
}

fn renew_at_value(value: &Value, now: OffsetDateTime) -> Option<String> {
    for key in ["renewAt", "renew_at"] {
        if let Some(raw) = value.get(key) {
            if let Some(timestamp) = parse_date_value(raw, now) {
                return Some(timestamp);
            }
        }
    }
    None
}

fn parse_date_value(value: &Value, now: OffsetDateTime) -> Option<String> {
    if let Some(number) = json_number(value) {
        let seconds = if number > 1_000_000_000_000.0 {
            (number / 1000.0) as i64
        } else if number > 1_000_000_000.0 {
            number as i64
        } else {
            return None;
        };
        return OffsetDateTime::from_unix_timestamp(seconds)
            .ok()
            .and_then(|timestamp| timestamp.format(&Rfc3339).ok());
    }

    if let Some(text) = value.as_str() {
        if let Ok(parsed) = OffsetDateTime::parse(text, &Rfc3339) {
            return parsed.format(&Rfc3339).ok();
        }
        if let Ok(number) = text.parse::<f64>() {
            return parse_date_value(&Value::from(number), now);
        }
    }

    let _ = now;
    None
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
            renews_at: None,
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

    #[test]
    fn parses_open_code_go_payload_with_monthly_window() {
        let text = r#"$R[16]($R[30],$R[41]={rollingUsage:$R[42]={status:"ok",resetInSec:5944,usagePercent:100},weeklyUsage:$R[43]={status:"ok",resetInSec:278201,usagePercent:99},monthlyUsage:$R[44]={status:"ok",resetInSec:880201,usagePercent:12}});"#;
        let data = parse_subscription(text, "1970-01-01T00:00:00Z").expect("parse");
        assert_eq!(data.rolling_usage_percent, 100.0);
        assert_eq!(data.weekly_usage_percent, 99.0);
        assert_eq!(data.monthly_usage_percent, Some(12.0));

        let snapshot = snapshot_from_usage(
            &data,
            ProviderId::OpenCodeGo,
            "2026-05-22T12:00:00Z",
            "opencode-go-web",
        )
        .expect("snapshot");
        assert_eq!(snapshot.primary.label, "5-hour");
        assert_eq!(snapshot.tertiary.as_ref().expect("monthly").label, "Monthly");
    }

    #[test]
    fn parses_live_go_page_hydration_payload() {
        let text = r#"_$HY.r["lite.subscription.get[\"wrk_LIVE123\"]"]=$R[17]=$R[2]($R[18]={p:0,s:0,f:0});$R[24]($R[18],$R[27]={mine:!0,useBalance:!1,rollingUsage:$R[28]={status:"ok",resetInSec:17591,usagePercent:0},weeklyUsage:$R[29]={status:"ok",resetInSec:444552,usagePercent:0},monthlyUsage:$R[30]={status:"ok",resetInSec:2591424,usagePercent:0}});"#;
        let data = parse_subscription(text, "1970-01-01T00:00:00Z").expect("parse");
        assert_eq!(data.rolling_reset_in_sec, 17_591);
        assert_eq!(data.weekly_reset_in_sec, 444_552);
        assert_eq!(data.monthly_reset_in_sec, Some(2_591_424));
    }
}
