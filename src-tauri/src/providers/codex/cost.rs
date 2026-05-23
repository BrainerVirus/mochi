//! Local Codex session JSONL cost scan (CodexBar-derived, MIT).
//! Reference: CodexBar `Sources/CodexBarCore/Vendored/CostUsage/*`.

use std::path::{Path, PathBuf};

use time::{Duration, OffsetDateTime};

use crate::core::models::SessionCostSummary;
use crate::core::provider::{ProviderError, ProviderResult};

const DEFAULT_WINDOW_DAYS: u32 = 30;

#[derive(Debug, Clone, Default)]
struct TokenTotals {
    input: u64,
    cached: u64,
    output: u64,
}

pub fn scan_session_cost(window_days: u32) -> ProviderResult<SessionCostSummary> {
    let sessions_root = codex_sessions_root();
    if !sessions_root.exists() {
        return Ok(empty_summary(window_days));
    }

    let cutoff = OffsetDateTime::now_utc() - Duration::days(i64::from(window_days));
    let mut totals = TokenTotals::default();
    let mut files_scanned = 0u32;

    for file in discover_jsonl_files(&sessions_root) {
        if !should_scan_file(&file, cutoff) {
            continue;
        }

        let parsed = parse_jsonl_file(&file, cutoff)?;
        totals.input += parsed.input;
        totals.cached += parsed.cached;
        totals.output += parsed.output;
        files_scanned += 1;
    }

    Ok(SessionCostSummary {
        window_days,
        input_tokens: totals.input,
        cached_input_tokens: totals.cached,
        output_tokens: totals.output,
        session_files_scanned: files_scanned,
    })
}

pub fn codex_home() -> PathBuf {
    if let Some(codex_home) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(codex_home);
    }

    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| PathBuf::from(home).join(".codex"))
        .unwrap_or_else(|| PathBuf::from(".codex"))
}

fn codex_sessions_root() -> PathBuf {
    codex_home().join("sessions")
}

fn empty_summary(window_days: u32) -> SessionCostSummary {
    SessionCostSummary {
        window_days,
        input_tokens: 0,
        cached_input_tokens: 0,
        output_tokens: 0,
        session_files_scanned: 0,
    }
}

fn discover_jsonl_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_jsonl_files(root, &mut files);

    let archived = codex_home().join("archived_sessions");
    if archived.is_dir() {
        collect_jsonl_files(&archived, &mut files);
    }

    files
}

fn collect_jsonl_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
}

fn should_scan_file(path: &Path, cutoff: OffsetDateTime) -> bool {
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            let modified = OffsetDateTime::from(modified);
            return modified >= cutoff;
        }
    }

    true
}

fn parse_jsonl_file(path: &Path, cutoff: OffsetDateTime) -> ProviderResult<TokenTotals> {
    let data = std::fs::read_to_string(path)
        .map_err(|error| ProviderError::Fetch(format!("read codex session log: {error}")))?;

    let mut latest_totals = TokenTotals::default();
    let mut current_model = None::<String>;

    for line in data.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let event_type = value
            .get("type")
            .and_then(|entry| entry.as_str())
            .unwrap_or("");
        let timestamp = value
            .get("timestamp")
            .and_then(|entry| entry.as_str())
            .and_then(parse_timestamp);

        if let Some(timestamp) = timestamp {
            if timestamp < cutoff {
                continue;
            }
        }

        match event_type {
            "turn_context" => {
                current_model = value
                    .pointer("/payload/model")
                    .and_then(|entry| entry.as_str())
                    .map(str::to_string);
            }
            "event_msg" => {
                if value
                    .pointer("/payload/type")
                    .and_then(|entry| entry.as_str())
                    != Some("token_count")
                {
                    continue;
                }

                let usage = value.pointer("/payload/info/total_token_usage");
                let Some(usage) = usage else { continue };

                latest_totals.input = usage
                    .get("input_tokens")
                    .and_then(|entry| entry.as_u64())
                    .unwrap_or(0);
                latest_totals.cached = usage
                    .get("cached_input_tokens")
                    .and_then(|entry| entry.as_u64())
                    .unwrap_or(0);
                latest_totals.output = usage
                    .get("output_tokens")
                    .and_then(|entry| entry.as_u64())
                    .unwrap_or(0);

                if current_model.is_none() {
                    current_model = value
                        .pointer("/payload/info/model")
                        .and_then(|entry| entry.as_str())
                        .map(str::to_string);
                }
            }
            _ => {}
        }
    }

    let _ = current_model;
    Ok(latest_totals)
}

fn parse_timestamp(raw: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(raw, &time::format_description::well_known::Rfc3339).ok()
}

pub fn default_window_days() -> u32 {
    DEFAULT_WINDOW_DAYS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_sessions_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("mochi-codex-sessions-{nanos}"))
    }

    #[test]
    fn scan_session_cost_sums_token_count_events_from_fixture() {
        let root = temp_sessions_dir();
        let day_dir = root.join("2026").join("05").join("20");
        fs::create_dir_all(&day_dir).expect("create session dir");
        fs::write(
            day_dir.join("session.jsonl"),
            include_str!("fixtures/session.jsonl"),
        )
        .expect("write session fixture");

        let summary = scan_session_cost_at_root(&root, 30).expect("scan");

        assert_eq!(summary.input_tokens, 160);
        assert_eq!(summary.cached_input_tokens, 40);
        assert_eq!(summary.output_tokens, 16);
        assert_eq!(summary.session_files_scanned, 1);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn scan_session_cost_returns_zero_when_sessions_missing() {
        let root = temp_sessions_dir();
        let summary = scan_session_cost_at_root(&root, 7).expect("scan");
        assert_eq!(summary.session_files_scanned, 0);
        assert_eq!(summary.input_tokens, 0);
    }

    fn scan_session_cost_at_root(
        root: &Path,
        window_days: u32,
    ) -> ProviderResult<SessionCostSummary> {
        if !root.exists() {
            return Ok(empty_summary(window_days));
        }

        let cutoff = OffsetDateTime::now_utc() - Duration::days(i64::from(window_days));
        let mut totals = TokenTotals::default();
        let mut files_scanned = 0u32;

        for file in discover_jsonl_files(root) {
            if !should_scan_file(&file, cutoff) {
                continue;
            }
            let parsed = parse_jsonl_file(&file, cutoff)?;
            totals.input += parsed.input;
            totals.cached += parsed.cached;
            totals.output += parsed.output;
            files_scanned += 1;
        }

        Ok(SessionCostSummary {
            window_days,
            input_tokens: totals.input,
            cached_input_tokens: totals.cached,
            output_tokens: totals.output,
            session_files_scanned: files_scanned,
        })
    }
}
