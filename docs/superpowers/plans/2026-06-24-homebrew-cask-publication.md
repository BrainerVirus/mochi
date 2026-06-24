# Homebrew Cask Publication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make stable and unstable Homebrew cask publication deterministic, idempotent, protected-branch compliant, and free of unstable self-release loops.

**Architecture:** Release jobs keep using `GITHUB_TOKEN` to create a deterministic cask branch and PR. Because bot-created pull-request workflows require approval, the publisher explicitly dispatches the existing PR workflow on the cask branch, waits for the uniquely named run at the exact head SHA, and squash-merges with a head-SHA guard. Cask-only PR and main pushes are path-ignored by automatic workflows, preventing approval-gated duplicate CI and unstable release recursion.

**Tech Stack:** Bash 3.2-compatible shell, GitHub CLI, GitHub Actions, Vitest

---

### Task 1: Add publication regression coverage

**Files:**

- Modify: `scripts/release/publish-homebrew-cask-pr.test.mjs`
- Create: `scripts/release/workflow-homebrew.test.mjs`

- [ ] **Step 1: Write a failing end-to-end shell test**

Create fake `git` and `gh` executables in a temporary directory, execute the real publication script, and assert that the recorded commands include:

```text
gh pr create --base main --head chore/homebrew-test
gh pr view <created-url> --json number --jq .number
gh workflow run pr.yml --ref chore/homebrew-test -f validation_id=<unique-id>
gh run watch <run-id> --exit-status
gh pr merge <pr-number> --squash --delete-branch --match-head-commit <sha>
```

Make the fake `gh pr create` reject `--json`, reproducing the production failure.

- [ ] **Step 2: Cover retry states**

Add cases proving that an existing remote branch is updated with an explicit force-with-lease SHA, an open PR is reused, a closed unmerged PR is reopened, and a no-change run exits without GitHub calls.

- [ ] **Step 3: Cover workflow contracts**

Assert that:

```text
pr.yml supports workflow_dispatch and ignores Casks/** pull requests
release-unstable.yml ignores Casks/** main pushes
both Homebrew jobs grant actions: write
```

- [ ] **Step 4: Run the focused tests and verify RED**

Run:

```bash
pnpm vitest run scripts/release/publish-homebrew-cask-pr.test.mjs scripts/release/workflow-homebrew.test.mjs
```

Expected: failures caused by the unsupported `gh pr create --json` call and missing dispatch/loop-guard workflow configuration.

### Task 2: Make the publisher deterministic and idempotent

**Files:**

- Modify: `scripts/release/publish-homebrew-cask-pr.sh`

- [ ] **Step 1: Push with an explicit remote lease**

Read the existing remote branch SHA with `git ls-remote`. Create the branch from the checked-out `main` commit, commit the generated cask, and either create the remote branch normally or update it with:

```bash
git push --force-with-lease="refs/heads/${BRANCH_NAME}:${REMOTE_SHA}" -u origin "${BRANCH_NAME}"
```

- [ ] **Step 2: Reuse or recover the PR**

Prefer an open PR. If none exists, reopen the latest closed, unmerged PR for the branch. Otherwise create a new PR, capture the URL emitted by `gh pr create`, and resolve its number with `gh pr view --json number --jq .number`.

- [ ] **Step 3: Dispatch and identify validation**

Build a unique validation ID from `GITHUB_RUN_ID`, `GITHUB_RUN_ATTEMPT`, and the branch head SHA. Dispatch `pr.yml` at the cask branch, poll `gh run list` for the matching run title and commit, fail clearly if it never appears, then run:

```bash
gh run watch "${RUN_ID}" --compact --exit-status --interval 15
```

- [ ] **Step 4: Merge safely and clean up**

After validation passes, squash-merge and delete the branch while requiring the PR head to still match the validated commit:

```bash
gh pr merge "${PR_NUM}" --squash --delete-branch --match-head-commit "${HEAD_SHA}"
```

- [ ] **Step 5: Run focused tests and verify GREEN**

Run the Task 1 focused Vitest command. Expected: all Homebrew publication tests pass.

### Task 3: Wire explicit validation and loop prevention

**Files:**

- Modify: `.github/workflows/pr.yml`
- Modify: `.github/workflows/release-stable.yml`
- Modify: `.github/workflows/release-unstable.yml`
- Modify: `docs/releasing.md`

- [ ] **Step 1: Make PR validation dispatchable**

Add a `workflow_dispatch` input named `validation_id` and a run name that includes it. Ignore `Casks/**` only for automatic `pull_request` events so bot-created cask PRs do not leave approval-required duplicate runs.

- [ ] **Step 2: Grant scoped dispatch permission**

Give each `update-homebrew-cask` job these explicit permissions:

```yaml
permissions:
  actions: write
  checks: read
  contents: write
  pull-requests: write
```

- [ ] **Step 3: Prevent unstable recursion**

Add `paths-ignore: ["Casks/**"]` to the unstable workflow's `push` trigger. A cask-only squash merge then updates `main` without publishing another unstable release.

- [ ] **Step 4: Document the token and retry model**

Document that no PAT or GitHub App secret is required: `workflow_dispatch` is the supported `GITHUB_TOKEN` recursion exception. Explain deterministic branches, PR reuse/reopen behavior, exact-SHA validation, squash merge, branch deletion, and cask-only loop prevention.

- [ ] **Step 5: Run focused tests**

Run:

```bash
pnpm test:release
```

Expected: all release tests pass.

### Task 4: Verify and publish through GitHub Flow

**Files:**

- Verify all changed files

- [ ] **Step 1: Run required local validation**

Run every command required by `AGENTS.md`:

```bash
pnpm lint
pnpm format:check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

- [ ] **Step 2: Commit and push**

Commit only the scoped files with a conventional subject under 50 characters, then push `fix/homebrew-cask-publication`.

- [ ] **Step 3: Open a ready PR**

Use `.github/PULL_REQUEST_TEMPLATE.md`, explain the observed root cause and token architecture, and include exact validation results.

- [ ] **Step 4: Wait for required checks**

Confirm Frontend Lint/Format/Tests/Build, React Doctor, Rust Format/Clippy/Tests all pass. Fix failures on the branch and rerun validation if needed.

- [ ] **Step 5: Squash merge and clean branches**

Squash-merge the PR, delete the remote branch, return the worktree to detached `origin/main`, delete the local branch, and verify the repository has no stale task branch.
