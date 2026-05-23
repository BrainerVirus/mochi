//! Gemini quota API response mapping.
//!
//! Logic derived from CodexBar `GeminiStatusProbe.swift` (MIT).

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone)]
pub struct GeminiModelQuota {
    pub model_id: String,
    pub percent_left: f32,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct QuotaBucket {
    #[serde(rename = "modelId", default)]
    model_id: Option<String>,
    #[serde(rename = "remainingFraction", default)]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime", default)]
    reset_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct QuotaResponse {
    buckets: Option<Vec<QuotaBucket>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminiUserTier {
    Free,
    Standard,
    Legacy,
}

#[derive(Debug, Clone, Default)]
pub struct JwtClaims {
    pub email: Option<String>,
    pub hosted_domain: Option<String>,
}

pub fn parse_quota_response(data: &str) -> ProviderResult<Vec<GeminiModelQuota>> {
    let response: QuotaResponse =
        serde_json::from_str(data).map_err(|error| ProviderError::Parse(error.to_string()))?;

    let buckets = response
        .buckets
        .filter(|items| !items.is_empty())
        .ok_or_else(|| ProviderError::Parse("no quota buckets in response".into()))?;

    let mut model_quota_map: std::collections::BTreeMap<String, (f64, Option<String>)> =
        std::collections::BTreeMap::new();

    for bucket in buckets {
        let Some(model_id) = bucket.model_id else {
            continue;
        };
        let Some(fraction) = bucket.remaining_fraction else {
            continue;
        };

        model_quota_map
            .entry(model_id)
            .and_modify(|existing| {
                if fraction < existing.0 {
                    *existing = (fraction, bucket.reset_time.clone());
                }
            })
            .or_insert((fraction, bucket.reset_time));
    }

    if model_quota_map.is_empty() {
        return Err(ProviderError::Parse(
            "no usable quota buckets in response".into(),
        ));
    }

    Ok(model_quota_map
        .into_iter()
        .map(|(model_id, (fraction, reset_time))| GeminiModelQuota {
            model_id,
            percent_left: (fraction * 100.0) as f32,
            resets_at: reset_time.and_then(|value| parse_reset_timestamp(&value)),
        })
        .collect())
}

pub fn snapshot_from_quotas(
    quotas: &[GeminiModelQuota],
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> {
    let pro_min = quotas
        .iter()
        .filter(|quota| is_pro_model(&quota.model_id))
        .min_by(|left, right| {
            left.percent_left
                .partial_cmp(&right.percent_left)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    let flash_min = quotas
        .iter()
        .filter(|quota| is_flash_model(&quota.model_id))
        .min_by(|left, right| {
            left.percent_left
                .partial_cmp(&right.percent_left)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

    let primary = pro_min
        .map(|quota| usage_window_from_quota("Pro", quota))
        .or_else(|| flash_min.map(|quota| usage_window_from_quota("Flash", quota)))
        .ok_or_else(|| ProviderError::Parse("gemini usage missing rate windows".into()))?;

    let secondary =
        pro_min.and_then(|_| flash_min.map(|quota| usage_window_from_quota("Flash", quota)));

    Ok(UsageSnapshot::new(
        ProviderId::Gemini,
        primary,
        secondary,
        updated_at,
        source,
    ))
}

pub fn parse_tier_id(data: &str) -> Option<GeminiUserTier> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    let tier_id = json
        .get("currentTier")
        .and_then(|tier| tier.get("id"))
        .and_then(|value| value.as_str())?;

    match tier_id {
        "standard-tier" => Some(GeminiUserTier::Standard),
        "free-tier" => Some(GeminiUserTier::Free),
        "legacy-tier" => Some(GeminiUserTier::Legacy),
        _ => None,
    }
}

pub fn parse_code_assist_project_id(data: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    let raw = json.get("cloudaicompanionProject")?;

    match raw {
        serde_json::Value::String(project_id) => normalize_project_id(project_id),
        serde_json::Value::Object(object) => object
            .get("id")
            .or_else(|| object.get("projectId"))
            .and_then(|value| value.as_str())
            .and_then(normalize_project_id),
        _ => None,
    }
}

pub fn plan_label(
    tier: Option<GeminiUserTier>,
    hosted_domain: Option<&str>,
) -> Option<&'static str> {
    match tier {
        Some(GeminiUserTier::Standard) => Some("Paid"),
        Some(GeminiUserTier::Free) if hosted_domain.is_some() => Some("Workspace"),
        Some(GeminiUserTier::Free) => Some("Free"),
        Some(GeminiUserTier::Legacy) => Some("Legacy"),
        None => None,
    }
}

pub fn extract_jwt_claims(id_token: Option<&str>) -> JwtClaims {
    let Some(token) = id_token else {
        return JwtClaims::default();
    };

    let payload = token.split('.').nth(1).unwrap_or_default();
    let Ok(decoded) = decode_base64url(payload) else {
        return JwtClaims::default();
    };
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(&decoded) else {
        return JwtClaims::default();
    };

    JwtClaims {
        email: json
            .get("email")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        hosted_domain: json
            .get("hd")
            .and_then(|value| value.as_str())
            .map(str::to_string),
    }
}

fn usage_window_from_quota(label: &str, quota: &GeminiModelQuota) -> UsageWindow {
    UsageWindow::new(
        label,
        (100.0 - quota.percent_left).clamp(0.0, 100.0),
        quota.resets_at.clone(),
    )
}

fn is_flash_lite_model(model_id: &str) -> bool {
    model_id.to_ascii_lowercase().contains("flash-lite")
}

fn is_flash_model(model_id: &str) -> bool {
    let lower = model_id.to_ascii_lowercase();
    lower.contains("flash") && !is_flash_lite_model(model_id)
}

fn is_pro_model(model_id: &str) -> bool {
    model_id.to_ascii_lowercase().contains("pro")
}

fn normalize_project_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_reset_timestamp(raw: &str) -> Option<String> {
    if OffsetDateTime::parse(raw, &Rfc3339).is_ok() {
        return Some(raw.to_string());
    }

    let format =
        time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z").ok()?;
    OffsetDateTime::parse(raw, &format)
        .ok()
        .and_then(|value| value.format(&Rfc3339).ok())
}

fn decode_base64url(input: &str) -> Result<Vec<u8>, ProviderError> {
    let mut padded = input.replace('-', "+").replace('_', "/");
    while !padded.len().is_multiple_of(4) {
        padded.push('=');
    }

    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(padded)
        .map_err(|error| ProviderError::Parse(format!("jwt decode failed: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_pro_and_flash_buckets_from_fixture() {
        let quotas =
            parse_quota_response(include_str!("fixtures/quota_response.json")).expect("parse");
        let snapshot = snapshot_from_quotas(&quotas, "2026-05-22T12:00:00Z", "gemini-oauth-quota")
            .expect("snapshot");

        assert_eq!(snapshot.primary.label, "Pro");
        assert_eq!(snapshot.primary.used_percent, 40.0);
        assert_eq!(snapshot.secondary.as_ref().expect("flash").label, "Flash");
        assert_eq!(snapshot.secondary.expect("flash").used_percent, 10.0);
    }

    #[test]
    fn picks_lowest_remaining_fraction_per_model() {
        let data = r#"{
            "buckets": [
                {"modelId": "gemini-2.5-flash", "remainingFraction": 0.9},
                {"modelId": "gemini-2.5-flash", "remainingFraction": 0.4}
            ]
        }"#;
        let quotas = parse_quota_response(data).expect("parse");
        assert_eq!(quotas.len(), 1);
        assert_eq!(quotas[0].percent_left, 40.0);
    }

    #[test]
    fn parses_code_assist_project_and_tier() {
        let fixture = include_str!("fixtures/load_code_assist_standard.json");
        assert_eq!(parse_tier_id(fixture), Some(GeminiUserTier::Standard));
        assert_eq!(
            parse_code_assist_project_id(fixture).as_deref(),
            Some("cloudaicompanion-123")
        );
    }

    #[test]
    fn maps_plan_labels_from_tier_and_hosted_domain() {
        assert_eq!(
            plan_label(Some(GeminiUserTier::Standard), None),
            Some("Paid")
        );
        assert_eq!(
            plan_label(Some(GeminiUserTier::Free), Some("example.com")),
            Some("Workspace")
        );
        assert_eq!(plan_label(Some(GeminiUserTier::Free), None), Some("Free"));
        assert_eq!(
            plan_label(Some(GeminiUserTier::Legacy), None),
            Some("Legacy")
        );
    }

    #[test]
    fn extracts_email_from_id_token() {
        let token = "header.eyJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20ifQ.sig";
        let claims = extract_jwt_claims(Some(token));
        assert_eq!(claims.email.as_deref(), Some("user@example.com"));
    }

    #[test]
    fn rejects_empty_quota_payload() {
        let error = parse_quota_response(r#"{"buckets": []}"#).expect_err("empty");
        assert!(matches!(error, ProviderError::Parse(_)));
    }
}
