# Agent Notes

This repo is pre-scaffold. Do not infer the implementation stack from missing manifests.

- Product spec: [docs/superpowers/specs/2026-05-19-mochi-design.md](docs/superpowers/specs/2026-05-19-mochi-design.md).
- Current implementation stack source of truth: [docs/tech-stack.md](docs/tech-stack.md).
- The old generated plan at [docs/superpowers/plans/2026-05-19-mochi.md](docs/superpowers/plans/2026-05-19-mochi.md) predates the Vite/Oxc direction; use it only for product/module intent, not package versions or scaffold commands.

## Cross-Platform Requirement

**Mandatory:** All code must work correctly on **macOS, Windows, and Linux**. Do not ship macOS-only assumptions — paths, shell commands, file locations, tray/window behavior, or platform APIs — without equivalent cross-platform handling or explicit guards (`#[cfg(...)]`, runtime detection). Tray panel, widget, settings, CLI, and build tooling must remain functional on all three desktop targets. When a platform needs a special case (e.g. macOS private API for transparent tray windows), document it and keep Win/Linux behavior correct without that API.

## Stack To Preserve

- Frontend: React 19 + Vite 8 static SPA for Tauri; do not add TanStack Start, Nitro, or Vinxi.
- Router/data layer: TanStack Router file routes plus TanStack Query 5; use Zod validators for Tauri IPC responses and untrusted JSON.
- Server/cache state: TanStack Query 5 for provider/status/update data, refetching, stale states, and retries.
- Local UI state: Zustand 5 for small client-only stores; do not put server snapshots in Zustand.
- Desktop shell: Tauri v2; keep Rust in `src-tauri/` and frontend app code in `app/`.
- Build tool: Vite 8 with the React Compiler wired through `@vitejs/plugin-react` and `@rolldown/plugin-babel`; prefer ESM config.
- Styling: Tailwind CSS 4 + shadcn/ui. Generate/maintain `DESIGN.md` with `.agents/skills/design-md` before building the shadcn design system.
- Tailwind tooling: use official `@tailwindcss/vite`, `oxlint-tailwindcss` for Tailwind linting, and oxfmt `sortTailwindcss` for class ordering.
- Oxc quality gate: oxlint must cover TypeScript, imports, React, accessibility, JavaScript correctness, and Tailwind; oxfmt must format JS/TS/HTML/CSS and sort imports/classes.
- Animation: GSAP + `@gsap/react` for non-trivial React animation; use scoped `useGSAP()` and reduced-motion-aware patterns.
- Validation: Zod 4 at untrusted boundaries.
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

## Shared Agent Rules

These rules are mirrored for Cursor and OpenCode. Treat the Markdown files under `docs/agent-rules/` as the source of truth.

- Commit messages: follow [docs/agent-rules/commit-messages.md](docs/agent-rules/commit-messages.md) for every agent-generated git commit.
- Design system: before UI, frontend, CSS, Tailwind, component, or accessibility work, follow [docs/agent-rules/design-system.md](docs/agent-rules/design-system.md) and read `DESIGN.md`.

## GitHub Flow

This repository uses GitHub Flow. `main` must always be deployable.

### Branch Rules

1. **Always branch from `main`.** Never commit or push directly to `main`.
2. **Branch names** must describe the work: `feature/*`, `fix/*`, `chore/*`, or `docs/*`.
3. **No agent/tool prefixes** such as `codex/`, `cursor/`, or `opencode/` in branch names.
4. **One concern per branch.** Keep PRs focused and reviewable.

### Pull Request Rules

1. **Squash merge every PR.** The PR title becomes the commit message on `main` — use conventional commit format (`feat(ui): add dark mode toggle`).
2. **Delete the branch immediately after merge.** Both local and remote. Never leave stale branches.
3. **All CI must pass before merge.** Treat required GitHub checks as merge blockers. Fix failures on the branch before merging.
4. Follow `.github/PULL_REQUEST_TEMPLATE.md` when preparing PR descriptions.

### Required Validation (run locally before opening a PR)

```
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

### Agent Merge Workflow

When completing work for the user:

1. Create the branch from `main`.
2. Make changes and commit with conventional messages.
3. Push the branch and open a PR.
4. Run validation commands. Fix any failures.
5. Squash merge the PR.
6. Delete the local and remote branch.
7. If a release is needed, proceed to the Release Process section below.

## Release Process

Two GitHub Actions workflows handle releases. Both are triggered automatically but can also be run manually via `workflow_dispatch`.

### Unstable Release

- **Triggers:** push to `main`, manual dispatch.
- **Tag:** auto-generated as `unstable-YYYYMMDD.HHMMSS`.
- **Channel:** `unstable` (prerelease).
- **Updater feeds:** deploys `unstable.json` for recovery versions (0.1.7, 0.2.0, latest) to GitHub Pages.
- **Notes location:** `releaseBody` in `.github/workflows/release-unstable.yml` and the `release-notes` job at the bottom of the same file. Both must match.

### Stable Release

- **Triggers:** push of a `vMAJOR.MINOR.PATCH` tag, manual dispatch (requires the tag to exist).
- **Tag:** must be created manually after all changes are on `main`.
- **Channel:** `stable`.
- **Updater feeds:** deploys both `stable.json` and `unstable.json` for recovery versions (0.1.7, 0.2.0, latest) to GitHub Pages. This is the only workflow that deploys to Pages, ensuring feeds are always consistent.
- **Notes location:** `releaseBody` in `.github/workflows/release-stable.yml` and the `release-notes` job at the bottom of the same file. Both must match.

### How to Cut a Stable Release

1. Ensure all changes are merged to `main` via PRs (never push directly).
2. Bump `version` in `src-tauri/tauri.conf.json` to match the new tag (without `v` prefix).
3. Commit the version bump on a `chore/release-*` branch, PR it into `main`.
4. After the PR merges, pull `main`, then create and push the tag:
   ```bash
   git tag vMAJOR.MINOR.PATCH
   git push origin vMAJOR.MINOR.PATCH
   ```
   This triggers the stable release workflow automatically.
5. **Never** trigger the stable workflow via `workflow_dispatch` on `main` — it requires a `v*` tag ref to pass validation.

### Updating Release Notes

Release notes must be **user-facing highlights**, not internal changelog entries. Update them in **both** places inside each workflow file:

1. The `releaseBody` field in the `tauri-action` step (used for the GitHub Release body).
2. The `body` array in the `release-notes` job (used to overwrite the release body after publish).

Keep stable and unstable notes in sync for the same set of changes. Focus on what the user experiences: new features, behavior changes, install instructions. Do not mention CI fixes, refactors, or internal tooling unless they affect the user.

## Structure And Maintainability

- Frontend app entry and routes live under `app/`; shared UI/domain code goes under `src/` as defined in [docs/tech-stack.md](docs/tech-stack.md).
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
