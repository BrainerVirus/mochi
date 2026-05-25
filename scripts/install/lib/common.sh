#!/usr/bin/env bash
# Shared helpers for Mochi install scripts.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_GITHUB_API:=https://api.github.com/repos/${MOCHI_GITHUB_REPO}}"

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

# Resolve release tag: explicit arg, MOCHI_VERSION env, or latest prerelease.
mochi_resolve_release_tag() {
  local requested="${1:-${MOCHI_VERSION:-}}"

  if [[ -n "${requested}" ]]; then
    echo "${requested}"
    return
  fi

  mochi_need_cmd jq
  local releases
  releases="$(mochi_curl_json "${MOCHI_GITHUB_API}/releases?per_page=30")"
  local tag
  tag="$(echo "${releases}" | jq -r '[.[] | select(.prerelease == true and .draft == false)][0].tag_name // empty')"
  if [[ -z "${tag}" ]]; then
    tag="$(echo "${releases}" | jq -r '[.[] | select(.draft == false)][0].tag_name // empty')"
  fi
  [[ -n "${tag}" ]] || mochi_die "no GitHub release found for ${MOCHI_GITHUB_REPO}; try MOCHI_VERSION=<tag>"
  echo "${tag}"
}

mochi_release_json() {
  local tag="$1"
  mochi_curl_json "${MOCHI_GITHUB_API}/releases/tags/${tag}"
}

# Pick first asset whose name matches any of the given extended-regex patterns.
mochi_pick_asset_url() {
  local release_json="$1"
  shift
  local pattern asset
  for pattern in "$@"; do
    asset="$(echo "${release_json}" | jq -r --arg re "${pattern}" '
      [.assets[] | select(.name | test($re;"i"))][0].browser_download_url // empty
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
