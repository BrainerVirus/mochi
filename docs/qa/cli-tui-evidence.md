# CLI/TUI evidence — 2026-09-04

Task 10 of the cli-tui workstream. Every command below was run against the
actual debug binary (`src-tauri/target/debug/mochi`, freshly built this
session via `cargo build --manifest-path src-tauri/Cargo.toml`) on this
machine. Scripted (piped, non-TTY) output was captured verbatim; anything
requiring eyes on a live terminal is marked **PENDING-OWNER** with repro
steps. Nothing is invented.

## Environment

| Field      | Value                                                                                                                      |
| ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| Date (UTC) | 2026-09-04T16:49Z                                                                                                          |
| OS         | Ubuntu 24.04 (noble), kernel 6.8.0-136-generic, x86_64                                                                     |
| Desktop    | GNOME (ubuntu:GNOME), Wayland session (`XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-0`, XWayland via `DISPLAY=:0`) |
| App build  | `mochi 0.2.6`, debug profile                                                                                               |
| Shell      | Non-interactive agent shell (stdout is a pipe, not a TTY — so every run below exercises the non-TTY fallback path)         |

Build note: Node 24 + system GTK/WebKit dev packages are installed on this
machine, so a plain `cargo build --manifest-path src-tauri/Cargo.toml`
succeeds (finished in-cache this session, no sysroot workaround needed).
Reconciliation (2026-09-04): this supersedes the `linux-ubuntu-evidence.md`
(2026-09-03) build note, which said the GTK/WebKit dev packages were NOT
installed and cargo needed a deb-extracted sysroot. The packages were
installed system-wide on 2026-09-04 via apt — verified this session:
`dpkg -l` shows `libwebkit2gtk-4.1-dev` + `libjavascriptcoregtk-4.1-dev`
installed and `pkg-config --modversion webkit2gtk-4.1` reports `2.52.6`.
The 2026-09-03 doc was accurate for its date; this doc is current.

## Exit-code matrix (real binary)

