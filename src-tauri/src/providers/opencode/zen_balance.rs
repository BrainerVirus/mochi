//! OpenCode Go optional Zen pay-as-you-go balance parsing.
//!
//! Logic derived from CodexBar `OpenCodeGoZenBalanceParser.swift` (MIT).

use regex::Regex;
use serde_json::Value;

use crate::core::models::ProviderCostSnapshot;
use crate::core::provider::{ProviderError, ProviderResult};

pub fn parse_zen_balance(text: &str) -> Option<f64> {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        if let Some(balance) = find_balance_value(&value) {
            return Some(balance);
        }
    }

    let localized = Regex::new(
        r"(?i)(?:current\s+balance|zen\s+balance|現在の残高)[^$]{0,80}\$\s*([0-9][0-9,]*(?:\.[0-9]+)?)",
    )
    .ok()?;
    if let Some(capture) = localized.captures(text) {
        return capture
            .get(1)
            .and_then(|value| parse_amount(value.as_str()));
    }

    let nearby = Regex::new(r"(?i)(?:balance|残高)[\s\S]{0,120}?\$\s*([0-9][0-9,]*(?:\.[0-9]+)?)").ok()?;
    nearby
        .captures(text)
        .and_then(|capture| capture.get(1))
        .and_then(|value| parse_amount(value.as_str()))
}

pub fn zen_balance_cost(balance: f64) -> ProviderResult<ProviderCostSnapshot> {
    if balance <= 0.0 {
        return Err(ProviderError::Parse("zen balance missing".into()));
    }

    Ok(ProviderCostSnapshot::new(
        balance,
        0.0,
        "USD",
        Some("Zen balance".into()),
        None,
    ))
}

fn find_balance_value(value: &Value) -> Option<f64> {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                if is_balance_key(key) {
                    if let Some(number) = json_amount(nested) {
                        return Some(number);
                    }
                }
                if let Some(found) = find_balance_value(nested) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(find_balance_value),
        _ => None,
    }
}

fn is_balance_key(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect();
    matches!(
        normalized.as_str(),
        "zenbalance"
            | "zencurrentbalance"
            | "currentbalance"
            | "currentbalanceusd"
            | "balanceusd"
            | "usdbalance"
    )
}

fn json_amount(value: &Value) -> Option<f64> {
    match value {
        Value::Bool(_) => None,
        Value::Number(number) => number.as_f64(),
        Value::String(text) => parse_amount(text),
        _ => None,
    }
}

fn parse_amount(raw: &str) -> Option<f64> {
    raw.trim()
        .replace(',', "")
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zen_balance_from_workspace_text() {
        let text = "Current balance $1,234.56 available for Zen pay-as-you-go usage.";
        assert_eq!(parse_zen_balance(text), Some(1234.56));
    }

    #[test]
    fn parses_zen_balance_from_nested_json() {
        let text = r#"{"account":{"zenBalance":"1,042.75"}}"#;
        assert_eq!(parse_zen_balance(text), Some(1042.75));
    }
}
