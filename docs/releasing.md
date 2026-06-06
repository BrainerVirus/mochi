# Releasing Mochi

Mochi uses GitHub Flow.

## Unstable

Every successful merge to `main` publishes unstable artifacts, a GitHub **prerelease** tagged `unstable`, and updates the unstable feed.

Install the latest unstable build with the `-i` flag — see [Install](../README.md#install) in the README.

## Stable

Stable releases are created by tagging a commit on `main` with a semver tag such as `v1.0.0`.

Install scripts default to the latest stable release — see [Install](../README.md#install) in the README.

Release notes should describe only the tag being published. Put upgrade warnings in docs or issues, not in the body for unrelated future releases.

## Update Channels

Stable users receive only stable updates. Unstable users receive builds from `main`. Users can switch channels in Settings.

## Required Secrets

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `MOCHI_UPDATER_PUBLIC_KEY`
- macOS signing and notarization secrets for stable releases
- Windows signing secrets for stable releases when available
- GitHub Pages publication token if `GITHUB_TOKEN` cannot write the Pages source branch

## Updater Feed

Release workflows must generate signed Tauri updater artifacts and publish versioned feeds under `https://brainervirus.github.io/mochi/updates/{target}/{arch}/{current_version}/{channel}.json`.

Required secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `MOCHI_UPDATER_PUBLIC_KEY`
- GitHub Pages publication token if `GITHUB_TOKEN` cannot write the Pages source branch.

The feed is backfilled for supported installed versions, currently `0.1.7` and `0.2.0`, so older installed apps can recover through in-app update. Every supported recovery version must have both `stable.json` and `unstable.json` for macOS arm64/x64, Linux x64, and Windows x64.

The first stable repair release publishes both `stable.json` and `unstable.json` from tags like `v0.2.1` so installed unstable-channel apps recover from missing feeds. Later unstable releases publish `unstable.json` from tags like `unstable-20260606.123456` using a feed version such as `0.2.1-unstable.20260606.123456`, where `0.2.1` is the explicit `MOCHI_UNSTABLE_BASE_VERSION`. Workflows must fail when any required updater bundle or `.sig` file is missing.

Validate representative endpoints after publication:

```bash
curl -fsS https://brainervirus.github.io/mochi/updates/darwin/aarch64/0.1.7/stable.json
curl -fsS https://brainervirus.github.io/mochi/updates/linux/x86_64/0.1.7/stable.json
curl -fsS https://brainervirus.github.io/mochi/updates/windows/x86_64/0.1.7/stable.json
```
