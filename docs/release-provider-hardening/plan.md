# Release, Provider, Linux, Deps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Spec:** `docs/release-provider-hardening/spec.md`
**Branch:** `feature/release-provider-hardening`

**Goal:** Workit-style semantic-release automation (stable-only), a Command Code usage provider, Ubuntu-verified Linux fixes with enforced core/platform seam, and a full dependency refresh headlined by Zod 4.5.

**Architecture:** Four sequential workstreams: (1) dependency refresh to a green modern baseline, (2) semantic-release replacing the dual-channel workflows with a tag-driven stable pipeline, (3) Command Code provider following the existing zai/claude-web provider module pattern, (4) discover-by-running Linux hardening with all platform quirks confined to platform modules.

**Tech Stack:** semantic-release + @semantic-release/exec (release), Tauri v2 + reqwest (provider), Zod ≥4.5 + Vitest (validation), GitHub Actions (CI).

## Global Constraints

- Branch: `feature/release-provider-hardening`; never commit directly to `main`.
- Conventional commits per `docs/agent-rules/commit-messages.md`; subject < 50 chars; no agent footers.
- After every task: `pnpm lint && pnpm format:check && pnpm test` green before commit; `cargo` suite green for Rust-touching tasks.
- Zod floor: `^4.5.0` (CA-11). TypeScript: attempt 7.x, pin latest 5.x with recorded reason if incompatible (CA-12).
- No new `#[cfg(target_os)]` branches in `src-tauri/src/core/` (CA-09).
- Product paths for the release gate: `app/`, `src/`, `src-tauri/`, `scripts/install/`, `Casks/`, `packaging/`.
- Provider endpoints (fixed): `https://api.commandcode.ai/internal/billing/credits`, `https://api.commandcode.ai/internal/usage/summary`; session cookie `__Secure-commandcode_prod_.session_token`.
- Unstable channel must not survive: workflows, feeds, casks, install scripts, tray menu, settings, updater (CA-04).

---

### Task 1: Dependency refresh baseline

**Files:**

- Modify: `package.json`
- Modify: `pnpm-lock.yaml` (via pnpm)
- Modify: `docs/tech-stack.md`
- Modify: `src/**` imports only if a major migration requires (vitest 4, lucide 1.x)

**Interfaces:**

- Consumes: existing `package.json` scripts (`lint`, `format:check`, `test`, `build`).
- Produces: lockfile + manifests on the new baseline; `zod@^4.5.4` importable as `import { z } from "zod"` unchanged; `docs/tech-stack.md` "Package Versions" section listing final versions.

- [ ] **Step 1: Write the failing check for the Zod floor**

Add to `scripts/release/version-consistency.test.mjs` a new test in the same describe block style:

```js
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("dependency floors", () => {
  it("keeps zod on 4.5 or newer", () => {
    const pkg = JSON.parse(readFileSync("package.json", "utf8"));
    const minor = pkg.dependencies.zod.match(/^[\^~]?(\d+)\.(\d+)/);
    expect([Number(minor[1]), Number(minor[2])]).toBeGreaterThanOrEqual([4, 5]);
  });
});
```

Note: `toBeGreaterThanOrEqual` does not compare arrays element-wise in vitest — compare numerically instead:

```js
const [maj, min] = pkg.dependencies.zod
  .match(/^[\^~]?(\d+)\.(\d+)/)
  .slice(1)
  .map(Number);
expect(maj * 100 + min).toBeGreaterThanOrEqual(405);
```

- [ ] **Step 2: Run to verify failure**

Run: `pnpm vitest run scripts/release/version-consistency.test.mjs`
Expected: FAIL — zod is `^4.4.3`, computed 404 < 405.

- [ ] **Step 3: Bump dependencies tier by tier**

Tier 1 (semver-compatible, all: `pnpm up <pkg>@latest`):
`zod@^4.5.4`, `@tanstack/react-router`, `@tanstack/router-plugin`, `@tanstack/react-query`, `@tauri-apps/api`, `@tauri-apps/cli`, `react`, `react-dom`, `@types/react`, `@types/react-dom`, `tailwindcss`, `@tailwindcss/vite`, `vite`, `oxlint`, `oxfmt`, `radix-ui`, `shadcn`, `lucide-react` (0.x → 1.x is breaking: check icon renames via `pnpm build`, fix or pin `^0.511` with reason), `gsap`, `@gsap/react`, `class-variance-authority`, `clsx`, `tailwind-merge`, `tw-animate-css`, `@fontsource-variable/geist`, `zustand`.

Tier 2 (majors, one at a time, full suite between each): `vitest@^4` + `@vitest/coverage-v8@^4` (update `vitest.config.ts` per v4 migration guide; coverage config keys changed), `happy-dom@^20`, `@babel/core@^8` (verify `babel-plugin-react-compiler` peer range; if incompatible keep babel 7 and record), `oxlint-tailwindcss@^1`, `oxlint-tsgolint` latest, `@types/node` matching `.nvmrc` node major.

Tier 3: `typescript@latest` (7.x). Run `pnpm build`. If tsc, `@tanstack/router-plugin`, or oxlint type-aware fails → `pnpm up typescript@^5` (latest 5.x) and record the reason in `docs/tech-stack.md`.

- [ ] **Step 4: Full validation**

Run: `pnpm lint && pnpm format:check && pnpm test && pnpm build`
Expected: all green. Fix breakage (vitest config migration, icon renames) until green; any package that cannot go latest gets pinned + one-line reason in `docs/tech-stack.md`.

