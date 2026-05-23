//! Copilot `copilot_internal/user` JSON mapping.
//!
//! Logic derived from CodexBar `CopilotUsageModels.swift` and `CopilotUsageFetcher.swift` (MIT).

use serde::Deserialize;
use serde_json::Value;

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone)]
pub struct CopilotUsageResponse {
    pub quota_snapshots: QuotaSnapshots,
    #[allow(dead_code)]
    pub copilot_plan: String,
}

#[derive(Debug, Clone, Default)]
pub struct QuotaSnapshots {
    pub premium_interactions: Option<QuotaSnapshot>,
    pub chat: Option<QuotaSnapshot>,
}

#[derive(Debug, Clone)]
pub struct QuotaSnapshot {
    pub entitlement: f64,
    pub remaining: f64,
    pub percent_remaining: f64,
    pub quota_id: String,
    pub has_percent_remaining: bool,
}

impl QuotaSnapshot {
    pub fn used_percent(&self) -> f32 {
        if self.has_percent_remaining {
            (100.0 - self.percent_remaining).clamp(0.0, 100.0) as f32
        } else {
            0.0
        }
    }

    fn is_placeholder(&self) -> bool {
        self.entitlement == 0.0
            && self.remaining == 0.0
            && self.percent_remaining == 0.0
            && self.quota_id.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawCopilotUsageResponse {
    #[serde(rename = "quota_snapshots", default)]
    quota_snapshots: Option<RawQuotaSnapshots>,
    #[serde(rename = "copilot_plan", default)]
    copilot_plan: Option<String>,
    #[serde(rename = "monthly_quotas", default)]
    monthly_quotas: Option<RawQuotaCounts>,
    #[serde(rename = "limited_user_quotas", default)]
    limited_user_quotas: Option<RawQuotaCounts>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawQuotaSnapshots {
    #[serde(rename = "premium_interactions", default)]
    premium_interactions: Option<RawQuotaSnapshot>,
    chat: Option<RawQuotaSnapshot>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawQuotaSnapshot {
    entitlement: Option<Value>,
    remaining: Option<Value>,
    #[serde(rename = "percent_remaining", default)]
    percent_remaining: Option<Value>,
    #[serde(rename = "quota_id", default)]
    quota_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawQuotaCounts {
    chat: Option<Value>,
    completions: Option<Value>,
}

pub fn parse_usage_response(value: &Value) -> ProviderResult<CopilotUsageResponse> {
    let raw: RawCopilotUsageResponse = serde_json::from_value(value.clone())
        .map_err(|error| ProviderError::Parse(error.to_string()))?;

    let direct = raw
        .quota_snapshots
        .as_ref()
        .map(parse_raw_snapshots)
        .transpose()?
        .unwrap_or_default();

    let monthly = make_quota_snapshots(
        raw.monthly_quotas.as_ref(),
        raw.limited_user_quotas.as_ref(),
    );

    let premium = usable_snapshot(direct.premium_interactions.as_ref()).or_else(|| {
        monthly
            .as_ref()
            .and_then(|s| usable_snapshot(s.premium_interactions.as_ref()))
    });
    let chat = usable_snapshot(direct.chat.as_ref()).or_else(|| {
        monthly
            .as_ref()
            .and_then(|s| usable_snapshot(s.chat.as_ref()))
    });

    let mut quota_snapshots = QuotaSnapshots {
        premium_interactions: premium,
        chat,
    };

    if quota_snapshots.premium_interactions.is_none() || quota_snapshots.chat.is_none() {
        apply_dynamic_snapshot_fallback(value, &mut quota_snapshots);
    }

    if quota_snapshots.premium_interactions.is_none() && quota_snapshots.chat.is_none() {
        return Err(ProviderError::Parse(
            "copilot usage missing usable quota snapshots".into(),
        ));
    }

    Ok(CopilotUsageResponse {
        quota_snapshots,
        copilot_plan: raw.copilot_plan.unwrap_or_else(|| "unknown".into()),
    })
}

pub fn snapshot_from_usage_response(
    response: &CopilotUsageResponse,
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let premium = response
        .quota_snapshots
        .premium_interactions
        .as_ref()
        .filter(|snapshot| snapshot.has_percent_remaining)
        .map(|snapshot| UsageWindow::new("Premium", snapshot.used_percent(), None));

    let chat = response
        .quota_snapshots
        .chat
        .as_ref()
        .filter(|snapshot| snapshot.has_percent_remaining)
        .map(|snapshot| UsageWindow::new("Chat", snapshot.used_percent(), None));

    let (primary, secondary) = match (premium, chat) {
        (Some(primary), secondary) => (primary, secondary),
        (None, Some(chat)) => (chat, None),
        (None, None) => {
            return Err(ProviderError::Parse(
                "copilot usage missing rate windows".into(),
            ));
        }
    };

    Ok(UsageSnapshot::new(
        ProviderId::Copilot,
        primary,
        secondary,
        updated_at,
        source,
    ))
}

fn parse_raw_snapshots(raw: &RawQuotaSnapshots) -> ProviderResult<QuotaSnapshots> {
    Ok(QuotaSnapshots {
        premium_interactions: raw
            .premium_interactions
            .as_ref()
            .map(parse_raw_snapshot)
            .transpose()?,
        chat: raw.chat.as_ref().map(parse_raw_snapshot).transpose()?,
    })
}

fn parse_raw_snapshot(raw: &RawQuotaSnapshot) -> ProviderResult<QuotaSnapshot> {
    let entitlement = decode_number(raw.entitlement.as_ref()).unwrap_or(0.0);
    let remaining = decode_number(raw.remaining.as_ref()).unwrap_or(0.0);
    let quota_id = raw.quota_id.clone().unwrap_or_default();

    let (percent_remaining, has_percent_remaining) =
        if let Some(percent) = decode_number(raw.percent_remaining.as_ref()) {
            (percent, true)
        } else if entitlement > 0.0 {
            ((remaining / entitlement) * 100.0, true)
        } else {
            (0.0, false)
        };

    Ok(QuotaSnapshot {
        entitlement,
        remaining,
        percent_remaining,
        quota_id,
        has_percent_remaining,
    })
}

fn make_quota_snapshots(
    monthly: Option<&RawQuotaCounts>,
    limited: Option<&RawQuotaCounts>,
) -> Option<QuotaSnapshots> {
    let premium = make_quota_snapshot(
        monthly.and_then(|counts| decode_number(counts.completions.as_ref())),
        limited.and_then(|counts| decode_number(counts.completions.as_ref())),
        "completions",
    );
    let chat = make_quota_snapshot(
        monthly.and_then(|counts| decode_number(counts.chat.as_ref())),
        limited.and_then(|counts| decode_number(counts.chat.as_ref())),
        "chat",
    );

    if premium.is_none() && chat.is_none() {
        return None;
    }

    Some(QuotaSnapshots {
        premium_interactions: premium,
        chat,
    })
}

fn make_quota_snapshot(
    monthly: Option<f64>,
    limited: Option<f64>,
    quota_id: &str,
) -> Option<QuotaSnapshot> {
    let monthly = monthly?;
    let limited = limited?;
    if monthly <= 0.0 {
        return None;
    }

    let remaining = limited.max(0.0);
    let percent_remaining = ((remaining / monthly) * 100.0).clamp(0.0, 100.0);

    Some(QuotaSnapshot {
        entitlement: monthly,
        remaining,
        percent_remaining,
        quota_id: quota_id.to_string(),
        has_percent_remaining: true,
    })
}

fn usable_snapshot(snapshot: Option<&QuotaSnapshot>) -> Option<QuotaSnapshot> {
    let snapshot = snapshot?;
    if snapshot.is_placeholder() || !snapshot.has_percent_remaining {
        return None;
    }
    Some(snapshot.clone())
}

fn apply_dynamic_snapshot_fallback(value: &Value, snapshots: &mut QuotaSnapshots) {
    let Some(object) = value.as_object() else {
        return;
    };

    let mut fallback_premium = None;
    let mut fallback_chat = None;
    let mut first_usable = None;

    for (key, nested) in object {
        if matches!(
            key.as_str(),
            "copilot_plan" | "assigned_date" | "quota_reset_date"
        ) {
            continue;
        }

        let Ok(raw) = serde_json::from_value::<RawQuotaSnapshot>(nested.clone()) else {
            continue;
        };
        let Ok(snapshot) = parse_raw_snapshot(&raw) else {
            continue;
        };
        if snapshot.is_placeholder() || !snapshot.has_percent_remaining {
            continue;
        }

        if first_usable.is_none() {
            first_usable = Some(snapshot.clone());
        }

        let name = key.to_lowercase();
        if fallback_chat.is_none() && name.contains("chat") {
            fallback_chat = Some(snapshot.clone());
            continue;
        }
        if fallback_premium.is_none()
            && (name.contains("premium") || name.contains("completion") || name.contains("code"))
        {
            fallback_premium = Some(snapshot);
        }
    }

    if snapshots.premium_interactions.is_none() {
        snapshots.premium_interactions = fallback_premium;
    }
    if snapshots.chat.is_none() {
        snapshots.chat = fallback_chat.or(first_usable);
    }
}

fn decode_number(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_premium_and_chat_fixture() {
        let value: Value =
            serde_json::from_str(include_str!("fixtures/usage_premium_chat.json")).expect("json");
        let response = parse_usage_response(&value).expect("parse");
        let snapshot = snapshot_from_usage_response(
            &response,
            "2026-05-22T12:00:00Z",
            "copilot-oauth-internal",
        )
        .expect("snapshot");

        assert_eq!(snapshot.primary.label, "Premium");
        assert_eq!(snapshot.primary.used_percent, 10.0);
        assert_eq!(snapshot.secondary.as_ref().expect("chat").label, "Chat");
        assert_eq!(snapshot.secondary.expect("chat").used_percent, 50.0);
    }

    #[test]
    fn maps_chat_only_fixture_to_primary_chat() {
        let value: Value =
            serde_json::from_str(include_str!("fixtures/usage_chat_only.json")).expect("json");
        let response = parse_usage_response(&value).expect("parse");
        let snapshot = snapshot_from_usage_response(
            &response,
            "2026-05-22T12:00:00Z",
            "copilot-oauth-internal",
        )
        .expect("snapshot");

        assert_eq!(snapshot.primary.label, "Chat");
        assert_eq!(snapshot.primary.used_percent, 62.5);
        assert!(snapshot.secondary.is_none());
    }

    #[test]
    fn derives_snapshots_from_monthly_and_limited_quotas() {
        let value: Value =
            serde_json::from_str(include_str!("fixtures/usage_monthly_quotas.json")).expect("json");
        let response = parse_usage_response(&value).expect("parse");

        let premium = response
            .quota_snapshots
            .premium_interactions
            .as_ref()
            .expect("premium");
        assert_eq!(premium.remaining, 60.0);
        assert!((premium.percent_remaining - 20.0).abs() < f64::EPSILON);

        let chat = response.quota_snapshots.chat.as_ref().expect("chat");
        assert_eq!(chat.remaining, 125.0);
        assert!((chat.percent_remaining - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rejects_empty_quota_payload() {
        let value = serde_json::json!({"copilot_plan": "free", "quota_snapshots": {}});
        let error = parse_usage_response(&value).expect_err("empty quotas");
        assert!(matches!(error, ProviderError::Parse(_)));
    }
}
