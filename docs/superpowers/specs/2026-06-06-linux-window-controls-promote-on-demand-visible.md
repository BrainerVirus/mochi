# Linux Window Controls On-Demand Promotion Spec

**Status:** Ready for implementation.

**Problem:** Ubuntu Wayland native window controls for decorated Mochi windows failed when those windows were hidden-precreated. The controls rendered, but hover/click hit regions did not work until the user manually maximized/restored the window. This affected settings/about/update and widget windows.

**Proven result:** The installed unstable build `unstable-20260606.233705`, compiled with `MOCHI_LINUX_WINDOW_EXPERIMENT=on-demand-visible`, fixed native controls for both settings and widget. Diagnostics confirmed the binary was truly compiled as `on-demand-visible` after the Cargo build-cache fix.

---

## Evidence

### Broken Baseline

Baseline builds logged hidden precreation and broken native controls:

```text
label=settings phase=created experiment=baseline-sequenced-logs creation=startup-precreate initial_visibility=hidden outer=26x23 inner=520x560
label=widget phase=created experiment=baseline-sequenced-logs creation=startup-precreate initial_visibility=hidden outer=26x23 inner=360x420
```

Manual result: native minimize/maximize/close controls did not hover or click until the maximize/restore workaround.

### False Positive Release Notes

Some unstable releases said `Linux window experiment: on-demand-visible` in GitHub release notes but still logged:

```text
experiment=baseline-sequenced-logs
```

Root cause: `src-tauri/build.rs` did not declare `MOCHI_LINUX_WINDOW_EXPERIMENT` as a Cargo build input, so CI reused cached Rust artifacts.

### Fixed Experiment Build

After adding Cargo env invalidation, `unstable-20260606.233705` logged:

```text
label=widget phase=created experiment=on-demand-visible creation=on-demand initial_visibility=visible
```

Manual result: settings and widget opened, and native controls worked.

---

## Required Product Behavior

On Linux only:

- Decorated app windows must not be hidden-precreated at startup.
- Settings/about/update window must be created only when opened by the user.
- Widget window must be created only when opened by the user.
- The widget must not be declared in `src-tauri/tauri.conf.json` as a hidden static window.
- Linux decorated windows must be created visible.
- First-open sequence for Linux decorated windows must be `set_focus` only after visible creation.
- Linux must not run the old `show -> unminimize -> set_focus` sequence for these decorated windows.
- Linux must not run first-open `set_size()` / `set_min_size()` mutations before the first visible map.

On macOS and Windows:

- Preserve current startup precreation behavior unless existing tests prove otherwise.
- Preserve current hidden initial visibility.
- Preserve the existing `show -> unminimize -> set_focus` sequence.

All platforms:

- Keep native decorations and resizability for settings/about/update/widget.
- Keep the tray panel undecorated and unaffected.
- Keep close behavior native; do not convert close into hide.
- Keep passive Linux window-control diagnostics. Do not reintroduce runtime `set_decorations(true)` or `set_resizable(true)` calls for decorated windows.

---

## Required Cleanup

Remove experiment scaffolding now that `on-demand-visible` is proven:

- Remove `MOCHI_LINUX_WINDOW_EXPERIMENT` workflow input/env plumbing.
- Remove release-note text that labels unstable builds with an experiment.
- Remove `LinuxWindowExperiment` and unused variants:
  - `baseline-sequenced-logs`
  - `on-demand-visible`
  - `on-demand-hidden`
  - `builder-size-only`
  - `show-focus-only`
  - `unminimize-show-focus`
- Remove Cargo env wiring for `MOCHI_LINUX_WINDOW_EXPERIMENT` after no code depends on it.
- Keep a small permanent `window_policy` layer if useful, but it must describe platform policy, not temporary experiments.

---

## Diagnostics Requirements

Permanent diagnostics should log platform policy, not experiment names.

Preferred lifecycle detail examples:

```text
label=settings phase=created policy=linux-on-demand-visible creation=on-demand initial_visibility=visible outer=0x0 inner=520x513
label=widget phase=created policy=linux-on-demand-visible creation=on-demand initial_visibility=visible outer=0x0 inner=360x373
```

For non-Linux:

```text
policy=startup-hidden creation=startup-precreate initial_visibility=hidden
```

The exact outer size may be `0x0` immediately after visible builder creation on Wayland. That is acceptable if manual native controls work.

---

## Acceptance Criteria

Automated:

- `pnpm test` passes.
- `pnpm build` passes.
- `pnpm format:check` passes.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passes.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` passes.
- `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` passes.
- Source tests prove no `MOCHI_LINUX_WINDOW_EXPERIMENT` usage remains.
- Source tests prove `tauri.conf.json` does not predeclare the widget window.
- Rust tests prove Linux policy is on-demand/visible/focus-only and macOS/Windows policy stays startup-hidden/show-unminimize-focus.

Manual Ubuntu Wayland:

- Install a new unstable or stable build containing the promoted behavior.
- Open settings from tray.
- Native minimize/maximize/close hover and click immediately.
- Close and reopen settings.
- Native controls still work.
- Open widget from tray.
- Native minimize/maximize/close hover and click immediately.
- Close and reopen widget.
- Native controls still work.
- Run `mochi diagnostics --bundle`.
- Confirm logs show `policy=linux-on-demand-visible` for settings and widget.

---

## Non-Goals

- Do not continue the unstable experiment matrix.
- Do not change tray panel behavior.
- Do not add custom client-side window controls.
- Do not hide native titlebar controls.
- Do not make Linux behavior depend on a runtime environment variable.
- Do not alter provider fetching, usage storage, or update-channel selection.
