# Superpowers → Trae IDE 移植 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 Claude Code 上的 Superpowers 5.0.7（14 skills + 3 commands + code-reviewer agent + SessionStart hook）移植到 Trae IDE，产出一条可重跑的转换流水线 + 双形态安装产物（用户级 / 项目级）。

**Architecture:** 约定驱动转换器：`upstream/`（superpowers 原版，入 git）+ `src/`（mappings.json + 模板，人工维护）→ `scripts/build.sh`（机械转换）→ `dist/{user,project}/`（入 git，便于 review）→ `scripts/install.sh`（拷贝至 `~/.trae/` 或 `<project>/.trae/`）。SessionStart hook 由 `rules/{user,project}_rules.md` 等价替代。

**Tech Stack:** Python 3（仅 stdlib：json、re、pathlib、argparse）做字符串转换与模板渲染；Bash 做编排（sync / build / install / verify）；POSIX 工具（cp、mv、grep、sed）；Python `unittest` 做单元测试。

**Spec:** [`docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md`](../specs/2026-04-28-superpowers-to-trae-design.md)

---

## File Structure

| 路径 | 责任 |
|---|---|
| `.gitignore` | 忽略 `dist/` 之外的临时产物（`*.bak`、`__pycache__` 等） |
| `README.md` | 项目总览、安装、升级流程 |
| `upstream/` | superpowers 原版源码副本（由 `sync-upstream.sh` 维护，入 git） |
| `src/mappings.json` | 转换配置：tool 名映射、phrase 替换、文件跳过清单、agents_to_generate |
| `src/bootstrap-rule.template.md` | `user_rules.md` / `project_rules.md` 模板，含 `{{ skill_index }}` 占位符 |
| `src/agent-wrapper.template.md` | Custom Agent 模板，含 `{{ name }}` `{{ description }}` `{{ skill }}` 占位符 |
| `src/trae-tools-reference.md` | Trae 工具速查（拷到 `dist/.../references/trae-tools.md`） |
| `src/transform.py` | 字符串替换库：apply_phrase_replacements、apply_tool_replacements |
| `src/render.py` | 模板渲染库：parse_skill_frontmatter、render_template |
| `scripts/sync-upstream.sh` | 从 `~/.claude/plugins/cache/.../superpowers/<ver>/` 拷贝到 `upstream/` |
| `scripts/build.sh` | 编排：调 transform.py + render.py，输出 `dist/{user,project}/`；末尾跑 verify |
| `scripts/install.sh` | 拷贝 `dist/user/` → `~/.trae/`，或 `dist/project/` → `<target>/.trae/`；备份 + 日志 |
| `tests/test_transform.py` | 单测 transform.py |
| `tests/test_render.py` | 单测 render.py |
| `tests/fixtures/mini-upstream/` | 假 superpowers 源码（2 个 skill + 1 agent + 1 command），驱动 build smoke test |
| `tests/verify-build.sh` | 静态校验 dist：frontmatter、引用完整、黑名单、size 预算 |
| `tests/forbidden-strings.txt` | Claude-only 字眼黑名单 |
| `tests/test_build_smoke.sh` | 端到端：用 fixture 驱动 build → verify |
| `docs/superpowers/manual-smoke-test.md` | 真机手工冒烟步骤 |
| `dist/user/` | 构建产物（用户级），入 git |
| `dist/project/` | 构建产物（项目级），入 git |

---

## Task 1: 仓库骨架

**Files:**
- Create: `.gitignore`
- Create: `README.md`（占位）
- Create: 空目录 `src/`、`scripts/`、`tests/fixtures/`、`upstream/`

- [ ] **Step 1: 创建目录骨架**

```bash
cd /home/xiaojin/workshop/cc-superpower-to-trae
mkdir -p src scripts tests/fixtures upstream
```

- [ ] **Step 2: 写 `.gitignore`**

```
# Python
__pycache__/
*.pyc
.pytest_cache/

# Backup files from install.sh
*.bak.*

# Local install logs
.superpowers-install.log

# Editor
.vscode/
.idea/
*.swp

# Worktrees
.worktrees/
```

- [ ] **Step 3: 写 README.md 占位**

```markdown
# cc-superpower-to-trae

把 [Superpowers](https://github.com/obra/superpowers) 移植到 [Trae IDE](https://www.trae.ai/)。

设计文档：[docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md](docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md)

实施计划：[docs/superpowers/plans/2026-04-28-superpowers-to-trae.md](docs/superpowers/plans/2026-04-28-superpowers-to-trae.md)

> 完整 README 在实施完成后由 Task 12 写入。
```

- [ ] **Step 4: 验证布局**

```bash
ls -la
```

Expected：含 `.gitignore`、`README.md`、`src/`、`scripts/`、`tests/`、`upstream/`、`docs/`。

- [ ] **Step 5: Commit**

```bash
git add .gitignore README.md
git commit -m "chore: 初始化仓库骨架"
```

---

## Task 2: 测试 fixture——假的 mini-upstream 树

**目标：** 提供小可控的 fixture 让 build/verify 测试可以独立跑，不依赖真实 5.0.7。

