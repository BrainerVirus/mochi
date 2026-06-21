#!/usr/bin/env bash
# Ensure the GitHub-backed Homebrew tap for Mochi is installed.
# Fixes "remote mismatch" when an old file:// tap is still registered.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"

TAP_USER="${MOCHI_GITHUB_REPO%%/*}"
TAP="${TAP_USER}/mochi"
URL="https://github.com/${MOCHI_GITHUB_REPO}"

if ! command -v brew >/dev/null 2>&1; then
  echo "error: Homebrew is required (https://brew.sh/)" >&2
  exit 1
fi

tap_installed=0
while IFS= read -r line; do
  [[ "${line,,}" == "${TAP,,}" ]] && tap_installed=1
done < <(brew tap 2>/dev/null || true)

if [[ "${tap_installed}" -eq 1 ]]; then
  if brew tap --custom-remote "${TAP}" "${URL}" 2>/dev/null; then
    echo "Updated tap ${TAP} -> ${URL}"
    exit 0
  fi

  echo "Replacing mismatched tap ${TAP} with ${URL}"
  brew untap "${TAP}"
fi

brew tap "${TAP}" "${URL}"
echo "Tapped ${TAP} -> ${URL}"
