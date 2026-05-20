# Mochi Design Spec

Date: 2026-05-19

## Summary

Mochi is a friendly, cross-platform usage companion for AI coding tools. It tracks session limits, weekly limits, reset windows, provider status, and local usage signals for tools such as Codex, Claude, Cursor, Gemini, Copilot, Antigravity, Factory/Droid, z.ai, Kiro, and Augment.

Mochi is inspired by CodexBar's menu-bar usage model, but it is built from scratch around first-class support for macOS, Windows, and Linux. The product must feel like the macOS app or better on every supported OS, not like a single-platform app with ports bolted on.

Tagline: **Soft alerts before hard limits.**

## Goals

- Provide a clear, calm view of AI coding usage across the top 10 v1 providers.
- Support macOS, Windows, and Linux as first-class desktop platforms.
- Offer four usage surfaces: tray app, desktop widget, CLI, and status-bar integration.
- Ship installers/packages for all supported OS targets through CI.
- Use GitHub Flow: short-lived branches, PRs into `main`, and tags from `main` for stable releases.
- Provide two update channels: stable and unstable.
- Support auto-update checks, user-visible update prompts, one-click update installation, and app restart/reload after update.
- Keep privacy-first behavior: no server-side aggregation, no broad filesystem crawling, opt-in browser cookie access, and local-only credential storage.

## Non-Goals For V1

- Hosted sync, cloud account, or telemetry backend.
- Mobile apps.
- Marketplace-style third-party provider plugins.
- Full parity with every CodexBar and Win-CodexBar provider beyond the top 10 selected for v1.
- Reusing Swift UI code from CodexBar. Mochi can reference behavior and MIT-licensed logic with attribution, but the implementation is Rust/TypeScript-first.

## Supported Platforms

Mochi treats these platforms as release blockers for v1:

- macOS 14+ on Apple Silicon and Intel.
- Windows 10/11 on x64.
- Linux x64 on modern desktop distributions with AppIndicator-compatible tray support where available.

Linux release artifacts must include:

- AppImage.
- Debian package (`.deb`).
- RPM package (`.rpm`).
- Flatpak bundle and Flatpak repository metadata.

Linux tray support varies by desktop environment. Mochi must therefore provide fallback surfaces that are not second-class: desktop widget, CLI, and status-bar output.

## Product Surfaces

### Tray App

The tray app is the default desktop experience on all OSes.

- Shows a dynamic tray icon with session and weekly usage bars.
- Click opens a compact usage panel.
- Right-click or secondary action opens a menu with refresh, settings, channel, update, and quit actions.
- Supports merged overview mode for multiple providers.
- Shows stale/error/incident state without hiding the last known good usage.

### Desktop Widget

The widget is a small always-on-top or optionally floating window.

- Shows a compact provider overview suitable for Linux desktops without reliable tray behavior.
- Can be pinned, moved, resized within defined min/max dimensions, and hidden.
- Uses the same data model as the tray panel.
- Supports compact, normal, and expanded density modes.

### CLI

The CLI is the automation and fallback interface.

Core commands:

- `mochi usage` shows usage for enabled providers.
- `mochi usage --provider claude` filters to one provider.
- `mochi usage --all --json` prints machine-readable usage for all known providers.
- `mochi status` shows provider incidents and stale/error states.
- `mochi cost --provider codex --days 30` shows local cost usage where supported.
- `mochi config get|set|list` reads and writes local configuration.
- `mochi update check` checks the selected update channel.
- `mochi update install` installs a discovered update when the package type supports in-app updates.
- `mochi status-bar --format waybar` prints Waybar JSON.

### Status-Bar Integration

Linux status-bar support is a first-class surface, not an afterthought.

Supported v1 outputs:

- Waybar JSON via `mochi status-bar --format waybar`.
- Generic JSON via `mochi status-bar --format json`.
- Plain text via `mochi status-bar --format text`.

The command must be fast, non-interactive, and safe for repeated execution from a status bar. It reads cached usage by default and only refreshes if explicitly requested.

## V1 Providers

Mochi ships with these providers in v1:

