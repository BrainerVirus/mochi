# Window & Menu UX Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Spec:** `docs/window-ux-polish/spec.md`
**Branch:** `feature/window-ux-polish`

**Goal:** Make Mochi's windows open instantly without layout shift, fit their content, and speak up via desktop notifications.

**Architecture:** Declarative menu/size tweaks in `tray/`; TanStack Query flag tuning for cache-first opens plus skeleton rows in widget/panel; release-notes collapse in the Update view; a new `notifications` Rust module (plugin-notification) fed by the refresh pipeline, updater, and a threshold evaluator driven by new settings fields.

**Tech Stack:** Tauri v2 + `tauri-plugin-notification` v2 / `@tauri-apps/plugin-notification` v2, React 19, TanStack Query 5, Tailwind 4 + shadcn skeleton, Zod 4, Vitest, Cargo tests.

## Global Constraints

- Cross-platform (macOS/Windows/Linux): no platform-only assumptions without guards; Linux-first verification, nothing macOS-only.
- TS/TSX files under 250 lines (split before 350 unless shadcn-owned); Rust modules under 300 lines (split before 450 unless data).
- No new `#[cfg(target_os)]` in `src-tauri/src/core/`.
- TDD: failing test first for every behavior change; full gate green before handoff (`pnpm lint`, `format:check`, `test`, `build`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all-targets`).
- Conventional commits, imperative, <50 chars, no footers; one concern per commit.
- Before UI work read `DESIGN.md` + `.agents/skills/shadcn/SKILL.md`; before query work read `.agents/skills/tanstack-query/SKILL.md`; before Tauri work read `.agents/skills/tauri-v2/SKILL.md`; before schema work read `.agents/skills/zod/SKILL.md`.
- Every new Tauri command registered in `generate_handler!`; new plugin permission added to `src-tauri/capabilities/default.json`.
- Lockfiles (`pnpm-lock.yaml`, `src-tauri/Cargo.lock`) committed with dependency changes (CI uses `--frozen-lockfile`).

---

### Task 1: Native menu About entry

**Files:**

- Modify: `src-tauri/src/tray/mod.rs:56-82` (menu model), `:284-286` (match arms), `:374` (ids test)

**Interfaces:**

- Consumes: `open_app_window` (already imported in `tray/mod.rs`), pending-route handoff (no change).
- Produces: `"about"` menu-event id routed to `/about` (Task 7 QA uses it).

- [ ] **Step 1:** Add the failing test. Extend the ids assertion at `src-tauri/src/tray/mod.rs:374`:

```rust
assert_eq!(
    ids,
    vec!["widget", "refresh", "settings", "update", "about", "quit"]
);
```

- [ ] **Step 2:** Run it, expect FAIL (`"about"` missing from model).

```bash
export RUSTUP_TOOLCHAIN=stable PATH="$HOME/.cargo/bin:$PATH"
cargo test --manifest-path src-tauri/Cargo.toml tray:: -- --nocapture 2>&1 | tail -5
```

Expected: assertion failure listing actual ids without `about`.

- [ ] **Step 3:** Add the model entry after the `update` item, before the separator:

```rust
TrayMenuEntry::Item {
    id: "about",
    label: "About Mochi",
},
```

- [ ] **Step 4:** Add the match arm after the `"update"` arm (`mod.rs:284-286`):

```rust
"about" => {
    let _ = open_app_window(app.clone(), "/about".to_string());
}
```

- [ ] **Step 5:** Run tests, expect PASS.

```bash
cargo test --manifest-path src-tauri/Cargo.toml tray:: 2>&1 | tail -3
```

Expected: `test result: ok`.

- [ ] **Step 6:** Commit.

```bash
git add src-tauri/src/tray/mod.rs
git commit -m "fix(tray): add About entry to native menu"
```

---

### Task 2: Warm-open cache-first usage queries

**Files:**

- Modify: `src/lib/query/usage-snapshots/usage-snapshots.ts:8-19`
- Test: `src/lib/query/usage-snapshots/usage-snapshots.test.ts` (extend)

**Interfaces:**

- Consumes: `queryKeys.usageSnapshots`, `getUsageStates`, `usageRefreshIntervalMs` (unchanged).
- Produces: mount-safe query options (Tasks 3 and 7 rely on `isLoading && !data` meaning truly-empty cache).

- [ ] **Step 1:** Read `.agents/skills/tanstack-query/SKILL.md`, then add failing assertions to `usage-snapshots.test.ts`:

```ts
const options = createUsageSnapshotsQueryOptions(300);
expect(options.refetchOnMount).toBe(false);
expect(options.refetchOnWindowFocus).toBe(false);
```

Keep the existing `refetchInterval` / `refetchIntervalInBackground` assertions untouched.

- [ ] **Step 2:** Run, expect FAIL (flags undefined).

```bash
pnpm vitest run src/lib/query/usage-snapshots/usage-snapshots.test.ts 2>&1 | tail -4
```

- [ ] **Step 3:** Minimal implementation in `createUsageSnapshotsQueryOptions`:

```ts
return queryOptions({
  queryKey: queryKeys.usageSnapshots,
  queryFn: getUsageStates,
  refetchOnMount: false,
  refetchOnWindowFocus: false,
  ...(refreshIntervalSeconds === undefined
    ? {}
    : {
        refetchInterval: usageRefreshIntervalMs(refreshIntervalSeconds),
        refetchIntervalInBackground: true,
      }),
});
```

Rationale: windows remount on every open; `refetchOnMount: false` serves cache when present and fetches only when empty; `refetchOnWindowFocus: false` stops Tauri focus events from refetching on open; interval polling is untouched so data stays fresh while mounted.

- [ ] **Step 4:** Run tests, expect PASS (full file, not just new assertions).
- [ ] **Step 5:** Commit.

```bash
git add src/lib/query/usage-snapshots/usage-snapshots.ts src/lib/query/usage-snapshots/usage-snapshots.test.ts
git commit -m "fix(ui): serve usage cache on window open"
```

---

### Task 3: Provider-list skeletons in widget + panel

**Files:**

- Create: `src/features/usage/components/provider-list-skeleton/provider-list-skeleton.tsx` (+ colocated `provider-list-skeleton.test.tsx`)
- Modify: `src/features/widget/components/widget-window/widget-window.tsx`, tray panel provider list (find via `grep -rn "usageSnapshotsQueryOptions" src/features/tray app/routes`), `src/features/usage/components/provider-usage-section/provider-usage-section.tsx` if it owns the row list (read it first; it already mentions skeletons)

**Interfaces:**

- Consumes: shadcn `src/components/ui/skeleton.tsx` (`Skeleton`), enabled-provider count (synchronous, from settings/selection already in memory), Task 2 options (`isLoading && data === undefined` ⟺ empty cache).
- Produces: skeleton rows that swap 1:1 for real rows (Task 7 QA).

- [ ] **Step 1:** Read `DESIGN.md` + `.agents/skills/shadcn/SKILL.md`. Write the failing test first:

```tsx
import { render, screen } from "@testing-library/react";
import { ProviderListSkeleton } from "./provider-list-skeleton";

it("renders one skeleton row per configured provider", () => {
  render(<ProviderListSkeleton providerCount={3} />);
  expect(screen.getAllByTestId("provider-skeleton-row")).toHaveLength(3);
});

it("renders nothing for zero providers", () => {
  const { container } = render(<ProviderListSkeleton providerCount={0} />);
  expect(container).toBeEmptyDOMElement();
});
```

- [ ] **Step 2:** Run, expect FAIL (component missing).
- [ ] **Step 3:** Minimal component — N shimmer rows shaped like provider rows, motion-safe:

```tsx
import { Skeleton } from "@/components/ui/skeleton";

export function ProviderListSkeleton({ providerCount }: { providerCount: number }) {
  if (providerCount <= 0) {
    return null;
  }
  return (
    <div aria-hidden="true">
      {Array.from({ length: providerCount }, (_, index) => (
        <div
          key={index}
          data-testid="provider-skeleton-row"
          className="flex items-center gap-3 py-2"
        >
          <Skeleton className="h-8 w-8 rounded-full" />
          <div className="flex-1 space-y-1.5">
            <Skeleton className="h-3 w-2/3" />
            <Skeleton className="h-2 w-full" />
          </div>
        </div>
      ))}
    </div>
  );
}
```

Keep the row geometry (avatar + two lines + padding) identical to the real provider row so the swap has zero layout shift; verify against the real row while wiring. Shimmer comes from shadcn's `Skeleton` (CSS animation; honor reduced motion via `motion-safe:` prefix on the animated class if the base component does not already).

- [ ] **Step 4:** Wire both surfaces with the empty-cache-only rule:

```tsx
const { data, isLoading } = useQuery(createUsageSnapshotsQueryOptions(refreshSeconds));
if (isLoading && data === undefined) {
  return <ProviderListSkeleton providerCount={enabledProviders.length} />;
}
```

Background refetches keep old `data` defined, so skeletons never flash over content. Get `enabledProviders` from whatever each surface already has in memory (settings query / selection) — no new fetch.

- [ ] **Step 5:** Run tests + lint on touched files, expect PASS.
- [ ] **Step 6:** Commit.

```bash
git add src/features/usage/components/provider-list-skeleton src/features/widget/components/widget-window/widget-window.tsx <panel-file(s)>
git commit -m "feat(ui): skeleton rows for empty provider lists"
```

---

### Task 4: Compact About/Update windows + notes expander

**Files:**

- Modify: `src-tauri/src/tray/panel.rs` (`ABOUT_WINDOW_*` / `UPDATE_WINDOW_*` consts near top, `app_window_size_for_path:235-243`, min-size block `:382-397`, size tests `:788-798`)
- Modify: `src/features/updates/components/update-page-content/update-page-content.tsx` (collapse notes behind expander)
- Test: Rust size tests + colocated vitest for the expander

**Interfaces:**

- Consumes: existing size-fn dispatch and min-size block (same shape, new numbers).
- Produces: compact dialogs (Task 7 QA measures no-scroll fit).

- [ ] **Step 1:** Read `DESIGN.md`. Add failing Rust test expectations: update the `/about` and `/update` cases at `panel.rs:788-798` to the new compact dims — About `400x300`, Update `420x260` (measure content at implementation; if content needs ±40px, adjust test + consts together and note why in the commit body… no — keep exact: implementer measures the rendered content and picks the smallest fitting dims, test and consts must match exactly).
- [ ] **Step 2:** Run, expect FAIL.
- [ ] **Step 3:** Update the four consts + confirm `app_window_size_for_path` and the min-size block pick them up with no logic change (both dispatch on the same path prefixes).
- [ ] **Step 4:** Update window content — replace inline release-notes render with a collapsed expander; notes region scrolls internally so the window never resizes:

```tsx
const [notesOpen, setNotesOpen] = useState(false);
<button type="button" onClick={() => setNotesOpen((open) => !open)} aria-expanded={notesOpen}>
  Release notes
</button>;
{
  notesOpen ? <div className="max-h-40 overflow-y-auto">{notes}</div> : null;
}
```

Adapt element names to the actual file (read it first); keep the Update button + progress-bar-only-while-updating behavior unchanged.

- [ ] **Step 5:** Colocated test: notes hidden by default, button toggles `aria-expanded` and reveals notes, window-size-irrelevant (no resize API involved).
- [ ] **Step 6:** Full file tests green (Rust size tests + vitest), then commit:

```bash
git add src-tauri/src/tray/panel.rs src/features/updates/components/update-page-content/
git commit -m "fix(ui): compact About and Update windows"
```

---

### Task 5: Notification plumbing + update/failure triggers

**Files:**

- Modify: `src-tauri/Cargo.toml`, `package.json` + `pnpm-lock.yaml`, `src-tauri/Cargo.lock`, `src-tauri/src/lib.rs` (plugin registration), `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/notifications/notifications.rs` (+ `#[cfg(test)] mod tests` inline, file must stay <300 lines)
- Modify: updater ready path (`src-tauri/src/updater/mod.rs` `install_update` success / ready branch — read it), refresh-failure path (`src-tauri/src/tray/mod.rs:253-277` refresh arm error branch)

**Interfaces:**

- Consumes: `MochiSettings.show_notifications` (`src-tauri/src/settings/mod.rs:171`, default true `:185`).
- Produces: `crate::notifications::send_notification(app: &AppHandle, title: &str, body: &str)` — checks master toggle, then sends via `NotificationExt`; Task 6 calls it for thresholds.

- [ ] **Step 1:** Read `.agents/skills/tauri-v2/SKILL.md`. Add deps: `tauri-plugin-notification` v2 line (cargo) + `@tauri-apps/plugin-notification` v2 (pnpm); commit lockfiles. Register `.plugin(tauri_plugin_notification::init())` in `lib.rs` beside the other `.plugin(...)` calls. Add `"notification:default"` to `permissions` in `default.json` (same shape as `"updater:default"`).
- [ ] **Step 2:** Failing test first — pure toggle logic needs no OS notification server. Structure `notifications.rs` so the gate is a pure fn:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_master_toggle_suppresses_notification() {
        assert!(!should_send_notification(false));
    }
    #[test]
    fn enabled_master_toggle_allows_notification() {
        assert!(should_send_notification(true));
    }
}
```

with `fn should_send_notification(show_notifications: bool) -> bool { show_notifications }` as the gate (permission-denied path degrades inside the Tauri call — log line only, never an error to the caller).

- [ ] **Step 3:** Run, expect FAIL (module missing), then implement:

```rust
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

