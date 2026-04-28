#!/usr/bin/env bash
# 把 dist/user/ 装到 ~/.trae/，或 dist/project/ 装到 <project>/.trae/。
# 策略：覆盖 + 自动备份 .bak.YYYYMMDD-HHMMSS；写日志便于回滚。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
    cat <<EOF
Usage:
  $0 --user                  Install dist/user/ to \$HOME/.trae/
  $0 --project <path>        Install dist/project/ to <path>/.trae/
  $0 --help                  Show this message
EOF
    exit "${1:-0}"
}

mode=""
project_path=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --user) mode="user"; shift;;
        --project)
            mode="project"
            project_path="${2:-}"
            [[ -z "$project_path" ]] && { echo "ERROR: --project needs a path" >&2; usage 1; }
            [[ "$project_path" == --* ]] && { echo "ERROR: --project needs a path, not a flag: $project_path" >&2; usage 1; }
            shift 2;;
        --help|-h) usage 0;;
        *) echo "ERROR: unknown arg: $1" >&2; usage 1;;
    esac
done

[[ -z "$mode" ]] && usage 1

if [[ "$mode" == "user" ]]; then
    SRC="$ROOT/dist/user"
    DEST="$HOME/.trae"
else
    SRC="$ROOT/dist/project"
    DEST="$project_path/.trae"
fi

if [[ ! -d "$SRC" ]]; then
    echo "ERROR: $SRC not found. Run scripts/build.sh first." >&2
    exit 1
fi

mkdir -p "$DEST"
ts=$(date +%Y%m%d-%H%M%S)
LOG="$DEST/.superpowers-install.log"

echo "# install run at $ts" >> "$LOG"

# 拷贝；若目标已有同名文件，先备份
copy_with_backup() {
    local src="$1" dst="$2"
    if [[ -e "$dst" ]]; then
        local bak="$dst.bak.$ts"
        mv "$dst" "$bak"
        echo "BACKUP: $bak" | tee -a "$LOG"
    fi
    mkdir -p "$(dirname "$dst")"
    cp -r "$src" "$dst"
    echo "INSTALL: $dst" >> "$LOG"
}

# 顶层目录：rules/ skills/ agents/ —— 文件粒度合并
while IFS= read -r f; do
    rel="${f#$SRC/}"
    copy_with_backup "$f" "$DEST/$rel"
done < <(find "$SRC" -type f)

echo
echo "Install complete. Log: $LOG"
echo "  Source: $SRC"
echo "  Dest:   $DEST"
