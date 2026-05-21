# Releasing Mochi

Mochi uses GitHub Flow.

## Unstable

Every successful merge to `main` publishes unstable artifacts and updates the unstable feed.

## Stable

Stable releases are created by tagging a commit on `main` with a semver tag such as `v1.0.0`.

## Update Channels

Stable users receive only stable updates. Unstable users receive builds from `main`. Users can switch channels in Settings.

## Required Secrets

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- macOS signing and notarization secrets for stable releases
- Windows signing secrets for stable releases when available