1. Codex.
2. Claude.
3. Cursor.
4. Gemini.
5. Copilot.
6. Antigravity.
7. Factory/Droid.
8. z.ai.
9. Kiro.
10. Augment.

Each provider must declare:

- Stable provider ID.
- Display name and icon metadata.
- Supported usage windows: session, weekly, monthly, credits, or provider-specific windows.
- Supported auth sources: CLI, OAuth, API key, browser cookies, local config, or local probe.
- Fetch strategy order.
- Whether fallback between strategies is allowed.
- Provider status URL or incident source if available.
- Redaction rules for diagnostics.

## Architecture

Mochi uses a unified Tauri v2 architecture:

- Rust owns provider fetching, secure storage, CLI, tray state, update checks, status-bar output, notifications, and filesystem access.
- React + TypeScript owns rendering for the tray panel, settings, provider details, and widget window.
- The frontend communicates with Rust through typed Tauri commands.
- The same Rust data model powers GUI, CLI, widget, tray icon, and status-bar output.

Recommended stack:

- Tauri v2.
- Rust stable toolchain.
- React + TypeScript.
- Tailwind CSS.
- Vite.
- `clap` for CLI parsing.
- `serde`/`serde_json` for data models.
- `reqwest` for HTTP.
- `tokio` for async runtime.
- `keyring` or platform-specific bindings for secure credential storage.
- Tauri updater plugin for non-Flatpak desktop packages.

## Module Boundaries

Expected top-level structure:

```text
mochi/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── cli/
│   │   ├── core/
│   │   ├── providers/
│   │   ├── browser/
│   │   ├── auth/
│   │   ├── fetch/
│   │   ├── settings/
│   │   ├── tray/
│   │   ├── widget/
│   │   ├── status_bar/
│   │   ├── status/
│   │   ├── notifications/
│   │   └── updater/
│   └── tauri.conf.json
├── src/
│   ├── components/
│   ├── hooks/
│   ├── lib/
│   └── styles/
├── .github/workflows/
└── docs/
```

### Core Rust Model

The Rust core exposes provider-agnostic data models:

- `ProviderId`.
- `ProviderMetadata`.
- `UsageSnapshot`.
- `UsageWindow`.
- `ProviderIdentity`.
- `ProviderHealth`.
- `ProviderStatus`.
- `FetchAttempt`.
- `FetchOutcome`.
- `MochiSettings`.

Providers implement a trait similar to:

```rust
pub trait Provider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn strategies(&self) -> Vec<Box<dyn FetchStrategy>>;
}

#[async_trait::async_trait]
pub trait FetchStrategy: Send + Sync {
    fn id(&self) -> &'static str;
    fn kind(&self) -> FetchKind;
    async fn is_available(&self, ctx: &FetchContext) -> ProviderResult<bool>;
    async fn fetch(&self, ctx: &FetchContext) -> ProviderResult<FetchOutcome>;
    fn should_fallback(&self, error: &ProviderError) -> bool;
}
```

The provider registry should be compile-time and explicit. Dynamic runtime plugin loading is excluded from v1.

### Frontend Model

React components must remain view-focused. They should not parse provider responses, read credentials, or access browser data.

Primary UI components:

- `TrayPanel`.
- `WidgetWindow`.
- `SettingsWindow`.
- `UsageCard`.
- `UsageMeter`.
- `ProviderList`.
- `ProviderDetail`.
- `MochiMascot`.

The mascot is a visual companion to the data. Usage state drives mascot expressions:

- Normal: happy mochi.
- Warning: worried mochi.
- Critical: flattened or sweating mochi.
- Reset soon: excited mochi with clock.
- All good: bouncy mochi.

## Data Flow

```text
Provider source
  ├─ CLI / PTY
  ├─ OAuth / API
  ├─ browser cookies
  ├─ local config
  └─ local logs
       ↓
FetchStrategy
       ↓
Provider registry
       ↓
UsageStore cache
       ↓
Tray icon / tray panel / widget / CLI / status-bar / notifications
```

