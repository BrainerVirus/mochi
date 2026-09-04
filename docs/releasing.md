# Releasing Mochi

Releases are fully automated: conventional commits on `main` drive semantic-release, which tags, publishes the GitHub Release, syncs manifests, and triggers the installer/updater pipeline. There is no unstable channel and no manual version bump.

## How a release happens

1. A PR with conventional commits (`feat:`, `fix:`, `perf:` — or `!` in the subject, e.g. `feat!:`, for major) merges to `main`.
2. `.github/workflows/release.yml` runs semantic-release with the path gate in `scripts/release/analyze-release-scope.mjs` (configured via `analyzeCommitsCmd` in [release.config.cjs](../release.config.cjs)):
   - The gate prints `major|minor|patch` **only when** commits since the last `v*` tag contain a release-type commit **and** touch product paths: `app/`, `src/`, `src-tauri/`, `scripts/install/`, `Casks/`, `packaging/`.
   - Docs, CI, and chore-only pushes print nothing and skip the release entirely — no tag, no release, no sync PR.
3. semantic-release pushes the `vX.Y.Z` tag and creates the GitHub Release with generated notes.
4. The workflow opens a **manifest-sync PR** (`chore(release): sync manifests to vX.Y.Z`) setting `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json` to the released version on `main`, then auto-merge squashes it.
5. The tag triggers `.github/workflows/release-stable.yml`:
   - 4-platform build matrix (macOS arm64/x64, Windows x64, Linux x64). The build **injects the tag version** into the manifests at build time (`scripts/release/sync-manifest-version.mjs --set`); it never trusts manifest versions.
   - Verifies updater signing configuration, builds and publishes installers + signed Tauri updater artifacts to the GitHub Release.
   - Deploys `stable.json` updater feeds to GitHub Pages (via `publish-updater-pages.yml@main`) and opens the Homebrew cask PR (`chore(homebrew): update stable cask for vX.Y.Z`).

## Dry run

Run **Release** (`release.yml`) via `workflow_dispatch` with **dry-run = true**: it runs the gate and version computation without tagging or publishing.

## Requirements

- **`RELEASE_SYNC_TOKEN` secret** (fine-grained PAT, this repo only, Contents: read/write, Pull requests: read/write). `GITHUB_TOKEN`-created tags do not trigger other workflows, so the PAT is required for the tag to fire `release-stable.yml` and for the sync PR to run CI. The workflow fails fast if it is missing.
- `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `MOCHI_UPDATER_PUBLIC_KEY` — verified before any build.
- `HOMEBREW_PR_TOKEN` (fine-grained token scoped to this repository; Actions: read, Contents: read and write, Pull requests: read and write) for the cask PR. Rotate before expiry.

## Commit discipline

Because commit messages drive versioning:

- Use conventional commit types only (`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`).
- `feat:` → minor, `fix:`/`perf:` → patch, `!` in the subject (e.g. `feat!:` / `fix(scope)!:`) → major — but only when product paths change. The release gate scans commit subjects only; a `BREAKING CHANGE:` footer does not trigger a major bump.
- `docs:`, `chore:`, `ci:`, `test:`, `refactor:`, `style:` commits never release, even on product paths.
- Never reword or edit commits on `main` after a release; semantic-release reads history from the last tag.

## Updater feeds

Feeds live under `https://brainervirus.github.io/mochi/updates/{target}/{arch}/{current_version}/stable.json`. Only the stable pipeline deploys to Pages (full-site replacement — no other release workflow may call `deploy-pages`). Feeds are backfilled for supported recovery versions (currently `0.1.7`, `0.2.0`).

If binaries for `vX.Y.Z` are published but the Pages deploy failed, run **Republish Updater Pages** (`republish-updater-pages.yml`) via `workflow_dispatch` with `release_tag=vX.Y.Z` — no new tag needed.

Validate representative endpoints after publication:

```bash
curl -fsS https://brainervirus.github.io/mochi/updates/darwin/aarch64/0.1.7/stable.json
curl -fsS https://brainervirus.github.io/mochi/updates/linux/x86_64/0.1.7/stable.json
curl -fsS https://brainervirus.github.io/mochi/updates/windows/x86_64/0.1.7/stable.json
```

## Release notes

Notes are generated from conventional commits by `@semantic-release/release-notes-generator`. User-facing highlights are curated in `.github/workflows/release-stable.yml` in both places (they must match): the `releaseBody` field in the `tauri-action` step and the `body` array in the `release-notes` job. Focus on what users experience; do not mention CI fixes, refactors, or internal tooling.

## macOS distribution (no Apple Developer account)

macOS builds are **ad-hoc signed** in CI (`APPLE_SIGNING_IDENTITY=-`). They are not notarized. Homebrew and direct `.dmg` installers remove the download quarantine flag so Gatekeeper does not show the misleading “damaged” dialog. If a manual install still fails to open, run:

```bash
xattr -dr com.apple.quarantine /Applications/Mochi.app
```

## Linux Window Controls

Linux decorated app windows are created on demand and visible. This avoids Ubuntu Wayland native titlebar hit-region failures caused by hidden precreation of decorated windows.

Do not add `MOCHI_LINUX_WINDOW_EXPERIMENT` back to release workflows. The proven behavior is now the default Linux behavior.
