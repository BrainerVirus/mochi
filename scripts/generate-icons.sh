#!/usr/bin/env bash
# Regenerate Tauri icon assets from the MochiChibi source SVG.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="${ROOT}/assets/icon/mochi-app-icon.svg"
OUTPUT_DIR="${ROOT}/src-tauri/icons"

if [[ ! -f "${SOURCE}" ]]; then
  echo "error: missing icon source at ${SOURCE}" >&2
  exit 1
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "error: pnpm is required (see docs/tech-stack.md)" >&2
  exit 1
fi

cd "${ROOT}"
pnpm tauri icon "${SOURCE}" --output "${OUTPUT_DIR}"

# Desktop-only app: remove mobile / Store assets not referenced by tauri.conf.json.
rm -rf "${OUTPUT_DIR}/android" "${OUTPUT_DIR}/ios"
rm -f "${OUTPUT_DIR}"/Square*.png "${OUTPUT_DIR}/StoreLogo.png" "${OUTPUT_DIR}/64x64.png"

echo "Generated Tauri icons in ${OUTPUT_DIR}"
