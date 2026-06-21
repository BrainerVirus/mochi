#!/usr/bin/env bash
# Ensure the GitHub-backed Homebrew tap for Mochi is installed.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"
: "${MOCHI_INSTALL_REF:=main}"

_mochi_homebrew_loaded=0
for _i in "${!BASH_SOURCE[@]}"; do
  _src="${BASH_SOURCE[_i]:-}"
  if [[ -n "${_src}" && "${_src}" == *.sh && -f "${_src}" ]]; then
    _dir="$(cd "$(dirname "${_src}")" && pwd)"
    if [[ -f "${_dir}/lib/homebrew-tap.sh" ]]; then
      # shellcheck source=lib/homebrew-tap.sh
      source "${_dir}/lib/homebrew-tap.sh"
      _mochi_homebrew_loaded=1
      break
    fi
  fi
done
unset _i _src _dir

if [[ "${_mochi_homebrew_loaded}" -eq 0 ]]; then
  _tmp="$(mktemp)"
  curl -fsSL \
    "https://raw.githubusercontent.com/${MOCHI_GITHUB_REPO}/${MOCHI_INSTALL_REF}/scripts/install/lib/homebrew-tap.sh" \
    -o "${_tmp}"
  # shellcheck source=/dev/null
  source "${_tmp}"
  rm -f "${_tmp}"
fi
unset _mochi_homebrew_loaded _tmp

if ! command -v brew >/dev/null 2>&1; then
  echo "error: Homebrew is required (https://brew.sh/)" >&2
  exit 1
fi

TAP="$(mochi_homebrew_tap_name)"
URL="$(mochi_homebrew_tap_url)"
TAP_LIST="$(brew tap 2>/dev/null || true)"
PLAN="$(mochi_homebrew_tap_plan "${TAP_LIST}")"
READY=0

while IFS= read -r line; do
  [ -z "${line}" ] && continue
  action="${line%%$'\t'*}"
  case "${action}" in
    remove-stale)
      IFS=$'\t' read -r _action stale_tap <<<"${line}"
      echo "Removing stale tap ${stale_tap}"
      mochi_homebrew_apply_tap_plan_line "${action}" "${stale_tap}"
      ;;
    replace)
      IFS=$'\t' read -r _action tap_name current_url target_url <<<"${line}"
      echo "Replacing tap ${tap_name} (${current_url} -> ${target_url})"
      mochi_homebrew_apply_tap_plan_line "${action}" "${tap_name}" "${current_url}" "${target_url}"
      echo "Tapped ${tap_name} -> ${target_url}"
      ;;
    install)
      IFS=$'\t' read -r _action tap_name target_url <<<"${line}"
      mochi_homebrew_apply_tap_plan_line "${action}" "${tap_name}" "${target_url}"
      echo "Tapped ${tap_name} -> ${target_url}"
      ;;
    ready)
      IFS=$'\t' read -r _action tap_name target_url <<<"${line}"
      echo "Tap ${tap_name} already points to ${target_url}"
      READY=1
      ;;
  esac
done <<EOF
${PLAN}
EOF
