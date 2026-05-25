#!/usr/bin/env bash
# Install Mochi from GitHub Releases (macOS).
# Usage: install-macos.sh [-i|--unstable] [release-tag]
# Env: MOCHI_VERSION, MOCHI_UNSTABLE=1, MOCHI_INSTALL_DIR (default /Applications), GITHUB_TOKEN
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

MOCHI_INSTALL_SCRIPT_NAME="install-macos.sh"
mochi_parse_install_args "$@"

CHANNEL="$(mochi_install_channel_label)"
TAG="$(mochi_resolve_release_tag)"
echo "Installing Mochi (${CHANNEL} channel, release ${TAG})"

RELEASE_JSON="$(mochi_release_json "${TAG}")"

ARCH="$(uname -m)"
case "${ARCH}" in
  arm64) ASSET_PATTERNS=('aarch64.*\.dmg$' 'arm64.*\.dmg$') ;;
  x86_64) ASSET_PATTERNS=('x64.*\.dmg$' 'x86_64.*\.dmg$') ;;
  *) mochi_die "unsupported macOS architecture: ${ARCH}" ;;
esac

ASSET_URL="$(mochi_pick_asset_url "${RELEASE_JSON}" "${ASSET_PATTERNS[@]}")" \
  || mochi_die "no macOS .dmg asset for ${ARCH} in release ${TAG}"

INSTALL_DIR="${MOCHI_INSTALL_DIR:-/Applications}"
TMP_DIR="$(mktemp -d)"
DMG="${TMP_DIR}/mochi.dmg"
MOUNT="${TMP_DIR}/mount"

cleanup() {
  if mount | grep -q "${MOUNT}"; then
    hdiutil detach "${MOUNT}" -quiet || hdiutil detach "${MOUNT}" -force -quiet || true
  fi
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

mochi_download "${ASSET_URL}" "${DMG}"
mkdir -p "${MOUNT}"
hdiutil attach "${DMG}" -nobrowse -readonly -mountpoint "${MOUNT}" >/dev/null

APP_SRC="$(find "${MOUNT}" -maxdepth 1 -name '*.app' -print -quit)"
[[ -n "${APP_SRC}" ]] || mochi_die "no .app bundle found inside DMG"

APP_NAME="$(basename "${APP_SRC}")"
DEST="${INSTALL_DIR}/${APP_NAME}"

if [[ -d "${DEST}" ]]; then
  echo "Removing existing ${DEST}"
  rm -rf "${DEST}"
fi

mkdir -p "${INSTALL_DIR}"
echo "Installing ${APP_NAME} to ${INSTALL_DIR}"
ditto "${APP_SRC}" "${DEST}"

echo "Installed Mochi ${TAG} (${CHANNEL}, ${ARCH}) to ${DEST}"
