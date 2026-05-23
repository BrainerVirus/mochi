//! Factory/Droid usage API response mapping.

use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
use crate::core::provider::{ProviderError, ProviderResult};

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFactoryUsage {
    pub standard_used_percent: f32,
    pub premium_used_percent: f32,
    pub period_end: Option<String>,
    pub account_email: Option<String>,
    pub organization_name: Option<String>,
    pub plan_name: Option<String>,
    pub tier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthMeResponse {
    #[serde(default, rename = "userProfile")]
    user_profile: Option<UserProfile>,
    #[serde(default)]
    organization: Option<Organization>,
}

#[derive(Debug, Deserialize)]
struct UserProfile {
    #[serde(default)]
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Organization {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    subscription: Option<Subscription>,
}

#[derive(Debug, Deserialize)]
struct Subscription {
    #[serde(default, rename = "factoryTier")]
    factory_tier: Option<String>,
    #[serde(default, rename = "orbSubscription")]
    orb_subscription: Option<OrbSubscription>,
}

#[derive(Debug, Deserialize)]
struct OrbSubscription {
    #[serde(default)]
    plan: Option<Plan>,
}

#[derive(Debug, Deserialize)]
struct Plan {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageApiResponse {
    #[serde(default)]
    usage: Option<UsageData>,
}

#[derive(Debug, Deserialize)]
struct UsageData {
    #[serde(default, rename = "endDate")]
    end_date: Option<i64>,
    #[serde(default)]
    standard: Option<TokenUsage>,
    #[serde(default)]
    premium: Option<TokenUsage>,
}

#[derive(Debug, Deserialize)]
struct TokenUsage {
    #[serde(default, rename = "userTokens")]
    user_tokens: Option<i64>,
    #[serde(default, rename = "totalAllowance")]
    total_allowance: Option<i64>,
    #[serde(default, rename = "usedRatio")]
    used_ratio: Option<f64>,
}

type AuthMeFields = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub fn parse_auth_me(body: &str) -> ProviderResult<AuthMeFields> {
    let response: AuthMeResponse =
        serde_json::from_str(body).map_err(|error| ProviderError::Parse(error.to_string()))?;
    let email = response.user_profile.and_then(|profile| profile.email);
    let organization_name = response
        .organization
        .as_ref()
        .and_then(|org| org.name.clone());
    let tier = response
        .organization
        .as_ref()
        .and_then(|org| org.subscription.as_ref())
        .and_then(|sub| sub.factory_tier.clone());
    let plan_name = response
        .organization
        .and_then(|org| org.subscription)
        .and_then(|sub| sub.orb_subscription)
        .and_then(|orb| orb.plan)
        .and_then(|plan| plan.name);
    Ok((email, organization_name, tier, plan_name))
}

pub fn parse_usage_response(body: &str) -> ProviderResult<ParsedFactoryUsage> {
    let response: UsageApiResponse =
        serde_json::from_str(body).map_err(|error| ProviderError::Parse(error.to_string()))?;
    let usage = response
        .usage
        .ok_or_else(|| ProviderError::Parse("Factory usage missing usage block".into()))?;

    let period_end = usage.end_date.and_then(format_epoch_ms);
    let standard_used_percent = usage_percent(usage.standard.as_ref());
    let premium_used_percent = usage_percent(usage.premium.as_ref());

    Ok(ParsedFactoryUsage {
        standard_used_percent,
        premium_used_percent,
        period_end,
        account_email: None,
        organization_name: None,
        plan_name: None,
        tier: None,
    })
}

pub fn snapshot_from_parsed(
    parsed: &ParsedFactoryUsage,
    updated_at: &str,
    source: &str,
) -> UsageSnapshot {
    UsageSnapshot::new(
        ProviderId::Factory,
        UsageWindow::new(
            "Standard",
            parsed.standard_used_percent,
            parsed.period_end.clone(),
        ),
        Some(UsageWindow::new(
            "Premium",
            parsed.premium_used_percent,
            parsed.period_end.clone(),
        )),
        updated_at,
        source,
    )
}

fn usage_percent(usage: Option<&TokenUsage>) -> f32 {
    let Some(usage) = usage else {
        return 0.0;
    };

    if let Some(ratio) = usage.used_ratio.filter(|value| (0.0..=1.0).contains(value)) {
        return (ratio * 100.0) as f32;
    }
    if let Some(ratio) = usage
        .used_ratio
        .filter(|value| *value > 1.0 && *value <= 100.0)
    {
        return ratio as f32;
    }

    match (usage.user_tokens, usage.total_allowance) {
        (Some(used), Some(allowance)) if allowance > 0 => {
            ((used as f64 / allowance as f64) * 100.0) as f32
        }
        _ => 0.0,
    }
}

fn format_epoch_ms(ms: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp(ms / 1000)
        .ok()?
        .format(&Rfc3339)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_usage_fixture_to_standard_and_premium() {
        let mut parsed = parse_usage_response(include_str!("fixtures/usage.json")).expect("parse");
        parsed.account_email = Some("user@example.com".into());
        let snapshot = snapshot_from_parsed(&parsed, "2026-05-23T00:00:00Z", "factory-web-cookies");
        assert_eq!(snapshot.primary.label, "Standard");
        assert_eq!(snapshot.primary.used_percent, 10.0);
        assert_eq!(
            snapshot.secondary.as_ref().expect("premium").label,
            "Premium"
        );
        assert_eq!(snapshot.secondary.expect("premium").used_percent, 50.0);
    }

    #[test]
    fn parses_auth_me_fixture() {
        let (email, org, tier, plan) =
            parse_auth_me(include_str!("fixtures/auth_me.json")).expect("parse");
        assert_eq!(email.as_deref(), Some("user@example.com"));
        assert_eq!(org.as_deref(), Some("Acme"));
        assert_eq!(tier.as_deref(), Some("enterprise"));
        assert_eq!(plan.as_deref(), Some("Pro"));
    }
}