fn should_send_notification(show_notifications: bool) -> bool {
    show_notifications
}

pub fn send_notification(app: &AppHandle, title: &str, body: &str) {
    let enabled = app
        .try_state::<crate::settings::SettingsState>()
        .and_then(|state| state.current().ok())
        .map(|settings| settings.show_notifications)
        .unwrap_or(true);
    if !should_send_notification(enabled) {
        return;
    }
    if let Err(error) = app.notification().builder().title(title).body(body).show() {
        crate::diagnostics::log_line("notify", &format!("send failed: {error}"));
    }
}
```

Mirror the exact `SettingsState` access pattern from the refresh arm (`mod.rs:256-260`). If the installed plugin's builder API differs, follow `node_modules/@tauri-apps/plugin-notification/README.md` + docs.rs for v2 and keep this same gate shape.

- [ ] **Step 4:** Wire the two triggers: after a stable update is staged/ready in `updater/mod.rs` call `send_notification(app, "Mochi update ready", "Restart to install <version>")`; in the refresh arm's failure path (where `refresh_all_providers_inner` errors before the `unwrap_or_else` fallback) call `send_notification(app, "Mochi refresh failed", <short error>)`. Keep messages short, no secrets, no URLs.
- [ ] **Step 5:** `cargo test`, `clippy -D warnings`, `cargo fmt --check` green; commit:

```bash
git add src-tauri/
git commit -m "feat(notify): desktop notifications plumbing"
```

Do NOT commit `package.json`/`pnpm-lock.yaml` here — separate commit `chore(deps): add notification plugin` if the worker prefers split commits; never mix.

---

### Task 6: Threshold settings + evaluator

**Files:**

- Modify: `src-tauri/src/settings/mod.rs:164-190` (`MochiSettings`), `ProviderConfig` struct (find it; add override), `src-tauri/src/settings/storage.rs:68` (defaults), settings tests
- Modify: frontend settings schema + `src/features/settings/components/settings-sections/settings-sections.tsx` (threshold fields; find `show_notifications` wiring via grep and follow it)
- Modify: refresh pipeline evaluation point (after `refresh_all_providers_inner` success in `mod.rs:253-277` or `status` module — read both, evaluate where snapshots + settings meet)
- Test: Rust settings defaults/validation tests + evaluator unit tests; vitest for fields/validation

**Interfaces:**

- Consumes: `crate::notifications::send_notification` (Task 5), snapshot usage percentages (same values the widget renders — no new computation).
- Produces: one-shot threshold notifications with re-arm (Task 7 QA).

- [ ] **Step 1:** Read `.agents/skills/zod/SKILL.md`. Failing tests first. Rust — global default 80, range validation helper is pure:

```rust
#[test]
fn default_global_warn_percent_is_80() {
    assert_eq!(MochiSettings::default().usage_warn_percent, 80);
}
```

```rust
#[test]
fn warn_percent_clamps_to_1_to_100() {
    assert_eq!(clamp_warn_percent(0), 1);
    assert_eq!(clamp_warn_percent(101), 100);
    assert_eq!(clamp_warn_percent(90), 90);
}
```

Evaluator crossing logic pure (armed-state in, decision out). Exact signatures the implementation must use:

```rust
fn clamp_warn_percent(value: u8) -> u8 {
    value.clamp(1, 100)
}
fn should_notify_threshold(usage: f64, threshold: u8, armed: bool) -> bool {
    armed && usage >= threshold as f64
}
fn rearmed_below_threshold(usage: f64, threshold: u8) -> bool {
    usage < threshold as f64
}
```

```rust
#[test]
fn crossing_armed_threshold_fires_once_then_disarms() {
    assert!(should_notify_threshold(85.0, 80, true));
    assert!(!should_notify_threshold(86.0, 80, false));
}
#[test]
fn dropping_below_rearms() {
    assert!(!should_notify_threshold(79.9, 80, false));
    // re-arm is the caller storing armed=true again; assert helper:
    assert!(rearmed_below_threshold(79.9, 80));
}
```

- [ ] **Step 2:** Run, expect FAIL (fields/fns missing).
- [ ] **Step 3:** Implement settings fields. `MochiSettings` gains:

```rust
#[serde(default = "default_usage_warn_percent")]
pub usage_warn_percent: u8,
```

```rust
fn default_usage_warn_percent() -> u8 {
    80
}
```

`ProviderConfig` gains `#[serde(default, skip_serializing_if = "Option::is_none")] pub warn_percent: Option<u8>`. Effective threshold = provider override else global; clamp both through `clamp_warn_percent` at read time (never reject stored settings — old files without the fields deserialize via defaults). Update `storage.rs` default test expectations (`show_notifications: false` fixtures stay as-is; add warn-percent assertions beside them).

