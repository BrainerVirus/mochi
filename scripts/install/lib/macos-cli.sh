#!/usr/bin/env bash
# macOS install helpers shared by install-macos.sh and install-macos-brew.sh
set -euo pipefail

if ! declare -F mochi_die >/dev/null 2>&1; then
  mochi_die() {
    echo "error: $*" >&2
    exit 1
  }
fi

mochi_clear_macos_app_quarantine() {
  local app_path="${1:-/Applications/Mochi.app}"
  [[ -d "${app_path}" ]] || return 0
  if xattr -p com.apple.quarantine "${app_path}" >/dev/null 2>&1; then
    xattr -dr com.apple.quarantine "${app_path}"
    echo "Removed macOS quarantine from ${app_path}"
  fi
}

mochi_macos_path_contains_dir() {
  local dir="$1"
  [[ ":${PATH}:" == *":${dir}:"* ]]
}

mochi_macos_try_cli_link() {
  local target="$1"
  local link_path="$2"
  local link_dir
  link_dir="$(dirname "${link_path}")"

  mkdir -p "${link_dir}" 2>/dev/null && ln -sf "${target}" "${link_path}" 2>/dev/null
}

mochi_macos_sudo_cmd() {
  printf '%s' "${MOCHI_CLI_SUDO:-sudo}"
}

mochi_macos_sudo_cli_link() {
  local target="$1"
  local link_path="$2"
  local link_dir sudo_cmd
  link_dir="$(dirname "${link_path}")"
  sudo_cmd="$(mochi_macos_sudo_cmd)"

  command -v "${sudo_cmd%% *}" >/dev/null 2>&1 || return 1
  "${sudo_cmd}" mkdir -p "${link_dir}" && "${sudo_cmd}" ln -sf "${target}" "${link_path}"
}

mochi_macos_ensure_path_entry() {
  local dir="$1"
  local marker="# mochi-install-path"
  local export_line="export PATH=\"${dir}:\$PATH\""
  local rc

  mochi_macos_path_contains_dir "${dir}" && return 0

  for rc in "${HOME}/.zprofile" "${HOME}/.zshrc"; do
    if [[ -f "${rc}" ]] && grep -Fq "${dir}" "${rc}" 2>/dev/null; then
      echo "PATH already includes ${dir} (${rc})"
      return 0
    fi
  done

  rc="${HOME}/.zprofile"
  {
    echo ""
    echo "${marker}"
    echo "${export_line}"
  } >> "${rc}"
  echo "Configured PATH in ${rc}"
}

mochi_install_cli_link() {
  local app_path="$1"
  local target="${app_path}/Contents/MacOS/mochi"
  local system_link="${MOCHI_CLI_USR_LOCAL:-/usr/local/bin/mochi}"
  local user_link="${HOME}/.local/bin/mochi"
  local link_path="${MOCHI_CLI_LINK:-}"

  if [[ ! -x "${target}" ]]; then
    echo "Skipping CLI link: ${target} is not executable"
    return 0
  fi

  if [[ -n "${link_path}" ]]; then
    if mochi_macos_try_cli_link "${target}" "${link_path}"; then
      echo "Installed CLI command to ${link_path}"
      return 0
    fi
    if mochi_macos_sudo_cli_link "${target}" "${link_path}"; then
      echo "Installed CLI command to ${link_path}"
      return 0
    fi
    mochi_die "could not install CLI to ${link_path}"
  fi

  if mochi_macos_try_cli_link "${target}" "${system_link}"; then
    echo "Installed CLI command to ${system_link}"
    return 0
  fi

  echo "Installing CLI to ${system_link} (administrator password required)"
  if mochi_macos_sudo_cli_link "${target}" "${system_link}"; then
    echo "Installed CLI command to ${system_link}"
    return 0
  fi

  if mochi_macos_try_cli_link "${target}" "${user_link}"; then
    echo "Installed CLI command to ${user_link}"
    mochi_macos_ensure_path_entry "$(dirname "${user_link}")"
    return 0
  fi

  mochi_die "could not install CLI (tried ${system_link} and ${user_link})"
}
