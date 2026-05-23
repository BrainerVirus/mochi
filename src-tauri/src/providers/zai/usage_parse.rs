//! z.ai quota API response mapping.
//!
//! Logic derived from CodexBar `ZaiUsageStats.swift` (MIT).

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LimitType {
    TimeLimit,
    TokensLimit,
}

#[derive(Debug, Clone)]
pub(crate) struct LimitEntry {
    limit_type: LimitType,
    unit: i32,
    number: i32,
    used_percent: f32,
    resets_at: Option<String>,
    label: String,
}

#[derive(Debug, Deserialize)]
struct QuotaResponse {
    code: i32,
    msg: String,
    success: bool,
    data: Option<QuotaData>,
}

#[derive(Debug, Deserialize)]
struct QuotaData {
    limits: Vec<LimitRaw>,
    #[serde(default, rename = "planName")]
    plan_name: Option<String>,
    #[serde(default)]
    plan: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LimitRaw {
    #[serde(rename = "type")]
    limit_type: String,
    unit: i32,
    number: i32,
    #[serde(default)]
    usage: Option<i64>,
    #[serde(rename = "currentValue", default)]
    current_value: Option<i64>,
    #[serde(default)]
    remaining: Option<i64>,
    percentage: i32,
    #[serde(rename = "nextResetTime", default)]
    next_reset_time: Option<i64>,
}

pub fn parse_quota_response(body: &str) -> ProviderResult<(Vec<LimitEntry>, Option<String>)> {
    let response: QuotaResponse =
        serde_json::from_str(body).map_err(|error| ProviderError::Parse(error.to_string()))?;

    if !response.success || response.code != 200 {
        return Err(ProviderError::Parse(format!(
            "z.ai quota API error: {}",
            response.msg
        )));
    }

    let data = response
        .data
        .ok_or_else(|| ProviderError::Parse("z.ai quota missing data".into()))?;

    let plan_name = data
        .plan_name
        .or(data.plan)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut token_limits = Vec::new();
    let mut time_limit = None;

    for raw in data.limits {
        let Some(entry) = limit_from_raw(raw) else {
            continue;
        };
        match entry.limit_type {
            LimitType::TokensLimit => token_limits.push(entry),
            LimitType::TimeLimit => time_limit = Some(entry),
        }
    }

    if token_limits.is_empty() && time_limit.is_none() {
        return Err(ProviderError::Parse(
            "z.ai quota missing usable limits".into(),
        ));
    }

    token_limits.sort_by_key(|entry| entry.window_minutes().unwrap_or(i32::MAX));

    let mut limits = Vec::new();
    if token_limits.len() >= 2 {
        let shortest = token_limits.first().expect("sorted");
        let longest = token_limits.last().expect("sorted");
        limits.push(longest.clone());
        if let Some(time) = time_limit {
            limits.push(time);
        }
        limits.push(shortest.clone());
    } else if let Some(token) = token_limits.into_iter().next() {
        limits.push(token);
        if let Some(time) = time_limit {
            limits.push(time);
        }
    } else if let Some(time) = time_limit {
        limits.push(time);
    }

    Ok((limits, plan_name))
}

pub fn snapshot_from_limits(
    limits: &[LimitEntry],
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let primary = limits
        .first()
        .ok_or_else(|| ProviderError::Parse("z.ai usage missing rate windows".into()))?;

    let mut snapshot = UsageSnapshot::new(
        ProviderId::Zai,
        usage_window_from_entry(primary),
        limits.get(1).map(usage_window_from_entry),
        updated_at,
        source,
    );

    if let Some(tertiary) = limits.get(2) {
        snapshot = snapshot.with_tertiary(usage_window_from_entry(tertiary));
    }

    Ok(snapshot)
}

fn limit_from_raw(raw: LimitRaw) -> Option<LimitEntry> {
    let limit_type = match raw.limit_type.as_str() {
        "TOKENS_LIMIT" => LimitType::TokensLimit,
        "TIME_LIMIT" => LimitType::TimeLimit,
        _ => return None,
    };

    let used_percent = used_percent_from_raw(&raw)?;
    let label = window_label(limit_type, raw.unit, raw.number);

    Some(LimitEntry {
        limit_type,
        unit: raw.unit,
        number: raw.number,
        used_percent,
        resets_at: raw.next_reset_time.and_then(format_reset_ms),
        label,
    })
}

fn used_percent_from_raw(raw: &LimitRaw) -> Option<f32> {
    if raw.percentage > 0 {
        return Some(raw.percentage as f32);
    }
    if let (Some(current), Some(usage)) = (raw.current_value, raw.usage) {
        if usage > 0 {
            return Some(((current as f32 / usage as f32) * 100.0).clamp(0.0, 100.0));
        }
    }
    if let (Some(remaining), Some(usage)) = (raw.remaining, raw.usage) {
        if usage > 0 {
            let used = usage.saturating_sub(remaining.max(0)) as f32;
            return Some(((used / usage as f32) * 100.0).clamp(0.0, 100.0));
        }
    }
    Some(0.0)
}

fn window_label(limit_type: LimitType, unit: i32, number: i32) -> String {
    if limit_type == LimitType::TimeLimit && unit == 5 && number == 1 {
        return "Monthly".into();
    }

    let unit_label = match unit {
        5 => "minute",
        3 => "hour",
        1 => "day",
        6 => "week",
        _ => return "Quota".into(),
    };
    let plural = if number == 1 {
        unit_label.to_string()
    } else {
        format!("{unit_label}s")
    };
    format!("{number} {plural}")
}

fn usage_window_from_entry(entry: &LimitEntry) -> UsageWindow {
    UsageWindow::new(&entry.label, entry.used_percent, entry.resets_at.clone())
}

impl LimitEntry {
    fn window_minutes(&self) -> Option<i32> {
        if self.number <= 0 {
            return None;
        }
        match self.unit {
            5 => Some(self.number),
            3 => Some(self.number * 60),
            1 => Some(self.number * 24 * 60),
            6 => Some(self.number * 7 * 24 * 60),
            _ => None,
        }
    }
}

fn format_reset_ms(ms: i64) -> Option<String> {
    let secs = ms / 1000;
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()?
        .format(&Rfc3339)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_quota_fixture_to_token_and_monthly_windows() {
        let (limits, plan) =
            parse_quota_response(include_str!("fixtures/quota_response.json")).expect("parse");
        assert_eq!(plan.as_deref(), Some("Pro"));
        assert_eq!(limits.len(), 2);

        let snapshot =
            snapshot_from_limits(&limits, "2026-05-22T12:00:00Z", "zai-api-quota").expect("snap");

        assert_eq!(snapshot.provider, ProviderId::Zai);
        assert_eq!(snapshot.primary.label, "5 hours");
        assert_eq!(snapshot.primary.used_percent, 34.0);
        assert_eq!(snapshot.secondary.as_ref().expect("monthly").label, "Monthly");
        assert_eq!(snapshot.secondary.expect("monthly").used_percent, 40.0);
    }
}
