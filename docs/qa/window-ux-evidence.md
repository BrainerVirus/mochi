# Window UX evidence

Branch: `feature/window-ux-polish` — base `9935661`.
Date (UTC): 2026-09-05.

## Automated gate (exact outputs)

All commands run from the repo root. Cargo commands used
`RUSTUP_TOOLCHAIN=stable` + `PATH="$HOME/.cargo/bin:$PATH"`.

### 1. `pnpm lint` — PASS (exit 0)

```text
> mochi@0.4.4 lint /home/cristhofer-pincetti/Documents/projects/personal/mochi
> oxlint --type-aware --react-plugin --jsx-a11y-plugin --import-plugin --deny-warnings app src
```

### 2. `pnpm format:check` — KNOWN EXCEPTION (exit 1, sole file below)

```text
> mochi@0.4.4 format:check /home/cristhofer-pincetti/Documents/projects/personal/mochi
> oxfmt --check

Checking formatting...

docs/window-ux-polish/plan.md (1628ms)

Format issues found in above 1 files. Run without `--check` to fix.
Finished in 3224ms on 370 files using 14 threads.
ELIFECYCLE  Command failed with exit code 1.
```

Known exception (per task brief, do not fix): untracked
`docs/window-ux-polish/plan.md` is flagged. Editing it post-approval
would void the approved digest. Every other file (369/370) is clean.
Re-run after creating this file (371 files total): `plan.md` is still
the sole flagged file; this evidence file is clean.

### 3. `pnpm test` — PASS (exit 0)

```text
> mochi@0.4.4 test /home/cristhofer-pincetti/Documents/projects/personal/mochi
> vitest run --config vitest.config.ts

 Test Files  96 passed | 1 skipped (97)
      Tests  462 passed | 6 skipped (468)
   Start at  15:20:48
   Duration  9.04s (transform 17.54s, setup 0ms, import 43.62s, tests 8.65s, environment 9.70s)
```

### 4. `pnpm build` — PASS (exit 0), no 500 kB chunk warning

```text
> mochi@0.4.4 build /home/cristhofer-pincetti/Documents/projects/personal/mochi
> vite build && tsc --noEmit

vite v8.2.2 building client environment for production...
transforming...
✓ 2329 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                                              1.04 kB │ gzip:  0.56 kB
dist/assets/geist-cyrillic-ext-wght-normal-DjL33-gN.woff2    7.42 kB
dist/assets/geist-vietnamese-wght-normal-6IgcOCM7.woff2      8.00 kB
dist/assets/geist-cyrillic-wght-normal-BEAKL7Jp.woff2       15.08 kB
dist/assets/geist-latin-ext-wght-normal-DC-KSUi6.woff2      16.51 kB
dist/assets/geist-latin-wght-normal-BgDaEnEv.woff2          29.40 kB
dist/assets/index-U5qTbDn_.css                             116.42 kB │ gzip: 18.33 kB
dist/assets/app-window-shell-Dt9-5Xv_.js                     0.93 kB │ gzip:  0.49 kB
dist/assets/about-BuDJCvuj.js                                1.07 kB │ gzip:  0.62 kB
dist/assets/format-patch-notes-B8J3dScQ.js                   1.30 kB │ gzip:  0.71 kB
dist/assets/widget-r3gN7x83.js                               1.85 kB │ gzip:  1.02 kB
dist/assets/routes-Dtq5ZhDo.js                               3.23 kB │ gzip:  1.59 kB
dist/assets/update-DQXbMFrQ.js                               6.84 kB │ gzip:  2.82 kB
dist/assets/tray-panel-spacing-jfSTwMvm.js                  27.51 kB │ gzip:  8.81 kB
dist/assets/use-tray-panel-state-BBrs_8sD.js                40.23 kB │ gzip: 14.18 kB
dist/assets/settings-BtJRX0zS.js                            68.27 kB │ gzip: 21.54 kB
dist/assets/tray-segmented-control-DcySg-o6.js              68.91 kB │ gzip: 26.59 kB
dist/assets/progress-C91vhkUP.js                            81.87 kB │ gzip: 31.83 kB
dist/assets/preload-helper-J-MxFT9e.js                     153.57 kB │ gzip: 46.19 kB
dist/assets/index-DJ-FDLpk.js                              281.53 kB │ gzip: 88.67 kB

✓ built in 4.40s
```

Largest JS chunk is `index-DJ-FDLpk.js` at 281.53 kB — under the
500 kB Vite chunk-size warning threshold. No
"Some chunks are larger than 500 kBs" warning was emitted.
`tsc --noEmit` passed with no errors.

### 5. `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS (exit 0)

No output (clean).

### 6. `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — PASS (exit 0)

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s
```

### 7. `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` — PASS (exit 0)

```text
test result: ok. 457 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.38s
```

Per-suite tallies:

- `src/lib.rs` (mochi_lib): 457 passed, 0 failed.
- `src/main.rs` (mochi): 0 passed, 0 failed (no tests).
- `tests/cli_smoke.rs` (cli_smoke): 2 passed
  (`help_lists_diagnostics_subcommand`, `diagnostics_cli_runs`), 0 failed.

## Static checks

- `notification:default` is present in
  `src-tauri/capabilities/default.json` permissions
  (`core:default`, `core:window:allow-start-dragging`,
  `notification:default`, `opener:default`, `updater:default`).
- Window dims from `src-tauri/src/tray/panel.rs` consts (logical px):
  - Settings: 520.0 x 560.0
    (`SETTINGS_WINDOW_WIDTH` x `SETTINGS_WINDOW_HEIGHT`).
  - About: 400.0 x 300.0
    (`ABOUT_WINDOW_WIDTH` x `ABOUT_WINDOW_HEIGHT`).
  - Update: 420.0 x 260.0
    (`UPDATE_WINDOW_WIDTH` x `UPDATE_WINDOW_HEIGHT`).
  - Tray panel: width 360.0 (`TRAY_PANEL_WIDTH`), min height 160.0
    (`TRAY_PANEL_MIN_HEIGHT`), default max height 496.0
    (`TRAY_PANEL_DEFAULT_MAX_HEIGHT`), viewport margin 16.0.

## Owner-run manual checklist (not performed by agent)

- [ ] GNOME cold open shows one skeleton row per configured provider
      (minimum 1) via `ProviderListSkeleton`.
- [ ] Warm reopen renders instantly with no usage fetch in the
      diagnostics log.
- [ ] About window fits without scroll at 400x300.
- [ ] Update window fits without scroll at 420x260.
- [ ] Update-ready notification fires exactly once.
- [ ] Refresh-failure notification fires exactly once.
- [ ] Threshold-cross notification fires exactly once per crossing
      (re-arms only after dropping below).
- [ ] Master toggle off silences all notifications.
- [ ] Denied permission logs only (no crash, no retry storm).
