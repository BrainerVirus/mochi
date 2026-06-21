# Releasing Mochi

Mochi uses GitHub Flow.

## Unstable

Every successful merge to `main` publishes unstable artifacts and a GitHub **prerelease** tagged `unstable-*`. The unstable workflow validates updater feed JSON locally but **does not deploy to GitHub Pages**.

Install the latest unstable build with the `-i` flag — see [Install](../README.md#install) in the README.

## Stable

Stable releases are created by tagging a commit on `main` with a semver tag such as `v1.0.0`.

Install scripts default to the latest stable release — see [Install](../README.md#install) in the README.

Release notes should describe only the tag being published. Put upgrade warnings in docs or issues, not in the body for unrelated future releases.

## GitHub Pages Publish Rules

GitHub Pages deploys are **full-site replacements**: whatever artifact is uploaded becomes the entire site. Two workflows publishing different subsets of updater JSON will clobber each other.

Rules:

1. **Only the stable release pipeline deploys to GitHub Pages.** Unstable builds must never call `deploy-pages`.
2. **Stable publishes both channels.** The stable Pages artifact includes `stable.json` and `unstable.json` for every supported recovery version so older installs can recover.
3. **Publish logic lives in `.github/workflows/publish-updater-pages.yml` on `main`.** Stable release calls that reusable workflow at `@main` so feed deploy fixes can ship without cutting a new tag.
4. **Do not gate Pages deploy on a protected `github-pages` environment** unless tag refs and `workflow_dispatch` are explicitly allowed. Otherwise stable tag releases fail after binaries are already published.
5. **`GITHUB_SHA` cannot be overridden in Actions.** `deploy-pages` keys deployments by commit SHA. Unstable must not deploy Pages on the same commit as a stable tag, or the stable deploy can be skipped while the live site still serves the wrong feed set.

### Republish feeds for an existing stable tag

If binaries for `vX.Y.Z` are already on GitHub Releases but Pages deploy failed, run **Republish Updater Pages** (`republish-updater-pages.yml`) via `workflow_dispatch` with `release_tag=vX.Y.Z`. That workflow downloads the published release assets, rebuilds feeds, deploys Pages, and retries live validation. No new tag or version bump is required.

## Update Channels

Stable users receive only stable updates. Unstable users receive builds from `main`. Users can switch channels in Settings.

## Required Secrets

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `MOCHI_UPDATER_PUBLIC_KEY`
- Windows code signing secrets for stable releases when available
- GitHub Pages publication token if `GITHUB_TOKEN` cannot write the Pages source branch

## macOS distribution (no Apple Developer account)

macOS builds are **ad-hoc signed** in CI (`APPLE_SIGNING_IDENTITY=-`). They are not notarized.

Homebrew and direct `.dmg` installers remove the download quarantine flag so Gatekeeper does not show the misleading “damaged” dialog. If a manual install still fails to open, run:

```bash
xattr -dr com.apple.quarantine /Applications/Mochi.app
```

## Updater Feed

Release workflows must generate signed Tauri updater artifacts and publish versioned feeds under `https://brainervirus.github.io/mochi/updates/{target}/{arch}/{current_version}/{channel}.json`.

Required secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `MOCHI_UPDATER_PUBLIC_KEY`
- GitHub Pages publication token if `GITHUB_TOKEN` cannot write the Pages source branch.

The feed is backfilled for supported installed versions, currently `0.1.7` and `0.2.0`, so older installed apps can recover through in-app update. Every supported recovery version must have both `stable.json` and `unstable.json` for macOS arm64/x64, Linux x64, and Windows x64.

The first stable repair release publishes both `stable.json` and `unstable.json` from tags like `v0.2.1` so installed unstable-channel apps recover from missing feeds. Unstable prereleases on `main` continue to publish GitHub Release assets, but **Pages feeds for both channels are updated only by stable release or Republish Updater Pages**. Workflows must fail when any required updater bundle or `.sig` file is missing.

Validate representative endpoints after publication:

```bash
curl -fsS https://brainervirus.github.io/mochi/updates/darwin/aarch64/0.1.7/stable.json
curl -fsS https://brainervirus.github.io/mochi/updates/linux/x86_64/0.1.7/stable.json
curl -fsS https://brainervirus.github.io/mochi/updates/windows/x86_64/0.1.7/stable.json
```

## Linux Window Controls

Linux decorated app windows are created on demand and visible. This avoids Ubuntu Wayland native titlebar hit-region failures caused by hidden precreation of decorated windows.

Do not add `MOCHI_LINUX_WINDOW_EXPERIMENT` back to release workflows. The proven behavior is now the default Linux behavior.
