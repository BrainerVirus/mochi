#!/usr/bin/env bash
# Install Mochi via Homebrew cask (macOS).
# Writes a local git-backed tap (Homebrew 6 rejects bare cask files outside taps).
# Usage: install-macos-brew.sh [-i|--unstable] [release-tag]
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"
: "${MOCHI_HOMEBREW_TAP_DIR:=${HOME}/.local/share/mochi/homebrew-tap}"
: "${MOCHI_HOMEBREW_TAP:=BrainerVirus/mochi-install}"

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
mochi_need_cmd git

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

if [[ "${CHANNEL}" == "unstable" ]]; then
  CASK_ID="mochi-unstable"
  CASK_NAME="Mochi (Unstable)"
  CASK_DESC="Cross-platform desktop companion for AI coding tool usage (unstable channel)"
else
  CASK_ID="mochi"
  CASK_NAME="Mochi"
  CASK_DESC="Cross-platform desktop companion for AI coding tool usage"
fi

if [[ "${TAG}" == v* ]]; then
  CASK_VERSION="${TAG#v}"
else
  CASK_VERSION="${TAG}"
fi

TMP_DIR="$(mktemp -d)"
DMG="${TMP_DIR}/$(basename "${ASSET_URL}")"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

mochi_download "${ASSET_URL}" "${DMG}"
SHA256="$(mochi_sha256 "${DMG}")"

mkdir -p "${MOCHI_HOMEBREW_TAP_DIR}/Casks"
if [[ ! -d "${MOCHI_HOMEBREW_TAP_DIR}/.git" ]]; then
  git -C "${MOCHI_HOMEBREW_TAP_DIR}" init -q
  git -C "${MOCHI_HOMEBREW_TAP_DIR}" config user.email "mochi-installer@localhost"
  git -C "${MOCHI_HOMEBREW_TAP_DIR}" config user.name "Mochi Installer"
fi

CASK_FILE="${MOCHI_HOMEBREW_TAP_DIR}/Casks/${CASK_ID}.rb"
cat >"${CASK_FILE}" <<RUBY
cask "${CASK_ID}" do
  version "${CASK_VERSION}"
  sha256 "${SHA256}"

  url "${ASSET_URL}",
      verified: "github.com/${MOCHI_GITHUB_REPO}/"
  name "${CASK_NAME}"
  desc "${CASK_DESC}"
  homepage "https://github.com/${MOCHI_GITHUB_REPO}"

  app "Mochi.app"
end
RUBY

git -C "${MOCHI_HOMEBREW_TAP_DIR}" add "Casks/${CASK_ID}.rb"
if ! git -C "${MOCHI_HOMEBREW_TAP_DIR}" diff --staged --quiet; then
  git -C "${MOCHI_HOMEBREW_TAP_DIR}" commit -qm "Update ${CASK_ID} to ${TAG}"
fi

TAP_URL="file://${MOCHI_HOMEBREW_TAP_DIR}"
echo "Installing Homebrew cask ${MOCHI_HOMEBREW_TAP}/${CASK_ID} (Mochi ${TAG}, ${ARCH})"
brew tap --force "${MOCHI_HOMEBREW_TAP}" "${TAP_URL}"
brew install --cask "${MOCHI_HOMEBREW_TAP}/${CASK_ID}" --force

cat <<EOF
Installed Mochi ${TAG} (${CHANNEL}) with Homebrew.

Upgrade later:
  brew upgrade --cask ${CASK_ID}
  # or re-run this installer to refresh the local tap cask

Install or upgrade from the GitHub tap instead:
  brew tap ${MOCHI_GITHUB_REPO%%/*}/mochi https://github.com/${MOCHI_GITHUB_REPO}
  brew install --cask ${MOCHI_GITHUB_REPO%%/*}/mochi/${CASK_ID}
  brew upgrade --cask ${CASK_ID}
EOF
