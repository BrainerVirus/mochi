# Mochi Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the gap between Mochi and CodexBar by establishing shared provider foundations, achieving Codex reference parity, porting v1 providers in phases, pivoting to OS-native shells, and hardening CLI/release paths.

**Architecture:** Declarative provider metadata drives settings UI and fetch strategy ordering; a cross-platform `UsageStore` caches last-known-good snapshots with stale/error health; credential storage abstracts platform keyrings; provider fetch logic lives in Rust under `src-tauri/src/providers/` with Zod-validated IPC to React. CodexBar fetch/parsing logic is ported incrementally with MIT attribution in docs.

**Tech Stack:** Tauri v2, Rust (tokio, thiserror, optional keyring), TanStack Start/Query, Zod 4, Vitest, cargo test/clippy.

**References:** [Mochi design spec](../specs/2026-05-19-mochi-design.md), [provider parity matrix](../../provider-parity-matrix.md), [CodexBar providers.md](../../../CodexBar/docs/providers.md) (MIT).

---

## Phase 0: Inventory + docs

**Files:**

- Create: `docs/provider-parity-matrix.md`
- Create: `docs/superpowers/plans/2026-05-20-mochi-gap-closure.md` (this file)
- Read: `docs/superpowers/specs/2026-05-19-mochi-design.md`
- Read: `/Users/cristhoferpincetti/Documents/projects/personal/CodexBar/docs/providers.md`

### Task 0.1: Provider parity matrix

- [x] **Step 1:** Draft matrix for all 46 CodexBar providers (condensed) plus detailed rows for Mochi v1 10.
- [x] **Step 2:** Save to `docs/provider-parity-matrix.md`.
- [x] **Step 3:** Commit\*\*

```bash
git add docs/provider-parity-matrix.md
git commit -m "docs: add provider parity matrix"
```

### Task 0.2: Gap closure plan (this document)

- [x] **Step 1:** Write phases 0–7 with file paths and TDD steps.
- [x] **Step 2:** Commit\*\*

```bash
git add docs/superpowers/plans/2026-05-20-mochi-gap-closure.md
git commit -m "docs: add gap closure plan"
```

---

## Phase 1: Shared foundations

**Branch:** `feat/gap-closure-foundations`

**Files:**

- Modify: `src-tauri/src/core/models.rs`
- Create: `src-tauri/src/core/usage_store.rs`
- Create: `src-tauri/src/core/provider_metadata.rs`
- Modify: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/auth/mod.rs`
- Create: `src-tauri/src/auth/credential_store.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/status/mod.rs`
- Modify: `src-tauri/src/tray/mod.rs` (`sync_tray_usage`)
- Modify: `src/lib/schemas/usage.ts`
- Modify: `src/lib/schemas/usage.test.ts`

### Task 1.1: Expand core health models

**Files:**

- Modify: `src-tauri/src/core/models.rs`
- Test: inline `#[cfg(test)]` in `models.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn usage_snapshot_defaults_health_to_ok() {
    let json = r#"{"provider":"claude","primary":{"label":"Session","used_percent":0.0,"remaining_percent":100.0,"resets_at":null},"secondary":null,"updated_at":"2026-05-20T00:00:00Z","source":"test"}"#;
    let snapshot: UsageSnapshot = serde_json::from_str(json).unwrap();
    assert_eq!(snapshot.health, ProviderHealth::Ok);
    assert!(!snapshot.is_stale);
    assert!(snapshot.error.is_none());
}
```

- [ ] **Step 2: Run test — expect FAIL** (`ProviderHealth` missing)

Run: `cargo test --manifest-path src-tauri/Cargo.toml usage_snapshot_defaults_health_to_ok -- --nocapture`

- [ ] **Step 3: Add types and optional serde fields**

Add `ProviderHealth`, `ProviderStatus`, `FetchAttempt`; extend `UsageSnapshot` with `#[serde(default)]` on new fields.

- [ ] **Step 4: Run tests — expect PASS**

