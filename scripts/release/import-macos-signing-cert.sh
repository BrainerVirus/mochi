#!/usr/bin/env bash
# Import Developer ID certificate into a CI keychain for stable macOS builds.
set -euo pipefail

require_nonempty() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    echo "missing ${name}" >&2
    exit 1
  fi
}

require_nonempty APPLE_CERTIFICATE
require_nonempty APPLE_CERTIFICATE_PASSWORD
require_nonempty KEYCHAIN_PASSWORD

KEYCHAIN="${MOCHI_MACOS_KEYCHAIN_PATH:-build.keychain}"
CERTIFICATE_FILE="$(mktemp -t mochi-cert.XXXXXX.p12)"

cleanup() {
  rm -f "${CERTIFICATE_FILE}"
}
trap cleanup EXIT

echo "${APPLE_CERTIFICATE}" | openssl base64 -d -A > "${CERTIFICATE_FILE}"

security create-keychain -p "${KEYCHAIN_PASSWORD}" "${KEYCHAIN}"
security default-keychain -s "${KEYCHAIN}"
security unlock-keychain -p "${KEYCHAIN_PASSWORD}" "${KEYCHAIN}"
security set-keychain-settings -t 3600 -u "${KEYCHAIN}"
security import "${CERTIFICATE_FILE}" -k "${KEYCHAIN}" -P "${APPLE_CERTIFICATE_PASSWORD}" -T /usr/bin/codesign -T /usr/bin/security
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "${KEYCHAIN_PASSWORD}" "${KEYCHAIN}"

if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  CERT_INFO="$(security find-identity -v -p codesigning "${KEYCHAIN}" | grep "Developer ID Application" | head -1 || true)"
  if [[ -z "${CERT_INFO}" ]]; then
    echo "Developer ID Application certificate not found in ${KEYCHAIN}" >&2
    exit 1
  fi
  APPLE_SIGNING_IDENTITY="$(echo "${CERT_INFO}" | awk -F'"' '{print $2}')"
fi

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "APPLE_SIGNING_IDENTITY=${APPLE_SIGNING_IDENTITY}" >> "${GITHUB_ENV}"
fi
export APPLE_SIGNING_IDENTITY

if [[ -n "${APPLE_API_KEY_PRIVATE:-}" ]]; then
  require_nonempty APPLE_API_KEY
  require_nonempty APPLE_API_ISSUER
  api_key_path="${RUNNER_TEMP:-${TMPDIR:-/tmp}}/AuthKey_${APPLE_API_KEY}.p8"
  echo "${APPLE_API_KEY_PRIVATE}" | openssl base64 -d -A > "${api_key_path}"
  chmod 600 "${api_key_path}"
  export APPLE_API_KEY_PATH="${api_key_path}"
  if [[ -n "${GITHUB_ENV:-}" ]]; then
    echo "APPLE_API_KEY_PATH=${api_key_path}" >> "${GITHUB_ENV}"
  fi
fi

echo "Imported macOS signing certificate into ${KEYCHAIN}"