**Files:**
- Create: `tests/fixtures/mini-upstream/skills/fake-alpha/SKILL.md`
- Create: `tests/fixtures/mini-upstream/skills/fake-beta/SKILL.md`
- Create: `tests/fixtures/mini-upstream/skills/fake-alpha/references/copilot-tools.md`
- Create: `tests/fixtures/mini-upstream/agents/code-reviewer.md`
- Create: `tests/fixtures/mini-upstream/commands/brainstorm.md`

- [ ] **Step 1: 写 fake-alpha skill（含可触发所有替换的字眼）**

`tests/fixtures/mini-upstream/skills/fake-alpha/SKILL.md`：

````markdown
---
name: fake-alpha
description: A fake skill for fixture testing — turning A into B
---

# Fake Alpha

This skill runs in Claude Code. Use the Skill tool to invoke other skills.

When you start, run `TaskCreate` to record progress, then `Bash` to check status.
Use `Read` to inspect files, `Edit` to modify them, `Write` to create new ones.

See references/copilot-tools.md for tool name details.

Refer to `hookSpecificOutput` if running under SessionStart.
````

- [ ] **Step 2: 写 fake-alpha 的 references/copilot-tools.md**

`tests/fixtures/mini-upstream/skills/fake-alpha/references/copilot-tools.md`：

```markdown
# Copilot CLI tool mapping (placeholder for fixture)
```

- [ ] **Step 3: 写 fake-beta skill（最小化，仅 frontmatter）**

`tests/fixtures/mini-upstream/skills/fake-beta/SKILL.md`：

```markdown
---
name: fake-beta
description: Another fake skill, minimal body
---

Body intentionally short.
```

- [ ] **Step 4: 写 fixture code-reviewer agent**

`tests/fixtures/mini-upstream/agents/code-reviewer.md`：

```markdown
---
name: code-reviewer
description: Review code for fixture
---

You are a code reviewer running in Claude Code. Use TaskCreate when planning.
```

- [ ] **Step 5: 写 fixture brainstorm command**

`tests/fixtures/mini-upstream/commands/brainstorm.md`：

```markdown
---
name: brainstorm
description: Fixture brainstorm command
---

Use the brainstorming skill.
```

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/
git commit -m "test: 添加 mini-upstream fixture"
```

---

## Task 3: `src/mappings.json`——转换配置

**Files:**
- Create: `src/mappings.json`

- [ ] **Step 1: 写 mappings.json 初版（tool 名先用占位 `<TBD-Trae-X>`，待 §11 TBD-1 落实后替换）**

```json
{
  "tool_name_replacements": {
    "TaskCreate": "<TBD-Trae-TaskCreate>",
    "TaskUpdate": "<TBD-Trae-TaskUpdate>",
    "TaskList": "<TBD-Trae-TaskList>",
    "WebSearch": "<TBD-Trae-WebSearch>",
    "WebFetch": "<TBD-Trae-WebFetch>"
  },
  "phrase_replacements": [
    { "from": "Claude Code", "to": "Trae IDE" },
    { "from": "use the Skill tool to invoke", "to": "read the SKILL.md file directly" },
    { "from": "Use the Skill tool to invoke", "to": "Read the SKILL.md file directly" },
    { "from": "the Skill tool", "to": "Read on the SKILL.md file" },
    { "from": "references/copilot-tools.md", "to": "references/trae-tools.md" },
    { "from": "references/codex-tools.md", "to": "references/trae-tools.md" },
    { "from": "hookSpecificOutput", "to": "rules-bootstrap" },
    { "from": "hookEventName", "to": "rule-event" },
    { "from": "CLAUDE_PLUGIN_ROOT", "to": "TRAE_PLUGIN_ROOT"}
  ],
  "files_to_skip": [
    "hooks",
    "scripts",
    "tests",
    "package.json",
    "AGENTS.md",
    "CLAUDE.md",
    "GEMINI.md",
    "gemini-extension.json",
    "RELEASE-NOTES.md",
    "CODE_OF_CONDUCT.md",
    "LICENSE"
  ],
  "agents_to_generate": [
    { "name": "brainstorm",   "skill": "brainstorming" },
    { "name": "write-plan",   "skill": "writing-plans" },
    { "name": "execute-plan", "skill": "executing-plans" }
  ]
}
```

> 注：保留 `Read`/`Edit`/`Write`/`Bash` 不动——这些工具名 Trae 多半保留同名（待 TBD-1 验证；如不同，添加对应映射即可）。

- [ ] **Step 2: 验证 JSON 合法**

```bash
python3 -c "import json; json.load(open('src/mappings.json'))"
```

Expected：无输出（无错误即通过）。

- [ ] **Step 3: Commit**

```bash
git add src/mappings.json
git commit -m "feat(src): 添加 mappings.json 转换配置"
```

---

## Task 4: `src/transform.py`——字符串转换库（TDD）

**Files:**
- Create: `src/transform.py`
- Create: `tests/test_transform.py`

- [ ] **Step 1: 写失败的测试**

`tests/test_transform.py`：

```python
import unittest
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "src"))
import transform


