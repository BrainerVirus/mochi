# SQLite Persistence And Refresh Flow Design

Date: 2026-06-04

## Summary

Mochi usage data is currently ephemeral because `UsageStore` is an in-memory cache and the app reads cached snapshots separately from live refresh commands. This creates confusing flows: first launch can detect provider credentials but show no fetched data until manual refresh, provider enable/disable changes can disagree with the Usage view, and repeated user interactions can stack refresh work.

This design makes Rust the authority for usage persistence and refresh orchestration. SQLite stores latest provider state and successful usage history. A refresh controller coordinates startup refresh, manual refresh, tray refresh, settings changes, and CLI refresh so each provider has at most one active fetch.

## Goals

- Show cached usage immediately when the app opens, then refresh eligible providers.
- Persist usage across restarts with SQLite.
- Auto-fetch only enabled providers with detected credentials.
- On first-ever launch only, auto-enable providers with detected credentials.
- After first launch, treat user enabled/disabled provider choices as authoritative.
- Delete usage data immediately when a provider is disabled.
- Keep last successful data visible when a refresh fails, marked stale/error.
- Show enabled providers even when credentials are missing or invalid.
- Use the same persisted data model for GUI, tray, widget, status-bar, and CLI.
- Fix macOS installer behavior so the `mochi` CLI is available after app install.
- Require TDD for every implementation task.

## Non-Goals

- Cloud sync or telemetry.
- Persisting raw credentials, cookies, tokens, or API keys in SQLite.
- Full analytics UI in this pass. The schema should keep bounded successful history for later analytics, but this work does not build charts or reports.
- Fetching usage from disabled providers.
- Deep links from missing-credential rows into provider settings.

## Architecture

Rust owns persisted usage state, provider credential detection, and refresh orchestration. React and TanStack Query remain the GUI view cache over typed Tauri commands.

`UsageStore` should gain a SQLite-backed repository behind its existing read/write interface. During Tauri setup, Mochi opens SQLite from the app data/config area, runs migrations, loads latest snapshots into memory, prunes old history, and records whether first-run provider detection has completed. If SQLite fails to open or migrate, Mochi falls back to in-memory mode for the current session and exposes a compact warning state instead of crashing.

A new refresh controller owns all live fetches. Startup refresh, manual GUI refresh, tray refresh, settings save reconciliation, and CLI `mochi usage --refresh` must use this controller. The controller dedupes by provider so repeated triggers join or no-op while a provider fetch is already running.

Reads must not start network work accidentally. These flows read cached/latest state only:

- `get_usage_snapshots`.
- tray sync.
- widget reads.
- status-bar output.
- `mochi usage` without `--refresh`.

## Startup Flow

On app startup:

1. Load settings.
2. Open SQLite or fall back to in-memory mode with warning state.
3. Load latest provider states from SQLite into `UsageStore`.
4. Prune successful history older than 90 days.
5. If `initial_provider_detection_completed` is false, detect provider credentials, auto-enable detected providers, persist settings, and set the flag true.
6. Return cached/latest state to the UI immediately.
7. Start background refresh only for enabled providers with detected credentials.
8. Skip network fetches for enabled providers with missing credentials and show local missing-credentials state.

After first-ever launch, provider detection must never auto-enable providers again. User preference is law.

## SQLite Data Model

SQLite stores usage state, not secrets.

### `usage_latest`

One row per provider. This powers app startup, GUI reads, tray sync, widget reads, status-bar output, and normal CLI reads.

Fields include:

- provider id.
- serialized `UsageSnapshot` when a snapshot exists.
- display state: `fresh`, `fetching`, `stale_error`, `missing_credentials`, or `credentials_need_refresh`.
- health.
- source.
- updated timestamp.
- last successful timestamp.
- last error message.
- last fetch attempt metadata.

### `usage_history`

Append-only successful snapshots for future analytics. Retention defaults to 90 days.

Only successful snapshots are stored here. Fetching, missing credentials, stale/error markers, and invalid-credential states do not enter history.

### `app_state`

Small key/value state, including:

- `initial_provider_detection_completed = true | false`.

## Provider State Rules

When a provider is disabled:

- Remove it from the Usage view immediately.
- Delete its `usage_latest` row.
- Delete its `usage_history` rows.
- Do not fetch it.
- Sync tray, widget, status-bar, and queries from the new enabled-provider list.
- Keep provider settings/configuration unless the user explicitly clears them elsewhere.

When a provider is enabled:

- Show it in Usage immediately.
- If credentials are detected, show a real-card-shaped fetching skeleton and refresh through the controller.
- If credentials are missing, show compact missing-credentials state and skip network fetch.

When detected credentials are expired or invalid:

- If a previous successful snapshot exists, keep it visible and mark it stale/error with a compact credentials-refresh message.
- If no successful snapshot exists, show a compact credentials-need-refresh state with no usage meters.
- Persist this latest state so the user sees the provider status on next launch.
- Do not add anything to `usage_history`.

When a non-auth refresh failure occurs:

