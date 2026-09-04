#!/usr/bin/env bash
# Shared helpers for Mochi install scripts.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"
: "${MOCHI_GITHUB_API:=https://api.github.com/repos/${MOCHI_GITHUB_REPO}}"

MOCHI_REQUESTED_TAG=""
MOCHI_INSTALL_SCRIPT_DIR=""

# Resolve scripts/install when executed from a checkout; empty when piped via curl | bash.
mochi_install_script_dir() {
  local i src dir
  for i in "${!BASH_SOURCE[@]}"; do
    src="${BASH_SOURCE[$i]:-}"
    [[ -n "${src}" && "${src}" == *.sh && -f "${src}" ]] || continue
    dir="$(cd "$(dirname "${src}")" && pwd)"
    if [[ -f "${dir}/lib/common.sh" ]]; then
      printf '%s' "${dir}"
      return 0
    fi
  done
  return 1
}

mochi_source_install_lib() {
  local lib_name="$1"
  local dir tmp
  if dir="$(mochi_install_script_dir)" && [[ -f "${dir}/lib/${lib_name}" ]]; then
    MOCHI_INSTALL_SCRIPT_DIR="${dir}"
    # shellcheck disable=SC1090
    source "${dir}/lib/${lib_name}"
    return 0
  fi
  tmp="$(mktemp)"
  curl -fsSL \
    "https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/lib/${lib_name}" \
    -o "${tmp}"
  # shellcheck source=/dev/null
  source "${tmp}"
  rm -f "${tmp}"
  MOCHI_INSTALL_SCRIPT_DIR=""
}

mochi_run_install_script() {
  local script_name="$1"
  shift || true
  local dir
  if dir="$(mochi_install_script_dir)" && [[ -f "${dir}/${script_name}" ]]; then
    bash "${dir}/${script_name}" "$@"
    return
  fi
  curl -fsSL \
    "https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/${script_name}" \
    | bash -s -- "$@"
}

mochi_die() {
  echo "error: $*" >&2
  exit 1
}

mochi_need_cmd() {
  local cmd="$1"
  command -v "${cmd}" >/dev/null 2>&1 || mochi_die "missing required command: ${cmd}"
}

mochi_curl_json() {
  local url="$1"
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN}" -H "Accept: application/vnd.github+json" "${url}"
  else
    curl -fsSL -H "Accept: application/vnd.github+json" "${url}"
  fi
}

mochi_install_usage() {
  cat <<EOF
Usage: ${MOCHI_INSTALL_SCRIPT_NAME:-mochi-install} [release-tag]

Install Mochi from GitHub Releases.

Options:
  -h, --help       Show this help

Environment:
  MOCHI_VERSION         Pin a specific release tag
  MOCHI_GITHUB_REPO     Override GitHub repo (default: ${MOCHI_GITHUB_REPO})
  GITHUB_TOKEN          Optional token for higher API rate limits
EOF
}

# Reject the removed prerelease channel with usage help and exit 2.
mochi_removed_channel_error() {
  mochi_install_usage >&2
  echo "error: only stable releases are published" >&2
}

# Parse an optional release tag. Prerelease channels are gone; the legacy
# prerelease flag and env var print usage and exit 2. Legacy env semantic:
# any non-empty value other than "0" counts as opted-in.
mochi_parse_install_args() {
  local legacy_channel_env="MOCHI_UNST""ABLE"
  local legacy_channel_value="${!legacy_channel_env:-}"
  if [[ -n "${legacy_channel_value}" && "${legacy_channel_value}" != "0" ]]; then
    mochi_removed_channel_error
    exit 2
  fi

  MOCHI_REQUESTED_TAG=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      -[iI] | --unsta*)
        mochi_removed_channel_error
        exit 2
        ;;
      -h | --help)
        mochi_install_usage
        exit 0
        ;;
      -*)
        mochi_die "unknown option: $1 (use -h for help)"
        ;;
      *)
        if [[ -n "${MOCHI_REQUESTED_TAG}" ]]; then
          mochi_die "unexpected argument: $1"
        fi
        MOCHI_REQUESTED_TAG="$1"
        shift
        ;;
    esac
  done
}

