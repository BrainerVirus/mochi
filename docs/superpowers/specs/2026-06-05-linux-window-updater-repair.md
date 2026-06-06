# Linux Window and Updater Repair Spec

**Goal:** Fix Linux tray/widget/settings parity, restore native window behavior, clean release-notes UX, and make stable/unstable in-app updates work from installed 0.1.x/0.2.x builds.

**Scope:** Tauri v2 desktop shell, React tray/widget/settings/update UI, release workflows, and updater feed publication. No provider-fetching behavior changes are included.

---

## Evidence Summary

### Tauri facts used as constraints

- Tauri native menus support checked items through `CheckMenuItemBuilder` and runtime check-state updates; plain `MenuItem` cannot show the selected channel. See [Tauri window/menu docs](https://v2.tauri.app/learn/window-menu/).
- Tauri updater requires `bundle.createUpdaterArtifacts`, a real updater `pubkey`, HTTPS endpoints, and JSON containing `version`, platform URL, and platform `signature`. See [Tauri updater docs](https://v2.tauri.app/plugin/updater/).
- Tauri updater endpoint variables are limited to `{{current_version}}`, `{{target}}`, and `{{arch}}`; custom variables such as channel are not supported by config substitution, so Mochi's current Rust-side channel-specific endpoint construction is the right place for channel selection.
- Tauri custom titlebars require explicit minimize/maximize/close button wiring when app chrome replaces native chrome. See [Tauri window customization docs](https://v2.tauri.app/learn/window-customization/).

### Repository evidence

- Native tray menu is built in `src-tauri/src/tray/mod.rs`.
  - `Show usage` is hard-coded first (`MenuItem::with_id(app, "usage", "Show usage", ...)`).
  - `Show widget` is third and named ambiguously.
  - Stable/unstable channels are plain `MenuItem`s, so no checked state can appear.
  - `setup_tray` currently does not receive settings, does not read `SettingsState.current().update_channel`, and does not retain check-item handles for later updates.
- `Show usage` calls `open_tray_panel(app, "/")` from a native menu event. It duplicates the direct tray-icon behavior and is likely the crash path reported on Linux indicator menus.
- Native tray percentage selection is not durable across every refresh path.
  - The selected provider tab is stored only in frontend Zustand/localStorage by `src/lib/stores/tray-ui-store.ts`.
  - Normal data-change sync in `src/hooks/use-tray-usage-sync.ts` correctly calls `syncTrayUsage(selectedTab)`.
  - `src/hooks/use-cold-start-provider-refresh.ts` calls `syncTrayUsage()` with no selected tab after boot refresh.
  - `src/hooks/use-tray-events.ts` calls `syncTrayUsage()` with no selected tab after native `tray-refresh` and after settings save.
  - Rust `TraySelection::parse(None)` in `src-tauri/src/tray/presentation.rs` resolves to `TraySelection::Overview`, so any selection-less sync explicitly repaints the native tray as the general overview percentage.
  - Default `refresh_interval_seconds` is 300 seconds, which matches the reported "after a few minutes" timing for refresh-related repaint bugs.
- The widget window is created in `src-tauri/src/widget/commands.rs` with `inner_size(320.0, 420.0)` and then immediately height-synced by the frontend. On Linux screenshots, the native decorated outer window remains larger than the actual tray-panel content.
- The widget window is also predeclared in `src-tauri/tauri.conf.json`; `setup_widget` reuses an existing configured window when present, so production fixes may need both the config entry and Rust builder path.
- `src/components/widget/widget-window.tsx` wraps the tray panel in `bg-background flex h-full ... overflow-hidden`; `TrayPanelShell` adds the inner rounded panel. This creates a separate outer webview background plus an inner bordered container on Linux.
- `src/components/layout/root-component.tsx` only applies full-height flex classes to the React root wrapper when `platform === "macos" || platform === "windows"`. Linux app/tray/widget routes still rely on `h-full` descendants, which can produce the empty lower region and broken scroll containers.
- `src/components/settings/settings-form.tsx` always renders `LinuxTrayHint` inside settings on Linux. The requested behavior is no top Linux tray message in settings.
- Vertical scrolling is handled by `ScrollFadeRegion` in `src/components/tray/scroll-fade-region.tsx`. It hides native scrollbars with `scrollbar-none`, adds `overscroll-y-contain`, and overlays absolute chevron buttons from `scroll-fade-overlays.tsx`. Linux users see a scrollbar but wheel scrolling fails in widget/settings/update notes, so this shared scroll primitive is the highest-leverage fix target.
- The "What's new" dialog is `src/components/updates/release-notes-dialog.tsx`. It only enables `ScrollFadeRegion` when `sections.length > 2`; long one-section notes can overflow without a scrollport.
- Release-note parsing in `src/lib/updates/format-patch-notes.ts` keeps every non-empty markdown line. It does not remove install-command sections, binary lists, or artifact sections before rendering "What's new".
- Updater config in `src-tauri/tauri.conf.json` has `"pubkey": "MOCHI_UPDATER_PUBLIC_KEY_REPLACED_BY_CI"` and lacks `"bundle": { "createUpdaterArtifacts": true }`.
- Release workflows use `tauri-apps/tauri-action@v0.6.2`, but neither `.github/workflows/release-stable.yml` nor `.github/workflows/release-unstable.yml` publishes updater JSON to `https://mochi-app.github.io/mochi/updates/...`.
- `src-tauri/tauri.conf.json` currently bundles multiple artifact formats (`dmg`, `msi`, `nsis`, `appimage`, `deb`, `rpm`), so updater feed generation must choose the signed updater artifact intentionally instead of guessing from release assets.
- Public checks on 2026-06-05 returned 404 for:
  - `https://mochi-app.github.io/mochi/updates/darwin/aarch64/0.1.7/stable.json`
  - `https://mochi-app.github.io/mochi/updates/linux/x86_64/0.1.7/stable.json`
  - `https://mochi-app.github.io/mochi/updates/darwin/aarch64/0.1.7/unstable.json`
  - `https://mochi-app.github.io/mochi/updates/linux/x86_64/0.1.7/unstable.json`
  - `https://mochi-app.github.io/mochi/updates/darwin/aarch64/0.2.0/stable.json`
  - `https://mochi-app.github.io/mochi/updates/linux/x86_64/0.2.0/stable.json`
- GitHub release `v0.2.0` has installer assets (`.dmg`, `.AppImage`, `.deb`, `.rpm`, `.msi`, `.exe`, `.app.tar.gz`) but no visible updater manifest/signature assets. The `unstable` release similarly accumulates installer assets but no updater metadata.
- The release matrix includes Windows x64 in addition to macOS and Linux, so updater-feed recovery should include Windows endpoints even though Windows user testing is still pending.

## Root Causes

1. **Native tray menu drift:** The Linux indicator fallback menu still exposes an obsolete `Show usage` item and uses plain menu items for update channels.
2. **Tray provider selection is lost at sync boundaries:** Some refresh flows call `syncTrayUsage()` without the current tab, and the Rust command treats a missing selection as overview/general.
3. **Linux layout not getting full-height shell classes:** Linux is excluded from the full-height root wrapper even though Linux shell CSS and scroll containers depend on the same height chain.
4. **Widget shell duplicates tray panel chrome:** The widget route combines a full webview background with the inner tray panel styling, while native Linux decorations add another outer frame.
5. **Shared scroll primitive is too aggressive for Linux:** Hidden scrollbars, edge controls, mask overlays, and `overscroll-y-contain` are bundled together for every vertical scroll surface. This makes Linux wheel behavior fragile and hides the real scroll affordance.
6. **Settings includes Linux onboarding copy in the persistent settings UI:** `LinuxTrayHint` belongs in docs or first-run diagnostics, not inside the normal settings window.
7. **Release notes render release bodies instead of patch notes:** The current release bodies intentionally include install commands; the app shows them because no filtering/sanitization exists.
8. **Updater artifacts and feed are missing:** The app checks a GitHub Pages feed that is not published. Builds are not configured to create updater artifacts, and the public key replacement is not guaranteed by the workflows.

## Required Behavior

### Tray menu

- Remove native menu item `Show usage`.
- Make the first native tray menu item `Open Mochi` or `Open usage panel`; it should open the same rich tray panel as direct tray-icon activation without using the crash-prone duplicate `usage` path.
- Rename `Show widget` to `Open widget` and place it first if the native indicator menu must prioritize the persistent window. Preferred ordering:
  1. `Open widget`
  2. `Refresh usage`
  3. `Settings`
  4. `Update channel`
  5. `Check for updates`
  6. separator
  7. `Quit Mochi`
- Convert channel entries to checked menu items and synchronize checked state after settings load and after `tray-set-channel`.
- Build initial checked state from `SettingsState.current().update_channel` in Rust during tray setup.
- Retain native channel check-item handles in Rust state, or add a command such as `sync_tray_update_channel(channel)` that updates checked states after frontend settings save. Tests must cover startup state and post-save state.

### Tray provider percentage persistence

- When the user selects a provider tab, the native tray title/icon must keep showing that provider until the user selects another tab or the provider becomes disabled/unavailable.
- Scheduled polling, cold-start refresh, manual refresh, settings saves, and background usage-cache invalidations must not repaint the native tray to overview unless the stored selected tab is invalid.
- Every frontend caller of `syncTrayUsage` must pass the current selected tab, or a central helper must read `useTrayUiStore.getState().selectedTab` and pass it.
- Add a pure helper for selection validity: a selected provider remains valid if it is still enabled in settings. Missing snapshots, loading states, empty cache reads, or temporary fetch failures must not clear the stored tab. Fall back to `overview` only when the provider is disabled, removed from the provider schema, or the stored value is invalid.
- Add diagnostics for selection-less native tray sync attempts during development, because missing selection means "reset to overview" in the current Rust API.

### Linux widget and settings layout

- Linux app/tray/widget windows must receive the same full-height root wrapper used by native-glass shells, while keeping Linux body backgrounds opaque.
- Widget content should have a single visible container. On Linux, remove the separate outer empty surface by making the widget route either:
  - render the same panel content without an additional full-window background, or
  - make the native widget window size exactly match the inner panel and remove redundant outer padding/background.
- Widget width and x padding should match macOS visual density. The effective content max width should stay `360px`, and Linux should not add extra horizontal padding beyond tray-panel spacing tokens.
- Update both widget creation sources: `src-tauri/tauri.conf.json` app window defaults and `src-tauri/src/widget/commands.rs`. Confirm which source creates the production widget on each platform before finalizing size/decorations fixes.
- Settings must remove `LinuxTrayHint` from the normal page.

### Scrolling

- Vertical wheel/trackpad scrolling must work in:
  - widget usage content,
  - settings form,
  - update page notes,
  - "What's new" dialog.
- Vertical scroll surfaces should use visible, native scroll behavior on Linux/Windows unless a platform test proves the custom fade region is safe.
- Keep fade affordances optional and non-blocking: overlays must not intercept wheel events, and scrollports must not depend on section count to become scrollable.

### Native window controls

- Settings and widget windows on Linux and Windows should use native decorations by default.
- First audit existing `src-tauri/src/linux_window_controls.rs` behavior, which already calls `set_decorations(true)` and `set_resizable(true)` for Linux settings/widget windows.
- Define a diagnostic field for each decorated window label that records attempted decoration/resizable calls, returned result, window label, platform, and whether the window was created from config or Rust builder.
- Only add a fallback custom titlebar if diagnostics show native decoration setup succeeded but the affected Linux window manager still does not expose minimize/maximize/close controls. Eligible fallback labels are `settings` and `widget`; the tray popover remains undecorated.
- Do not add custom controls on macOS, where current native/overlay behavior is acceptable.

### What's new

- The dialog must always cap height and scroll notes when content exceeds available space.
- Render only patch notes for the current installed/update version.
- Remove sections whose headings or body are install/distribution-oriented, including install commands, binary/artifact lists, and stable/unstable install instructions.
- Sanitize notes in both updater and fallback sources: `src/lib/query/update-check.ts` for updater manifests, `src/lib/updates/current-release-notes.ts` for GitHub release fallback, and read/render paths so old cached raw notes cannot leak install instructions.
- If updater check fails and the app falls back to GitHub release notes, the fallback can only fetch the installed `appVersion()` release notes unless a newer version is known from a successful manifest. The UI must label this honestly as current installed-version notes, not available-update notes.
- The release workflows should publish release notes that separate `### What's changed` from `### Install ...`; the app should extract only the change section.

### Updater

- Configure Tauri to create updater artifacts.
- Replace the updater public key at build time with the real public key and fail CI if signing secrets or pubkey replacement are missing.
- Publish stable and unstable updater JSON files to the exact endpoint family the app checks:
  - `/updates/{{target}}/{{arch}}/{{current_version}}/stable.json`
  - `/updates/{{target}}/{{arch}}/{{current_version}}/unstable.json`
- Backfill feeds for every supported installed `current_version` that should recover through in-app update. Minimum recovery set: `0.1.7` and `0.2.0`, plus any newer released version that ships before this fix lands. Do not only publish the newest version directory.
- Publish both `stable.json` and `unstable.json` for macOS arm64/x64, Linux x64, and Windows x64 recovery endpoints.
- Feed JSON must contain the newest version, notes, pub date, and signed platform artifact entries using Tauri's expected `OS-ARCH` platform keys.
- Define and test exact endpoint-to-platform mapping:
  - endpoint `darwin/aarch64/...` -> Tauri platform key `darwin-aarch64`;
  - endpoint `darwin/x86_64/...` -> `darwin-x86_64`;
  - endpoint `linux/x86_64/...` -> `linux-x86_64`;
  - endpoint `windows/x86_64/...` -> `windows-x86_64`.
- Define which generated updater artifact URL and signature each platform key uses. The feed script must not infer updater artifacts from arbitrary installer assets when multiple package types exist.
- Preserve Rust-side channel selection in `src-tauri/src/updater/mod.rs`, but add tests for exact URL generation and unsupported channels.
- Add release validation that curls representative macOS, Linux, and Windows endpoints for older supported versions and verifies HTTP 200 plus required updater fields.

## Implementation Plan Outline

### Task 1: Tray native menu cleanup

- Modify `src-tauri/src/tray/mod.rs`.
- Replace `MenuItem` channel entries with `CheckMenuItemBuilder`.
- Remove `"usage"` event branch.
- Add a small Rust helper that builds the tray menu from current settings channel, so tests can assert ordering and checked state.
- Add Rust state or a command/event bridge for updating native channel check state after frontend settings saves.
- Add Rust/TS tests for initial menu item labels, initial checked channel, and post-save channel check state.

### Task 2: Root layout and Linux scroll repair

- Modify `src/components/layout/root-component.tsx` so Linux tray/app/widget windows also receive full-height flex wrapper classes.
- Keep Linux `html/body` opaque via existing CSS selectors.
- Modify `ScrollFadeRegion` to support a `mode="native"` or `showControls={false}` path for vertical app-window/widget scroll regions.
- Update `settings-form.tsx`, `tray-panel-shell.tsx`, `update-page-content.tsx`, and `release-notes-dialog.tsx` to use the safer vertical mode.
- Add component/CSS tests proving Linux shells get full-height classes and vertical scroll regions do not render clickable overlays over the scrollport.

### Task 3: Tray selected-provider persistence

- Modify `src/hooks/use-cold-start-provider-refresh.ts`, `src/hooks/use-tray-events.ts`, and any helper used by settings-save reconciliation so they call tray sync with the current selected tab.
- Prefer a central frontend helper such as `syncCurrentTrayUsage()` under `src/lib/tauri/` or `src/lib/stores/` that reads `useTrayUiStore.getState().selectedTab` outside React render and calls `syncTrayUsage(selectedTab)`.
- Keep `src/hooks/use-tray-usage-sync.ts` as the normal data-change sync path, but ensure it also uses the same helper or same selection-validity rules.
- Selection validity must be settings-driven, not snapshot-driven: keep an enabled provider selected through missing/empty snapshots and in-flight refresh states.
- Add tests proving:
  - cold-start refresh syncs the current selected provider, not `undefined`;
  - native `tray-refresh` syncs the current selected provider after refreshing;
  - settings-save reconciliation does not reset provider selection;
  - invalid/disabled selected providers still fall back to `overview`.
- Add Rust tests only if the command semantics change; if `None` continues to mean overview, frontend tests must enforce that no refresh path passes `None` accidentally.

### Task 4: Widget visual parity

- Modify `src/components/widget/widget-window.tsx`, `src/lib/utils/tray-panel-layout.ts`, `src-tauri/src/widget/commands.rs`, and the `widget` entry in `src-tauri/tauri.conf.json`.
- Add a test or diagnostic assertion documenting whether production uses the configured `tauri.conf.json` widget window or the Rust builder fallback.
- Remove redundant outer background/padding in the widget route.
- Ensure native widget height sync includes exactly the content shell height and clamps to monitor height.
- Rename UI/native labels from "Show widget" to "Open widget" or a chosen final name.
- Add tests for widget height clamp and route shell class names.

### Task 5: Settings cleanup

- Remove `LinuxTrayHint` from `src/components/settings/settings-form.tsx`.
- Keep Linux tray guidance in `docs/linux.md` and diagnostics only.
- Add/update a test to assert settings does not render `data-linux-tray-hint`.

### Task 6: Release-note sanitization

- Add a `sanitizeReleaseNotesForApp(notes: string, version?: string)` helper under `src/lib/updates/`.
- Keep headings/items under "What's changed", "Changes", "Fixes", "Features", and similar patch-note sections.
- Drop "Install", "Install stable", "Install unstable", "Binaries", "Assets", "Downloads", command blocks, and raw artifact names.
- Use the sanitizer before caching in `src/lib/query/update-check.ts` and `src/lib/updates/current-release-notes.ts`, and again when reading/rendering cached notes in `ReleaseNotesDialog`/`UpdatePageContent`.
- Add unit tests with the current `v0.2.0` release-body shape.

### Task 7: Updater feed generation and validation

- Modify `src-tauri/tauri.conf.json` to include `bundle.createUpdaterArtifacts`.
- Add CI steps in both release workflows to:
  - assert `TAURI_SIGNING_PRIVATE_KEY` and password are present,
  - inject the real updater public key or fail,
  - collect generated updater artifacts/signatures,
  - publish channel JSON to the GitHub Pages branch or another HTTPS feed.
- Add a script such as `scripts/release/build-updater-feed.mjs` that maps Tauri updater artifacts into Mochi's `/updates/{target}/{arch}/{current_version}/{channel}.json` layout.
- The script must backfill endpoint directories for every supported installed version and channel, with at least `0.1.7` and `0.2.0` in the test fixture.
- Add workflow validation that curls representative old-version endpoints for macOS arm64/x64, Linux x64, and Windows x64 and checks JSON fields.

## Verification Commands

- `pnpm lint`
- `pnpm format:check`
- `pnpm test`
- `pnpm build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
- Linux manual QA:
  - install `.deb` or AppImage,
  - open tray native menu,
  - verify no `Show usage`,
  - verify checked channel state,
  - open widget/settings/update notes,
  - verify wheel and trackpad scrolling in settings, widget, update page, and "What's new" dialog,
  - verify native controls on settings and widget.
- Release QA:
  - after publishing, curl representative macOS, Linux, and Windows update endpoints for `0.1.7` and `0.2.0`,
  - run installed 0.1.7 macOS and Linux apps and confirm update available.
  - when Windows testing is available, run installed Windows 0.1.7/0.2.0 and confirm update available.

## Open Decisions

- Final label for the first tray action: recommended `Open widget` if the widget is intended as the primary Linux fallback; otherwise `Open usage panel`.
- Whether updater feed should live in GitHub Pages as currently coded or be moved to GitHub release static JSON assets. GitHub Pages matches existing app code but requires branch publication; release assets are simpler but require changing endpoint construction.
