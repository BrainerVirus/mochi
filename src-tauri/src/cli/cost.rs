use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::core::models::ProviderId;
use crate::core::usage_state::ProviderUsageState;
use crate::tray::provider_display_name;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CostEntry {
    pub provider: ProviderId,
    pub used: f64,
    pub limit: f64,
    pub currency_code: String,
    pub period: String,
}

/// Single parse site for CLI provider filters (Cost, Status, Usage).
pub fn parse_provider_filter(name: &str) -> anyhow::Result<ProviderId> {
    ProviderId::parse(name).ok_or_else(|| anyhow::anyhow!("unknown provider: {name}"))
}

fn currency_symbol(currency_code: &str) -> Option<&'static str> {
    match currency_code.trim().to_ascii_uppercase().as_str() {
        "USD" => Some("$"),
        "EUR" => Some("€"),
        "GBP" => Some("£"),
        "JPY" => Some("¥"),
        _ => None,
    }
}

pub fn format_money(amount: f64, currency_code: &str) -> String {
    if !amount.is_finite() {
        return "n/a".to_string();
    }
    let sign = if amount < 0.0 { "-" } else { "" };
    let absolute = amount.abs();
    match currency_symbol(currency_code) {
        Some(symbol) => format!("{sign}{symbol}{absolute:.2}"),
        None => {
            let code = currency_code.trim();
            if code.is_empty() {
                format!("{sign}{absolute:.2}")
            } else {
                format!("{sign}{absolute:.2} {}", code.to_ascii_uppercase())
            }
        }
    }
}

/// Mirrors the frontend `formatCostDetail`: a non-positive limit means used-only.
pub fn format_cost_detail(used: f64, limit: f64, currency_code: &str) -> String {
    if limit > 0.0 {
        format!(
            "{} / {}",
            format_money(used, currency_code),
            format_money(limit, currency_code)
        )
    } else {
        format_money(used, currency_code)
    }
}

/// Mirror of the frontend `formatCostPeriodLabel`: raw period ids
/// ("billing-period") never render as labels; human labels
/// ("Billing period") do, with "On-demand" for missing periods.
/// Shared by the headless `format_cost_text` and the cost TUI so both
/// render identical labels.
pub fn cost_period_label(period: Option<&str>) -> String {
    let raw = period.map(str::trim).unwrap_or("");
    let words: Vec<&str> = raw.split('-').filter(|word| !word.is_empty()).collect();
    let [first, rest @ ..] = words.as_slice() else {
        return "On-demand".to_string();
    };
    let mut label = String::with_capacity(raw.len() + 1);
    let mut chars = first.chars();
    match chars.next() {
        Some(head) => label.extend(head.to_uppercase()),
        None => return "On-demand".to_string(),
    }
    label.push_str(chars.as_str());
    for word in rest {
        label.push(' ');
        label.push_str(word);
    }
    label
}