- If a previous successful snapshot exists, keep it visible and mark it stale/error.
- If no previous successful snapshot exists, show a compact provider error state.
- Do not add failure states to `usage_history`.

## Refresh Behavior

Startup refresh:

- Runs automatically after cached state is available.
- Refreshes only enabled providers with detected credentials.

Manual GUI/tray refresh:

- Refreshes only enabled providers with detected credentials.
- Skips missing-credential providers and updates their local display state.
- Coalesces repeated refresh requests per provider.

Settings save reconciliation:

- Applies the settings change first.
- Deletes disabled providers' usage data immediately.
- Creates visible latest-state rows for newly enabled providers.
- Starts fetches only for newly enabled providers with detected credentials.
- Syncs tray/widget/status-bar after reconciliation.

Scheduled/polling refresh:

- Must use the refresh controller.
- Must obey the same enabled-provider and detected-credential rules.

## GUI Behavior

Usage should support these provider display states:

- fresh successful usage.
- fetching skeleton.
- stale/error with prior successful usage.
- missing credentials.
- credentials need refresh with no prior usage.
- generic error with no prior usage.

Fetching should use a skeleton shaped like the real usage card with a brief fetching message and smooth transition into the final provider state. Disabled providers disappear immediately because their persisted rows are deleted.

The UI should continue using TanStack Query for command results and stale/loading state, but it must not be the authority for server/cache state. Zustand remains only for local UI preferences.

## CLI And Status-Bar Behavior

`mochi usage` reads cached SQLite data only. It must be fast and must not trigger network calls.

`mochi usage --refresh` refreshes enabled providers with detected credentials through the same controller, shows a clean TUI-style progress indicator while work is active, then prints the updated usage.

`mochi usage --refresh --json` also refreshes first. Progress output must go to `stderr`; final JSON must go to `stdout` so scripts can consume it safely.

Missing-credential providers are skipped during refresh and represented as provider status in output.

Status-bar output reads cached/latest state by default so repeated bar polling is safe. Any future explicit refresh option must use the same controller rules.

## macOS CLI Installer

The macOS installer currently copies `Mochi.app` into `/Applications` but does not make `mochi` available on `PATH`.

The installer should create or update:

```text
/usr/local/bin/mochi -> /Applications/Mochi.app/Contents/MacOS/mochi
```

If `/usr/local/bin` is not writable, the installer should print a clear fallback command:

```bash
sudo ln -sf /Applications/Mochi.app/Contents/MacOS/mochi /usr/local/bin/mochi
```

When `MOCHI_INSTALL_DIR` is customized, both the symlink target and fallback command must use the actual installed app path.

## Error Handling

- SQLite open or migration failure falls back to in-memory mode for the session.
- The app surfaces a compact warning instead of a hard error screen.
- Provider fetch failures never panic.
- Auth/expired-credential errors are distinguishable from generic provider/network errors so the UI can show concise credential-refresh text.
- Refresh controller lock/dedupe failures must fail gracefully and not crash the app.

## Testing Strategy

Implementation must follow TDD. For every behavior change:

1. Write the failing test first.
2. Run it and confirm the expected failure.
3. Implement the minimal code to pass.
4. Run the test and confirm it passes.
5. Refactor only while tests stay green.

Rust unit tests should cover:

- SQLite migrations.
- latest snapshot writes and reads.
- successful history writes only.
- 90-day history pruning.
- disabled-provider deletion from latest and history.
- app-state first-run flag.
- SQLite failure fallback to in-memory mode.
- stale/error marking with prior successful data.
- expired credentials with prior successful data.
- expired credentials without prior successful data.
- missing-credentials state.
- per-provider refresh dedupe.

Rust command/controller tests should cover:

- `get_usage_snapshots` reads cache only.
- startup reconciliation auto-enables detected providers only on first launch.
- startup refresh fetches only enabled providers with detected credentials.
- manual refresh skips missing-credential providers.
- settings save deletes disabled provider data.
- settings save shows newly enabled providers immediately.
- refresh failure preserves last successful snapshot as stale/error.
- concurrent refresh requests coalesce.

Frontend tests should cover:

- usage display state mapping.
- fetching skeleton rendering.
- stale snapshot rendering.
- missing-credentials rendering.
- credentials-need-refresh rendering.
- provider enable/disable UI state.
- settings save query invalidation and sync behavior.

CLI tests should cover:

- `mochi usage` reads cache without fetching.
- `mochi usage --refresh` waits for refresh before printing.
- `mochi usage --refresh --json` writes progress to `stderr` and JSON to `stdout`.
- missing-credential providers are skipped during refresh.
- stale/error data prints cleanly.

Installer tests should cover:

- macOS install creates the CLI link when possible.
- custom `MOCHI_INSTALL_DIR` changes the symlink target.
- unwritable `/usr/local/bin` prints the fallback `sudo ln -sf` command.

## Verification Commands

Before claiming implementation completion, run the relevant full validation:

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Report the actual commands run and any failures.