- [ ] **Step 5: Update docs/tech-stack.md**

Rewrite the "Package Versions" list with final `npm view <pkg> version`-verified values and today's date. Add pin reasons where applicable.

- [ ] **Step 6: Run floor test to verify pass**

Run: `pnpm vitest run scripts/release/version-consistency.test.mjs`
Expected: PASS (405 ≥ 405).

- [ ] **Step 7: Commit**

```bash
git add package.json pnpm-lock.yaml docs/tech-stack.md vitest.config.ts src/ scripts/release/version-consistency.test.mjs
git commit -m "chore(deps): refresh packages, zod 4.5 floor"
```

---

### Task 2: Release gate script (semantic-release analyzeCommits)

**Files:**

- Create: `scripts/release/analyze-release-scope.mjs`
- Test: `scripts/release/analyze-release-scope.test.mjs`
- Modify: `package.json` (add `"test:release"` already exists — no change needed)

**Interfaces:**

- Consumes: git CLI; env `GITHUB_REF` not required (local invocation).
- Produces: `node scripts/release/analyze-release-scope.mjs [<lastTag>]` prints `major|minor|patch` or nothing (exit 0). Commit-list input via `git log <lastTag>..HEAD --no-merges --pretty=format:%s`. Reads product paths from a module export `PRODUCT_PATHS`.

- [ ] **Step 1: Write the failing tests**

`scripts/release/analyze-release-scope.test.mjs`:

```js
import { describe, expect, it } from "vitest";
import { bumpFromSubjects, changedProductPaths, PRODUCT_PATHS } from "./analyze-release-scope.mjs";

describe("bumpFromSubjects", () => {
  it("returns major for breaking change", () => {
    expect(bumpFromSubjects(["feat!: x", "fix: y"])).toBe("major");
  });
  it("returns feat over fix", () => {
    expect(bumpFromSubjects(["fix: y", "feat: x"])).toBe("minor");
  });
  it("returns patch for fix/perf only", () => {
    expect(bumpFromSubjects(["fix: y", "perf: z"])).toBe("patch");
  });
  it("returns null for docs/chore/ci only", () => {
    expect(bumpFromSubjects(["docs: x", "chore: y", "ci: z"])).toBeNull();
  });
});

describe("changedProductPaths", () => {
  it("detects product path changes", () => {
    const files = ["docs/tech-stack.md", "src-tauri/src/lib.rs", "README.md"];
    expect(changedProductPaths(files, PRODUCT_PATHS)).toEqual(["src-tauri/"]);
  });
  it("returns empty for docs-only change", () => {
    expect(changedProductPaths(["docs/tech-stack.md"], PRODUCT_PATHS)).toEqual([]);
  });
  it("matches nested app routes", () => {
    expect(changedProductPaths(["app/routes/index.tsx"], PRODUCT_PATHS)).toEqual(["app/"]);
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run: `pnpm vitest run scripts/release/analyze-release-scope.test.mjs`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement analyze-release-scope.mjs**

```js
#!/usr/bin/env node
// Path-gated release bump: prints major|minor|patch when conventional-commit
// product changes exist since <lastTag>; prints nothing to skip the release.
import { execFileSync } from "node:child_process";

export const PRODUCT_PATHS = [
  "app/",
  "src/",
  "src-tauri/",
  "scripts/install/",
  "Casks/",
  "packaging/",
];

const BUMPS = [
  ["major", /^(\w+)(\(.+\))?!:/],
  ["major", /^break(ing)?( change)?:/],
  ["minor", /^feat(\(.+\))?:/],
  ["patch", /^(fix|perf)(\(.+\))?:/],
];

export function bumpFromSubjects(subjects) {
  for (const [bump, re] of BUMPS) {
    if (subjects.some((s) => re.test(s))) return bump;
  }
  return null;
}

export function changedProductPaths(files, productPaths = PRODUCT_PATHS) {
  return productPaths.filter((prefix) => files.some((f) => f.startsWith(prefix)));
}