class TransformTests(unittest.TestCase):
    def test_phrase_replacements_applied_in_order(self):
        text = "This runs in Claude Code."
        mappings = {
            "tool_name_replacements": {},
            "phrase_replacements": [{"from": "Claude Code", "to": "Trae IDE"}],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, "This runs in Trae IDE.")

    def test_tool_name_word_boundary(self):
        text = "Run TaskCreate now. TaskCreated is a different word."
        mappings = {
            "tool_name_replacements": {"TaskCreate": "TraeTask"},
            "phrase_replacements": [],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, "Run TraeTask now. TaskCreated is a different word.")

    def test_phrase_replaced_before_tool_to_avoid_double_substitution(self):
        text = "use the Skill tool to invoke X"
        mappings = {
            "tool_name_replacements": {"Skill": "TraeSkill"},
            "phrase_replacements": [{"from": "use the Skill tool to invoke", "to": "read the SKILL.md directly for"}],
        }
        out = transform.apply(text, mappings)
        self.assertEqual(out, "read the SKILL.md directly for X")

    def test_no_change_when_no_match(self):
        text = "nothing matches here"
        mappings = {"tool_name_replacements": {"Foo": "Bar"}, "phrase_replacements": []}
        self.assertEqual(transform.apply(text, mappings), text)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: 跑测试看它失败（transform 还不存在）**

```bash
python3 -m unittest tests.test_transform -v
```

Expected：`ModuleNotFoundError: No module named 'transform'`。

- [ ] **Step 3: 实现 `src/transform.py`**

```python
"""Apply mappings.json transformations to a string.

Order matters:
  1. phrase_replacements (longer multi-word patterns) — applied first
  2. tool_name_replacements (single tokens with word boundaries) — applied second

Rationale: phrases often contain tool tokens, so substituting tools first
would corrupt phrase matching.
"""
import re


def apply(text: str, mappings: dict) -> str:
    for entry in mappings.get("phrase_replacements", []):
        text = text.replace(entry["from"], entry["to"])
    for tool, replacement in mappings.get("tool_name_replacements", {}).items():
        text = re.sub(rf"\b{re.escape(tool)}\b", replacement, text)
    return text


def apply_to_file(path, mappings: dict) -> bool:
    import pathlib
    p = pathlib.Path(path)
    original = p.read_text(encoding="utf-8")
    transformed = apply(original, mappings)
    if transformed != original:
        p.write_text(transformed, encoding="utf-8")
        return True
    return False
```

- [ ] **Step 4: 跑测试看它通过**

```bash
python3 -m unittest tests.test_transform -v
```

Expected：4 个测试全 PASS。

- [ ] **Step 5: Commit**

```bash
git add src/transform.py tests/test_transform.py
git commit -m "feat(src): transform.py + 单测"
```

---

## Task 5: 模板文件（bootstrap rule + agent wrapper + trae-tools-reference）

**Files:**
- Create: `src/bootstrap-rule.template.md`
- Create: `src/agent-wrapper.template.md`
- Create: `src/trae-tools-reference.md`

- [ ] **Step 1: 写 `src/bootstrap-rule.template.md`**

```markdown
---
description: Superpowers methodology - skill discovery and use protocol
---

# You have superpowers.

You have access to a library of methodology skills under `.trae/skills/superpowers/`.
Each skill is a folder containing a SKILL.md describing when and how to use it.

## The Rule

When the user's request matches a skill's `description`, you MUST read that
skill's SKILL.md and follow it before acting — including for clarifying
questions, exploration, and planning.

## Skill Index

{{ skill_index }}

## How to load a skill

Read the file `.trae/skills/superpowers/<skill-name>/SKILL.md` and follow it
exactly. Skills evolve — do not paraphrase from memory.

## Tool name reference

Trae IDE tool names may differ from those mentioned in skill text. Before
following any skill that names a tool, see
`.trae/skills/superpowers/references/trae-tools.md` for mappings.
```

- [ ] **Step 2: 写 `src/agent-wrapper.template.md`**

```markdown
---
name: {{ name }}
description: {{ description }}
---

You are running the {{ skill }} workflow. Read
`.trae/skills/superpowers/{{ skill }}/SKILL.md` and follow it exactly.

Skills evolve — always read the file fresh, do not work from memory.
```

- [ ] **Step 3: 写 `src/trae-tools-reference.md`（含 TBD 标注，待 §11 TBD-1 落实后填）**

```markdown
# Trae IDE tool name mapping

Skills in this library were authored against Claude Code tool names. Use this
table when a skill mentions a tool name.

| Skill text mentions | Trae IDE equivalent | Notes |
|---|---|---|
| Read | Read | Same name (verify on install) |
| Edit | Edit | Same name (verify on install) |
| Write | Write | Same name (verify on install) |
| Bash | Bash | Same name (verify on install) |
| TaskCreate / TaskUpdate / TaskList | (TBD — see project README) | Trae's todo/task mechanism |
| WebSearch | (TBD) | If unavailable, do search manually |
| WebFetch | (TBD) | If unavailable, ask user to provide content |
| Skill (tool invocation) | Read the SKILL.md file directly | Trae has no "Skill tool"; load via Read |

If a skill references a tool not available in Trae, do the equivalent manually
(e.g., maintain a markdown todo list when there is no TaskCreate).

> The TBD entries are filled by the porting maintainer after verifying against
> a running Trae IDE. See `src/mappings.json` `tool_name_replacements`.
```

- [ ] **Step 4: Commit**

```bash
git add src/bootstrap-rule.template.md src/agent-wrapper.template.md src/trae-tools-reference.md
git commit -m "feat(src): 添加 bootstrap rule / agent wrapper / trae-tools 模板"
```

---

## Task 6: `src/render.py`——模板渲染库（TDD）

**Files:**
- Create: `src/render.py`
- Create: `tests/test_render.py`

