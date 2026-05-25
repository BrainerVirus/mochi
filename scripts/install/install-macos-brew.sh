#!/usr/bin/env bash
# Install Mochi via Homebrew cask (macOS).
# Generates a temporary cask from the latest stable or unstable GitHub release.
# Usage: install-macos-brew.sh [-i|--unstable] [release-tag]
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"

_mochi_common_loaded=0
for _i in "${!BASH_SOURCE[@]}"; do
  _src="${BASH_SOURCE[_i]}"
  if [[ "${_src}" == *.sh ]] && [[ -f "${_src}" ]]; then
    _dir="$(cd "$(dirname "${_src}")" && pwd)"
    if [[ -f "${_dir}/lib/common.sh" ]]; then
      # shellcheck source=lib/common.sh
      source "${_dir}/lib/common.sh"
      _mochi_common_loaded=1
      break
    fi
  fi
done
unset _i _src _dir

if [[ "${_mochi_common_loaded}" -eq 0 ]]; then
  _tmp="$(mktemp)"
  curl -fsSL \
    "https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/lib/common.sh" \
    -o "${_tmp}"
  # shellcheck source=/dev/null
  source "${_tmp}"
  rm -f "${_tmp}"
fi
unset _mochi_common_loaded _tmp

MOCHI_INSTALL_SCRIPT_NAME="install-macos-brew.sh"
mochi_parse_install_args "$@"

mochi_need_cmd brew
mochi_need_cmd jq

CHANNEL="$(mochi_install_channel_label)"
TAG="$(mochi_resolve_release_tag)"
echo "Installing Mochi via Homebrew (${CHANNEL} channel, release ${TAG})"

RELEASE_JSON="$(mochi_release_json "${TAG}")"

ARCH="$(uname -m)"
case "${ARCH}" in
  arm64) ASSET_PATTERNS=('aarch64.*\.dmg$' 'arm64.*\.dmg$') ;;
  x86_64) ASSET_PATTERNS=('x64.*\.dmg$' 'x86_64.*\.dmg$') ;;
  *) mochi_die "unsupported macOS architecture: ${ARCH}" ;;
esac

ASSET_URL="$(mochi_pick_asset_url "${RELEASE_JSON}" "${ASSET_PATTERNS[@]}")" \
  || mochi_die "no macOS .dmg asset for ${ARCH} in release ${TAG}"

ASSET_NAME="$(printf '%s' "${RELEASE_JSON}" | jq -r --arg url "${ASSET_URL}" '
  [.assets[] | select(.browser_download_url == $url)][0].name
')"
[[ -n "${ASSET_NAME}" ]] || mochi_die "could not resolve asset name"

if [[ "${CHANNEL}" == "unstable" ]]; then
  CASK_ID="mochi-unstable"
  CASK_NAME="Mochi (Unstable)"
  CASK_DESC="Cross-platform desktop companion for AI coding tool usage (unstable channel)"
else
  CASK_ID="mochi"
  CASK_NAME="Mochi"
  CASK_DESC="Cross-platform desktop companion for AI coding tool usage"
fi

TMP_DIR="$(mktemp -d)"
DMG="${TMP_DIR}/${ASSET_NAME}"
CASK="${TMP_DIR}/${CASK_ID}.rb"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

mochi_download "${ASSET_URL}" "${DMG}"
SHA256="$(mochi_sha256 "${DMG}")"

cat >"${CASK}" <<RUBY
cask "${CASK_ID}" do
  version "${TAG}"
  sha256 "${SHA256}"

  url "${ASSET_URL}"
  name "${CASK_NAME}"
  desc "${CASK_DESC}"
  homepage "https://github.com/${MOCHI_GITHUB_REPO}"

  app "Mochi.app"
end
RUBY

echo "Installing Homebrew cask ${CASK_ID} (Mochi ${TAG}, ${ARCH})"
brew install --cask "${CASK}" --force

echo "Installed Mochi ${TAG} (${CHANNEL}) with Homebrew"
