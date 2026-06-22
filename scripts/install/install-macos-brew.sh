#!/usr/bin/env bash
# Install Mochi via Homebrew cask from the GitHub tap (macOS).
# Usage: install-macos-brew.sh [-i|--unstable]
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"
: "${MOCHI_STABLE_CASK:=mochi-desktop}"
: "${MOCHI_UNSTABLE_CASK:=mochi-unstable}"

_mochi_common_loaded=0
for _i in "${!BASH_SOURCE[@]}"; do
  _src="${BASH_SOURCE[_i]:-}"
  if [[ -n "${_src}" && "${_src}" == *.sh && -f "${_src}" ]]; then
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

mochi_source_install_lib "homebrew-tap.sh"
mochi_source_install_lib "macos-cli.sh"

MOCHI_INSTALL_SCRIPT_NAME="install-macos-brew.sh"
mochi_parse_install_args "$@"

mochi_need_cmd brew

if [[ "${MOCHI_INSTALL_UNSTABLE}" == "1" ]]; then
  CHANNEL="unstable"
else
  CHANNEL="stable"
fi

CASK_REF="$(mochi_homebrew_install_cask_ref "${CHANNEL}")"
CASK_TOKEN="$(mochi_homebrew_cask_token "${CASK_REF}")"

mochi_run_install_script "setup-macos-brew-tap.sh"

echo "Installing Homebrew cask ${CASK_REF} (${CHANNEL})"
mochi_brew_install_cask "${CASK_REF}"

MOCHI_APP_PATH="$(mochi_homebrew_cask_app_path "${CASK_TOKEN}" || true)"
if [[ -z "${MOCHI_APP_PATH}" || ! -d "${MOCHI_APP_PATH}" ]]; then
  echo "error: Mochi.app was not found after Homebrew install." >&2
  echo "Try: brew reinstall --cask --force ${CASK_REF}" >&2
  exit 1
fi

mochi_clear_macos_app_quarantine "${MOCHI_APP_PATH}"

cat <<EOF
Installed Mochi (${CHANNEL}) with Homebrew at ${MOCHI_APP_PATH}.

Mochi is ad-hoc signed (no Apple Developer notarization). If macOS says the app
is damaged, clear download quarantine:
  xattr -dr com.apple.quarantine ${MOCHI_APP_PATH}

Upgrade later:
  curl -fsSL https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/setup-macos-brew-tap.sh | bash
  brew upgrade --cask ${CASK_REF}
EOF
