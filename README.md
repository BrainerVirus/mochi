# Mochi

**Soft alerts before hard limits.**

Mochi is a cross-platform desktop companion for AI coding tools. It tracks session and weekly usage, reset windows, provider status, and local usage signals for tools like Codex, Claude, Cursor, Gemini, Copilot, and more—without sending your data to a cloud backend.

Inspired by menu-bar usage trackers, Mochi is built from scratch for macOS, Windows, and Linux with tray, widget, CLI, and status-bar surfaces that feel native on each platform.

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
pnpm lint
pnpm format:check
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

Agent and stack details: [AGENTS.md](AGENTS.md).

## Contributing

This project uses **GitHub Flow**: short-lived branches from `main`, pull requests, no direct merges by automation. See [docs/contributing.md](docs/contributing.md).

## License

[MIT](LICENSE) © [BrainerVirus](https://github.com/BrainerVirus)