pub fn format_cost_text(entries: &[CostEntry], days: u16, provider: Option<ProviderId>) -> String {
    if entries.is_empty() {
        if let Some(id) = provider {
            return format!(
                "{}: no cost data in the last {days} days",
                provider_display_name(id)
            );
        }
        return format!("No cost data in the last {days} days.");
    }
    entries
        .iter()
        .map(|entry| {
            format!(
                "{} {} ({})",
                provider_display_name(entry.provider),
                format_cost_detail(entry.used, entry.limit, &entry.currency_code),
                cost_period_label(Some(&entry.period))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_cost_json(entries: &[CostEntry]) -> Result<String, serde_json::Error> {
    serde_json::to_string(entries)
}

fn snapshot_within_days(updated_at: &str, days: u16, now: OffsetDateTime) -> bool {
    let Ok(updated) = OffsetDateTime::parse(updated_at, &Rfc3339) else {
        return true;
    };
    updated >= now - Duration::days(i64::from(days))
}

pub fn cost_entries_from_states(
    states: &[ProviderUsageState],
    days: u16,
    now: OffsetDateTime,
) -> Vec<CostEntry> {
    states
        .iter()
        .filter_map(|state| {
            let snapshot = state.snapshot.as_ref()?;
            if !snapshot_within_days(&snapshot.updated_at, days, now) {
                return None;
            }
            let cost = snapshot.provider_cost.as_ref()?;
            Some(CostEntry {
                provider: snapshot.provider,
                used: cost.used,
                limit: cost.limit,
                currency_code: cost.currency_code.clone(),
                period: cost
                    .period
                    .clone()
                    .unwrap_or_else(|| "current period".to_string()),
            })
        })
        .collect()
}

pub fn load_cost_entries(days: u16) -> anyhow::Result<Vec<CostEntry>> {
    // Same usage.sqlite3 store the other CLI helpers read (see `cli_usage_states`
    // in lib.rs); spend rides on the cached snapshots, so `days` scopes them to
    // the trailing window via their snapshot timestamps.
    let states = crate::cli_usage_states(None, false)?;
    Ok(cost_entries_from_states(
        &states,
        days,
        OffsetDateTime::now_utc(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderCostSnapshot, ProviderId, UsageSnapshot, UsageWindow};
    use time::format_description::well_known::Rfc3339;

    fn test_now() -> OffsetDateTime {
        OffsetDateTime::parse("2026-06-04T12:00:00Z", &Rfc3339).expect("test now")
    }

    #[test]
    fn cost_line_shows_used_vs_limit() {
        let entries = vec![CostEntry {
            provider: ProviderId::CommandCode,
            used: 7.54,
            limit: 71.93,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(output.contains("$7.54 / $71.93"));
        assert!(output.contains("(Billing period)"));
        assert!(!output.contains("billing-period"));
    }

    #[test]
    fn cost_period_label_formats_like_widget() {
        assert_eq!(
            cost_period_label(Some("billing-period")),
            "Billing period".to_string()
        );
        assert_eq!(cost_period_label(None), "On-demand".to_string());
        assert_eq!(cost_period_label(Some("")), "On-demand".to_string());
        assert_eq!(cost_period_label(Some("--")), "On-demand".to_string());
    }

    #[test]
    fn cost_headless_uses_shared_period_label() {
        let entries = vec![CostEntry {
            provider: ProviderId::CommandCode,
            used: 7.54,
            limit: 71.93,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(output.contains("$7.54 / $71.93"));
        assert!(output.contains("Billing period"));
    }

    #[test]
    fn cost_empty_reports_range() {
        assert!(format_cost_text(&[], 30, None).contains("30 days"));
    }

    #[test]
    fn cost_entries_excludes_snapshots_older_than_days() {
        let now = test_now();
        let fresh = state_with_cost_at(ProviderId::Cursor, "2026-06-03T12:00:00Z");
        let old = state_with_cost_at(ProviderId::Claude, "2026-04-01T12:00:00Z");
        let entries = cost_entries_from_states(&[fresh, old], 7, now);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider, ProviderId::Cursor);
    }

    #[test]
    fn cost_entries_includes_snapshots_within_days() {
        let now = test_now();
        let entries = cost_entries_from_states(&[snapshot_with_cost()], 30, now);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn cost_line_uses_euro_symbol_for_eur() {
        let entries = vec![CostEntry {
            provider: ProviderId::Cursor,
            used: 7.54,
            limit: 71.93,
            currency_code: "EUR".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(output.contains("€7.54 / €71.93"), "unexpected: {output}");
        assert!(!output.contains('$'), "bogus $ in: {output}");
    }

    #[test]
    fn cost_line_uses_symbols_for_gbp_and_jpy() {
        for (code, symbol) in [("GBP", "£"), ("JPY", "¥")] {
            let entries = vec![CostEntry {
                provider: ProviderId::Cursor,
                used: 7.54,
                limit: 71.93,
                currency_code: code.to_string(),
                period: "billing-period".to_string(),
            }];
            let output = format_cost_text(&entries, 30, None);
            assert!(
                output.contains(&format!("{symbol}7.54 / {symbol}71.93")),
                "unexpected {code}: {output}"
            );
        }
    }

    #[test]
    fn cost_line_falls_back_to_amount_code_without_bogus_dollar() {
        let entries = vec![CostEntry {
            provider: ProviderId::Cursor,
            used: 7.54,
            limit: 71.93,
            currency_code: "CHF".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(
            output.contains("7.54 CHF / 71.93 CHF"),
            "unexpected: {output}"
        );
        assert!(!output.contains('$'), "bogus $ in: {output}");
    }

    #[test]
    fn cost_line_with_empty_currency_has_no_dangling_space() {
        let entries = vec![CostEntry {
            provider: ProviderId::Cursor,
            used: 7.54,
            limit: 71.93,
            currency_code: String::new(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(output.contains("7.54 / 71.93"), "unexpected: {output}");
        assert!(!output.contains("  "), "double space in: {output}");
        assert!(
            output.contains("71.93 (Billing period)"),
            "dangling space in: {output}"
        );
    }

    #[test]
    fn cost_line_with_zero_limit_shows_used_only() {
        let entries = vec![CostEntry {
            provider: ProviderId::Cursor,
            used: 7.54,
            limit: 0.0,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(output.contains("$7.54"), "unexpected: {output}");
        assert!(
            !output.contains('/'),
            "limit shown for zero limit: {output}"
        );
    }

    #[test]
    fn cost_line_renders_negative_with_sign_before_symbol() {
        let entries = vec![CostEntry {
            provider: ProviderId::Cursor,
            used: -1.0,
            limit: 71.93,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }];
        let output = format_cost_text(&entries, 30, None);
        assert!(output.contains("-$1.00"), "unexpected: {output}");
    }

    #[test]
    fn cost_money_guards_non_finite_amounts() {
        assert_eq!(format_money(f64::NAN, "USD"), "n/a");
        assert_eq!(format_money(f64::INFINITY, "USD"), "n/a");
        assert_eq!(format_money(f64::NEG_INFINITY, "EUR"), "n/a");
    }

    #[test]
    fn provider_filter_accepts_case_and_alias_variants() {
        assert_eq!(
            parse_provider_filter("Cursor").expect("Cursor"),
            ProviderId::Cursor
        );
        assert_eq!(
            parse_provider_filter("command-code").expect("command-code"),
            ProviderId::CommandCode
        );
        assert_eq!(
            parse_provider_filter(" COMMANDCODE ").expect("padded"),
            ProviderId::CommandCode
        );
    }

    #[test]
    fn provider_filter_rejects_unknown() {
        let error = parse_provider_filter("gibberish").expect_err("unknown should fail");
        assert!(error.to_string().contains("unknown provider"));
    }

    #[test]
    fn filtered_empty_names_provider_instead_of_generic_message() {
        let output = format_cost_text(&[], 7, Some(ProviderId::Cursor));
        assert!(output.contains("Cursor"), "unexpected: {output}");
        assert!(output.contains("7 days"), "unexpected: {output}");
        assert!(output.contains("no cost data"), "unexpected: {output}");
    }

    #[test]
    fn loader_reports_unreadable_database_with_path() {
        let path = std::env::temp_dir().join(format!(
            "mochi-cost-corrupt-{}-loader_reports.sqlite3",
            std::process::id()
        ));
        std::fs::write(&path, b"not a sqlite database").expect("seed corrupt db");
        let error = crate::cli_usage_states_with_db_path(None, false, Some(path.clone()))
            .expect_err("corrupt db should fail");
        let message = error.to_string();
        std::fs::remove_file(&path).ok();
        assert!(
            message.contains("cannot open usage database"),
            "unexpected: {message}"
        );
        assert!(
            message.contains(&path.display().to_string()),
            "missing path in: {message}"
        );
    }

    #[test]
    fn loader_reads_persisted_states_from_repository() {
        use crate::core::usage_repository::{SqliteUsageRepository, UsageRepository};
        use crate::core::usage_store::UsageStore;

        let repository = SqliteUsageRepository::from_connection(
            rusqlite::Connection::open_in_memory().expect("sqlite"),
        )
        .expect("repo");
        let state = state_with_cost_at(ProviderId::Cursor, "2026-06-03T12:00:00Z");
        repository.put_latest(&state).expect("put latest");
        let store = UsageStore::with_repository(std::sync::Arc::new(repository));
        let states = store
            .load_latest_states(&["cursor".to_string()])
            .expect("load");
        let entries = cost_entries_from_states(&states, 30, test_now());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider, ProviderId::Cursor);
    }

    fn state_with_cost_at(provider: ProviderId, updated_at: &str) -> ProviderUsageState {
        let snapshot = UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", 10.0, None),
            None,
            updated_at,
            "test",
        )
        .with_provider_cost(ProviderCostSnapshot::new(
            7.54,
            71.93,
            "USD",
            Some("billing-period".to_string()),
            None,
        ));
        ProviderUsageState::fresh(snapshot)
    }

    fn snapshot_with_cost() -> ProviderUsageState {
        state_with_cost_at(ProviderId::Cursor, "2026-06-04T12:00:00Z")
    }

    #[test]
    fn cost_entries_from_states_extracts_provider_cost() {
        let entries = cost_entries_from_states(&[snapshot_with_cost()], 30, test_now());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider, ProviderId::Cursor);
        assert_eq!(entries[0].currency_code, "USD");
        assert_eq!(entries[0].period, "billing-period");
    }

    #[test]
    fn cost_entries_from_states_skips_snapshots_without_cost() {
        let snapshot = UsageSnapshot::new(
            ProviderId::Claude,
            UsageWindow::new("Session", 10.0, None),
            None,
            "2026-06-04T12:00:00Z",
            "test",
        );
        let entries =
            cost_entries_from_states(&[ProviderUsageState::fresh(snapshot)], 30, test_now());
        assert!(entries.is_empty());
    }

    #[test]
    fn cost_json_parses_and_matches_text_content() {
        let entries = vec![CostEntry {
            provider: ProviderId::Cursor,
            used: 7.54,
            limit: 71.93,
            currency_code: "USD".to_string(),
            period: "billing-period".to_string(),
        }];
        let text = format_cost_text(&entries, 30, None);
        let json = format_cost_json(&entries).expect("json");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parses");
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["provider"], serde_json::json!("cursor"));
        assert_eq!(parsed[0]["used"], serde_json::json!(7.54));
        assert!(text.contains("$7.54"), "unexpected: {text}");
    }
}
