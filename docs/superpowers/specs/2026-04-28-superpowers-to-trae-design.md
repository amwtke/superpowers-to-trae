# Superpowers → Trae IDE 移植：设计文档

- 日期：2026-04-28
- 状态：Draft（待用户复审）
- Upstream：`superpowers` 5.0.7（来源 `claude-plugins-official` 插件缓存）
- 目标平台：Trae IDE（ByteDance）

## 1. 背景与问题

`superpowers` 是 Claude Code / Cursor / Copilot CLI / Gemini CLI 上一套以 SKILL.md 为核心的方法论插件，提供 brainstorming、TDD、systematic-debugging、writing-plans 等 14 个工作流 skill，配合 SessionStart hook 自动建立"如何使用 skill"的元规则。

Trae IDE（ByteDance 的 AI 编辑器）虽然有 `.trae/skills/` 目录概念，但其内置 skill 加载器只识别 `npx skills` CLI 生态，**不能直接解析 superpowers 的 SKILL.md+YAML frontmatter 文件夹结构**（参考 [Trae-AI/TRAE#2253](https://github.com/Trae-AI/TRAE/issues/2253)，开放中无官方进展）。同时 Trae 没有 SessionStart hook，需要用 `.trae/rules/` 的"始终在线"机制等价替代。

因此需要一个移植项目：在 Trae IDE 上获得与 Claude Code 上 superpowers 等效的体验。

## 2. 目标与非目标

**目标**

1. 14 个 superpowers skills 在 Trae IDE 上可用，**保持按需加载**（非启动注入全文）。
2. 3 条 superpowers slash 命令（`/brainstorm`、`/write-plan`、`/execute-plan`）在 Trae 上以 `@`-mention Custom Agent 形式可用。
3. `code-reviewer` 子 Agent 移植到 Trae Custom Agent。
4. SessionStart hook 用 `user_rules.md` / `project_rules.md` 等价替代——Agent 启动即知"如何发现并使用 skill"。
5. 工具名差异（Claude Code → Trae IDE）通过 `references/trae-tools.md` + 转换器映射统一处理。
6. 整套是**可重跑的 porting 流水线**：upstream 升级（5.0.8 / 6.x）只需重跑同步与构建。
7. 同时产出 `dist/user/`（拷到 `~/.trae/`）和 `dist/project/`（拷到 `<project>/.trae/`）两种安装形态。

**非目标**

- 不翻译 skills 内容（保留英文原文，避免语义漂移）。
- 不复刻 upstream 的 `tests/`（那是上游 skill 自测，与本项目转换流程无关）。
- 不复刻 Windows 平台的 `hooks/run-hook.cmd`（Trae 无 hook 概念）。
- 不端到端跑 LLM 验证（提供静态校验 + 手工 smoke test 文档）。
- 不做 hook → Trae 的运行时桥接（rules 已是等价方案）。

## 3. 顶层架构

```
cc-superpower-to-trae/
├── upstream/                          # superpowers 原版源码（入 git）
│   ├── skills/  commands/  agents/  hooks/  README.md  CHANGELOG.md
│   └── VERSION                        # 当前同步的版本号（如 "5.0.7"）
├── src/                               # 转换"知识"——人工维护
│   ├── mappings.json                  # 工具名映射 + 字符串替换 + 跳过清单
│   ├── bootstrap-rule.template.md     # using-superpowers 的 Trae 等价物模板
│   ├── agent-wrapper.template.md      # @brainstorm 等 Custom Agent 模板
│   └── trae-tools-reference.md        # Trae 工具速查（被 skills 引用）
├── scripts/
│   ├── sync-upstream.sh               # 拉取/更新 upstream/
│   ├── build.sh                       # upstream/ + src/ → dist/
│   └── install.sh                     # dist/ → ~/.trae/ 或 <project>/.trae/
├── dist/                              # 构建产物（入 git，便于 PR 中 diff）
│   ├── user/                          # cp -r * ~/.trae/
│   │   ├── rules/user_rules.md
│   │   ├── skills/superpowers/<14 个 skill 子目录>
│   │   ├── skills/superpowers/references/trae-tools.md
│   │   └── agents/{brainstorm,write-plan,execute-plan,code-reviewer}.md
│   └── project/                       # cp -r .trae/ <target>/
│       └── (镜像 user/，仅 rules/user_rules.md 改名为 rules/project_rules.md)
├── tests/
│   ├── verify-build.sh                # 静态校验 build 产物
│   └── forbidden-strings.txt          # Claude-only 字眼黑名单
└── docs/superpowers/
    ├── specs/2026-04-28-superpowers-to-trae-design.md  ← 本文档
    └── manual-smoke-test.md           # 手工冒烟测试步骤
```

**核心数据流**

```
sync-upstream.sh  →  upstream/
                       ↓  （读 src/mappings.json + templates）
                    build.sh
                       ↓
                    dist/{user,project}/
                       ↓
                    install.sh [--user | --project <path>]
                       ↓
                    ~/.trae/  或  <target>/.trae/
```

**关键设计点**

- `dist/` 入 git——任何转换变化都能在 PR 中 review。
- `mappings.json` 与各 `*.template.md` 是唯二需要人工调整的"知识"；`build.sh` 本身只做机械替换与渲染。
- 默认保留 skills 英文原文；bootstrap rule 与 README 用中文。

## 4. 关键组件

### 4.1 Bootstrap Rule（替代 SessionStart hook）

`dist/user/rules/user_rules.md`（项目级版同形，文件名为 `project_rules.md`）：

- frontmatter `description: Superpowers methodology - skill discovery and use protocol`
- 正文包含：
  - "你拥有 superpowers" 元规则
  - 「The Rule」：用户请求匹配某 skill 的 `description` 时必须先读 SKILL.md 后再行动
  - **Skill 索引**：14 个 skill 的 `name + 一句话 description`（约 1 KB），由 `build.sh` 扫描 SKILL.md frontmatter 自动生成
  - "如何加载 skill"：用 Read 工具读 `.trae/skills/superpowers/<name>/SKILL.md` 并直接执行
  - 工具名映射指向 `.trae/skills/superpowers/references/trae-tools.md`

**大小预算**：≤ 4 KB 为软目标（始终在线，需克制），> 6 KB 视为构建失败。具体阈值与执行强度见 §9.4。

### 4.2 Skills 目录（核心，14 个）

`dist/user/skills/superpowers/<skill>/SKILL.md` —— 由 `build.sh` 从 `upstream/skills/` 拷贝并经三类机械改写：

| upstream 写法 | 转换后 |
|---|---|
| `Use the Skill tool to invoke...` | `Read the file .trae/skills/superpowers/<name>/SKILL.md and follow it.` |
| 引用 `references/copilot-tools.md` / `codex-tools.md` | 替换为 `references/trae-tools.md` |
| `TaskCreate` / `TaskUpdate` 等工具名 | 替换为 Trae 等价工具名（见 4.3 与 §11 TBD-1）|
| Claude-Code-only 措辞（`Claude Code`、`hookSpecificOutput` 等） | 删/改 |

skill 子目录内其他文件（`references/*.md`、`visual-companion.md`、`assets/*` 等）原样保留。

### 4.3 `references/trae-tools.md`（工具名映射表）

`dist/user/skills/superpowers/references/trae-tools.md`——单一来源的 Trae 工具速查：

- 表格列出 `skill 文档里的 X` → `Trae 里实际可调用的 Y`
- 不可用工具（如 Trae 无对应 WebSearch）显式标注 "unavailable; do equivalent manually"
- 由 `src/trae-tools-reference.md` 拷贝而来（人工维护）

具体工具名见 §11 TBD-1。

### 4.4 Custom Agents（替代 slash commands + code-reviewer）

`dist/user/agents/<name>.md`——Trae Custom Agent 文件，**轻量壳，委托读 SKILL.md**：

```markdown
---
name: brainstorm
description: Turn an idea into a design through collaborative dialogue
---

You are running the brainstorming workflow. Read
`.trae/skills/superpowers/brainstorming/SKILL.md` and follow it exactly.

Skills evolve — always read the file fresh, do not work from memory.
```

四个 Agent：

- `brainstorm` → `brainstorming` skill
- `write-plan` → `writing-plans` skill
- `execute-plan` → `executing-plans` skill
- `code-reviewer` → 直接从 `upstream/agents/code-reviewer.md` 转换（保留原 system prompt，仅工具名替换）

**为何不把 skill 全文塞进 Agent system prompt**：双写每次升级要改两处，单一来源更稳。

### 4.5 `src/mappings.json`（驱动 build.sh 的配置）

```json
{
  "tool_name_replacements": {
    "TaskCreate": "...",
    "TaskUpdate": "...",
    "Bash":       "..."
  },
  "phrase_replacements": [
    { "from": "Claude Code", "to": "Trae IDE" },
    { "from": "use the Skill tool to invoke", "to": "read the SKILL.md file directly" },
    { "from": "references/copilot-tools.md",  "to": "references/trae-tools.md" },
    { "from": "references/codex-tools.md",    "to": "references/trae-tools.md" }
  ],
  "files_to_skip": [
    "hooks/", "scripts/", "tests/", "package.json",
    "AGENTS.md", "CLAUDE.md", "GEMINI.md", "gemini-extension.json"
  ],
  "agents_to_generate": [
    { "name": "brainstorm",    "skill": "brainstorming" },
    { "name": "write-plan",    "skill": "writing-plans" },
    { "name": "execute-plan",  "skill": "executing-plans" }
  ]
}
```

转换器只读这个 JSON 决定行为，不在脚本里写死规则——新增映射仅改 JSON。

## 5. Build 流程（`scripts/build.sh`）

幂等，可重复执行。流程：

1. **清理 dist/**：`rm -rf dist/{user,project}/` 后建 `rules/`、`skills/superpowers/`、`agents/` 三个子目录。
2. **拷贝 skills**：`upstream/skills/<name>/` → `dist/user/skills/superpowers/<name>/`，对所有 `*.md` 跑 `mappings.json` 的替换。
3. **注入 trae-tools.md**：`cp src/trae-tools-reference.md dist/user/skills/superpowers/references/trae-tools.md`。
4. **生成 bootstrap rule**：渲染 `src/bootstrap-rule.template.md`，扫描所有 SKILL.md 的 frontmatter 填入 `{{ skill_index }}`，输出 `dist/user/rules/user_rules.md`。
5. **生成 Custom Agents**：对 `mappings.json` 的 `agents_to_generate` 中每项渲染 `src/agent-wrapper.template.md`，输出 `dist/user/agents/<name>.md`。
6. **转换 code-reviewer**：对 `upstream/agents/code-reviewer.md` 跑替换 → `dist/user/agents/code-reviewer.md`。
7. **镜像 user/ → project/**：整树拷贝，仅 `rules/user_rules.md` 改名为 `rules/project_rules.md`。
8. **跑 verify**：`bash tests/verify-build.sh`，失败则 build 失败。

## 6. Install 流程（`scripts/install.sh`）

```
用法：
  bash install.sh --user                    # 装到 ~/.trae/
  bash install.sh --project /path/to/proj   # 装到 <project>/.trae/
```

策略：**覆盖 + 自动备份**。

- 目标路径已存在同名文件（如 `user_rules.md`、`agents/brainstorm.md`、`skills/superpowers/`）→ 备份为 `<file>.bak.YYYYMMDD-HHMMSS`，再写入新内容；屏幕打印备份位置。
- 操作日志写到 `~/.trae/.superpowers-install.log`（或 `<project>/.trae/.superpowers-install.log`），列出本次写入与备份的所有路径，便于回滚。
- 安装中途失败：日志已记录已写入路径，便于人工反向操作（不实现自动回滚）。

## 7. Upstream 升级流程

```
git checkout -b upgrade-superpowers-<version>
bash scripts/sync-upstream.sh           # 拉新版 → upstream/
git diff upstream/                      # review upstream 自身的变化
bash scripts/build.sh                   # 重新生成 dist/
git diff dist/                          # review 转换后产物的变化（抓 mappings 漏改）
bash tests/verify-build.sh              # 静态校验
# 必要时调整 src/mappings.json，回到 build.sh 那步
# 一切 OK → 合并、打 tag
```

`build.sh` 在结束前会跑一次 `grep -nE "<黑名单>" dist/`，发现漏改的 Claude-only 字眼立即列出并以非零状态退出。

## 8. 错误处理

| 失败点 | 处理 |
|---|---|
| `sync-upstream.sh` 找不到 superpowers cache 目录 | 报错并提示用户先安装 superpowers，或显式给路径参数 |
| `build.sh` 发现 `mappings.json` 中某 `from` 在 upstream 里 0 命中 | warning（条目可能过时），不终止 |
| `build.sh` 在 dist 里检出黑名单字眼 | 列文件:行号，**非零退出**；build 不通过 |
| `install.sh` 目标路径有同名文件 | 备份为 `<file>.bak.<时间戳>` 后覆盖；屏幕提示备份位置 |
| `install.sh` 中途失败 | 已写入路径已记入 install log，便于人工回滚 |
| skill 内的 `references/copilot-tools.md` / `codex-tools.md` 引用 | mappings 中替换为 `trae-tools.md`；upstream 原文件保留作 fallback |

不做：upstream 升级时的"自动迁移 mappings"——脆弱，必须人工 review。

## 9. 测试

`tests/verify-build.sh`——纯静态校验，不调 LLM：

1. **Frontmatter 完整性**：每个 `dist/**/SKILL.md` 必须有非空 `name` 与 `description` 字段。
2. **链接/引用完整性**：grep 出 markdown 链接 `[*](path)` 与 `Read the file <path>` 形式的相对引用，逐条 stat 检查文件存在。
3. **黑名单扫描**：对 `tests/forbidden-strings.txt` 中每条做 `grep -rn`，命中即失败。初始包含：`Claude Code`、`hookSpecificOutput`、`hookEventName`、`CLAUDE_PLUGIN_ROOT`、`SubagentStop`、原始未替换的工具名等。
4. **`user_rules.md` 大小预算**：≤ 4 KB，超出告警（不致命）；≤ 6 KB 致命。
5. **`mappings.json` 覆盖率**：每个 `phrase_replacements[].from` 在 `upstream/` 的命中次数；为 0 时给 warning（条目过时）。

不做：端到端 LLM 验证。提供 `docs/superpowers/manual-smoke-test.md` 列最小手工验证步骤（装一次 → 在 Trae IDE 里说"帮我设计 X" → 确认 Agent 读了 brainstorming SKILL.md）。

## 10. 不做（明确边界）

- 不翻译 skills；中文仅出现在 bootstrap rule（必要部分）、README、install.sh 提示。
- 不复刻 upstream 的 `tests/`（与本项目无关）。
- 不复刻 Windows hook 适配。
- 保留 skills 内 `<SUBAGENT-STOP>` 等 Claude-Code-only 标签——LLM 见到会忽略，无害。
- 不做 Trae IDE 内的端到端自动验证（依赖外部 GUI，超本仓库范围）。

## 11. 已知 TBD（不阻塞 spec，留待 plan 阶段）

**TBD-1：Trae IDE Agent 实际可调用工具的精确名字**

影响 `src/mappings.json` 的 `tool_name_replacements` 与 `src/trae-tools-reference.md` 的内容。需要：

- 读 [docs.trae.ai/ide/agent](https://docs.trae.ai/ide/agent) 全文（WebFetch 当前对此页有限制，可能要换路径）
- 必要时实际安装 Trae IDE 并查看 Agent 暴露的工具清单
- 待映射工具至少包括：Read / Edit / Write / Bash / TaskCreate / TaskUpdate / TaskList / WebSearch / WebFetch / Skill 调用语义

**TBD-2：Trae IDE rules 是否有官方大小硬限**

影响 `tests/verify-build.sh` 的 size 预算阈值。当前暂定 ≤ 4 KB（warning）/ ≤ 6 KB（致命）。

两个 TBD 都是"内容填充"问题而非"架构决策"问题，骨架先成型即可推进。

## 12. 验收标准

- 在干净环境跑 `sync-upstream.sh && build.sh && tests/verify-build.sh` 全部通过。
- `dist/user/` 与 `dist/project/` 都生成了完整的 `rules/`、`skills/superpowers/`、`agents/`。
- `dist/user/rules/user_rules.md` 大小 ≤ 4 KB。
- `tests/forbidden-strings.txt` 列表在 `dist/` 中 0 命中。
- 手工冒烟测试：在一台装有 Trae IDE 的机器上 `bash install.sh --user`，新开 Trae 会话说一句"帮我设计 X"，能观察到 Agent 主动 Read `.trae/skills/superpowers/brainstorming/SKILL.md`。