Run: `cargo test --manifest-path src-tauri/Cargo.toml core::models`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/models.rs
git commit -m "feat(tauri): add provider health models"
```

### Task 1.2: UsageStore with last-known-good

**Files:**

- Create: `src-tauri/src/core/usage_store.rs`
- Modify: `src-tauri/src/core/mod.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn record_failure_returns_stale_last_known_good() {
    let store = UsageStore::new(None);
    let good = sample_snapshot(ProviderId::Claude, ProviderHealth::Ok);
    store.record_success(good.clone());
    let stale = store
        .record_failure(ProviderId::Claude, &ProviderError::Fetch("network".into()), attempt())
        .expect("lkg");
    assert!(stale.is_stale);
    assert_eq!(stale.health, ProviderHealth::Stale);
    assert_eq!(stale.primary.used_percent, good.primary.used_percent);
}
```

- [ ] **Step 2: Run test — expect FAIL**

Run: `cargo test --manifest-path src-tauri/Cargo.toml record_failure_returns_stale_last_known_good`

- [ ] **Step 3: Implement `UsageStore`** (in-memory `HashMap`, optional persistence path stub, `record_success`, `record_failure`, `get_snapshots`)

- [ ] **Step 4: Run tests — expect PASS**

Run: `cargo test --manifest-path src-tauri/Cargo.toml usage_store`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/usage_store.rs src-tauri/src/core/mod.rs
git commit -m "feat(tauri): add usage store with LKG"
```

### Task 1.3: Declarative provider metadata registry

**Files:**

- Create: `src-tauri/src/core/provider_metadata.rs`
- Modify: `src-tauri/src/core/mod.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn registry_contains_all_v1_providers() {
    let ids: HashSet<_> = provider_registry().iter().map(|d| d.id).collect();
    for id in ProviderId::all() {
        assert!(ids.contains(&id), "missing metadata for {id:?}");
    }
}
```

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Implement registry** with strategies, auth, settings fields, status URL, cost flag, `ImplementationStatus` for each v1 provider.

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/provider_metadata.rs
git commit -m "feat(tauri): add provider metadata registry"
```

### Task 1.4: Credential storage abstraction (stub)

**Files:**

- Create: `src-tauri/src/auth/mod.rs`
- Create: `src-tauri/src/auth/credential_store.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn dev_store_round_trips_secrets() {
    let store = DevCredentialStore::new();
    store.set("codex.oauth", "token").unwrap();
    assert_eq!(store.get("codex.oauth").unwrap(), Some("token".into()));
    store.delete("codex.oauth").unwrap();
    assert_eq!(store.get("codex.oauth").unwrap(), None);
}
```

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Implement `CredentialStore` trait + `DevCredentialStore`**; document Keychain/libsecret/Windows paths in module docs (no browser cookie import yet).

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/auth/
git commit -m "feat(tauri): add credential store stub"
```

### Task 1.5: Wire commands to UsageStore

**Files:**

- Modify: `src-tauri/src/status/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/tray/mod.rs`

- [ ] **Step 1: Write failing test** — `get_usage_snapshots` returns cached snapshot without live fetch when cache populated.

- [ ] **Step 2: Run test — expect FAIL**

- [ ] **Step 3: Manage `UsageStore` in Tauri state; `get_usage_snapshots` reads cache; `refresh_provider` / `sync_tray_usage` fetch live and update cache.**

- [ ] **Step 4: Run full Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/status/mod.rs src-tauri/src/lib.rs src-tauri/src/tray/mod.rs
git commit -m "feat(tauri): wire usage store to status commands"
```

### Task 1.6: Frontend Zod schema sync

**Files:**

- Modify: `src/lib/schemas/usage.ts`
- Modify: `src/lib/schemas/usage.test.ts`

- [ ] **Step 1: Write failing test** — parse snapshot with `health`, `is_stale`, optional `error`.

- [ ] **Step 2: Run test — expect FAIL**

Run: `pnpm test src/lib/schemas/usage.test.ts`

- [ ] **Step 3: Extend schemas** with `ProviderHealthSchema`, optional fields with defaults.

- [ ] **Step 4: Run frontend verification**

Run: `pnpm lint && pnpm test && pnpm build`

- [ ] **Step 5: Commit**

```bash
git add src/lib/schemas/usage.ts src/lib/schemas/usage.test.ts
git commit -m "feat(ui): extend usage snapshot schema"
```

### Task 1.7: Phase 1 verification gate

- [ ] Run: `pnpm lint`
- [ ] Run: `pnpm format:check`
- [ ] Run: `pnpm test`
- [ ] Run: `pnpm build`
- [ ] Run: `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
- [ ] Run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] Push branch (do not merge)