Usage refreshes are bounded by timeout and refresh interval settings. UI surfaces read cached state and can request an explicit refresh. Failed refreshes keep the last known good snapshot and mark the provider stale or errored.

## Settings And Storage

Settings are stored in a local JSON file under the platform config directory:

- macOS: `~/Library/Application Support/Mochi/settings.json`.
- Windows: `%APPDATA%\Mochi\settings.json`.
- Linux: `${XDG_CONFIG_HOME:-~/.config}/mochi/settings.json`.

Secrets must not be stored in plaintext settings.

Secure storage:

- macOS: Keychain.
- Windows: DPAPI-backed credential storage.
- Linux: libsecret where available, with an explicit encrypted-file fallback only if the user opts in.

Browser cookie access is opt-in per provider and per browser. Diagnostics must redact tokens, cookies, auth headers, emails when configured, and organization identifiers when configured.

## Update Channels

Mochi has two update channels:

### Stable

- Default channel for normal users.
- Published from version tags such as `v1.0.0`.
- Requires passing all release checks.
- Requires signed updater metadata.
- Requires installer artifacts for macOS, Windows, AppImage, `.deb`, `.rpm`, and Flatpak.

### Unstable

- Opt-in channel for users who want latest builds.
- Published from every successful merge to `main`.
- Version format: `MAJOR.MINOR.PATCH-main.RUN+SHORT_SHA`, for example `0.1.0-main.42+abc1234`.
- Uses the same updater mechanism and artifact matrix as stable where possible.
- Clearly labels itself as unstable in the app UI, About panel, and update prompt.

## Auto-Update Behavior

On startup and then at a configurable interval, Mochi checks the selected channel for updates.

Expected user flow:

1. Mochi detects a newer version for the selected channel.
2. Mochi shows a non-blocking prompt with release notes, version, channel, and package type.
3. User clicks Update.
4. Mochi downloads and verifies the update.
5. Mochi installs the update.
6. Mochi restarts or reloads automatically.
7. If installation fails, Mochi keeps running and shows a clear recovery message.

Implementation rules:

- Non-Flatpak packages use the Tauri updater plugin.
- Flatpak builds use Flatpak-managed update flow. Mochi still presents the same update UI, but the install action invokes the Flatpak update mechanism and restarts the app after the update completes.
- Update manifests are channel-specific.
- Update artifacts are signed.
- Stable builds must never update to unstable unless the user explicitly switches channel.
- Unstable users can switch back to stable from settings.

## GitHub Flow

Branching strategy:

- `main` is the only long-lived branch.
- Feature work happens on short-lived branches.
- Pull requests are required before merging to `main`.
- Required checks must pass before merge.
- Squash merge is preferred for a clean history.
- Stable releases are created by tagging a commit already on `main`.

Release behavior:

- Push to `main` after PR merge publishes unstable builds and updates the unstable update feed.
- Push tag `v*` publishes stable builds and updates the stable update feed.
- Pull requests run validation but do not publish update feeds.

## CI/CD Requirements

Required workflows:

### Pull Request Validation

Runs on every PR:

- Rust format check.
- Rust clippy.
- Rust tests.
- TypeScript typecheck.
- frontend lint.
- frontend tests.
- Tauri build smoke test where feasible.
- Secret scanning and dependency audit.

### Unstable Release

Runs on push to `main`:

- Builds macOS app and DMG for Apple Silicon and Intel or a universal binary.
- Builds Windows MSI and NSIS installer if enabled.
- Builds Linux AppImage, `.deb`, `.rpm`, and Flatpak.
- Signs update artifacts.
- Publishes an unstable GitHub prerelease or channel artifact release.
- Updates the unstable update feed.

### Stable Release

Runs on `v*` tags:

- Repeats the full build matrix.
- Requires release signing credentials.
- Publishes GitHub Release artifacts.
- Updates the stable update feed.
- Generates checksums and release notes.

### Package Smoke Tests

Runs after release artifacts are built:

- Verify each installer/package exists.
- Verify checksums.
- Verify updater manifest points at existing artifacts.
- Verify CLI starts and prints version.
- Verify GUI binary starts in a headless-safe smoke mode where possible.

