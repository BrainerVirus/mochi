#!/usr/bin/env bash
# Ensure Linux runtime dependencies for Mochi (tray indicator, libsecret, SVG).
# Sourced from install-linux.sh. Idempotent; honors MOCHI_SKIP_DEPS=1.
# Set MOCHI_GNOME_TRAY=0 to skip the optional GNOME AppIndicator extension step.
set -euo pipefail

mochi_deps_log() {
  if [[ "${MOCHI_QUIET_DEPS:-0}" != "1" ]]; then
    echo "$*"
  fi
}

mochi_deps_warn() {
  echo "warning: $*" >&2
}

mochi_dpkg_installed() {
  local pkg="$1"
  dpkg-query -W -f='${Status}' "${pkg}" 2>/dev/null | grep -q 'install ok installed'
}

mochi_rpm_installed() {
  local pkg="$1"
  rpm -q "${pkg}" >/dev/null 2>&1
}

mochi_pacman_installed() {
  local pkg="$1"
  pacman -Q "${pkg}" >/dev/null 2>&1
}

mochi_ensure_debian_deps() {
  mochi_need_cmd sudo
  if [[ "${MOCHI_SKIP_APT_UPDATE:-0}" != "1" ]]; then
    mochi_deps_log "Updating package lists (apt)..."
    if [[ "${MOCHI_DEPS_DRY_RUN:-0}" == "1" ]]; then
      mochi_deps_log "[dry-run] sudo apt-get update"
    else
      sudo apt-get update -qq
    fi
  fi

  local tray_pkgs=(libayatana-appindicator3-1)
  if ! apt-cache show libayatana-appindicator3-1 >/dev/null 2>&1; then
    tray_pkgs=(libappindicator3-1)
  fi

  local -a required=(libsecret-1-0 librsvg2-2 "${tray_pkgs[@]}")
  local -a missing=()

  for pkg in "${required[@]}"; do
    if mochi_dpkg_installed "${pkg}"; then
      mochi_deps_log "ok (already installed): ${pkg}"
    else
      missing+=("${pkg}")
    fi
  done

  if [[ "${#missing[@]}" -gt 0 ]]; then
    mochi_deps_log "Installing: ${missing[*]}"
    if [[ "${MOCHI_DEPS_DRY_RUN:-0}" == "1" ]]; then
      mochi_deps_log "[dry-run] sudo apt-get install -y --no-install-recommends ${missing[*]}"
    else
      sudo apt-get install -y --no-install-recommends "${missing[@]}"
    fi
  fi

  mochi_ensure_gnome_tray_extension
}

mochi_is_gnome_desktop() {
  local desktop="${XDG_CURRENT_DESKTOP:-}"
  [[ "${desktop}" == *"GNOME"* || "${desktop}" == *"gnome"* ]]
}

mochi_ensure_gnome_tray_extension() {
  if [[ "${MOCHI_GNOME_TRAY:-1}" == "0" ]]; then
    mochi_deps_log "Skipping GNOME tray extension setup (MOCHI_GNOME_TRAY=0)"
    return 0
  fi

  if ! mochi_is_gnome_desktop; then
    return 0
  fi

  local ext_pkg=""
  if apt-cache show gnome-shell-extension-appindicator >/dev/null 2>&1; then
    ext_pkg="gnome-shell-extension-appindicator"
  elif apt-cache show gnome-shell-extension-ubuntu-appindicators >/dev/null 2>&1; then
    ext_pkg="gnome-shell-extension-ubuntu-appindicators"
  fi

  if [[ -z "${ext_pkg}" ]]; then
    mochi_deps_warn "GNOME detected but no AppIndicator package found; see docs/linux.md"
    return 0
  fi

  if mochi_dpkg_installed "${ext_pkg}"; then
    mochi_deps_log "ok (already installed): ${ext_pkg}"
  elif [[ "${MOCHI_DEPS_DRY_RUN:-0}" == "1" ]]; then
    mochi_deps_log "[dry-run] would install ${ext_pkg}"
  else
    sudo apt-get install -y --no-install-recommends "${ext_pkg}" || \
      mochi_deps_warn "could not install ${ext_pkg}; install a GNOME AppIndicator extension manually"
  fi

  if command -v gnome-extensions >/dev/null 2>&1; then
    local uuid=""
    if [[ "${ext_pkg}" == *"ubuntu-appindicators"* ]]; then
      uuid="ubuntu-appindicators@ubuntu.com"
    else
      uuid="appindicatorsupport@rgcjonas.gmail.com"
    fi
    if [[ "${MOCHI_DEPS_DRY_RUN:-0}" != "1" ]]; then
      gnome-extensions enable "${uuid}" 2>/dev/null || \
        mochi_deps_warn "enable extension ${uuid} in GNOME Extensions, then log out and back in"
    fi
  else
    mochi_deps_warn "log out and back in after installing ${ext_pkg} so the tray icon can appear"
  fi
}