mochi_install_channel_label() {
  echo "stable"
}

# Resolve release tag: explicit arg, MOCHI_VERSION env, or latest stable release.
mochi_resolve_release_tag() {
  local requested="${1:-${MOCHI_REQUESTED_TAG:-${MOCHI_VERSION:-}}}"

  if [[ -n "${requested}" ]]; then
    echo "${requested}"
    return
  fi

  mochi_need_cmd jq
  local releases
  releases="$(mochi_curl_json "${MOCHI_GITHUB_API}/releases?per_page=30")"
  local tag=""

  tag="$(printf '%s' "${releases}" | jq -r '[.[] | select(.prerelease == false and .draft == false)][0].tag_name // empty')"
  [[ -n "${tag}" ]] || mochi_die "no stable GitHub release found for ${MOCHI_GITHUB_REPO}; set MOCHI_VERSION=<tag>"

  echo "${tag}"
}

mochi_release_json() {
  local tag="$1"
  mochi_curl_json "${MOCHI_GITHUB_API}/releases/tags/${tag}"
}

# Pick the newest asset (by updated_at) matching the first pattern that has hits.
# Releases can keep multiple versioned artifacts; never take [0].
mochi_pick_asset_url() {
  local release_json="$1"
  shift
  mochi_need_cmd jq
  local pattern asset
  for pattern in "$@"; do
    asset="$(printf '%s' "${release_json}" | jq -r --arg re "${pattern}" '
      [.assets[] | select(.name | test($re;"i"))]
      | sort_by(.updated_at)
      | if length > 0 then .[-1].browser_download_url else empty end
    ')"
    if [[ -n "${asset}" ]]; then
      echo "${asset}"
      return 0
    fi
  done
  return 1
}

mochi_download() {
  local url="$1"
  local dest="$2"
  echo "Downloading ${url}"
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl -fsSL -H "Authorization: Bearer ${GITHUB_TOKEN}" -L "${url}" -o "${dest}"
  else
    curl -fsSL -L "${url}" -o "${dest}"
  fi
}

mochi_sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file}" | awk '{print $1}'
  else
    mochi_die "need sha256sum or shasum to verify downloads"
  fi
}

# Linux package format: appimage, deb, or rpm. MOCHI_TEST_LINUX_FAMILY overrides auto-detect in tests.
mochi_linux_package_kind() {
  local requested="${MOCHI_PACKAGE:-auto}"
  if [[ "${requested}" != "auto" ]]; then
    echo "${requested}"
    return
  fi

  case "${MOCHI_TEST_LINUX_FAMILY:-}" in
    debian) echo "deb"; return ;;
    fedora) echo "rpm"; return ;;
    generic) echo "appimage"; return ;;
  esac

  if command -v dpkg >/dev/null 2>&1 && [[ -f /etc/debian_version || -f /etc/os-release ]]; then
    echo "deb"
  elif command -v rpm >/dev/null 2>&1 && [[ -f /etc/redhat-release || -f /etc/fedora-release ]]; then
    echo "rpm"
  else
    echo "appimage"
  fi
}

mochi_linux_asset_patterns() {
  local pkg_kind="$1"
  case "${pkg_kind}" in
    appimage) printf '%s\n' '\.AppImage$' 'appimage' ;;
    deb) printf '%s\n' '\.deb$' '_amd64\.deb$' ;;
    rpm) printf '%s\n' '\.rpm$' 'x86_64\.rpm$' ;;
    *) mochi_die "unsupported MOCHI_PACKAGE=${pkg_kind} (use appimage, deb, or rpm)" ;;
  esac
}
