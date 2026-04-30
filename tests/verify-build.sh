#!/usr/bin/env bash
# 静态校验 dist/：frontmatter / 黑名单 / size 预算 / mappings 覆盖率 / 结构一致性。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
BLACKLIST="$ROOT/tests/forbidden-strings.txt"

fail=0
warn=0

if [[ ! -d "$DIST" ]]; then
    echo "FAIL: dist/ directory not found at $DIST"
    echo "Run scripts/build.sh first."
    exit 1
fi

# 1. Frontmatter 完整性：每个 SKILL.md 都要有非空 name 和 description
echo "[1/4] Frontmatter completeness ..."
while IFS= read -r f; do
    if ! head -20 "$f" | grep -q "^name:"; then
        echo "  FAIL: missing 'name:' in $f"; fail=1
    fi
    if ! head -20 "$f" | grep -q "^description:"; then
        echo "  FAIL: missing 'description:' in $f"; fail=1
    fi
done < <(find "$DIST" -name SKILL.md)

# 2. 黑名单扫描
echo "[2/4] Forbidden strings ..."
while IFS= read -r pat; do
    [[ -z "$pat" ]] && continue
    # Exclude the explanatory reference doc which intentionally names "Claude Code"
    hits=$(grep -rln --exclude="trae-tools.md" "$pat" "$DIST" || true)
    if [[ -n "$hits" ]]; then
        echo "  FAIL: forbidden pattern '$pat' found in:"
        echo "$hits" | sed 's/^/    /'
        fail=1
    fi
done < "$BLACKLIST"

# 3. user_rules.md size 预算（≤4KB warning，≤6KB fatal）
echo "[3/4] user_rules.md size budget ..."
for rules in "$DIST/user/rules/user_rules.md" "$DIST/project/rules/project_rules.md"; do
    [[ -f "$rules" ]] || continue
    size=$(wc -c < "$rules")
    if (( size > 6144 )); then
        echo "  FAIL: $rules is $size bytes (>6KB hard cap)"; fail=1
    elif (( size > 4096 )); then
        echo "  WARN: $rules is $size bytes (>4KB soft target)"; warn=1
    else
        echo "  OK: $rules is $size bytes"
    fi
done

# 4. Mappings 覆盖率
echo "[4/4] Mappings coverage ..."
rc=0
{ PROJECT_ROOT="$ROOT" python3 - <<'PY'
import json, os, pathlib, sys
root = pathlib.Path(os.environ["PROJECT_ROOT"])
mappings = json.loads((root/"src"/"mappings.json").read_text())
upstream = root / "upstream"
warnings = []
for entry in mappings["phrase_replacements"]:
    pat = entry["from"]
    found = False
    for f in upstream.rglob("*.md"):
        if pat in f.read_text(encoding="utf-8", errors="ignore"):
            found = True
            break
    if not found:
        warnings.append(pat)
if warnings:
    print("  WARN: phrase_replacements with 0 matches in upstream/:")
    for w in warnings:
        print(f"    - {w!r}")
    sys.exit(2)
print("  OK: all phrase_replacements have at least one match.")
PY
} || rc=$?
if (( rc == 2 )); then
    warn=1
elif (( rc != 0 )); then
    fail=1
fi

echo
if (( fail )); then
    echo "verify-build: FAIL"
    exit 1
fi
if (( warn )); then
    echo "verify-build: PASS with warnings"
    exit 0
fi
echo "verify-build: PASS"
