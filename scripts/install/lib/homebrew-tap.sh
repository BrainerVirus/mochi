#!/usr/bin/env bash
# Shared Homebrew tap helpers for Mochi install scripts.
# macOS /bin/bash is 3.2 — avoid bash 4+ syntax such as ${var,,}.
set -euo pipefail

: "${MOCHI_GITHUB_REPO:=BrainerVirus/mochi}"

mochi_lower() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

mochi_tap_listed_in() {
  local needle="$1"
  local tap_list="$2"
  local line
  while IFS= read -r line; do
    [ -z "${line}" ] && continue
    if [ "$(mochi_lower "${line}")" = "$(mochi_lower "${needle}")" ]; then
      return 0
    fi
  done <<EOF
${tap_list}
EOF
  return 1
}

mochi_homebrew_tap_user() {
  local repo="${1:-${MOCHI_GITHUB_REPO}}"
  printf '%s' "${repo%%/*}"
}

mochi_homebrew_tap_name() {
  local repo="${1:-${MOCHI_GITHUB_REPO}}"
  printf '%s/mochi' "$(mochi_homebrew_tap_user "${repo}")"
}

mochi_homebrew_tap_url() {
  local repo="${1:-${MOCHI_GITHUB_REPO}}"
  printf 'https://github.com/%s' "${repo}"
}

mochi_homebrew_stale_taps() {
  local tap_user="${1:-$(mochi_homebrew_tap_user)}"
  printf 'test/mochi-tap\n%s/mochi-install\n' "${tap_user}"
}

mochi_homebrew_tap_remote_override_key() {
  local tap="$1"
  printf 'MOCHI_TEST_TAP_REMOTE_%s' "$(mochi_lower "$(printf '%s' "${tap}" | tr '/-' '_')")"
}

mochi_homebrew_tap_remote() {
  local tap="$1"
  local env_key
  env_key="$(mochi_homebrew_tap_remote_override_key "${tap}")"
  local override="${!env_key:-}"
  if [ -n "${override}" ]; then
    printf '%s' "${override}"
    return 0
  fi

  local tap_repo=""
  tap_repo="$(brew --repo "${tap}" 2>/dev/null || true)"
  if [ -z "${tap_repo}" ] || [ ! -d "${tap_repo}/.git" ]; then
    return 0
  fi

  git -C "${tap_repo}" config --get remote.origin.url 2>/dev/null || true
}

# Prints tab-separated tap actions, one per line:
# remove-stale<TAB>tap
# replace<TAB>tap<TAB>current-url<TAB>target-url
# install<TAB>tap<TAB>target-url
# ready<TAB>tap<TAB>target-url
mochi_homebrew_tap_plan() {
  local tap_list="${1:-}"
  local repo="${MOCHI_GITHUB_REPO}"
  local tap
  local url
  local stale_tap
  local current_url

  tap="$(mochi_homebrew_tap_name "${repo}")"
  url="$(mochi_homebrew_tap_url "${repo}")"

  while IFS= read -r stale_tap; do
    [ -z "${stale_tap}" ] && continue
    if mochi_tap_listed_in "${stale_tap}" "${tap_list}"; then
      printf 'remove-stale\t%s\n' "${stale_tap}"
    fi
  done <<EOF
$(mochi_homebrew_stale_taps "$(mochi_homebrew_tap_user "${repo}")")
EOF

  if mochi_tap_listed_in "${tap}" "${tap_list}"; then
    current_url="$(mochi_homebrew_tap_remote "${tap}")"
    if [ "${current_url}" = "${url}" ]; then
      printf 'ready\t%s\t%s\n' "${tap}" "${url}"
      return 0
    fi
    printf 'replace\t%s\t%s\t%s\n' "${tap}" "${current_url:-unknown}" "${url}"
    return 0
  fi

  printf 'install\t%s\t%s\n' "${tap}" "${url}"
}

mochi_homebrew_apply_tap_plan_line() {
  local action="${1:-}"
  local arg2="${2:-}"
  local arg3="${3:-}"
  local arg4="${4:-}"

  case "${action}" in
    remove-stale)
      brew untap --force "${arg2}" 2>/dev/null || brew untap "${arg2}" 2>/dev/null || true
      ;;
    replace)
      brew untap --force "${arg2}"
      brew tap "${arg2}" "${arg4}"
      ;;
    install)
      brew tap "${arg2}" "${arg3}"
      ;;
    ready)
      return 0
      ;;
    *)
      printf 'error: unknown tap plan action: %s\n' "${action}" >&2
      return 1
      ;;
  esac
}

mochi_homebrew_apply_tap_plan() {
  local tap_list="${1:-}"
  local line
  local action
  local plan_output=""

  plan_output="$(mochi_homebrew_tap_plan "${tap_list}")"
  while IFS= read -r line; do
    [ -z "${line}" ] && continue
    IFS=$'\t' read -r action _rest <<<"${line}"
    case "${action}" in
      remove-stale)
        IFS=$'\t' read -r _action arg2 <<<"${line}"
        mochi_homebrew_apply_tap_plan_line "${action}" "${arg2}"
        ;;
      replace)
        IFS=$'\t' read -r _action arg2 arg3 arg4 <<<"${line}"
        mochi_homebrew_apply_tap_plan_line "${action}" "${arg2}" "${arg3}" "${arg4}"
        ;;
      install)
        IFS=$'\t' read -r _action arg2 arg3 <<<"${line}"
        mochi_homebrew_apply_tap_plan_line "${action}" "${arg2}" "${arg3}"
        ;;
      ready)
        return 0
        ;;
    esac
  done <<EOF
${plan_output}
EOF
}

mochi_homebrew_install_cask_ref() {
  local channel="${1:-stable}"
  local stable_cask="${MOCHI_STABLE_CASK:-mochi-desktop}"
  local unstable_cask="${MOCHI_UNSTABLE_CASK:-mochi-unstable}"
  local tap
  local cask_id

  tap="$(mochi_homebrew_tap_name)"
  if [ "${channel}" = "unstable" ]; then
    cask_id="${unstable_cask}"
  else
    cask_id="${stable_cask}"
  fi

  printf '%s/%s' "${tap}" "${cask_id}"
}