| Command                               | Exit | Output (verbatim)                                                                                                                                                                                                    |
| ------------------------------------- | ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mochi status --bad-flag`             | 2    | `error: unexpected argument '--bad-flag' found` + usage text                                                                                                                                                         |
| `mochi config <unknown-key>`          | 1    | `mochi failed: unknown key: <key>` (ran as `mochi config unknown-key`)                                                                                                                                               |
| `mochi update frobnicate`             | 2    | `usage: mochi update <check\|apply> [--confirm] (unknown action: frobnicate)`                                                                                                                                        |
| `mochi update` (missing action)       | 2    | clap required-argument error + usage text                                                                                                                                                                            |
| `mochi status`                        | 0    | `Codex 0% / Cursor 87% / OpenCode Go 8% / Command Code 50%` (one line each)                                                                                                                                          |
| `mochi config` (bare)                 | 0    | `update_channel = stable`, `enabled_providers = …`, `cursor.cookie_source = auto`                                                                                                                                    |
| `mochi update apply` (no `--confirm`) | 2    | `refusing to apply without --confirm` + `usage: mochi update <check\|apply> [--confirm] apply --confirm` (ran piped, `< /dev/null`)                                                                                  |
| `mochi config update_channel bogus`   | 1    | `mochi failed: unknown update channel: bogus` (ran piped, `< /dev/null`; note: `config` takes `[KEY] [VALUE]` with no `set` verb, so the invalid-value path is `mochi config <key> <bad-value>`, not `config set …`) |
| `mochi cost`                          | 0    | `Command Code $64.37 / $71.95 (Billing period)`                                                                                                                                                                      |

Contract note: the failure-domain split is **usage error → 2** (clap parse
failures, unknown `update` action) vs **domain failure → 1** (resolved CLI
shape but unknown config key). `mochi update frobnicate` lands on the usage
side (exit 2), not exit 1 — the task brief/worker instructions' "expect 1"
example holds for the `mochi config <unknown-key>` path instead (that
expectation text lived in the brief, not in plan.md Task 10, which contains
no such example). `mochi config list` also exits 1
(`mochi failed: unknown key: list`): `config` takes `[KEY] [VALUE]`, there is
no `list`/`get` verb.

## TTY-fallback proof (piped runs, no TUI escape codes)

All three runs below were piped (non-TTY), and the raw bytes were counted with
Python (`data.count(b'\x1b')`):

- `mochi status | cat` → plain lines, `ESC count: 0`:
  `Codex 0%\nCursor 87%\nOpenCode Go 8%\nCommand Code 50%\n`
- `mochi usage --json | head -c 200` → plain JSON, `ESC count: 0`:
  `[{"provider":"codex","kind":"fresh","snapshot":{"provider":"codex",…`
- `mochi cost | …` → plain line, `ESC count: 0`.

PASS. Piped output carries zero `0x1b` bytes — no TUI escape codes leak into
the fallback path. (Caveat: `od -c | grep '\033'` false-positives on this
output because od _offsets_ like `0003300` contain the substring `033`; the
Python byte count is the authoritative check.)

## Masked-secret proof

PASS (unit-test level, run this session). No live secret exists on this
machine to screenshot, so the proof is the masking tests, all green:

```
cargo test --manifest-path src-tauri/Cargo.toml --lib masks
→ review_masks_secret … ok
→ rendered_detail_masks_secret … ok
→ multibyte_secret_masks_per_char … ok
→ config_get_masks_secret_values … ok (4 passed, 0 failed)
```

What they cover: `mochi config <provider>.api_key` prints `<set>`/`<unset>`,
never the value (`src-tauri/src/cli/config.rs`; `Config` takes `[KEY] [VALUE]`
per `src-tauri/src/cli/mod.rs`, no `get` verb — verified live this session:
`mochi config cursor.api_key` → `<unset>`, exit 0); the wizard's
`masked_secret()` renders `•` per char (multibyte-safe) and the review/detail
lines use it (`src-tauri/src/cli/tui/config_wizard.rs`); diagnostics output
redacts sensitive markers (`src-tauri/src/diagnostics/report.rs`). The secret
paste buffer is never logged by design.

## Per-screen manual checklist

Scripted = run non-interactively in this session and output captured.
Eyeballed = a human watched the interactive TUI render. This session has no
PTY, so every interactive render below is PENDING-OWNER.

### GNOME terminal (this machine)

| Screen                                                              | Status                    | Evidence / repro                                                                                                                                                |
| ------------------------------------------------------------------- | ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Dashboard (`mochi status`, piped)                                   | PASS (scripted, fallback) | Plain provider lines, exit 0, zero ESC bytes (above)                                                                                                            |
| Cost view (`mochi cost`, piped)                                     | PASS (scripted, fallback) | `Command Code $64.37 / $71.95 (Billing period)`, exit 0, zero ESC bytes                                                                                         |
| Config read (`mochi config`, piped)                                 | PASS (scripted)           | Key/value lines, exit 0; unknown key → exit 1 with `mochi failed:`                                                                                              |
| Diagnostics (`mochi diagnostics`, piped)                            | PASS (scripted)           | Version/platform/env/config-hint/log-tail output, exit 0                                                                                                        |
| Config wizard (interactive TUI)                                     | PENDING-OWNER             | Repro: run `mochi config` (or the wizard entry) in GNOME Terminal on a TTY; confirm the secret field masks input, paste works, cancel leaves settings untouched |
| Update flow (`mochi update check` / `apply --confirm`, interactive) | PENDING-OWNER             | Repro: `mochi update check` on a TTY; confirm progress renders and `apply` requires `--confirm`. Not run here (live network fetch, no TTY)                      |
| Dashboard (interactive TUI)                                         | PENDING-OWNER             | Repro: `mochi status` on a TTY; confirm the ratatui dashboard renders and quits cleanly                                                                         |
| Cost view (interactive TUI)                                         | PENDING-OWNER             | Repro: `mochi cost` on a TTY; confirm the cost screen renders                                                                                                   |

### macOS Terminal

PENDING-OWNER by definition (no macOS hardware in this session). Repro:
same four interactive repros as GNOME above, plus confirm no
macOS-private-API assumptions in the TUI path (there are none — the TUI is
ratatui/crossterm only).

### Windows Terminal (Windows console path)

PENDING-OWNER by definition (no Windows host in this session). Repro: run
the four interactive screens under Windows Terminal (conhost + Windows
Terminal); confirm crossterm console rendering, no ANSI/raw-mode failures,
and graceful fallback when piped (`| cat` shows plain output, zero ESC
bytes — the same assertion scripted above on Linux).

## docs/releasing.md

Skipped per the brief: the file has no CLI section to extend (it covers the
release pipeline, updater feeds, release notes, macOS distribution, and
Linux window controls). No paragraph added.

## Validation

- `cargo build --manifest-path src-tauri/Cargo.toml` — clean (debug).
- `cargo test --manifest-path src-tauri/Cargo.toml --lib masks` — 4 passed.
- `pnpm format:check` — green, verbatim (2026-09-04):

```
> mochi@0.2.6 format:check /home/cristhofer-pincetti/Documents/projects/personal/mochi
> oxfmt --check

Checking formatting...

All matched files use the correct format.
Finished in 3251ms on 360 files using 14 threads.
```

- `pnpm lint` — green, verbatim (2026-09-04):

```
> mochi@0.2.6 lint /home/cristhofer-pincetti/Documents/projects/personal/mochi
> oxlint --type-aware --react-plugin --jsx-a11y-plugin --import-plugin --deny-warnings app src
```

(exit 0, no warnings — oxlint emits no output on success).
