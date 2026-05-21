# Agent Notes

This repo is pre-scaffold. Do not infer the implementation stack from missing manifests.

- Product spec: [docs/superpowers/specs/2026-05-19-mochi-design.md](docs/superpowers/specs/2026-05-19-mochi-design.md).
- Current implementation stack source of truth: [docs/tech-stack.md](docs/tech-stack.md).
- The old generated plan at [docs/superpowers/plans/2026-05-19-mochi.md](docs/superpowers/plans/2026-05-19-mochi.md) predates the TanStack Start/Oxc direction; use it only for product/module intent, not package versions or scaffold commands.

## Cross-Platform Requirement

**Mandatory:** All code must work correctly on **macOS, Windows, and Linux**. Do not ship macOS-only assumptions — paths, shell commands, file locations, tray/window behavior, or platform APIs — without equivalent cross-platform handling or explicit guards (`#[cfg(...)]`, runtime detection). Tray panel, widget, settings, CLI, and build tooling must remain functional on all three desktop targets. When a platform needs a special case (e.g. macOS private API for transparent tray windows), document it and keep Win/Linux behavior correct without that API.

## Stack To Preserve

- Frontend: React 19 + TanStack Start 1.x, not a plain Vite SPA.
- Router/data layer: TanStack Router from the TanStack Start setup; use Zod validators for server functions and untrusted JSON.
- Server/cache state: TanStack Query 5 for provider/status/update data, refetching, stale states, and retries.
- Local UI state: Zustand 5 for small client-only stores; do not put server snapshots in Zustand.
- Desktop shell: Tauri v2; keep Rust in `src-tauri/` and frontend app code in `app/`.
- Build tool: Vite 8 through TanStack Start/Vinxi; prefer ESM config.
- Styling: Tailwind CSS 4 + shadcn/ui. Generate/maintain `DESIGN.md` with `.agents/skills/design-md` before building the shadcn design system.
- Tailwind tooling: use official `@tailwindcss/vite`, `oxlint-tailwindcss` for Tailwind linting, and oxfmt `sortTailwindcss` for class ordering.
- Oxc quality gate: oxlint must cover TypeScript, imports, React, accessibility, JavaScript correctness, and Tailwind; oxfmt must format JS/TS/HTML/CSS and sort imports/classes.
- Animation: GSAP + `@gsap/react` for non-trivial React animation; use scoped `useGSAP()` and reduced-motion-aware patterns.
- Validation: Zod 4 at untrusted boundaries and for TanStack Start server function inputs.
- JS/TS lint/format: oxlint + oxfmt, not ESLint/Prettier/Biome.
- Rust: stable Rust, Tauri v2 command patterns, and explicit capabilities.

## Workflow

- Before frontend/design work, read `.agents/skills/design-md/SKILL.md`, `.agents/skills/frontend-design/SKILL.md`, `.agents/skills/shadcn/SKILL.md`, and `DESIGN.md` if it exists.
- Before animation work, read `.agents/skills/gsap-react/SKILL.md` plus the relevant GSAP skill (`gsap-core`, `gsap-timeline`, `gsap-scrolltrigger`, `gsap-plugins`, or `gsap-performance`).
- Before schema/validation work, read `.agents/skills/zod/SKILL.md`.
- Before async server/cache state work, read `.agents/skills/tanstack-query/SKILL.md`.
- Before Tauri work, read `.agents/skills/tauri-v2/SKILL.md`; every Tauri command must be registered in `generate_handler!` and covered by `src-tauri/capabilities/default.json` permissions when needed.
- Before Rust implementation/review, read `.agents/skills/rust-best-practices/SKILL.md`.
- Before behavior changes, follow `.agents/skills/test-driven-development/SKILL.md`; write the failing test first unless the user explicitly approves a config/prototype exception.
- Before claiming completion, run the relevant verification from [docs/tech-stack.md](docs/tech-stack.md) and report what actually ran.

## Structure And Maintainability

- Frontend app code follows TanStack Start structure under `app/`; shared UI/domain code goes under `src/` as defined in [docs/tech-stack.md](docs/tech-stack.md).
- Keep TanStack Query keys/options under `src/lib/query/` and Zustand stores under `src/lib/stores/`.
- Tauri/Rust code follows the `src-tauri/src/{core,providers,auth,fetch,settings,tray,widget,status_bar,...}` structure from [docs/tech-stack.md](docs/tech-stack.md).
- Keep TS/TSX files under 250 lines when practical; split before 350 lines unless generated or shadcn-owned.
- Keep Rust modules under 300 lines when practical; split before 450 lines unless mostly data definitions.
- Use kebab-case file names for TS modules/components; use Rust snake_case modules.

## Expected Commands After Scaffold

- Install: `pnpm install`
- Dev: `pnpm dev`
- Build/typecheck: `pnpm build`
- Lint: `pnpm lint`
- Format check: `pnpm format:check`
- Desktop dev: `pnpm tauri dev`
- Rust check: `cargo check --manifest-path src-tauri/Cargo.toml`
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
