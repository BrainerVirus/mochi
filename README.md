# Mochi

[![Pull Request](https://github.com/BrainerVirus/mochi/actions/workflows/pr.yml/badge.svg)](https://github.com/BrainerVirus/mochi/actions/workflows/pr.yml)
[![React Doctor](https://img.shields.io/badge/React%20Doctor-enabled-61dafb?logo=react&logoColor=111827)](https://react.doctor/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24c8db?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=111827)](https://react.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**Soft alerts before hard limits.**

Mochi is a cross-platform desktop companion for AI coding tools. It tracks session and weekly usage, reset windows, provider status, and local usage signals for tools like Codex, Claude, Cursor, Gemini, Copilot, and more—without sending your data to a cloud backend.

Mochi exists because of [steipete/codexbar](https://github.com/steipete/codexbar). CodexBar proved how useful a focused menu-bar usage tracker for Codex could be; Mochi takes that spark seriously and expands the idea into a cross-platform companion for macOS, Windows, and Linux, with tray, widget, CLI, and status-bar surfaces that feel native on each platform.

## Install

Install from [GitHub Releases](https://github.com/BrainerVirus/mochi/releases). Scripts default to the latest **stable** release. Pass **`-i`** (or **`--unstable`**) for the unstable channel (latest prerelease from `main`).

### macOS

**Direct** (downloads the release DMG and installs to `/Applications`):

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos.sh | bash
```

Unstable:

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos.sh | bash -s -- -i
```

**Homebrew** (temporary cask from the selected release; requires [Homebrew](https://brew.sh/)):

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos-brew.sh | bash
```

Unstable:

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-macos-brew.sh | bash -s -- -i
```

_Advanced:_ the direct script accepts `MOCHI_INSTALL_DIR` (default `/Applications`) if you need a non-system location.\_

### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash
```

Unstable:

```bash
curl -fsSL https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-linux.sh | bash -s -- -i
```

Package selection:

| Env             | Values                           | Default                                                          |
| --------------- | -------------------------------- | ---------------------------------------------------------------- |
| `MOCHI_PACKAGE` | `appimage`, `deb`, `rpm`, `auto` | `auto` (deb on Debian/Ubuntu, rpm on Fedora/RHEL, else AppImage) |

AppImage installs to `~/.local/bin/mochi`.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-windows.ps1 | iex
```

Unstable:

```powershell
$env:MOCHI_UNSTABLE = "1"; irm https://raw.githubusercontent.com/BrainerVirus/mochi/main/scripts/install/install-windows.ps1 | iex
```

Or download and run:

```powershell
.\scripts\install\install-windows.ps1
.\scripts\install\install-windows.ps1 -Unstable
```

Prefer MSI (default); set `$env:MOCHI_PACKAGE = "exe"` for the NSIS installer.

### Pin a release

```bash
MOCHI_VERSION=v1.0.0 ./scripts/install/install-macos.sh
MOCHI_VERSION=unstable ./scripts/install/install-macos.sh -i
```

```powershell
$env:MOCHI_VERSION = "v1.0.0"
.\scripts\install\install-windows.ps1
```

Set `MOCHI_UNSTABLE=1` instead of `-i` / `-Unstable` if you prefer environment variables.

### Requirements

- **macOS / Linux scripts:** `curl`, `jq`
- **macOS:** `hdiutil`, `ditto`
- **Linux `.deb` / `.rpm`:** `sudo` and the matching package manager
- **Windows:** PowerShell 5.1+
- **Optional:** `GITHUB_TOKEN` for higher GitHub API rate limits

See [docs/releasing.md](docs/releasing.md) for stable vs unstable release channels.

## Features

- **Tray app** — Dynamic icon with usage bars; click for a compact panel, secondary action for refresh, settings, updates, and quit.
- **Desktop widget** — Floating overview for desktops where tray support is unreliable (especially Linux).
- **Settings** — Enable providers, configure refresh intervals, update channel, and privacy-related options.
- **Providers** — Top v1 providers (Codex, Claude, Cursor, Gemini, Copilot, Antigravity, Factory/Droid, z.ai, Kiro, Augment) with clear stale/error/incident states.
- **CLI & status bar** — `mochi usage`, `mochi status`, Waybar JSON, and automation-friendly output.
- **Privacy-first** — Local-only storage, no server-side aggregation, opt-in browser cookie access.

## Tech stack

| Layer     | Choices                                                                               |
| --------- | ------------------------------------------------------------------------------------- |
| UI        | React 19, TanStack Start & Router, TanStack Query, Zustand, shadcn/ui, Tailwind CSS 4 |
| Desktop   | Tauri v2 (Rust) — tray, widget, providers, secure storage, updater                    |
| Quality   | oxlint, oxfmt, Zod 4, Vitest                                                          |
| Animation | GSAP + `@gsap/react`                                                                  |

See [docs/tech-stack.md](docs/tech-stack.md) for versions, folder layout, and conventions.

## Prerequisites

- **Node.js** 24 LTS or newer (`.nvmrc` pins **25** for local/CI parity; npm 11 ships with Node 25)
- **pnpm** 9.15.x — enable via [Corepack](https://nodejs.org/api/corepack.html) (`corepack enable`); version pinned in `package.json` `packageManager`
- **Rust** stable toolchain and [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

The stack (TanStack Start 1.x, Vite 8, React 19, Tauri v2) targets Node 20.12+, but this repo enforces **Node ≥ 24** so CI and contributors stay on current LTS or newer.

## Development

```bash
pnpm install
pnpm dev          # frontend dev server (port 1420)
pnpm tauri dev    # desktop app with Tauri
```

On macOS, `pnpm tauri dev` runs an unbundled debug binary. The Dock icon is restored from bundled PNG assets when settings/about/update opens. **Installed builds** (`.app`, `.msi`, `.deb`, etc.) use the OS icon pipeline — macOS squircle masking from `icon.icns`, Windows/Linux taskbar icons from `icon.ico` / PNG. Regenerate icons with `./scripts/generate-icons.sh`.

```bash
pnpm lint
pnpm format:check
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

Agent and stack details: [AGENTS.md](AGENTS.md).

## Contributing

This project uses **GitHub Flow**: short-lived branches from `main`, pull requests, no direct merges by automation. See [docs/contributing.md](docs/contributing.md).

## Kudos

Deep thanks to [Peter Steinberger](https://github.com/steipete) and [CodexBar](https://github.com/steipete/codexbar), the project that inspired Mochi’s original direction. Mochi is not a fork, but CodexBar is the origin point for this app’s menu-bar-first thinking and deserves clear credit.

## License

[MIT](LICENSE) © [BrainerVirus](https://github.com/BrainerVirus)