mochi_ensure_fedora_deps() {
  mochi_need_cmd sudo
  local -a pkgs=(libayatana-appindicator3 libsecret librsvg2)
  local -a install=()
  for pkg in "${pkgs[@]}"; do
    if mochi_rpm_installed "${pkg}"; then
      mochi_deps_log "ok (already installed): ${pkg}"
    else
      install+=("${pkg}")
    fi
  done
  if [[ "${#install[@]}" -eq 0 ]]; then
    return 0
  fi
  mochi_deps_log "Installing: ${install[*]}"
  if [[ "${MOCHI_DEPS_DRY_RUN:-0}" == "1" ]]; then
    mochi_deps_log "[dry-run] sudo dnf install -y ${install[*]}"
    return 0
  fi
  if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y "${install[@]}"
  elif command -v zypper >/dev/null 2>&1; then
    sudo zypper install -y "${install[@]}"
  else
    mochi_deps_warn "no dnf/zypper found; install ${install[*]} manually"
  fi
}

mochi_ensure_arch_deps() {
  mochi_need_cmd sudo
  local -a pkgs=(ayatana-appindicator libsecret librsvg)
  local -a install=()
  for pkg in "${pkgs[@]}"; do
    if mochi_pacman_installed "${pkg}"; then
      mochi_deps_log "ok (already installed): ${pkg}"
    else
      install+=("${pkg}")
    fi
  done
  if [[ "${#install[@]}" -eq 0 ]]; then
    return 0
  fi
  mochi_deps_log "Installing: ${install[*]}"
  if [[ "${MOCHI_DEPS_DRY_RUN:-0}" == "1" ]]; then
    mochi_deps_log "[dry-run] sudo pacman -S --needed --noconfirm ${install[*]}"
  else
    sudo pacman -S --needed --noconfirm "${install[@]}"
  fi
}

mochi_detect_linux_family() {
  if [[ ! -f /etc/os-release ]]; then
    echo "unknown"
    return
  fi
  # shellcheck disable=SC1091
  source /etc/os-release
  local id_like="${ID_LIKE:-}"
  local id="${ID:-}"
  case "${id} ${id_like}" in
    *debian* | *ubuntu*) echo "debian" ;;
    *fedora* | *rhel* | *centos*) echo "fedora" ;;
    *arch*) echo "arch" ;;
    *) echo "unknown" ;;
  esac
}

mochi_ensure_linux_runtime_deps() {
  if [[ "${MOCHI_SKIP_DEPS:-0}" == "1" ]]; then
    mochi_deps_log "Skipping Linux runtime dependencies (MOCHI_SKIP_DEPS=1)"
    return 0
  fi

  mochi_deps_log "Checking Linux runtime dependencies for Mochi..."
  local family
  family="$(mochi_detect_linux_family)"

  case "${family}" in
    debian)
      mochi_need_cmd apt-get
      mochi_ensure_debian_deps
      ;;
    fedora)
      mochi_ensure_fedora_deps
      ;;
    arch)
      mochi_need_cmd pacman
      mochi_ensure_arch_deps
      ;;
    *)
      mochi_deps_warn "unknown distro; install libsecret, Ayatana/AppIndicator, and librsvg manually"
      mochi_deps_warn "see https://github.com/${MOCHI_GITHUB_REPO}/blob/main/docs/linux.md"
      ;;
  esac
}
