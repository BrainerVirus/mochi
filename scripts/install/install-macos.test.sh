#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if ! grep -q "mochi_install_cli_link" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected macOS installer to define mochi_install_cli_link" >&2
  exit 1
fi

if ! grep -q "/usr/local/bin/mochi" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected default CLI link path" >&2
  exit 1
fi

if ! grep -q "Contents/MacOS/mochi" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected app bundle binary target" >&2
  exit 1
fi

if ! grep -q "sudo ln -sf" "${ROOT}/scripts/install/install-macos.sh"; then
  echo "expected sudo fallback command" >&2
  exit 1
fi
