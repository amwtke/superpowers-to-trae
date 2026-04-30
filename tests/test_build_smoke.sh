#!/usr/bin/env bash
# 在 tmp 工作区里以 fixture 为 upstream 跑一遍 build，断言关键产物。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# 复制项目骨架到 tmp（不含 upstream/，避免污染）
mkdir -p "$TMP/src" "$TMP/upstream" "$TMP/dist"
cp -r "$ROOT/src/"* "$TMP/src/"
# 把 fixture 当作 upstream
cp -r "$ROOT/tests/fixtures/mini-upstream/"* "$TMP/upstream/"
echo "fixture-0" > "$TMP/upstream/VERSION"

# Patch mappings.json so agents_to_generate references fixture skills, not real ones
python3 -c "
import json, pathlib
p = pathlib.Path('$TMP/src/mappings.json')
m = json.loads(p.read_text())
m['agents_to_generate'] = [{'name': 'fake-agent-alpha', 'skill': 'fake-alpha'}]
p.write_text(json.dumps(m, indent=2))
"

cd "$TMP"
PYTHONPATH="$TMP/src" python3 "$TMP/src/build_helpers.py" "$TMP"

# 断言
assert_file() { [[ -f "$1" ]] || { echo "MISSING: $1"; exit 1; }; }
assert_grep() { grep -q "$1" "$2" || { echo "GREP FAIL '$1' in $2"; exit 1; }; }

assert_file "$TMP/dist/user/rules/user_rules.md"
assert_file "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
assert_file "$TMP/dist/user/skills/superpowers/fake-beta/SKILL.md"
assert_file "$TMP/dist/user/skills/superpowers/references/trae-tools.md"
assert_file "$TMP/dist/user/agents/code-reviewer.md"
assert_file "$TMP/dist/user/agents/fake-agent-alpha.md"

# 转换效果验证
assert_grep "Trae IDE"  "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
! grep -q "Claude Code" "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
assert_grep "trae-tools.md" "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
! grep -q "copilot-tools.md" "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
# Phrase replacement: hookSpecificOutput → rules-bootstrap
assert_grep "rules-bootstrap"        "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
! grep -q "hookSpecificOutput"       "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
# Tool name replacement: TaskCreate → <TBD-Trae-TaskCreate>
assert_grep "<TBD-Trae-TaskCreate>"  "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
! grep -q "\bTaskCreate\b"           "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"

# bootstrap 索引应含两个 fake skill
assert_grep "fake-alpha" "$TMP/dist/user/rules/user_rules.md"
assert_grep "fake-beta"  "$TMP/dist/user/rules/user_rules.md"

echo "test_build_smoke: PASS"