---

## Phase 2: Codex full parity

**Goal:** Match CodexBar Codex provider: OAuth API, CLI RPC, optional web dashboard, session cost scan, status page.

**Files:**

- Modify: `src-tauri/src/providers/codex/` (`mod.rs`, `cli_rpc.rs`, `cookies.rs`, new `oauth.rs`, `cost.rs`)
- Create: `src-tauri/src/providers/codex/oauth.rs`
- Create: `src-tauri/src/providers/codex/cost.rs`
- Modify: `src-tauri/src/auth/credential_store.rs` (real keyring optional)
- Reference: `CodexBar/docs/codex.md`, `CodexBar/docs/codex-oauth.md`

### Task 2.1: OAuth strategy

- [ ] Port OAuth token read/refresh from CodexBar (MIT note in module header)
- [ ] TDD: `oauth_strategy_returns_snapshot_when_token_valid`
- [ ] Wire as first strategy in auto order

### Task 2.2: Web dashboard strategy

- [ ] Implement cookie-based web fetch (defer WebView to settings flag)
- [ ] TDD: parse fixture HTML/JSON from `src-tauri/tests/fixtures/codex/`

### Task 2.3: Session cost scan

- [ ] Scan `CODEX_HOME` JSONL for configured window
- [ ] Expose cost fields in snapshot or separate command (per spec)

### Task 2.4: Status page

- [ ] Fetch OpenAI Statuspage component status
- [ ] Surface in `ProviderStatus` on snapshot

### Task 2.5: Update matrix row Codex → **done**

---

## Phase 3: Phase A providers (Claude, Gemini, Copilot, Cursor)

**Goal:** Replace stubs with real fetch for highest-traffic providers.

| Provider | Primary files                      | CodexBar reference |
| -------- | ---------------------------------- | ------------------ |
| Claude   | `src-tauri/src/providers/claude/`  | `docs/claude.md`   |
| Gemini   | `src-tauri/src/providers/gemini/`  | `docs/gemini.md`   |
| Copilot  | `src-tauri/src/providers/copilot/` | `docs/copilot.md`  |
| Cursor   | `src-tauri/src/providers/cursor/`  | `docs/cursor.md`   |

Each provider task follows: failing integration test → strategy impl → register in `providers/mod.rs` → update matrix.

---

## Phase 4: Phase B providers

**Goal:** Port Antigravity, Factory, z.ai, Kiro, Augment.

**Files:** `src-tauri/src/providers/{antigravity,factory,zai,kiro,augment}/`

---

## Phase 5: Native OS design pivot

**Goal:** Rework UI toward platform-native shells (macOS vibrancy, Windows Mica, Linux GTK-adjacent) without rewriting DESIGN.md in Phase 1.

**Files:**

- Modify: `DESIGN.md` (via `.agents/skills/design-md`)
- Modify: `app/` layout components for platform-aware chrome
- Create: `src/lib/platform/` detection helpers

**Note:** Do not rewrite DESIGN.md until this phase.

---

## Phase 6: CLI + status-bar

**Goal:** Parity with CodexBar CLI surfaces.

**Files:**

- Modify: `src-tauri/src/cli/mod.rs`
- Modify: `src-tauri/src/status_bar/mod.rs`
- Add: `mochi status`, `mochi providers`, `mochi refresh <id>`

---

## Phase 7: Release hardening

**Goal:** Smoke tests, signing, updater channels, cross-platform CI matrix.

**Files:**

- Modify: `.github/workflows/`
- Create: `scripts/smoke-test.sh`
- Extend: `src-tauri/tests/` integration harness

---

## Self-review (spec coverage)

| Spec requirement          | Phase       |
| ------------------------- | ----------- |
| Provider parity inventory | 0           |
| Usage cache / LKG         | 1           |
| Credential store          | 1 stub → 2+ |
| Provider metadata         | 1           |
| Codex reference provider  | 2           |
| v1 provider fetch         | 3–4         |
| OS-native shells          | 5           |
| CLI                       | 6           |
| Release smoke             | 7           |

## Next session

Execute **Phase 2 Task 2.1** (Codex OAuth strategy) after Phase 1 lands on `feat/gap-closure-foundations`.
