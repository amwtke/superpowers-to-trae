#!/usr/bin/env bash
# 主构建：upstream/ + src/ → dist/{user,project}/，末尾跑 verify。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ ! -d "$ROOT/upstream/skills" ]]; then
    echo "ERROR: upstream/ is empty. Run scripts/sync-upstream.sh first." >&2
    exit 1
fi

cd "$ROOT"
PYTHONPATH="$ROOT/src" python3 "$ROOT/src/build_helpers.py" "$ROOT"

if [[ -x "$ROOT/tests/verify-build.sh" ]]; then
    bash "$ROOT/tests/verify-build.sh"
else
    echo "(verify-build.sh not yet present; skipping post-build verification)"
fi
