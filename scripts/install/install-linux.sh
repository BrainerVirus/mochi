#!/usr/bin/env bash
# Install Mochi from GitHub Releases (Linux).
# Usage: install-linux.sh [-i|--unstable] [release-tag]
# Env: MOCHI_VERSION, MOCHI_UNSTABLE=1, MOCHI_PACKAGE (appimage|deb|rpm; default auto), GITHUB_TOKEN
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"

# raw.githubusercontent.com can serve a stale branch HEAD; pin to the current commit SHA.
if [[ "${MOCHI_INSTALL_REF}" == "main" || "${MOCHI_INSTALL_REF}" == "master" ]]; then
  _sha="$(
    curl -fsSL -H "Accept: application/vnd.github+json" \
      "https://api.github.com/repos/${MOCHI_GITHUB_REPO}/commits/${MOCHI_INSTALL_REF}" \
      | grep -oE '"sha": "[a-f0-9]{40}"' | head -1 | grep -oE '[a-f0-9]{40}' || true
  )"
  if [[ -n "${_sha}" ]]; then
    MOCHI_INSTALL_REF="${_sha}"
  fi
fi
unset _sha

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

if [[ "${MOCHI_SKIP_DEPS:-0}" != "1" ]]; then
  for _i in "${!BASH_SOURCE[@]}"; do
    _src="${BASH_SOURCE[_i]}"
    if [[ "${_src}" == *.sh ]] && [[ -f "${_src}" ]]; then
      _install_dir="$(cd "$(dirname "${_src}")" && pwd)"
      if [[ -f "${_install_dir}/lib/linux-deps.sh" ]]; then
        # shellcheck source=lib/linux-deps.sh
        source "${_install_dir}/lib/linux-deps.sh"
        mochi_ensure_linux_runtime_deps
        break
      fi
    fi
  done
  unset _i _src _install_dir
fi

mochi_parse_install_args "$@"

CHANNEL="$(mochi_install_channel_label)"
TAG="$(mochi_resolve_release_tag)"
echo "Installing Mochi (${CHANNEL} channel, release ${TAG})"

RELEASE_JSON="$(mochi_release_json "${TAG}")"

PKG_KIND="$(mochi_linux_package_kind)"
mapfile -t ASSET_PATTERNS < <(mochi_linux_asset_patterns "${PKG_KIND}")

ASSET_URL="$(mochi_pick_asset_url "${RELEASE_JSON}" "${ASSET_PATTERNS[@]}")" \
  || mochi_die "no Linux ${PKG_KIND} asset in release ${TAG}"

ASSET_NAME="$(printf '%s' "${RELEASE_JSON}" | jq -r --arg url "${ASSET_URL}" '
  [.assets[] | select(.browser_download_url == $url)][0].name
')"
[[ -n "${ASSET_NAME}" ]] || mochi_die "could not resolve asset name"

TMP_DIR="$(mktemp -d)"
FILE="${TMP_DIR}/${ASSET_NAME}"
cleanup() { rm -rf "${TMP_DIR}"; }
trap cleanup EXIT

mochi_download "${ASSET_URL}" "${FILE}"

install_appimage() {
  case "${ASSET_NAME}" in
    *.AppImage | *.appimage) ;;
    *) mochi_die "expected an AppImage asset, got ${ASSET_NAME}" ;;
  esac
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
  case "${ASSET_NAME}" in
    *.deb) ;;
    *) mochi_die "expected a .deb asset, got ${ASSET_NAME}" ;;
  esac
  if ! sudo dpkg -i "${FILE}"; then
    sudo apt-get install -f -y
  fi
  if ! dpkg-query -W -f='${Status}' mochi 2>/dev/null | grep -q 'install ok installed'; then
    mochi_die "dpkg did not install mochi (wrong asset or broken package: ${ASSET_NAME})"
  fi
  echo "Installed .deb package for Mochi ${TAG}"
}

install_rpm() {
  mochi_need_cmd sudo
  case "${ASSET_NAME}" in
    *.rpm) ;;
    *) mochi_die "expected a .rpm asset, got ${ASSET_NAME}" ;;
  esac
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
