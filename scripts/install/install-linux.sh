#!/usr/bin/env bash
# Install Mochi from GitHub Releases (Linux).
# Usage: install-linux.sh [-i|--unstable] [release-tag]
# Env: MOCHI_VERSION, MOCHI_UNSTABLE=1, MOCHI_PACKAGE (appimage|deb|rpm; default auto), GITHUB_TOKEN
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

MOCHI_INSTALL_SCRIPT_NAME="install-linux.sh"
mochi_parse_install_args "$@"

CHANNEL="$(mochi_install_channel_label)"
TAG="$(mochi_resolve_release_tag)"
echo "Installing Mochi (${CHANNEL} channel, release ${TAG})"

RELEASE_JSON="$(mochi_release_json "${TAG}")"

detect_package_kind() {
  local requested="${MOCHI_PACKAGE:-auto}"
  if [[ "${requested}" != "auto" ]]; then
    echo "${requested}"
    return
  fi
  if command -v dpkg >/dev/null 2>&1 && [[ -f /etc/debian_version || -f /etc/os-release ]]; then
    echo "deb"
  elif command -v rpm >/dev/null 2>&1 && [[ -f /etc/redhat-release || -f /etc/fedora-release ]]; then
    echo "rpm"
  else
    echo "appimage"
  fi
}

PKG_KIND="$(detect_package_kind)"
case "${PKG_KIND}" in
  appimage) ASSET_PATTERNS=('\.AppImage$' 'appimage') ;;
  deb) ASSET_PATTERNS=('\.deb$' '_amd64\.deb$') ;;
  rpm) ASSET_PATTERNS=('\.rpm$' 'x86_64\.rpm$') ;;
  *) mochi_die "unsupported MOCHI_PACKAGE=${PKG_KIND} (use appimage, deb, or rpm)" ;;
esac

ASSET_URL="$(mochi_pick_asset_url "${RELEASE_JSON}" "${ASSET_PATTERNS[@]}")" \
  || mochi_die "no Linux ${PKG_KIND} asset in release ${TAG}"

ASSET_NAME="$(echo "${RELEASE_JSON}" | jq -r --arg url "${ASSET_URL}" '
  [.assets[] | select(.browser_download_url == $url)][0].name
')"
[[ -n "${ASSET_NAME}" ]] || mochi_die "could not resolve asset name"

TMP_DIR="$(mktemp -d)"
FILE="${TMP_DIR}/${ASSET_NAME}"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

mochi_download "${ASSET_URL}" "${FILE}"

install_appimage() {
  local dest_dir="${MOCHI_INSTALL_DIR:-${HOME}/.local/bin}"
  mkdir -p "${dest_dir}"
  local dest="${dest_dir}/mochi"
  install -m 755 "${FILE}" "${dest}"
  echo "Installed AppImage to ${dest}"
  if [[ ":${PATH}:" != *":${dest_dir}:"* ]]; then
    echo "Add ${dest_dir} to your PATH to run 'mochi'"
  fi
}

install_deb() {
  mochi_need_cmd sudo
  sudo dpkg -i "${FILE}" || sudo apt-get install -f -y
  echo "Installed .deb package for Mochi ${TAG}"
}

install_rpm() {
  mochi_need_cmd sudo
  if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y "${FILE}"
  elif command -v zypper >/dev/null 2>&1; then
    sudo zypper install -y "${FILE}"
  else
    sudo rpm -Uvh "${FILE}"
  fi
  echo "Installed .rpm package for Mochi ${TAG}"
}

case "${PKG_KIND}" in
  appimage) install_appimage ;;
  deb) install_deb ;;
  rpm) install_rpm ;;
esac

echo "Mochi ${TAG} (${CHANNEL}, ${PKG_KIND}) install complete"