- [ ] **Step 1: 写失败的测试**

`tests/test_render.py`：

```python
import unittest
import tempfile
import pathlib
import sys
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "src"))
import render


SKILL_FRONTMATTER_SAMPLE = """---
name: brainstorming
description: Use when starting any creative work — turn ideas into specs through dialogue
---

# Body content here
"""


class RenderTests(unittest.TestCase):
    def test_parse_skill_frontmatter_extracts_name_and_description(self):
        with tempfile.TemporaryDirectory() as td:
            p = pathlib.Path(td) / "SKILL.md"
            p.write_text(SKILL_FRONTMATTER_SAMPLE)
            fm = render.parse_skill_frontmatter(p)
        self.assertEqual(fm["name"], "brainstorming")
        self.assertIn("turn ideas", fm["description"])

    def test_render_template_substitutes_double_braces(self):
        out = render.render_template("Hello {{ name }}!", {"name": "world"})
        self.assertEqual(out, "Hello world!")

    def test_render_template_handles_multiple_vars(self):
        tpl = "{{ a }} and {{ b }}"
        self.assertEqual(render.render_template(tpl, {"a": "x", "b": "y"}), "x and y")

    def test_build_skill_index_formats_as_markdown_list(self):
        skills = [
            {"name": "alpha", "description": "Do alpha things"},
            {"name": "beta", "description": "Do beta things"},
        ]
        out = render.build_skill_index(skills)
        self.assertIn("- alpha — Do alpha things", out)
        self.assertIn("- beta — Do beta things", out)

    def test_render_template_missing_var_raises(self):
        with self.assertRaises(KeyError):
            render.render_template("{{ missing }}", {})


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: 跑测试看它失败**

```bash
python3 -m unittest tests.test_render -v
```

Expected：`ModuleNotFoundError: No module named 'render'`。

- [ ] **Step 3: 实现 `src/render.py`**

```python
"""Template rendering and skill-frontmatter parsing for the porting build."""
import re
import pathlib


_FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n", re.DOTALL)
_VAR_RE = re.compile(r"\{\{\s*(\w+)\s*\}\}")


def parse_skill_frontmatter(path) -> dict:
    """Read a SKILL.md and return its YAML-ish frontmatter as a dict.

    Only supports `key: value` lines (one per line). Sufficient for skills
    in this library — we don't need full YAML parsing.
    """
    text = pathlib.Path(path).read_text(encoding="utf-8")
    m = _FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError(f"No frontmatter in {path}")
    out = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            out[k.strip()] = v.strip()
    return out


def render_template(template: str, vars: dict) -> str:
    """Substitute `{{ name }}` placeholders. Raises KeyError on missing keys."""
    def sub(match):
        key = match.group(1)
        if key not in vars:
            raise KeyError(f"Template variable not provided: {key}")
        return str(vars[key])
    return _VAR_RE.sub(sub, template)


def build_skill_index(skills: list) -> str:
    """Render a list of {'name', 'description'} dicts as a markdown bullet list."""
    return "\n".join(f"- {s['name']} — {s['description']}" for s in skills)
