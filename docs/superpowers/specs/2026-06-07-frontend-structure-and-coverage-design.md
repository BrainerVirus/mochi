# Frontend Structure and Coverage Design

**Status:** Approved (2026-06-07).

**Problem:** The Mochi frontend has 205 TS/TSX files with only ~58 colocated test files (≈28% file-level coverage; line coverage is lower). The project uses a "feature folder + flat files" layout (`src/components/tray/tray-segment-indicator.ts` + `tray-segment-indicator.test.ts` side-by-side), but the colocated-tests convention isn't enforced and there's no coverage tooling — so untested code accumulates silently. The codebase has no CI gate, no coverage threshold, and a folder structure that mixes "feature folder" with "kind folder" (`src/hooks/`, `src/lib/`, `src/components/`) inconsistently, which makes it hard to reason about ownership and to find related code for a single feature.

**Goal:** Refactor the frontend into a per-unit feature-slices layout (`src/features/<name>/` and `src/shared/`), eliminate test coverage gaps, and turn coverage into a CI-blocking gate at ≥ 80% global lines.

---

## 1. Goals, scope, and non-goals

**In scope:**
- Move every frontend file in `src/components/`, `src/hooks/`, `src/lib/`, `src/styles/`, and `src/assets/` to a same-named folder alongside its test file (per-unit folders, no `index.ts` barrels).
- Configure Vite + tsconfig path mapping so the import surface (`@/components/tray/tray-segment-indicator`) is unchanged.
- Add missing unit tests so the frontend reaches ≥ 80% global line coverage.
- Wire `@vitest/coverage-v8` and a threshold into `pnpm test:coverage` and a new CI job (`frontend-coverage`) that blocks PRs under 80%.
- Add minimal Rust test improvements (new `#[cfg(test)]` blocks in modules that lack them) and a `src-tauri/tests/cli_smoke.rs` integration test — **no Rust coverage threshold this round**.

**Explicitly out of scope (this round):**
- Rust coverage gate (deferred to a follow-up spec).
- Refactoring existing app logic; only structure, tests, and CI wiring change.
- New shadcn components, new features, design-token changes.
- App route changes (`app/routes/` stays as TanStack Router file routes).
- Renaming files to shorten the verbose path (e.g. dropping `tray-` in folder names — separate decision).

**Cross-platform constraint (per `AGENTS.md`):** all code must continue to work on macOS, Windows, and Linux. Vitest runs identically on the three. The new Rust integration test is a smoke test that compiles on all three.

---

## 2. Target structure