function git(...args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

export function collect(lastTag) {
  const range = lastTag ? `${lastTag}..HEAD` : "HEAD";
  const subjects = git("log", range, "--no-merges", "--pretty=format:%s")
    .split("\n")
    .filter(Boolean);
  const files = git("log", range, "--no-merges", "--pretty=format:", "--name-only")
    .split("\n")
    .map((f) => f.trim())
    .filter(Boolean);
  return { subjects, files };
}

const lastTag =
  process.argv[2] || git("describe", "--tags", "--abbrev=0", "--match=v*").catch?.() || "";
const { subjects, files } = collect(lastTag || "");
const bump = bumpFromSubjects(subjects);
const productChanged = changedProductPaths(files).length > 0;
if (bump && productChanged) console.log(bump);
```

Note: `git describe` throws when no `v*` tag exists — wrap with try/catch in final code; empty `lastTag` then means "all commits since repo start".

- [ ] **Step 4: Run tests to verify pass**

Run: `pnpm vitest run scripts/release/analyze-release-scope.test.mjs`
Expected: PASS (6 tests).

- [ ] **Step 5: Smoke-test against real git**

Run: `node scripts/release/analyze-release-scope.mjs v0.2.6 && echo "GATE OPEN" || echo "GATE CLOSED"`
Expected: prints `fix` (commit 177cf47 `fix(ci):` touches `.github/` — not a product path, so GATE CLOSED is correct; verify manually that a fake product commit flips it). Also run with a docs-only range to confirm empty output.

- [ ] **Step 6: Commit**

```bash
git add scripts/release/analyze-release-scope.mjs scripts/release/analyze-release-scope.test.mjs
git commit -m "feat(release): add path-gated bump analyzer"
```

---

### Task 3: semantic-release workflow + tag-driven stable build

**Files:**

- Create: `.github/workflows/release.yml`
- Create: `release.config.cjs`
- Modify: `.github/workflows/release-stable.yml` (inject tag version into manifests; strip unstable references from release body)
- Delete: `.github/workflows/release-unstable.yml`
- Modify: `package.json` (devDeps: `semantic-release`, `@semantic-release/exec`, `@semantic-release/release-notes-generator`, `@semantic-release/github`, `@semantic-release/git` not needed)
- Modify: `.github/workflows/pr.yml`, `package-smoke.yml`, `publish-updater-pages.yml`, `republish-updater-pages.yml` (unstable feed references)

**Interfaces:**

- Consumes: Task 2's `analyze-release-scope.mjs` via `analyzeCommitsCmd`.
- Produces: `vX.Y.Z` tags trigger `release-stable.yml`; manifests injected at build time by `scripts/release/sync-manifest-version.mjs --from-tag`; post-release sync PR titled `chore(release): sync manifests to vX.Y.Z`.

- [ ] **Step 1: Write the failing workflow-consistency test**

Extend `scripts/release/workflow-updater.test.mjs` pattern — create `scripts/release/workflow-release.test.mjs`:

```js
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const release = () => readFileSync(".github/workflows/release.yml", "utf8");
const stable = () => readFileSync(".github/workflows/release-stable.yml", "utf8");

describe("semantic-release workflow", () => {
  it("gates releases through the analyzer", () => {
    expect(release()).toMatch(/analyze-release-scope\.mjs/);
  });
  it("does not reference the unstable channel", () => {
    for (const f of [".github/workflows/release.yml", ".github/workflows/release-stable.yml"]) {
      expect(readFileSync(f, "utf8").toLowerCase()).not.toContain("unstable");
    }
  });
  it("stable build injects the tag version", () => {
    expect(stable()).toMatch(/sync-manifest-version\.mjs/);
  });
});
```

- [ ] **Step 2: Run to verify failure**

Run: `pnpm vitest run scripts/release/workflow-release.test.mjs`
Expected: FAIL — `release.yml` does not exist.

- [ ] **Step 3: Create release.config.cjs**

```js
module.exports = {
  branches: ["main"],
  tagFormat: "v${version}",
  plugins: [
    // Must be first (workit AR-16: @semantic-release/exec v7 renamed
    // analyzeCmd -> analyzeCommitsCmd; old name is silently ignored).
    // Printing nothing skips the release entirely — no tag, no publish.
    [
      "@semantic-release/exec",
      {
        analyzeCommitsCmd: "node scripts/release/analyze-release-scope.mjs",
      },
    ],
    "@semantic-release/release-notes-generator",
    "@semantic-release/github",
  ],
};
```

Note: when `analyzeCommitsCmd` prints nothing, semantic-release skips the release — no tag, no publish, no sync.

- [ ] **Step 4: Create .github/workflows/release.yml**

```yaml
name: Release

on:
  push:
    branches: [main]
  workflow_dispatch:
    inputs:
      dry-run:
        description: "Dry run (no tag/publish)"
        type: boolean
        default: false

permissions:
  contents: write
  pull-requests: write

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false

jobs:
  release:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0
          persist-credentials: false
      - uses: actions/setup-node@v6
        with:
          node-version-file: .nvmrc
      - name: Install dependencies
        run: pnpm install --frozen-lockfile
      - name: Release
        run: npx semantic-release${{ github.event.inputs.dry-run == 'true' && ' --dry-run' || '' }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      - name: Sync release manifests to main
        if: success() && github.event.inputs.dry-run != 'true'
        env:
          GH_TOKEN: ${{ secrets.RELEASE_SYNC_TOKEN || secrets.GITHUB_TOKEN }}
        run: |
          TAG=$(git describe --tags --abbrev=0 --match="v*")
          VERSION=${TAG#v}
          node scripts/release/sync-manifest-version.mjs --set "$VERSION"
          if [[ -z "$(git status --porcelain -- package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json)" ]]; then
            echo "manifests already match ${TAG}"; exit 0
          fi
          BRANCH="chore/manifest-sync-v${VERSION}"
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
          git remote set-url origin "https://x-access-token:${GH_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
          git checkout -b "${BRANCH}"
          git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
          git commit -m "chore(release): sync manifests to v${VERSION}"
          git push -u origin "${BRANCH}"
          gh pr create --title "chore(release): sync manifests to v${VERSION}" \
            --body "Automated post-release manifest alignment. Sets tracked manifests to released ${VERSION}." \
            --base main --head "${BRANCH}"
          gh pr merge --auto --squash --delete-branch || echo "auto-merge unavailable — merge manually"
```

Note: `Cargo.lock` version follows `Cargo.toml` on next `cargo` run; the sync PR omits it deliberately (workit omits generated locks similarly) — CI's `cargo test` regenerates it and the version-consistency test tolerates the transient diff. If `version-consistency.test.mjs` fails on the sync PR, add `src-tauri/Cargo.lock` to the sync list instead (decide by running the test locally with a bumped Cargo.toml).

- [ ] **Step 5: Create scripts/release/sync-manifest-version.mjs with tests**

Test first — `scripts/release/sync-manifest-version.test.mjs`:

```js
import { describe, expect, it } from "vitest";
import { setPackageVersion, setCargoVersion, setTauriVersion } from "./sync-manifest-version.mjs";

describe("sync-manifest-version", () => {
  it("sets package.json version", () => {
    const src = '{\n  "name": "mochi",\n  "version": "0.0.1"\n}';
    expect(JSON.parse(setPackageVersion(src, "9.9.9")).version).toBe("9.9.9");
  });
  it("sets Cargo.toml version keeping formatting", () => {
    const src = '[package]\nname = "mochi"\nversion = "0.0.1"\nedition = "2021"\n';
    expect(setCargoVersion(src, "9.9.9")).toContain('version = "9.9.9"');
    expect(setCargoVersion(src, "9.9.9")).toContain('edition = "2021"');
  });
  it("sets tauri.conf.json version", () => {
    const src = '{\n  "version": "0.0.1",\n  "identifier": "app.mochi"\n}';
    expect(JSON.parse(setTauriVersion(src, "9.9.9")).version).toBe("9.9.9");
  });
});
```

Implement `scripts/release/sync-manifest-version.mjs`:

```js
#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";

export function setPackageVersion(src, version) {
  const json = JSON.parse(src);
  json.version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

export function setCargoVersion(src, version) {
  return src.replace(/^version = "[^"]+"/m, `version = "${version}"`);
}

export function setTauriVersion(src, version) {
  const json = JSON.parse(src);
  json.version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

const args = process.argv.slice(2);
if (args[0] === "--set" && args[1]) {
  const version = args[1];
  writeFileSync("package.json", setPackageVersion(readFileSync("package.json", "utf8"), version));
  writeFileSync(
    "src-tauri/Cargo.toml",
    setCargoVersion(readFileSync("src-tauri/Cargo.toml", "utf8"), version),
  );
  writeFileSync(
    "src-tauri/tauri.conf.json",
    setTauriVersion(readFileSync("src-tauri/tauri.conf.json", "utf8"), version),
  );
  console.log(`manifests set to ${version}`);
}
```

Run: `pnpm vitest run scripts/release/sync-manifest-version.test.mjs` → FAIL → implement → PASS.

- [ ] **Step 6: Modify release-stable.yml for tag-driven build**

Insert after the `Install frontend dependencies` step and before `Verify updater signing configuration`:

```yaml
- name: Inject tag version into manifests
  shell: bash
  run: |
    VERSION="${GITHUB_REF_NAME#v}"
    [[ "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "bad version from tag ${GITHUB_REF_NAME}"; exit 1; }
    node scripts/release/sync-manifest-version.mjs --set "${VERSION}"
    node scripts/release/version-consistency.test.mjs 2>/dev/null || true
```

(Keep it simple: the version-consistency vitest is not a shell script; drop that last line. The build itself validates.) Replace the `releaseBody` unstable install line and the trailing "Unstable: add `-i`" paragraph with stable-only install instructions. Update the `release-notes` job body the same way (both places must match per repo convention).

- [ ] **Step 7: Delete release-unstable.yml and strip unstable from other workflows**

`git rm .github/workflows/release-unstable.yml`. Grep remaining workflows for `unstable` (`grep -rn unstable .github/workflows/`) — fix `publish-updater-pages.yml` / `republish-updater-pages.yml` to deploy `stable.json` (+ recovery copies) only. Run: `pnpm test:release` — fix failing expectations in `workflow-updater.test.mjs`, `collect-updater-artifacts.test.mjs`, `build-updater-feed.test.mjs`, `generate-homebrew-casks.test.mjs` to stable-only channels (these tests assert channel lists; update expected values, delete unstable-only cases).

- [ ] **Step 8: Run full workflow test suite**

Run: `pnpm test:release && pnpm vitest run scripts/`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add .github/workflows/ release.config.cjs scripts/release/ package.json pnpm-lock.yaml
git commit -m "feat(release): semantic-release stable-only pipeline"
```

---

### Task 4: Kill the unstable channel in app code

**Files:**

- Modify: `src-tauri/src/updater/mod.rs` (channel validation, endpoint tests)
- Modify: `src-tauri/src/settings/mod.rs` (`update_channel` default/parse, test at line 213)
- Modify: `src-tauri/src/tray/mod.rs` (channel menu items, lines 63-196)
- Modify: `src/lib/updates/**`, `src/lib/query/update-check/**` (unstable feed parsing)
- Modify: `scripts/install/lib/common.sh`, `install-linux.sh`, `install-macos.sh`, `install-macos-brew.sh`, `install-windows.ps1`, `install-windows.test.mjs`, `install-common.test.mjs`, `homebrew-tap.test.mjs`, `install-linux.test.mjs`, `install-macos.test.mjs`
- Modify: `Casks/` generator inputs (`scripts/release/generate-homebrew-casks.mjs` cask list)
- Modify: `docs/linux.md`, `README.md` (install instructions)
- Test: colocated `*.test.mjs` / inline `#[cfg(test)]`

**Interfaces:**

- Consumes: `MOCHI_UPDATE_CHANNEL` env (now always `stable`).
- Produces: `update_channel` setting collapses to `"stable"`; tray has no channel submenu; install scripts reject `-i/--unstable` with usage error.

- [ ] **Step 1: Failing Rust test — updater rejects unstable**

In `src-tauri/src/updater/mod.rs` tests module, replace `update_endpoint_builds_exact_unstable_feed_url` with:

```rust
#[test]
fn update_endpoint_rejects_unknown_channel() {
    assert!(update_endpoint_for_channel("unstable").is_none());
    assert!(update_endpoint_for_channel("nightly").is_none());
}
```

Adjust `update_endpoint_for_channel` (line ~98) to return `Option<&str>` and match only `"stable"`. Run `cargo test --manifest-path src-tauri/Cargo.toml updater` → FAIL → implement → PASS.

- [ ] **Step 2: Failing Rust test — settings default stable, no unstable parse**

In `src-tauri/src/settings/mod.rs` replace the line-213 test assertion and any `update_channel: "unstable"` fixtures:

```rust
#[test]
fn update_channel_defaults_to_stable() {
    let json = serde_json::json!({});
    let settings: MochiSettings = serde_json::from_value(json).expect("parse");
    assert_eq!(settings.update_channel, UpdateChannel::Stable);
}

#[test]
fn update_channel_rejects_unstable() {
    let json = serde_json::json!({ "update_channel": "unstable" });
    let result: Result<MochiSettings, _> = serde_json::from_value(json);
    assert!(result.is_err());
}
```

(Adapt to actual `MochiSettings` shape — `update_channel` may be `String`; keep the existing type, just make `"unstable"` fail validation via a serde deny-list or explicit enum match.) FAIL → implement → PASS.

- [ ] **Step 3: Remove tray channel submenu**

In `src-tauri/src/tray/mod.rs`: delete the `unstable: CheckMenuItem` field (line 63), the `channel-unstable` menu item construction (lines 68-105), and the `channel-unstable` handling arm (line 195). Add/adjust a test asserting the channel items builder produces only the stable item — follow existing tray test patterns if present; otherwise assert via the extracted builder function the plan touches. Run `cargo test --manifest-path src-tauri/Cargo.toml tray` → green.

- [ ] **Step 4: Failing script tests — install scripts reject --unstable**

In `scripts/install/install-common.test.mjs` add:

```js
import { describe, expect, it } from "vitest";
import { execFileSync } from "node:child_process";

describe("install scripts reject unstable", () => {
  it("install-linux.sh exits non-zero on -i", () => {
    let code = 0;
    try {
      execFileSync("bash", ["scripts/install/install-linux.sh", "-i"], { stdio: "pipe" });
      code = 0;
    } catch (e) {
      code = e.status ?? 1;
    }
    expect(code).not.toBe(0);
  });
});
```

(If executing the real script is too heavy — it hits the network — instead unit-test the arg parser exported from `lib/common.sh` sourced in a bash subshell; follow the existing `install-common.test.mjs` sourcing pattern.) FAIL → change parsers in `lib/common.sh` + each entry script to print usage and exit 2 on `-i|--unstable|MOCHI_UNSTABLE` → PASS.

- [ ] **Step 5: Strip unstable from frontend update code**

In `src/lib/updates/**` and `src/lib/query/update-check/**`: remove unstable channel branches from `sanitize-release-notes.ts`, `current-release-notes`, `update-check` query options; update colocated tests (`update-check.test.ts`, `current-release-notes.test.ts`) to stable-only expectations. Run `pnpm test` → green.

- [ ] **Step 6: Docs and Casks**

Update `README.md` install section and `docs/linux.md`: remove unstable flags/mentions. `scripts/release/generate-homebrew-casks.mjs`: drop the `mochi-unstable` cask from the generator defaults; update `generate-homebrew-casks.test.mjs` and `homebrew-tap.test.mjs` expectations.

- [ ] **Step 7: CA-04 grep verification**

Run: `grep -rni unstable .github/workflows/ scripts/ src/ app/ src-tauri/src/ Casks/ packaging/ || echo CLEAN`
Expected: CLEAN (zero hits; historical docs under `docs/superpowers/` are exempt).

- [ ] **Step 8: Full validation**

Run: `pnpm lint && pnpm format:check && pnpm test && pnpm build && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
Expected: green.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(release): remove unstable channel everywhere"
```

---

### Task 5: Command Code provider — parsing + client

**Files:**

- Create: `src-tauri/src/providers/commandcode/mod.rs`
- Create: `src-tauri/src/providers/commandcode/usage_parse.rs`
- Create: `src-tauri/src/providers/commandcode/client.rs`
- Create: `src-tauri/src/providers/commandcode/credentials.rs`
- Create: `src-tauri/src/providers/commandcode/strategy.rs`
- Create: `src-tauri/src/providers/commandcode/fixtures/credits.json`, `summary.json`
- Modify: `src-tauri/src/providers/mod.rs`

**Interfaces:**

- Consumes: `crate::core::provider::{FetchStrategy, FetchKind, ProviderError, ProviderResult, FetchContext}`; `crate::core::models::{ProviderId, UsageSnapshot, UsageWindow, ProviderCostSnapshot}`; `crate::browser::{import_cookies, CookieImportQuery}` (factory pattern).
- Produces: `ProviderId::CommandCode` variant (Task 6); `parse_credits(json: &serde_json::Value) -> ProviderResult<CreditsResponse>`; `snapshot_from_commandcode(credits: &CreditsResponse, summary: &SummaryResponse, updated_at: &str, source: &str) -> ProviderResult<UsageSnapshot>`; `CommandCodeClient` trait with `fetch_credits(&self, cookie: &str) -> ProviderResult<serde_json::Value>` + `fetch_summary(&self, cookie: &str) -> ProviderResult<serde_json::Value>`; `resolve_session_cookie(config) -> ProviderResult<Option<String>>`.

- [ ] **Step 1: Failing parse tests from HAR fixtures**

Create `fixtures/credits.json` (from the captured HAR shape; windowLimits structure inferred from the UI code — verify live in Task 7):

```json
{
  "credits": { "purchasedCredits": 0, "monthlyCredits": 12.5 },
  "windowLimits": {
    "fiveHour": { "usedPercent": 0, "resetsAt": "2026-09-02T23:32:00Z", "limited": false },
    "weekly": { "usedPercent": 0, "resetsAt": "2026-09-06T18:59:00Z", "limited": false },
    "monthly": { "usedPercent": 64, "resetsAt": "2026-09-10T00:00:00Z", "limited": true }
  }
}
```

Create `fixtures/summary.json` (verbatim from HAR):

```json
{
  "totalCount": 6319,
  "totalCost": 46.80689824929999,
  "averageCost": 0.007407326831666401,
  "successRate": 99.96834942237696,
  "completedCount": 6317,
  "failedCount": 2,
  "totalTokensIn": 1305381978,
  "totalTokensOut": 3061181,
  "totalTokens": 1308443159,
  "totalCredits": 46.80689824929999,
  "periodBasis": "billing-period"
}
```

In `usage_parse.rs` write tests first (module skeleton with `#[cfg(test)]`):

```rust
use crate::core::models::{ProviderId, UsageSnapshot};
use crate::core::provider::ProviderResult;

#[derive(Debug, Clone)]
pub struct WindowLimit {
    pub used_percent: f32,
    pub resets_at: Option<String>,
    pub limited: bool,
}

#[derive(Debug, Clone)]
pub struct CreditsResponse {
    pub monthly_credits_remaining: f64,
    pub five_hour: Option<WindowLimit>,
    pub weekly: Option<WindowLimit>,
    pub monthly: Option<WindowLimit>,
}

#[derive(Debug, Clone)]
pub struct SummaryResponse {
    pub total_tokens: f64,
    pub total_tokens_in: f64,
    pub total_tokens_out: f64,
    pub run_count: u64,
    pub total_cost: f64,
    pub success_rate: f64,
}

pub fn parse_credits(value: &serde_json::Value) -> ProviderResult<CreditsResponse> { todo!() }
pub fn parse_summary(value: &serde_json::Value) -> ProviderResult<SummaryResponse> { todo!() }
pub fn snapshot_from_commandcode(
    credits: &CreditsResponse,
    summary: &SummaryResponse,
    updated_at: &str,
    source: &str,
) -> ProviderResult<UsageSnapshot> { todo!() }

#[cfg(test)]
mod tests {
    use super::*;

    fn credits_fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("fixtures/credits.json")).unwrap()
    }
    fn summary_fixture() -> serde_json::Value {
        serde_json::from_str(include_str!("fixtures/summary.json")).unwrap()
    }

    #[test]
    fn parses_window_limits() {
        let credits = parse_credits(&credits_fixture()).expect("parse credits");
        let monthly = credits.monthly.as_ref().expect("monthly window");
        assert_eq!(monthly.used_percent, 64.0);
        assert!(monthly.limited);
        assert_eq!(credits.monthly_credits_remaining, 12.5);
    }

    #[test]
    fn parses_summary_totals() {
        let summary = parse_summary(&summary_fixture()).expect("parse summary");
        assert_eq!(summary.run_count, 6319);
        assert_eq!(summary.total_tokens, 1_308_443_159.0);
    }

    #[test]
    fn builds_snapshot_with_three_windows() {
        let credits = parse_credits(&credits_fixture()).expect("credits");
        let summary = parse_summary(&summary_fixture()).expect("summary");
        let snapshot = snapshot_from_commandcode(&credits, &summary, "2026-09-02T19:00:00Z", "commandcode-web").expect("snap");
        assert_eq!(snapshot.provider, ProviderId::CommandCode);
        assert_eq!(snapshot.primary.label, "Monthly");
        assert_eq!(snapshot.primary.used_percent, 64.0);
        let five = snapshot.extra_windows.iter().find(|w| w.label == "5 hours").expect("5h window");
        assert_eq!(five.used_percent, 0.0);
        assert!(snapshot.provider_cost.is_some());
    }

    #[test]
    fn rejects_malformed_credits() {
        let bad = serde_json::json!({ "credits": "nope" });
        assert!(parse_credits(&bad).is_err());
    }
}
```

(Remove the `todo!`-with-env! confusion: final test module uses the two concrete fixture helpers shown.)

Run: `cargo test --manifest-path src-tauri/Cargo.toml commandcode` → FAIL (compile error: module not registered, parse fns are `todo!()`).

- [ ] **Step 2: Register module + enum variant**

In `src-tauri/src/providers/mod.rs` add `pub(crate) mod commandcode;` (alphabetical, after `claude`). Run the test again → FAIL (parse fns are `todo!`, `ProviderId::CommandCode` missing — add the enum variant + `ALL` entry + `"commandcode"` serde/FromStr mapping in `core/models.rs` now, since compilation requires it; the registry count test moves to Task 6).

- [ ] **Step 3: Implement parsers**

Implement `parse_credits`, `parse_summary`, `snapshot_from_commandcode`:

```rust
pub fn parse_credits(value: &serde_json::Value) -> ProviderResult<CreditsResponse> {
    let credits = value
        .get("credits")
        .ok_or_else(|| ProviderError::Parse("commandcode: missing credits".into()))?;
    let monthly_credits_remaining = credits
        .get("monthlyCredits")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| ProviderError::Parse("commandcode: missing monthlyCredits".into()))?;

    let window = |key: &str| -> Option<WindowLimit> {
        value.get("windowLimits")?.get(key).map(|raw| WindowLimit {
            used_percent: raw.get("usedPercent").and_then(serde_json::Value::as_f64).unwrap_or(0.0) as f32,
            resets_at: raw.get("resetsAt").and_then(serde_json::Value::as_str).map(str::to_string),
            limited: raw.get("limited").and_then(serde_json::Value::as_bool).unwrap_or(false),
        })
    };

    Ok(CreditsResponse {
        monthly_credits_remaining,
        five_hour: window("fiveHour"),
        weekly: window("weekly"),
        monthly: window("monthly"),
    })
}
```

`parse_summary`: field-by-field `as_f64`/`as_u64` with `ProviderError::Parse` on missing `totalCount`/`totalTokens`. `snapshot_from_commandcode`: primary = monthly window (label `"Monthly"`), secondary = weekly (label `"Weekly"`), five-hour into `extra_windows` (label `"5 hours"`); `provider_cost = Some(ProviderCostSnapshot { used: summary.total_cost, limit: used+remaining derived from monthly_credits, currency_code: "USD", period: Some("billing-period"), resets_at: monthly.resets_at })` — follow `ProviderCostSnapshot::new` if it exists. Run tests → PASS.

- [ ] **Step 4: Failing client + credentials tests**

`client.rs`: trait + HTTP impl following `claude/web/client.rs` (reqwest, 30s timeout, `Cookie` header built from session value). Test with a mock trait impl asserting both endpoints are called and cookie header is passed:

```rust
#[tokio::test]
async fn fetches_both_endpoints() {
    let client = MockClient::default(); // records urls
    let _ = client.fetch_credits("__Secure-commandcode_prod_.session_token=abc");
    let _ = client.fetch_summary("__Secure-commandcode_prod_.session_token=abc");
    // assert via recorded urls
}
```

`credentials.rs`: mirror `factory/credentials.rs` — `ENV_COOKIE = "MOCHI_COMMANDCODE_COOKIE"`, `DOMAINS = &["commandcode.ai"]`, `SESSION_COOKIE_NAMES = &["__Secure-commandcode_prod_.session_token"]`, manual-cookie first, env second, browser import last. Test cookie-header extraction from a raw `Cookie:` header string (copy the claude `session_key_from_cookie_header` test shape).

- [ ] **Step 5: Strategy wiring**

`strategy.rs`: `WebStrategy` with `FetchKind::BrowserCookies`, id `"commandcode-web"`, `is_available` = cookie resolvable, `fetch` = resolve cookie → fetch both endpoints → parse → `snapshot_from_commandcode`. Fixture-client tested like `zai/strategy.rs` `FixtureClient`. Run `cargo test --manifest-path src-tauri/Cargo.toml commandcode` → PASS.

- [ ] **Step 6: Rust validation**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml commandcode`
Expected: green.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/providers/commandcode src-tauri/src/providers/mod.rs src-tauri/src/core/models.rs
git commit -m "feat(providers): add commandcode parse and client"
```

---

### Task 6: Command Code provider — registry + frontend

**Files:**

- Modify: `src-tauri/src/core/models.rs` (done in Task 5 for the enum; finish `ALL`, `from_str`, serde aliases)
- Modify: `src-tauri/src/core/registry.rs`, `src-tauri/src/providers/mod.rs` (`built_in_providers`)
- Modify: `src/lib/schemas/usage/usage.ts` (add `"commandcode"` to the provider union)
- Modify: `src/lib/schemas/settings/settings.ts`
- Modify: `src/lib/providers/dashboard-urls/dashboard-urls.ts` (`commandcode: { url: "https://commandcode.ai/cristhoferpincefig0/settings/usage" }` — use the generic `https://commandcode.ai/<login>/settings/usage` form; check how other providers template the login)
- Modify: `src/lib/utils/provider-labels/provider-labels.ts` (`commandcode: "Command Code"`)
- Modify: `src/shared/components/providers/provider-icon-sources/provider-icon-sources.ts` (add `commandcode.svg` — export a simple mark from the brand or reuse a neutral icon until brand SVG is available)
- Modify: `src-tauri/src/providers/mod.rs` test `includes_twelve_v1_providers` → thirteen

**Interfaces:**

- Consumes: Task 5's `CommandCodeProvider` struct implementing `Provider`.
- Produces: provider selectable in tray/dashboard; `z.string()` provider schema accepts `"commandcode"`.

- [ ] **Step 1: Failing registry test**

In `src-tauri/src/providers/mod.rs` change the test:

```rust
#[test]
fn includes_thirteen_v1_providers() {
    assert_eq!(built_in_providers().len(), 13);
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers::` → FAIL (12 ≠ 13).

- [ ] **Step 2: Register the provider**

Create `CommandCodeProvider` in `commandcode/mod.rs` implementing `Provider` (metadata: id `ProviderId::CommandCode`, name "Command Code", `FetchKind::BrowserCookies` strategy list) mirroring `zai/mod.rs`. Add to `built_in_providers()`. Complete `ProviderId::ALL` (13 entries), the `"commandcode"` name mapping (line ~53/80 in `core/models.rs`). Run → PASS.

- [ ] **Step 3: Failing frontend schema test**

In `src/lib/schemas/usage/usage.test.ts` (colocated; create if absent following other schema test patterns) add:

```ts
import { describe, expect, it } from "vitest";
import { UsageSnapshotSchema } from "./usage";

describe("usage provider union", () => {
  it("accepts commandcode", () => {
    const base = {
      provider: "zai",
      primary: { label: "M", used_percent: 1, remaining_percent: 99, resets_at: null },
      updated_at: "",
      source: "test",
    };
    expect(UsageSnapshotSchema.safeParse({ ...base, provider: "commandcode" }).success).toBe(true);
  });
});
```

(Adapt the fixture to the actual schema shape after reading `usage.ts`.) Run `pnpm vitest run src/lib/schemas/usage` → FAIL → add `"commandcode"` to the union → PASS.

- [ ] **Step 4: Labels, dashboard URL, icon**

Add label `"Command Code"`, dashboard URL (match the login-templating pattern used by claude/z.ai entries), and an SVG source. For the icon: create `src/shared/assets/providers/commandcode.svg` — simple `>_` terminal glyph on transparent background is acceptable until brand asset; register in `provider-icon-sources.ts`. Run `pnpm lint && pnpm build` → green.

- [ ] **Step 5: Full validation**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --all-targets && pnpm lint && pnpm test && pnpm build`
Expected: green (CA-06 backend + frontend wiring; live-API verification happens in Task 8).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src src/
git commit -m "feat(providers): register commandcode provider"
```

---

### Task 7: Command Code live verification (manual, this machine)

**Files:**

- Modify: none (verification task; may fix parse bugs found)

**Interfaces:**

- Consumes: Tasks 5-6 provider; the captured session cookie from the user's browser profile on this machine (Firefox, per HAR user-agent).
- Produces: evidence entry in `docs/qa/` confirming live snapshot (CA-06) or a punch-list of parse fixes.

- [ ] **Step 1: Launch the app**

Run: `pnpm tauri:dev`
Expected: app builds and launches on Ubuntu (first live Linux data point for Task 9).

- [ ] **Step 2: Import the session cookie**

Use mochi's browser-cookie import for `commandcode.ai` from the local Firefox profile, or paste a raw cookie via the manual-cookie setting. Verify `resolve_session_cookie` finds it (add a temporary `dbg!` or use existing diagnostics CLI).

- [ ] **Step 3: Fetch live snapshot**

Trigger a provider refresh. Expected: 5h/weekly/monthly windows match the commandcode.ai usage page values. If `windowLimits` shape differs from the inferred fixture (field names like `usedPercent` vs `used_percent`, or nested `limits`), capture the real JSON and update fixtures + parser — TDD: adjust the failing test first, then the parser.

- [ ] **Step 4: Record evidence**

Append a dated evidence block to `docs/qa/` (existing QA doc structure) with the live values, or file punch-list items into Task 9 if Linux issues surface during launch.

---

### Task 8: Linux discover-by-running pass

**Files:**

- Modify: `docs/qa/linux-ubuntu-evidence.md` (create; evidence log)
- Modify: Linux-specific modules found faulty: `src-tauri/src/linux_webkit.rs`, `src-tauri/src/linux_window_controls.rs`, `src-tauri/src/window_policy.rs`, `src-tauri/src/tray/mod.rs` (appindicator paths), `src-tauri/src/diagnostics/**`, `src-tauri/src/cli/**`
- Test: colocated Rust `#[cfg(test)]` for any fixed pure logic; manual evidence otherwise

**Interfaces:**

- Consumes: running app from Task 7.
- Produces: evidence doc mapping each found issue → fix (or accepted quirk with reason); CA-08/CA-09 satisfied.

- [ ] **Step 1: Enumerate**

Launch and exercise: tray icon presence + menu (libappindicator), left/right click behavior, main window open/close/hide, widget window, settings window, provider refresh, updater check (should hit stable feed), CLI `diagnostics` output, notifications, single-instance behavior, window decorations/controls on GNOME (Wayland + X11 sessions if available). Log every failure with repro steps + desktop session in `docs/qa/linux-ubuntu-evidence.md`.

- [ ] **Step 2: Fix in priority order**

For each issue: root-cause in the platform module (no core `#[cfg]` per CA-09), TDD where logic is testable, then verify manually. Cross-platform guard: after each fix, mentally/CI-verify macOS + Windows paths unchanged (cfg-gated code is compile-checked by CI matrix).

- [ ] **Step 3: Regression suite**

Run: `pnpm lint && pnpm format:check && pnpm test && pnpm build && cargo fmt --manifest-path src-tauri/Cargo.toml -- --check && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
Expected: green (CA-10).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "fix(linux): ubuntu hardening pass"
```

---

### Task 9: Final verification + docs

**Files:**

- Modify: `docs/tech-stack.md` (final versions, if Task 1 left TODOs)
- Modify: `docs/qa/linux-ubuntu-evidence.md` (final state)
- Modify: `README.md` (release process section: conventional commits + auto-release)

**Interfaces:**

- Consumes: all prior tasks.
- Produces: green full suite; spec CA-01..CA-12 each addressed; README documents the new release contract.

- [ ] **Step 1: Full local validation**

Run:

```bash
pnpm lint && pnpm format:check && pnpm test && pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all green — record the outputs as evidence.

- [ ] **Step 2: CA sweep**

Walk CA-01..CA-12 in the spec; for each, note the evidence (test, grep, doc, workflow file). Any gap → fix now.

- [ ] **Step 3: README release section**

Document: conventional commits drive releases; `feat:`/`fix:`/`perf:` on product paths auto-release; docs/chore never release; manifests sync via automated PR; no unstable channel.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "docs(release): document semantic-release flow"
```
