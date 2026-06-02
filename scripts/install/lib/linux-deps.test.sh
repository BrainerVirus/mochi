#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
source "${ROOT}/scripts/install/lib/linux-deps.sh"

if ! declare -f mochi_ensure_gnome_tray_extension >/dev/null; then
  echo "expected mochi_ensure_gnome_tray_extension helper" >&2
  exit 1
fi

if ! grep -q "MOCHI_GNOME_TRAY=0" "${ROOT}/scripts/install/lib/linux-deps.sh"; then
  echo "expected documented MOCHI_GNOME_TRAY=0 opt-out" >&2
  exit 1
fi