```
app/                              (unchanged: Vite entry, routes, router)
src/
  features/
    tray/
      components/                 (tray-event-bridge, tray-panel, tray-segment-item, tray-segmented-control, tray-tab-chevron, scroll-fade-overlays, scroll-fade-region, tray-menu-row, tray-overview, tray-panel-content, tray-panel-divider, tray-panel-footer, tray-panel-shell, tray-panel-tab-list)
      hooks/                      (use-scroll-overflow, use-tray-segment-indicator-sync, use-tray-segment-indicators, use-gsap-overflow-visibility, use-tray-panel-focus-reset, use-tray-panel-height, use-tray-panel-refresh, use-tray-panel-shortcuts, use-tray-panel-state, use-tray-usage-sync, use-tab-fill-activation-key)
      lib/                        (scroll-fade-cycle, segment-indicator-animation, segment-track-resize-observer, tray-panel-tab-cycle, tray-segment-indicator, tray-segment-indicator-executor, tray-segment-indicator-machine, tray-segmented-control-config, tray-tab-chevron-class-name)
    widget/
      components/                 (widget-window)
    settings/
      components/                 (settings-form, settings-page, settings-page-content, settings-sections, settings-update-section, provider-config-fields, provider-token-fields, linux-tray-hint)
      lib/                        (settings-form-state, settings-tab-state, provider-field-visibility)
    usage/
      components/                 (usage-card, usage-meter, provider-usage-actions, provider-cost-section, provider-usage-section)
      hooks/                      (use-usage-data, use-usage-meter-fill, use-usage-meter-left-label)
    updates/
      components/                 (update-page, update-page-content, update-prompt, release-notes-dialog, update-check-prefetch)
      hooks/                      (use-update-install, use-post-update-refresh)
    layout/
      components/                 (app-window-shell, root-component)
      lib/                        (app-window-titlebar-policy, root-component-state)
    about/
      components/                 (about-page, about-page-content)
  shared/
    components/
      ui/                         (shadcn primitives: alert, badge, button, card, dialog, field, input, label, progress, separator, skeleton, switch, tabs, toggle, toggle-group, app-segmented-control + its segments/state-machine/utils)
      providers/                  (provider-icon + provider-icon-sources)
      mascot/                     (mochi-chibi, mochi-mark)
    hooks/
      (cross-feature hooks: use-diagnostics-boot, use-initial-window-route, use-cold-start-provider-refresh)
    lib/
      query/                      (client, keys, refresh-provider, settings, update-check, usage-refetch-interval, usage-snapshots, usage-snapshots-live-refresh)
      schemas/                    (settings, usage, provider-catalog)
      stores/                     (ui-store)
      tauri/                      (app-window, commands, diagnostics, initial-window-route, native-window-controls, runtime, tray-panel-window, widget-window, window-events)
      utils/                      (aggregate-used-percent, format-reset-countdown, format-reset-line, format-updated-ago, is-provider-configured, mascot-state, provider-labels, tray-panel-focus, tray-panel-height-animation, tray-panel-height-sync, tray-panel-layout, tray-panel-shortcut, tray-panel-spacing, tray-panel-tabs, tray-tab-fill-activation, tray-tab-fill-scheduler, tray-tab-selection, usage-meter-fill-animation, usage-meter-tone, usage-pace, usage-snapshots-empty-message, widget-density)
      platform/                   (detect, index, types, use-system-color-scheme)
      providers/                  (dashboard-urls — distinct from shared/components/providers)
      updates/                    (current-release-notes, format-patch-notes, release-notes-cache, sanitize-release-notes, settings-update-status, tray-update-footer-items)
    styles/                       (index.css, segment-tokens)
    assets/                       (images, fonts already imported via @fontsource)
```

**`tray-ui-store` lands in** `src/features/tray/lib/stores/tray-ui-store/tray-ui-store.ts` (tray-specific). **`use-update-install` lands in** `src/features/updates/hooks/use-update-install/use-update-install.ts` (updates-specific).

**Classification rules:**
1. A unit is **feature-owned** if ≥ 1 non-test import comes from a file in that feature folder today, OR if its name prefix matches the feature (`tray-*` → features/tray, `usage-*` → features/usage, etc.).
2. A unit is **shared** if it has imports from two or more features, OR if it's a primitive layer (ui/, query/, schemas/, tauri/, stores/, platform/).
3. Pre-existing test files migrate with their unit; new tests are colocated in the same folder.
4. Cross-feature hooks that today live in `src/hooks/` (use-usage-data, use-tray-*, use-update-install, etc.) move into the feature that owns them; only truly cross-feature hooks stay in `src/shared/hooks/`.

**Updated `components.json` aliases (shadcn):**
```json
{
  "aliases": {
    "components": "@/shared/components",
    "utils": "@/shared/lib/utils",
    "ui": "@/shared/components/ui",
    "lib": "@/shared/lib",
    "hooks": "@/shared/hooks"
  }
}
```

---

## 3. Import path resolution (the "magic")

**Contract:** importing a folder name resolves to the same-named file inside that folder. Concretely:

```ts
// Source
import { TrayPanel } from "@/features/tray/components/tray-panel";

// Resolves to
src/features/tray/components/tray-panel/tray-panel.tsx
```

This applies everywhere: `@/features/<feat>/components/<unit>`, `@/features/<feat>/hooks/<unit>`, `@/features/<feat>/lib/<unit>`, `@/shared/components/<area>/<unit>`, `@/shared/hooks/<unit>`, `@/shared/lib/<area>/<unit>`. The `tray-panel.tsx` and `tray-panel.test.tsx` sit side-by-side inside the `tray-panel/` folder with no `index.ts`.

