#!/usr/bin/env bash
# 把 superpowers 5.0.x 源拷贝到 upstream/，便于入 git 跟踪。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEFAULT_CACHE="${HOME}/.claude/plugins/cache/claude-plugins-official/superpowers"

# 允许通过参数显式指定源；否则用 cache 目录里最新版本号子目录
if [[ $# -ge 1 ]]; then
    SRC="$1"
else
    if [[ ! -d "$DEFAULT_CACHE" ]]; then
        echo "ERROR: superpowers cache not found at $DEFAULT_CACHE" >&2
        echo "Either install superpowers via Claude Code, or pass an explicit source path:" >&2
        echo "  $0 /path/to/superpowers/<version>" >&2
        exit 1
    fi
    SRC="$(ls -1d "$DEFAULT_CACHE"/*/ 2>/dev/null | sort -V | tail -n1)"
    SRC="${SRC%/}"
fi

if [[ ! -d "$SRC" ]]; then
    echo "ERROR: source dir does not exist: $SRC" >&2
    exit 1
fi

VERSION="$(basename "$SRC")"
echo "Syncing superpowers $VERSION from $SRC ..."

rm -rf "$ROOT/upstream"
mkdir -p "$ROOT/upstream"
# 仅同步 porting 流水线需要的顶层条目。
# 若 superpowers 将来新增需要跟踪的目录/文件，在此列表追加并重跑本脚本。
for item in skills commands agents hooks README.md CHANGELOG.md RELEASE-NOTES.md package.json LICENSE; do
    if [[ -e "$SRC/$item" ]]; then
        cp -r "$SRC/$item" "$ROOT/upstream/"
    fi
done
echo "$VERSION" > "$ROOT/upstream/VERSION"
echo "Done. upstream/VERSION = $VERSION"
