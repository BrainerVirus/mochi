# Cross-Platform Parity (2026-05-24)

## Goal

Deliver the same user-visible capabilities on **macOS**, **Windows**, and **Linux** (native packages): browser cookie auto-import for the full Chromium/Gecko catalog, readable settings/tray UI, one-command install scripts with runtime dependencies, and documented OS prerequisites.

## In scope

- **Browser import:** Chromium + Gecko paths and decryption on all three OSes; macOS Safari unchanged.
- **Secrets:** macOS Keychain, Windows DPAPI (`Local State` + credential store), Linux Secret Service (libsecret).
- **Linux shell:** Opaque GTK windows for tray/settings; CSS glass on solid backgrounds.
- **Install scripts:** `install-linux.sh` / `install-windows.ps1` install tray/libsecret/WebView2 deps idempotently (`MOCHI_SKIP_DEPS=1` to opt out).
- **Linux packages:** AppImage, `.deb`, `.rpm` only — Tauri updater on all native Linux artifacts.
- **UX:** Linux tray hint in Settings; [docs/linux.md](../../linux.md) troubleshooting.

## Out of scope

- **Flatpak** distribution and Flatpak-managed updates (sandbox blocks reliable cookie import; dropped for v1).
- **Safari** import on Windows/Linux.
- **Chrome 127+ app-bound encryption (v20)** on Windows.
- Perfect tray on every Linux DE without user indicator support.

## Non-goals / unchanged

- Privacy model: opt-in browser cookie access; local-only storage.
- Provider API surface unchanged; behavior fixes via `import_cookies`.

## Verification

| Check                           | macOS         | Windows     | Linux                |
| ------------------------------- | ------------- | ----------- | -------------------- |
| Settings readable               | yes           | yes         | yes                  |
| Tray panel styled               | yes           | yes         | yes (with indicator) |
| Cursor / OpenCode cookie import | Chrome/Safari | Chrome/Edge | Chrome/Firefox       |
| `pnpm test` + `cargo test`      | green         | green       | green                |

## Flatpak decision

Removed `FLATPAK_ID` updater branch, CI `flatpak-builder` deps, and Flatpak docs. Users on Flathub should uninstall and use `install-linux.sh`.
