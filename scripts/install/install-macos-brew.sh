#!/usr/bin/env bash
# Install Mochi via Homebrew cask from the GitHub tap (macOS).
# Usage: install-macos-brew.sh [-i|--unstable]
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"
: "${MOCHI_STABLE_CASK:=mochi-desktop}"
: "${MOCHI_UNSTABLE_CASK:=mochi-unstable}"

TAP_USER="${MOCHI_GITHUB_REPO%%/*}"
TAP="${TAP_USER}/mochi"

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
  CASK_ID="${MOCHI_UNSTABLE_CASK}"
  CHANNEL="unstable"
else
  CASK_ID="${MOCHI_STABLE_CASK}"
  CHANNEL="stable"
fi

if [[ -n "${_dir:-}" ]] && [[ -f "${_dir}/setup-macos-brew-tap.sh" ]]; then
  bash "${_dir}/setup-macos-brew-tap.sh"
else
  curl -fsSL \
    "https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/setup-macos-brew-tap.sh" \
    | bash
fi

echo "Installing Homebrew cask ${TAP}/${CASK_ID} (${CHANNEL})"
brew install --cask "${TAP}/${CASK_ID}" --force

cat <<EOF
Installed Mochi (${CHANNEL}) with Homebrew.

Upgrade later:
  curl -fsSL https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/setup-macos-brew-tap.sh | bash
  brew upgrade --cask ${TAP}/${CASK_ID}
EOF
