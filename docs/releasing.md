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

### All releases

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `MOCHI_UPDATER_PUBLIC_KEY`

### Stable macOS (Developer ID signing + notarization)

Stable macOS builds fail until these GitHub Actions secrets are configured. Unstable builds stay ad-hoc signed.

**Certificate import (required):**

- `APPLE_CERTIFICATE` — base64-encoded `.p12` export of your **Developer ID Application** certificate (include the private key). Generate with `openssl base64 -A -in certificate.p12 -out certificate-base64.txt`.
- `APPLE_CERTIFICATE_PASSWORD` — password used when exporting the `.p12`.
- `KEYCHAIN_PASSWORD` — arbitrary password for the temporary CI keychain.

**Notarization (choose one method):**

*Apple ID (simplest to start):*

- `APPLE_ID` — Apple ID email.
- `APPLE_PASSWORD` — [app-specific password](https://support.apple.com/en-us/HT204397).
- `APPLE_TEAM_ID` — Team ID from [Membership Details](https://developer.apple.com/account#MembershipDetailsCard).

*App Store Connect API key (recommended for CI):*

- `APPLE_API_KEY` — Key ID from App Store Connect → Users and Access → Integrations → Keys.
- `APPLE_API_ISSUER` — Issuer ID shown above the keys table.
- `APPLE_API_KEY_PRIVATE` — base64-encoded contents of the downloaded `.p8` private key (`openssl base64 -A -in AuthKey_XXXX.p8 -out key-base64.txt`).

**Optional:**

- `APPLE_SIGNING_IDENTITY` — overrides auto-detected identity (usually `Developer ID Application: Your Name (TEAMID)`).

### Windows (when available)

- Windows code signing secrets for stable releases

### GitHub Pages

- GitHub Pages publication token if `GITHUB_TOKEN` cannot write the Pages source branch

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
