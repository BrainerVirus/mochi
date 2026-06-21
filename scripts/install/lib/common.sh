#!/usr/bin/env bash
# Shared helpers for Mochi install scripts.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_GITHUB_API:=https://api.github.com/repos/${MOCHI_GITHUB_REPO}}"

MOCHI_INSTALL_UNSTABLE="${MOCHI_INSTALL_UNSTABLE:-0}"
MOCHI_REQUESTED_TAG=""

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
Usage: ${MOCHI_INSTALL_SCRIPT_NAME:-mochi-install} [-i|--unstable] [release-tag]

Install Mochi from GitHub Releases.

Options:
  -i, --unstable   Install the latest timestamped unstable prerelease
  -h, --help       Show this help

Environment:
  MOCHI_VERSION         Pin a specific release tag
  MOCHI_UNSTABLE=1      Same as -i / --unstable
  MOCHI_GITHUB_REPO     Override GitHub repo (default: ${MOCHI_GITHUB_REPO})
  GITHUB_TOKEN          Optional token for higher API rate limits
EOF
}

# Parse -i / --unstable and optional release tag. Sets MOCHI_INSTALL_UNSTABLE and MOCHI_REQUESTED_TAG.
mochi_parse_install_args() {
  MOCHI_INSTALL_UNSTABLE="${MOCHI_UNSTABLE:-0}"
  MOCHI_REQUESTED_TAG=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      -i | --unstable)
        MOCHI_INSTALL_UNSTABLE=1
        shift
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
  if [[ "${MOCHI_INSTALL_UNSTABLE}" == "1" ]]; then
    echo "unstable"
  else
    echo "stable"
  fi
}

# Resolve release tag: explicit arg, MOCHI_VERSION env, or latest stable/unstable release.
mochi_resolve_release_tag() {
  local requested="${1:-${MOCHI_REQUESTED_TAG:-${MOCHI_VERSION:-}}}"
  local unstable="${MOCHI_INSTALL_UNSTABLE:-0}"

  if [[ -n "${requested}" ]]; then
    echo "${requested}"
    return
  fi

  mochi_need_cmd jq
  local releases
  releases="$(mochi_curl_json "${MOCHI_GITHUB_API}/releases?per_page=30")"
  local tag=""

  if [[ "${unstable}" == "1" ]]; then
    tag="$(printf '%s' "${releases}" | jq -r '
      [.[] | select(
        .draft == false
        and .prerelease == true
        and (.tag_name | test("^unstable-[0-9]{8}\\.[0-9]{6}$"))
      )]
      | sort_by(.published_at // .created_at // "")
      | if length > 0 then .[-1].tag_name else empty end
    ')"
    if [[ -z "${tag}" ]]; then
      tag="$(printf '%s' "${releases}" | jq -r '[.[] | select(.tag_name == "unstable" and .draft == false)][0].tag_name // empty')"
    fi
    [[ -n "${tag}" ]] || mochi_die "no unstable GitHub release found for ${MOCHI_GITHUB_REPO}; try MOCHI_VERSION=unstable"
  else
    tag="$(printf '%s' "${releases}" | jq -r '[.[] | select(.prerelease == false and .draft == false)][0].tag_name // empty')"
    [[ -n "${tag}" ]] || mochi_die "no stable GitHub release found for ${MOCHI_GITHUB_REPO}; try -i for unstable or MOCHI_VERSION=<tag>"
  fi

  echo "${tag}"
}

mochi_release_json() {
  local tag="$1"
  mochi_curl_json "${MOCHI_GITHUB_API}/releases/tags/${tag}"
}

# Pick the newest asset (by updated_at) matching the first pattern that has hits.
# Rolling releases like `unstable` keep multiple versioned artifacts; never take [0].
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