**Mechanism (two parts, kept small):**

1. **Vite plugin** (~40 LOC in `scripts/vite-folder-resolver.ts`, no new dependency): on `resolveId`, if the requested source matches `@/X/Y/Z` and `src/X/Y/Z/Z.{ts,tsx,js,jsx}` exists, return that absolute path. Otherwise return `null` and let Vite's default resolver run. Wired into `vite.config.ts` and `vitest.config.ts` (same plugin file, imported by both). Runs before tsconfig resolution. Unit-tested in `scripts/vite-folder-resolver.test.ts`.

2. **tsconfig paths** (for `tsc --noEmit` + IDE): add a TypeScript Language Service Plugin in `compilerOptions.plugins` (`{ "name": "mochi-folder-resolver" }`, ~30 LOC in `scripts/tsconfig-folder-resolver.ts`) that reuses the same logic so editor IntelliSense and `tsc --noEmit` resolve identically. The existing `"@/*": ["./src/*"]` mapping is kept as-is.

**No barrel files anywhere** — explicit oxlint rule via the `import` plugin's `no-restricted-imports` rule that errors on imports matching `*/index` from inside the project to kill barrel-style imports.

---

## 4. Test coverage approach

**Tool:** `@vitest/coverage-v8` (added as a devDependency). Per [Vitest's official coverage docs](https://vitest.dev/guide/coverage.html), v8 is the recommended provider — faster than Istanbul, no pre-transpile, and as of Vitest 3.2 the V8 output is AST-remapped to match Istanbul accuracy.

**Config (in `vitest.config.ts`):**
```ts
test: {
  coverage: {
    provider: "v8",
    include: ["src/**/*.{ts,tsx}", "app/**/*.{ts,tsx}"],
    exclude: [
      "**/*.test.{ts,tsx}",
      "**/*.test-d.{ts,tsx}",
      "app/routeTree.gen.ts",
      "src/shared/components/ui/**",   // shadcn primitives: tested indirectly
      "**/*.d.ts",
      "**/types.ts",
    ],
    reporter: ["text", "text-summary", "html", "lcov"],
    thresholds: {
      lines: 80,
      functions: 80,
      statements: 80,
      branches: 75,    // branches 80 is aggressive on UI components; 75 is realistic
    },
  },
}
```

**Thresholds rationale:**
- **Lines, functions, statements at 80%** — matches the "80% global" requirement.
- **Branches at 75%** — branches are the metric that bites on React components (early returns, optional chaining in JSX). 75% gives the same signal without forcing tests for every conditional render. This is consistent with [Bulletproof React's testing guidance](https://github.com/alan2207/bulletproof-react/blob/master/docs/testing.md) and the broader Vitest `coverage.watermarks` defaults.
- **Negative numbers also supported** (e.g. `lines: -50` means "no more than 50 uncovered lines") — useful as a softer floor if 80% proves hard on day one. We start with positive percentages.

**Excluding untestable code:** for files that are inherently hard to unit test (Tauri command wrappers, GSAP setup, window-event bridge), two options:
1. **File-level ignore** in the source: `/* v8 ignore file -- @preserve */` at the top of the file. The `-- @preserve` keeps the comment through Vite's esbuild transpile (per Vitest docs).
2. **Coverage exclude** in vitest config for whole folders. We use option 1 by default (more honest — the file shows up as 0% rather than vanishing) and option 2 only for generated/third-party.

**Adding tests for untested code:** follow the project's TDD skill (`/test-driven-development`) — failing test first, then implementation, then refactor. New test files land in the same per-unit folder as the source.

**New scripts in `package.json`:**
```json
{
  "test": "vitest run --config vitest.config.ts",
  "test:coverage": "vitest run --config vitest.config.ts --coverage",
  "test:watch": "vitest --config vitest.config.ts"
}
```

`pnpm test` stays fast (no coverage). `pnpm test:coverage` is what CI runs.

---

## 5. CI gate

**Two-job shape** (replaces the current single `frontend-tests` job; both are required):

1. **`frontend-tests`** — runs `pnpm test` (no coverage). Fast feedback on test failures, no false positives from coverage threshold swings.
2. **`frontend-coverage`** — runs `pnpm test:coverage`. Fails the build if either (a) any test fails, or (b) coverage drops below the threshold. The Vitest threshold check exits non-zero automatically, so we don't need a custom script.

**New job in `.github/workflows/pr.yml`:**
```yaml
  frontend-coverage:
    name: Frontend Coverage Gate
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v6
      - uses: pnpm/action-setup@v6
        with:
          package_json_file: package.json
      - uses: actions/setup-node@v6
        with:
          node-version-file: .nvmrc
          cache: pnpm
          cache-dependency-path: pnpm-lock.yaml
      - name: Install frontend dependencies
        run: pnpm install --frozen-lockfile
      - name: Run tests with coverage gate
        run: pnpm test:coverage
      - name: Upload coverage report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/
          retention-days: 14
```

**The threshold is enforced by Vitest itself**, not by a post-script. If `pnpm test:coverage` exits non-zero, the job fails, the PR is blocked. This is the standard pattern (per Vitest's own `coverage.thresholds` docs) and avoids any custom parsing of the coverage output.

**Rust side stays as-is** for this round: `rust-tests` keeps running `cargo test` (the same job that's there today). No Rust coverage threshold this round. We do add a `src-tauri/tests/cli_smoke.rs` integration test (Phase 4).

**No coverage regression in main:** the same `frontend-coverage` job also runs on `push` to `main` (the `pr.yml` workflow already triggers on `push: branches: [main]`), so a bad merge can't slip the gate.

**README badges (added to `README.md`):**
```markdown
[![Frontend CI](https://github.com/BrainerVirus/mochi/actions/workflows/pr.yml/badge.svg?branch=main&event=push)](./.github/workflows/pr.yml)
[![Frontend Coverage](https://github.com/BrainerVirus/mochi/actions/workflows/pr.yml/badge.svg?branch=main&job=frontend-coverage)](./.github/workflows/pr.yml)
```

These use GitHub Actions' native workflow badge URL — no third-party service, no token, the SVG is regenerated on every workflow run.

---

## 6. Migration plan and phasing

**Five phases, each its own PR per GitHub Flow:**

| Phase | Scope | PR size | Risk |
|---|---|---|---|
| **1. Foundation** | Add `vite-folder-resolver` plugin; mirror it in tsconfig; add `@vitest/coverage-v8` to devDeps; add `test:coverage` / `test:watch` scripts; add oxlint `no-restricted-imports` rule killing `*/index` imports; add `frontend-coverage` CI job; add README badges. **No file moves.** | small | low |
| **2. Shared layer** | Move every file under `src/lib/`, `src/styles/`, the current `src/hooks/` cross-feature hooks, and `src/components/{ui,providers,mascot}/` into `src/shared/...` per Section 2. Update imports via the resolver (no path rewrites needed in callers). | large (many small files) | medium |
| **3. Features, one at a time** | `tray` → `widget` → `settings` → `usage` → `updates` → `layout` → `about`. Each feature moves its `components/`, `hooks/`, `lib/` into `src/features/<name>/...`. Each PR keeps the build green before merge. | 7 PRs, medium each | medium |
| **4. Test fill-in** | After Phase 3, run `pnpm test:coverage` to find files below 80%. Add tests for previously-untested units using TDD. Apply `/* v8 ignore file -- @preserve */` to genuinely untestable code. Iterate until ≥ 80% global. | 1-3 PRs | low |
| **5. Docs & cleanup** | Update `AGENTS.md`, `docs/tech-stack.md`, and the 2026-05-19-mochi-design spec to reflect the new layout. Delete the one-shot migration helper. Verify the 7 feature + shared layout against `tree src/`. | small | low |

**Why split this way:**
- **Phase 1 first, no moves** — proves the path resolver works against the existing import surface (the trickiest part) before touching a single file. If it breaks, we revert one PR.
- **Shared before features** — features depend on shared. Moving shared first means feature moves only need to fix their own imports.
- **Features in order of complexity** — `tray` is the most complex (28 files including state machines); doing it first de-risks the pattern. Smaller features come later as quick wins.
- **Test fill-in is its own phase** — coverage work is qualitatively different from a refactor; bundling it would balloon PR size and hide progress.

**Per-PR verification (per `AGENTS.md`):**
```
pnpm lint
pnpm format:check
pnpm test
pnpm build
pnpm test:coverage
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Phases 2-3 skip the explicit `pnpm test:coverage` step until Phase 4 (no threshold yet). Phase 1 sets up the gate but with a temporary "no threshold" branch so existing in-flight PRs aren't blocked; Phase 4 flips the threshold on.

---

## 7. Out of scope, risks, and success metrics

**Out of scope:**
- Rust coverage threshold (deferred; `cargo test` keeps running as today).
- App-level refactors (route logic, state shape, data fetching strategy).
- New shadcn components, design-token changes, AGENTS.md process changes.
- File renaming to shorten the verbose path (e.g. dropping `tray-` in folder names).
- New tooling dependencies beyond `@vitest/coverage-v8`. The path resolver is in-repo.

**Risk register:**

| Risk | Likelihood | Mitigation |
|---|---|---|
| Path resolver breaks a third-party import | low | The plugin only matches `@/...` imports; node_modules and bare imports are skipped. Unit-tested in `scripts/vite-folder-resolver.test.ts`. |
| 80% threshold can't be met with current untested code | medium | Threshold is staged: Phases 1-3 ship without a hard threshold; Phase 4 writes the missing tests. We don't merge Phase 4 until 80% is real. |
| Rust test additions pollute PR size | low | Rust changes are limited to new `#[cfg(test)]` blocks in modules that have none and one smoke integration test under `src-tauri/tests/`. No refactors. |
| Coverage report storage bloats CI | low | HTML + lcov only; 14-day retention; `coverage/` already in `.gitignore`. |
| Cross-platform test breakage (per `AGENTS.md`) | low | Vitest is pure Node — runs identically on all three. The Rust integration test is a smoke test that compiles but doesn't require platform features. |

**Success metrics (validate post-merge of Phase 5):**
- `pnpm test:coverage` exits 0 in CI on `main`.
- `tree src/` shows exactly seven feature folders under `src/features/` and exactly the documented `src/shared/` tree.
- No `index.ts` (or `.tsx`) re-exporting multiple symbols appears anywhere under `src/`. The new oxlint rule makes this a build break.
- Every file in `src/features/**/components/`, `src/features/**/hooks/`, `src/features/**/lib/`, and `src/shared/**` has a colocated `*.test.*` file (or an explicit `v8 ignore file` comment).
- README badges render green.

**Cross-cutting constraints (from `AGENTS.md` and `docs/tech-stack.md`):**
- All code continues to work on macOS, Windows, and Linux. Vitest and `cargo test` both run on the three GitHub-hosted runners.
- New oxlint rules go into the existing `.oxlintrc.json`, not a new config.
- New scripts go into the existing `package.json` `scripts` block; no new manifest files.
- Commit messages follow `docs/agent-rules/commit-messages.md` (conventional commits, ≤50 char subject).
- Each phase ships on its own branch from `main` (GitHub Flow), squash-merged, branch deleted.

---

## References

- Vitest coverage docs: <https://vitest.dev/guide/coverage.html>
- Vitest coverage config reference: <https://vitest.dev/config/coverage.html>
- Bulletproof React project structure: <https://github.com/alan2207/bulletproof-react/blob/master/docs/project-structure.md> (source of the "no barrel files" recommendation and unidirectional `shared ← features ← app` flow)
- Tauri v2 capabilities/permissions: `src-tauri/capabilities/default.json`
- Existing tech stack: `docs/tech-stack.md`
- Project agent rules: `AGENTS.md`, `docs/agent-rules/`
