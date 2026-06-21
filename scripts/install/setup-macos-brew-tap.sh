#!/usr/bin/env bash
# Ensure the GitHub-backed Homebrew tap for Mochi is installed.
# macOS /bin/bash is 3.2 — avoid bash 4+ syntax such as ${var,,}.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"

TAP_USER="${MOCHI_GITHUB_REPO%%/*}"
TAP="${TAP_USER}/mochi"
URL="https://github.com/${MOCHI_GITHUB_REPO}"

mochi_lower() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

mochi_tap_listed() {
  local needle="$1"
  local line
  while IFS= read -r line; do
    [ "$(mochi_lower "${line}")" = "$(mochi_lower "${needle}")" ] && return 0
  done
  return 1
}

if ! command -v brew >/dev/null 2>&1; then
  echo "error: Homebrew is required (https://brew.sh/)" >&2
  exit 1
fi

# Remove stale local/dev taps that break `brew update` or shadow the GitHub tap.
for stale_tap in test/mochi-tap "${TAP_USER}/mochi-install"; do
  if mochi_tap_listed "${stale_tap}"; then
    echo "Removing stale tap ${stale_tap}"
    brew untap --force "${stale_tap}" 2>/dev/null || brew untap "${stale_tap}" 2>/dev/null || true
  fi
done

if mochi_tap_listed "${TAP}"; then
  tap_repo="$(brew --repo "${TAP}" 2>/dev/null || true)"
  current_url=""
  if [ -n "${tap_repo}" ] && [ -d "${tap_repo}/.git" ]; then
    current_url="$(git -C "${tap_repo}" config --get remote.origin.url 2>/dev/null || true)"
  fi

  if [ "${current_url}" = "${URL}" ]; then
    echo "Tap ${TAP} already points to ${URL}"
    exit 0
  fi

  echo "Replacing tap ${TAP} (${current_url:-unknown} -> ${URL})"
  brew untap --force "${TAP}"
fi

brew tap "${TAP}" "${URL}"
echo "Tapped ${TAP} -> ${URL}"
