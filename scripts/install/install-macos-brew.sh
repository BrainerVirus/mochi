#!/usr/bin/env bash
# Install Mochi via Homebrew cask from the GitHub tap (macOS).
# Usage: install-macos-brew.sh [-i|--unstable]
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"
: "${MOCHI_STABLE_CASK:=mochi-desktop}"
: "${MOCHI_UNSTABLE_CASK:=mochi-unstable}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/homebrew-tap.sh
source "${SCRIPT_DIR}/lib/homebrew-tap.sh"

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
unset _i _src

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

if [[ "${MOCHI_INSTALL_UNSTABLE}" == "1" ]]; then
  CHANNEL="unstable"
else
  CHANNEL="stable"
fi

CASK_REF="$(mochi_homebrew_install_cask_ref "${CHANNEL}")"

bash "${SCRIPT_DIR}/setup-macos-brew-tap.sh"

echo "Installing Homebrew cask ${CASK_REF} (${CHANNEL})"
brew install --cask "${CASK_REF}" --force

cat <<EOF
Installed Mochi (${CHANNEL}) with Homebrew.

Upgrade later:
  curl -fsSL https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/setup-macos-brew-tap.sh | bash
  brew upgrade --cask ${CASK_REF}
EOF