- [ ] **Step 4:** Evaluator in the refresh-success path: for each enabled provider, compute effective threshold, compare against the snapshot percentage the widget already renders; keep armed-state in memory (HashMap<provider, bool>, default armed); on `usage >= threshold && armed` → `send_notification(app, "Mochi usage warning", "<Provider> at <pct>%")` + disarm; on `usage < threshold` → re-arm. General-usage threshold = same rule applied to the aggregate/overall usage value the Overview already computes (reuse it, no new metric).
- [ ] **Step 5:** Frontend: threshold number fields on provider rows + global default field, Zod integer min 1 max 100, follow the existing `show_notifications` wiring end to end (schema → section → save command). Vitest: invalid values rejected, valid persist shape.
- [ ] **Step 6:** Full gate green; commit(s) (settings backend + UI may split in two commits if cleaner):

```bash
git add src-tauri/src/settings/ <evaluator-file>
git commit -m "feat(notify): usage threshold evaluation"
```

---

### Task 7: QA evidence + full verification

**Files:**

- Create: `docs/qa/window-ux-evidence.md`
- Consumes: Tasks 1–6 on the branch.

- [ ] **Step 1:** Run the entire verification gate and record exact outputs:

```bash
pnpm lint && pnpm format:check && pnpm test && pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

- [ ] **Step 2:** Write `docs/qa/window-ux-evidence.md` with automated results (test counts, chunk sizes from `pnpm build` proving no 500 kB warning, capability permission present) plus the owner-run manual checklist: GNOME cold open (skeletons, N rows), warm reopen (instant, no fetch in log), About/Update sizes fit without scroll, each of the 3 notification triggers fires once, master toggle silences all, permission-denied path logs only.
- [ ] **Step 3:** Commit:

```bash
git add docs/qa/window-ux-evidence.md
git commit -m "docs(qa): window-UX evidence and checklist"
```
