#!/usr/bin/env bash
# Verify env for stable macOS Developer ID signing + notarization.
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

has_apple_id_auth=0
if [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]]; then
  has_apple_id_auth=1
fi

has_api_key_auth=0
if [[ -n "${APPLE_API_KEY:-}" && -n "${APPLE_API_ISSUER:-}" && -n "${APPLE_API_KEY_PRIVATE:-}" ]]; then
  has_api_key_auth=1
fi

if [[ "${has_apple_id_auth}" -eq 0 && "${has_api_key_auth}" -eq 0 ]]; then
  echo "missing notarization credentials: set APPLE_ID/APPLE_PASSWORD/APPLE_TEAM_ID or APPLE_API_KEY/APPLE_API_ISSUER/APPLE_API_KEY_PRIVATE" >&2
  exit 1
fi

echo "macOS signing environment OK"
