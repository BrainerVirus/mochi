#!/usr/bin/env bash
# Install Mochi unstable via Homebrew cask (macOS).
# Generates a temporary cask from the latest unstable GitHub release.
# Usage: install-macos-brew.sh [release-tag]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

mochi_need_cmd brew
mochi_need_cmd jq

TAG="$(mochi_resolve_release_tag "${1:-}")"
RELEASE_JSON="$(mochi_release_json "${TAG}")"

ARCH="$(uname -m)"
case "${ARCH}" in
  arm64) ASSET_PATTERNS=('aarch64.*\.dmg$' 'arm64.*\.dmg$') ;;
  x86_64) ASSET_PATTERNS=('x64.*\.dmg$' 'x86_64.*\.dmg$') ;;
  *) mochi_die "unsupported macOS architecture: ${ARCH}" ;;
esac

ASSET_URL="$(mochi_pick_asset_url "${RELEASE_JSON}" "${ASSET_PATTERNS[@]}")" \
  || mochi_die "no macOS .dmg asset for ${ARCH} in release ${TAG}"

ASSET_NAME="$(echo "${RELEASE_JSON}" | jq -r --arg url "${ASSET_URL}" '
  [.assets[] | select(.browser_download_url == $url)][0].name
')"
[[ -n "${ASSET_NAME}" ]] || mochi_die "could not resolve asset name"

TMP_DIR="$(mktemp -d)"
DMG="${TMP_DIR}/${ASSET_NAME}"
CASK="${TMP_DIR}/mochi-unstable.rb"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

mochi_download "${ASSET_URL}" "${DMG}"
SHA256="$(mochi_sha256 "${DMG}")"

cat >"${CASK}" <<RUBY
cask "mochi-unstable" do
  version "${TAG}"
  sha256 "${SHA256}"

  url "${ASSET_URL}"
  name "Mochi (Unstable)"
  desc "Cross-platform desktop companion for AI coding tool usage (unstable channel)"
  homepage "https://github.com/${MOCHI_GITHUB_REPO}"

  app "Mochi.app"
end
RUBY

echo "Installing via Homebrew cask (Mochi ${TAG}, ${ARCH})"
brew install --cask "${CASK}" --force

echo "Installed Mochi unstable ${TAG} with Homebrew"
