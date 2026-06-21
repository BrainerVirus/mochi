#!/usr/bin/env bash
# macOS install helpers shared by install-macos.sh and install-macos-brew.sh
set -euo pipefail

mochi_clear_macos_app_quarantine() {
  local app_path="${1:-/Applications/Mochi.app}"
  [[ -d "${app_path}" ]] || return 0
  if xattr -p com.apple.quarantine "${app_path}" >/dev/null 2>&1; then
    xattr -dr com.apple.quarantine "${app_path}"
    echo "Removed macOS quarantine from ${app_path}"
  fi
}

mochi_install_cli_link() {
  local app_path="$1"
  local link_path="${MOCHI_CLI_LINK:-/usr/local/bin/mochi}"
  local target="${app_path}/Contents/MacOS/mochi"
  local link_dir
  link_dir="$(dirname "${link_path}")"

  if [[ ! -x "${target}" ]]; then
    echo "Skipping CLI link: ${target} is not executable"
    return 0
  fi

  if mkdir -p "${link_dir}" 2>/dev/null && ln -sf "${target}" "${link_path}" 2>/dev/null; then
    echo "Installed CLI command to ${link_path}"
    return 0
  fi

  echo "Could not write ${link_path}. To enable the CLI, run:"
  echo "sudo ln -sf ${target} ${link_path}"
}