## Signing And Trust

Stable releases should be signed per platform:

- macOS: Developer ID signing and notarization.
- Windows: code-signing certificate for installer and executable.
- Linux: checksums and optional GPG signatures for artifacts and repository metadata.
- Tauri updater: signed update manifests and archives.

Unstable builds must use signed Tauri updater metadata. Platform code-signing may be omitted until stable release credentials are available, but the UI and release notes must make unstable trust status visible.

## Error Handling

Mochi must prefer clear degradation over silent failure.

- Provider fetch failure keeps last known good data and marks state stale.
- Auth failure explains which source failed and what action fixes it.
- Cookie import failure explains browser, profile, and permission issue without leaking secrets.
- Update failure keeps current version running and offers retry or manual download.
- Unsupported tray environments suggest widget or status-bar mode.

## Testing Strategy

Testing must cover core logic before UI polish.

- Unit tests for shared data models and percentage calculations.
- Provider parser tests with redacted fixtures.
- Strategy availability tests per provider.
- Settings migration tests.
- Secure-store abstraction tests with platform-specific mocked backends.
- CLI snapshot tests for text and JSON output.
- Frontend component tests for usage cards, update prompt, settings, and widget.
- CI artifact smoke tests for each OS package.

## Phased Delivery

### Phase 1: Foundation

- Scaffold Tauri + React + Tailwind.
- Define core Rust models and provider traits.
- Add settings and secure storage abstractions.
- Add CLI skeleton.
- Add basic CI validation.

### Phase 2: Core Providers And Cache

- Implement usage cache and refresh loop.
- Implement Codex, Claude, and Copilot first.
- Add provider tests and CLI usage output.

### Phase 3: Desktop Surfaces

- Implement tray icon and tray panel.
- Implement desktop widget.
- Implement notifications.
- Implement status-bar output.

### Phase 4: Complete V1 Provider Set

- Add Cursor, Gemini, Antigravity, Factory/Droid, z.ai, Kiro, and Augment.
- Add browser cookie import UI.
- Add provider detail diagnostics.

### Phase 5: CI Packaging And Release Channels

- Build all OS installers/packages in CI.
- Add stable and unstable update feeds.
- Add release signing.
- Add smoke tests for packages and update manifests.

### Phase 6: Auto-Update And Hardening

- Implement update check UI.
- Implement update download, install, and restart flow.
- Add Flatpak-managed update action.
- Harden error handling, redaction, and fallback behavior.

### Phase 7: Brand Polish And Release Readiness

- Add Mochi mascot states and theme.
- Complete docs.
- Validate first-run UX on each OS.
- Prepare stable v1 release.

## Acceptance Criteria

Mochi v1 is ready when:

- macOS, Windows, and Linux builds are produced by CI from the same commit.
- AppImage, `.deb`, `.rpm`, and Flatpak Linux packages are produced.
- Stable and unstable update feeds work independently.
- A user can switch channels in settings.
- The app detects an update, prompts the user, installs on click, and restarts/reloads.
- Tray app, desktop widget, CLI, and status-bar output all use the same cached provider data.
- The top 10 providers have tests for parser and strategy availability behavior.
- Provider secrets are stored outside plaintext settings.
- Failed provider refreshes and failed updates show actionable errors without losing last known usage.

## Risks

- Linux tray behavior differs across desktop environments. The widget and status-bar modes mitigate this.
- Flatpak self-update restrictions require a Flatpak-managed update path rather than the same binary replacement flow used by Tauri updater.
- Some provider APIs and dashboards are unofficial and may change. Provider strategies must be isolated, timeout-bounded, and testable with fixtures.
- Browser cookie decryption differs by OS and browser. Cookie access must be opt-in and diagnostics must guide users to manual auth alternatives.

## Open Decisions Locked By This Spec

- Tauri unified core is the chosen architecture.
- GitHub Flow is the chosen branching strategy.
- `main` is the unstable channel source.
- Semver tags are the stable channel source.
- Linux v1 includes Flatpak in addition to AppImage, `.deb`, and `.rpm`.