```

- [ ] **Step 4: 跑测试看它通过**

```bash
python3 -m unittest tests.test_render -v
```

Expected：5 个测试全 PASS。

- [ ] **Step 5: Commit**

```bash
git add src/render.py tests/test_render.py
git commit -m "feat(src): render.py + 单测"
```

---

## Task 7: `scripts/sync-upstream.sh`

**Files:**
- Create: `scripts/sync-upstream.sh`

- [ ] **Step 1: 写 `scripts/sync-upstream.sh`**

```bash
#!/usr/bin/env bash
# 把 superpowers 5.0.x 源拷贝到 upstream/，便于入 git 跟踪。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
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
    SRC=$(ls -1d "$DEFAULT_CACHE"/*/ 2>/dev/null | sort -V | tail -n1)
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
for item in skills commands agents hooks README.md CHANGELOG.md RELEASE-NOTES.md package.json LICENSE; do
    if [[ -e "$SRC/$item" ]]; then
        cp -r "$SRC/$item" "$ROOT/upstream/"
    fi
done
echo "$VERSION" > "$ROOT/upstream/VERSION"
echo "Done. upstream/VERSION = $VERSION"
```

- [ ] **Step 2: 设可执行权限并验证**

```bash
chmod +x scripts/sync-upstream.sh
ls -l scripts/sync-upstream.sh
```

Expected：文件有 `x` 权限。

- [ ] **Step 3: 实跑一次（首次填充 upstream/）**

```bash
bash scripts/sync-upstream.sh
ls upstream/ && cat upstream/VERSION
```

Expected：列出 `skills`、`commands`、`agents`、`hooks`、`README.md` 等；VERSION 显示如 `5.0.7`。

- [ ] **Step 4: Commit 脚本 + upstream 副本**

```bash
git add scripts/sync-upstream.sh upstream/
git commit -m "feat(scripts): sync-upstream.sh + 同步 superpowers 5.x 副本"
```

---

## Task 8: `scripts/build.sh`——主构建编排

**目标：** 调 transform.py + render.py，把 `upstream/` 转成 `dist/{user,project}/`，末尾跑 verify。

**Files:**
- Create: `scripts/build.sh`
- Create: `src/build_helpers.py`（构建辅助：扫 skill 列表、产出 agent、镜像目录）

- [ ] **Step 1: 写 `src/build_helpers.py`**

```python
"""Build helpers: orchestrate transform.py + render.py over upstream/.

This module is intentionally Python (not bash) because:
- It needs to walk directory trees applying per-file transforms.
- It needs to parse SKILL.md frontmatter to build the bootstrap index.
- It needs to render templates with multiple variables.
"""
import json
import shutil
import sys
import pathlib

import transform
import render


def load_mappings(root: pathlib.Path) -> dict:
    return json.loads((root / "src" / "mappings.json").read_text())


def copy_and_transform_skills(upstream: pathlib.Path, dist_skills: pathlib.Path,
                              mappings: dict) -> list:
    """Copy upstream/skills/* to dist/skills/superpowers/*, transform every .md.

    Returns a list of {'name', 'description'} dicts for the bootstrap index.
    """
    if dist_skills.exists():
        shutil.rmtree(dist_skills)
    dist_skills.mkdir(parents=True)
    skills_meta = []
    for skill_dir in sorted((upstream / "skills").iterdir()):
        if not skill_dir.is_dir():
            continue
        target = dist_skills / skill_dir.name
        shutil.copytree(skill_dir, target)
        for md in target.rglob("*.md"):
            transform.apply_to_file(md, mappings)
        fm = render.parse_skill_frontmatter(target / "SKILL.md")
        skills_meta.append({"name": fm["name"], "description": fm["description"]})
    return skills_meta


def write_bootstrap_rule(template_path: pathlib.Path, out_path: pathlib.Path,
                        skills_meta: list) -> None:
    template = template_path.read_text()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(render.render_template(
        template,
        {"skill_index": render.build_skill_index(skills_meta)},
    ))


def write_custom_agents(template_path: pathlib.Path, out_dir: pathlib.Path,
                        agents_to_generate: list, skills_meta: list) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    template = template_path.read_text()
    for agent in agents_to_generate:
        skill_meta = next((s for s in skills_meta if s["name"] == agent["skill"]), None)
        if skill_meta is None:
            raise ValueError(f"Agent {agent['name']} references unknown skill {agent['skill']}")
        rendered = render.render_template(template, {
            "name": agent["name"],
            "skill": agent["skill"],
            "description": skill_meta["description"],
        })
        (out_dir / f"{agent['name']}.md").write_text(rendered)


def transform_code_reviewer(upstream: pathlib.Path, out_dir: pathlib.Path,
                            mappings: dict) -> None:
    src = upstream / "agents" / "code-reviewer.md"
    if not src.exists():
        return
    dst = out_dir / "code-reviewer.md"
    out_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy(src, dst)
    transform.apply_to_file(dst, mappings)


def install_trae_tools_ref(src_ref: pathlib.Path, dist_skills: pathlib.Path) -> None:
    target = dist_skills / "references" / "trae-tools.md"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src_ref, target)


def mirror_user_to_project(user_dir: pathlib.Path, project_dir: pathlib.Path) -> None:
    if project_dir.exists():
        shutil.rmtree(project_dir)
    shutil.copytree(user_dir, project_dir)
    user_rules = project_dir / "rules" / "user_rules.md"
    project_rules = project_dir / "rules" / "project_rules.md"
    if user_rules.exists():
        user_rules.rename(project_rules)


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path.cwd()
    upstream = root / "upstream"
    src = root / "src"
    dist_user = root / "dist" / "user"
    dist_project = root / "dist" / "project"

    if (root / "dist").exists():
        shutil.rmtree(root / "dist")

    mappings = load_mappings(root)

    # Phase 1: skills
    skills_meta = copy_and_transform_skills(
        upstream, dist_user / "skills" / "superpowers", mappings,
    )

    # Phase 2: trae-tools.md reference
    install_trae_tools_ref(
        src / "trae-tools-reference.md",
        dist_user / "skills" / "superpowers",
    )

    # Phase 3: bootstrap rule
    write_bootstrap_rule(
        src / "bootstrap-rule.template.md",
        dist_user / "rules" / "user_rules.md",
        skills_meta,
    )

    # Phase 4: custom agents
    write_custom_agents(
        src / "agent-wrapper.template.md",
        dist_user / "agents",
        mappings["agents_to_generate"],
        skills_meta,
    )

    # Phase 5: code-reviewer
    transform_code_reviewer(upstream, dist_user / "agents", mappings)

    # Phase 6: mirror to project
    mirror_user_to_project(dist_user, dist_project)

    print(f"Built {len(skills_meta)} skills, {len(mappings['agents_to_generate']) + 1} agents")
    print(f"  -> {dist_user}")
    print(f"  -> {dist_project}")


if __name__ == "__main__":
    sys.path.insert(0, str(pathlib.Path(__file__).parent))
    main(sys.argv)
```

- [ ] **Step 2: 写 `scripts/build.sh`**

```bash
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
```

- [ ] **Step 3: 设可执行权限**

```bash
chmod +x scripts/build.sh
```

- [ ] **Step 4: 实跑一次构建（基于真实 upstream）**

```bash
bash scripts/build.sh
ls dist/user/ dist/project/
ls dist/user/skills/superpowers/ | head
cat dist/user/rules/user_rules.md | head -30
ls dist/user/agents/
```

Expected：
- `dist/user/`、`dist/project/` 都存在，含 `rules/`、`skills/`、`agents/`
- `dist/user/rules/user_rules.md` 含 `## Skill Index` 段，下面是 14 个 `- <name> — <description>` bullet
- `dist/user/agents/` 含 `brainstorm.md`、`write-plan.md`、`execute-plan.md`、`code-reviewer.md`

- [ ] **Step 5: Commit 脚本 + helpers + 首版 dist 产物**

```bash
git add src/build_helpers.py scripts/build.sh dist/
git commit -m "feat(scripts): build.sh + build_helpers.py + 首版 dist 产物"
```

---

## Task 9: `tests/verify-build.sh` + 黑名单

**Files:**
- Create: `tests/verify-build.sh`
- Create: `tests/forbidden-strings.txt`

- [ ] **Step 1: 写 `tests/forbidden-strings.txt`**

```
Claude Code
hookSpecificOutput
hookEventName
CLAUDE_PLUGIN_ROOT
COPILOT_CLI
CURSOR_PLUGIN_ROOT
```

> 故意排除 `TaskCreate`/`Bash`/`Read`/`Edit`/`Write`——这些（按 §4.3 与 TBD-1）期望在 dist 中保留或被映射后保留可读字面。

- [ ] **Step 2: 写 `tests/verify-build.sh`**

```bash
#!/usr/bin/env bash
# 静态校验 dist/：frontmatter / 引用 / 黑名单 / size 预算 / mappings 覆盖率。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
BLACKLIST="$ROOT/tests/forbidden-strings.txt"

fail=0
warn=0

# 1. Frontmatter 完整性：每个 SKILL.md 都要有非空 name 和 description
echo "[1/5] Frontmatter completeness ..."
while IFS= read -r f; do
    if ! head -20 "$f" | grep -q "^name:"; then
        echo "  FAIL: missing 'name:' in $f"; fail=1
    fi
    if ! head -20 "$f" | grep -q "^description:"; then
        echo "  FAIL: missing 'description:' in $f"; fail=1
    fi
done < <(find "$DIST" -name SKILL.md)

# 2. 黑名单扫描
echo "[2/5] Forbidden strings ..."
while IFS= read -r pat; do
    [[ -z "$pat" ]] && continue
    hits=$(grep -rln "$pat" "$DIST" || true)
    if [[ -n "$hits" ]]; then
        echo "  FAIL: forbidden pattern '$pat' found in:"
        echo "$hits" | sed 's/^/    /'
        fail=1
    fi
done < "$BLACKLIST"

# 3. user_rules.md size 预算（≤4KB warning，≤6KB fatal）
echo "[3/5] user_rules.md size budget ..."
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

# 4. Mappings 覆盖率（检测 phrase_replacements 中的 from 在 upstream/ 是否命中过）
echo "[4/5] Mappings coverage ..."
PROJECT_ROOT="$ROOT" python3 - <<'PY'
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
rc=$?
if (( rc == 2 )); then
    warn=1
elif (( rc != 0 )); then
    fail=1
fi

# 5. dist/user 与 dist/project 关键文件一致性
echo "[5/5] user/ vs project/ structure parity ..."
if [[ ! -f "$DIST/user/rules/user_rules.md" ]]; then
    echo "  FAIL: missing dist/user/rules/user_rules.md"; fail=1
fi
if [[ ! -f "$DIST/project/rules/project_rules.md" ]]; then
    echo "  FAIL: missing dist/project/rules/project_rules.md"; fail=1
fi
if [[ -f "$DIST/project/rules/user_rules.md" ]]; then
    echo "  FAIL: dist/project should not contain user_rules.md"; fail=1
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
```

- [ ] **Step 3: 设可执行权限**

```bash
chmod +x tests/verify-build.sh
```

- [ ] **Step 4: 实跑一次**

```bash
bash tests/verify-build.sh
```

Expected：所有 5 项 PASS（或 PASS with warnings——某些 phrase_replacements 可能在 upstream 中 0 命中，那是 mappings 过时，不致命）。

- [ ] **Step 5: Commit**

```bash
git add tests/verify-build.sh tests/forbidden-strings.txt
git commit -m "test: verify-build.sh + 黑名单"
```

---

## Task 10: 端到端 fixture 测试 `tests/test_build_smoke.sh`

**目标：** 用 `tests/fixtures/mini-upstream/` 驱动一遍 build，确认产物结构正确。

**Files:**
- Create: `tests/test_build_smoke.sh`

- [ ] **Step 1: 写测试**

```bash
#!/usr/bin/env bash
# 在 tmp 工作区里以 fixture 为 upstream 跑一遍 build，断言关键产物。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

# 复制项目骨架到 tmp（不含 upstream/，避免污染）
mkdir -p "$TMP/src" "$TMP/upstream" "$TMP/dist"
cp -r "$ROOT/src/"* "$TMP/src/"
# 把 fixture 当作 upstream
cp -r "$ROOT/tests/fixtures/mini-upstream/"* "$TMP/upstream/"
echo "fixture-0" > "$TMP/upstream/VERSION"

cd "$TMP"
PYTHONPATH="$TMP/src" python3 "$TMP/src/build_helpers.py" "$TMP"

# 断言
assert_file() { [[ -f "$1" ]] || { echo "MISSING: $1"; exit 1; }; }
assert_grep() { grep -q "$1" "$2" || { echo "GREP FAIL '$1' in $2"; exit 1; }; }

assert_file "$TMP/dist/user/rules/user_rules.md"
assert_file "$TMP/dist/project/rules/project_rules.md"
assert_file "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
assert_file "$TMP/dist/user/skills/superpowers/fake-beta/SKILL.md"
assert_file "$TMP/dist/user/skills/superpowers/references/trae-tools.md"
assert_file "$TMP/dist/user/agents/code-reviewer.md"

# 转换效果验证
assert_grep "Trae IDE"  "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
! grep -q "Claude Code" "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
assert_grep "trae-tools.md" "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"
! grep -q "copilot-tools.md" "$TMP/dist/user/skills/superpowers/fake-alpha/SKILL.md"

# bootstrap 索引应含两个 fake skill
assert_grep "fake-alpha" "$TMP/dist/user/rules/user_rules.md"
assert_grep "fake-beta"  "$TMP/dist/user/rules/user_rules.md"

# project_rules.md 不应有 user_rules.md 同名残留
[[ ! -f "$TMP/dist/project/rules/user_rules.md" ]] || { echo "stray user_rules.md in project/"; exit 1; }

echo "test_build_smoke: PASS"
```

- [ ] **Step 2: 设可执行权限并跑**

```bash
chmod +x tests/test_build_smoke.sh
bash tests/test_build_smoke.sh
```

Expected：`test_build_smoke: PASS`。

- [ ] **Step 3: Commit**

```bash
git add tests/test_build_smoke.sh
git commit -m "test: 端到端 fixture build smoke"
```

---

## Task 11: `scripts/install.sh`——安装器

**Files:**
- Create: `scripts/install.sh`

- [ ] **Step 1: 写 `scripts/install.sh`**

```bash
#!/usr/bin/env bash
# 把 dist/user/ 装到 ~/.trae/，或 dist/project/ 装到 <project>/.trae/。
# 策略：覆盖 + 自动备份 .bak.YYYYMMDD-HHMMSS；写日志便于回滚。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

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
```

- [ ] **Step 2: 设可执行权限并 dry-run 到 tmp**

```bash
chmod +x scripts/install.sh
TMPHOME=$(mktemp -d)
HOME="$TMPHOME" bash scripts/install.sh --user
ls "$TMPHOME/.trae/"
cat "$TMPHOME/.trae/.superpowers-install.log"
rm -rf "$TMPHOME"
```

Expected：`$TMPHOME/.trae/` 含 `rules/`、`skills/`、`agents/`，log 文件列出每个 INSTALL 行。

- [ ] **Step 3: 测项目级安装（再 dry-run）**

```bash
TMP_PROJ=$(mktemp -d)
bash scripts/install.sh --project "$TMP_PROJ"
ls "$TMP_PROJ/.trae/rules/"
[[ -f "$TMP_PROJ/.trae/rules/project_rules.md" ]] && echo "OK: project_rules.md present"
[[ ! -f "$TMP_PROJ/.trae/rules/user_rules.md" ]] && echo "OK: user_rules.md absent"
rm -rf "$TMP_PROJ"
```

Expected：`project_rules.md` 存在，`user_rules.md` 不存在。

- [ ] **Step 4: 测覆盖+备份语义**

```bash
TMPH=$(mktemp -d)
HOME="$TMPH" bash scripts/install.sh --user
HOME="$TMPH" bash scripts/install.sh --user      # 第二次：应产生 .bak.* 备份
ls "$TMPH/.trae/rules/" | grep "\.bak\." || { echo "FAIL: backup not created"; exit 1; }
echo "OK: backup created on second install"
rm -rf "$TMPH"
```

Expected：`OK: backup created on second install`。

- [ ] **Step 5: Commit**

```bash
git add scripts/install.sh
git commit -m "feat(scripts): install.sh + 覆盖备份语义"
```

---

## Task 12: README.md 完整版 + manual-smoke-test.md

**Files:**
- Modify: `README.md`（替换占位）
- Create: `docs/superpowers/manual-smoke-test.md`

- [ ] **Step 1: 写完整 README.md**

```markdown
# cc-superpower-to-trae

把 [Superpowers](https://github.com/obra/superpowers) 整套方法论（14 skills + 3 commands + code-reviewer agent + SessionStart hook）移植到 [Trae IDE](https://www.trae.ai/)。

## 目录

- 设计文档：[docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md](docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md)
- 实施计划：[docs/superpowers/plans/2026-04-28-superpowers-to-trae.md](docs/superpowers/plans/2026-04-28-superpowers-to-trae.md)
- 手工冒烟测试：[docs/superpowers/manual-smoke-test.md](docs/superpowers/manual-smoke-test.md)

## 快速开始

### 1. 同步 upstream（首次或升级时）

```bash
bash scripts/sync-upstream.sh
```

会把 `~/.claude/plugins/cache/claude-plugins-official/superpowers/<最新版>/` 拷贝到 `upstream/`。或显式指路径：

```bash
bash scripts/sync-upstream.sh /path/to/superpowers/5.0.7
```

### 2. 构建产物

```bash
bash scripts/build.sh
```

输出 `dist/user/` 与 `dist/project/`。

### 3. 安装

**用户级**（全局，所有 Trae 项目都吃到）：

```bash
bash scripts/install.sh --user
```

**项目级**（仅特定项目）：

```bash
bash scripts/install.sh --project /path/to/your/project
```

两种方式都会在目标位置已有同名文件时自动备份为 `<file>.bak.YYYYMMDD-HHMMSS`，并写日志到 `<dest>/.trae/.superpowers-install.log`。

## 升级流程

```bash
git checkout -b upgrade-superpowers-<新版本号>
bash scripts/sync-upstream.sh
git diff upstream/                # review upstream 自身的变化
bash scripts/build.sh             # 重新生成 dist/
git diff dist/                    # review 转换后产物的变化
# 必要时调整 src/mappings.json 或模板，回到 build.sh
```

## 已知 TBD

`src/mappings.json` 中含 `<TBD-Trae-X>` 占位（如 `TaskCreate` 的 Trae 等价）；`src/trae-tools-reference.md` 中含 `(TBD)` 标记。这些工具名需在能跑 Trae IDE 的环境中验证一次后填入。详见设计文档 §11。

## 项目结构

| 路径 | 说明 |
|---|---|
| `upstream/` | superpowers 原版源（入 git，便于 diff 升级） |
| `src/` | 转换知识：mappings.json + 模板 + transform.py / render.py |
| `scripts/` | sync-upstream.sh / build.sh / install.sh |
| `dist/` | 构建产物（入 git） |
| `tests/` | 单测 + 端到端 fixture smoke |
| `docs/superpowers/` | 设计文档、实施计划、手工测试步骤 |

## License

本项目仅做工具链与转换器；移植的 superpowers 内容版权归原作者所有，详见 `upstream/LICENSE`。
```

- [ ] **Step 2: 写 `docs/superpowers/manual-smoke-test.md`**

```markdown
# 手工冒烟测试

verify-build.sh 仅做静态校验。这份文档列出在 Trae IDE 上的真机验证步骤。

## 前置

- 已装 [Trae IDE](https://www.trae.ai/)
- 在本仓库跑过 `bash scripts/build.sh && bash tests/verify-build.sh`，无 FAIL

## 验证 1：用户级安装能被 Trae 识别

1. `bash scripts/install.sh --user`
2. 启动 Trae IDE，新建/打开任意项目
3. 在 Agent 聊天框输入：「我想做一个 X 工具，帮我设计一下」
4. **预期**：Agent 在回复前调用 Read 工具读取 `~/.trae/skills/superpowers/brainstorming/SKILL.md`（在工具调用日志中可见），然后按 brainstorming 流程逐个澄清问题，而非直接给方案。

## 验证 2：Custom Agent 触发

1. 在聊天框打 `@brainstorm`（或 Trae UI 的 Agent 选择菜单选 brainstorm）
2. **预期**：Agent 立刻读 `brainstorming/SKILL.md`，开始 brainstorming 流程。

## 验证 3：项目级安装与 user_rules 不冲突

1. `bash scripts/install.sh --project /path/to/test-proj`
2. 在 Trae IDE 打开 `/path/to/test-proj`
3. 验证 `.trae/rules/project_rules.md` 存在、`.trae/skills/superpowers/` 存在
4. **预期**：项目级 rules 与可能存在的用户级 rules 共存（按 Trae 文档，project rules 在冲突时优先）。

## 验证 4：升级路径

1. 改一行 `upstream/skills/brainstorming/SKILL.md`，重跑 `build.sh`
2. **预期**：`dist/user/skills/superpowers/brainstorming/SKILL.md` 反映改动；`git diff dist/` 能看到。

## 验证 5：卸载（手工）

1. 找到 `<dest>/.trae/.superpowers-install.log` 中所有 `INSTALL:` 行
2. 删除对应文件；如有 `BACKUP:` 行，将 `.bak.*` 文件 mv 回原名
3. **预期**：恢复到安装前状态。

如以上任一验证失败，记录失败步骤与现象到 GitHub issue 或团队渠道。
```

- [ ] **Step 3: Commit**

```bash
git add README.md docs/superpowers/manual-smoke-test.md
git commit -m "docs: 完整 README + 手工冒烟测试步骤"
```

---

## Self-Review

完成所有 12 个 Task 后，再做一次端到端：

- [ ] 跑全部测试

```bash
python3 -m unittest discover tests -v
bash tests/test_build_smoke.sh
bash scripts/sync-upstream.sh
bash scripts/build.sh
bash tests/verify-build.sh
```

Expected：全部 PASS（verify-build 可能 PASS with warnings 因 mappings 覆盖率，但不应 FAIL）。

- [ ] 把当前 spec 与 plan 中提到的"产物"逐一勾对：
  - [x] upstream/ 入 git ✔（Task 7）
  - [x] dist/user 与 dist/project 入 git ✔（Task 8）
  - [x] bootstrap rule 由模板生成 ✔（Task 8）
  - [x] 14 skills 转换 + references/trae-tools.md 注入 ✔（Task 8）
  - [x] 4 个 Custom Agent（含 code-reviewer） ✔（Task 8）
  - [x] verify-build 含 frontmatter / 黑名单 / size / 覆盖率 / 结构对齐 ✔（Task 9）
  - [x] install.sh 双形态 + 覆盖+备份 ✔（Task 11）
  - [x] manual-smoke-test.md ✔（Task 12）

如发现遗漏，回到对应 Task 补步骤；如 mappings.json 中某 `from` 在升级后 0 命中，由人工删除该条目（不阻塞构建）。
