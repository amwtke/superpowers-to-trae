# DDD plugin v0.2.0：设计文档

- 日期：2026-04-29
- 状态：Draft（待用户复审）
- 子项目：sub-project 2 of 3
- 上一版（v0.1.0）：Rust CLI 三件套 + DDD 仅 stub 占位
- 下一版（v0.3.x）：sub-project 3 — 文档收尾 + roadmap

## 1. 背景与问题

v0.1.0 已交付 Rust CLI `superpowers-trae`，提供 init/upgrade/status 三命令，把 14 个 superpowers skills + project_rules.md + AGENTS.md 装到目标项目。`--addons ddd` flag 已预留但**仅打印 "not yet implemented" 后继续**——不安装任何 DDD 内容。

参考项目 `~/workshop/ddd-run`（Rust + clap CLI）已有完整 DDD harness：
- 3 个 skill（`ddd-storm` / `ddd-model` / `ddd-spec`，共约 550 行 markdown）
- 3 个 root 文档（CLAUDE.md / DOMAIN.md / README-DDD-HARNESS.md，共约 505 行）
- 工作流：事件风暴 → 领域建模 → spec → 移交给 Superpowers brainstorming

ddd-run 当前装到 `.claude/skills/<name>/SKILL.md`（Claude Code 路径），不适合 Trae IDE 用。本设计把 ddd-run 的方法论 portable 进 superpowers-trae，作为 v0.2.0 的 `--addons ddd` 真实现。

## 2. 目标与非目标

**目标（v0.2.0）**

1. `superpowers-trae init --addons ddd` 在目标项目装：
   - `.trae/skills/ddd/<name>/SKILL.md`（3 个：ddd-storm / ddd-model / ddd-spec）
   - `<dir>/DOMAIN.md`（项目根，**install-once 语义**）
   - `<dir>/README-DDD-HARNESS.md`（项目根）
   - AGENTS.md 末尾追加 "DDD methodology" 段（含约束 + 调用方式）
2. `superpowers-trae upgrade --addons ddd` 刷新 DDD：
   - 备份 + 覆盖 SKILL.md（3 个）和 README-DDD-HARNESS.md
   - **DOMAIN.md 跳过**（用户数据，永不覆盖）
   - AGENTS.md 重新拼接（含 DDD 段）
3. `superpowers-trae upgrade`（不带 `--addons`）：只刷 superpowers，**不动 DDD 文件**
4. `superpowers-trae status`：扫到 `.trae/skills/ddd/` 时打印 `Addons:` 段，列 ddd 完整性（3 skills / DOMAIN.md / README）
5. v0.1 的所有现有行为保持兼容（不带 `--addons` 时输出与 v0.1 完全一致）
6. `Addon` trait 真实现，作为未来 plugin 的可扩展接口
7. cli 版本号升 0.2.0；Cargo.toml metadata 记录 ddd-run 同步的 commit
8. 集成测试覆盖 6 个新场景（DDD 装入 / DOMAIN.md install-once / upgrade 不动 ddd / 等）

**非目标（留 v0.3 / 后续）**

- DDD plugin 卸载（`remove ddd` 子命令）
- 按 plugin 升级（只刷 ddd 不刷 superpowers）
- 多 plugin 并存（v0.2 只 ddd 一个；trait 已为多 plugin 留接口）
- ddd-run 的工作目录 `docs/ddd/` / `docs/specs/` 自动创建（用户首次跑 ddd-storm 时自然创建即可）
- 把 ddd-run CLAUDE.md 完整内容并入 AGENTS.md（只摘核心约束；非约束的"项目定位"/"技术栈占位"等用户自填字段留 DOMAIN.md 处理）

## 3. 顶层架构

```
cli/
├── Cargo.toml                          # 修改：version 0.1.0 → 0.2.0；metadata 加 ddd_run_version
├── src/
│   ├── main.rs                         # 不动（addons flag 已在 v0.1）
│   ├── embed.rs                        # 修改：导出已有 DIST_USER；ddd 的嵌入挪到 addons/ddd.rs（局部）
│   ├── agents_md.rs                    # 修改：render_agents_md 接受 addons 参数 → 末尾追加 DDD 段
│   ├── rollback.rs                     # 不动（write 已支持 backup_existing）
│   ├── addons/
│   │   ├── mod.rs                      # 修改：trait 加 install + agents_md_segment；handle_addons → resolve_addons
│   │   └── ddd.rs                      # ★ 新增：DddAddon impl Addon
│   └── commands/
│       ├── init.rs                     # 修改：把 addons 真分发到 install
│       ├── upgrade.rs                  # 不动签名（已传 addons_list）
│       └── status.rs                   # 修改：扫 .trae/skills/ddd/ + DOMAIN.md
│
├── templates/                          # ★ 新增（与 cli/src/ 平级）
│   └── ddd/
│       ├── skills/
│       │   ├── ddd-storm.md            # 99 行（从 ddd-run 拷贝）
│       │   ├── ddd-model.md            # 180 行
│       │   └── ddd-spec.md             # 269 行
│       └── root/
│           ├── DOMAIN.md               # 120 行（install-once）
│           ├── README-DDD-HARNESS.md   # 206 行
│           └── claude-md-merge.md      # ★ 新增（手工抽取自 ddd-run CLAUDE.md），约束段落，并入 AGENTS.md
└── tests/
    ├── integration_init.rs             # 不动
    ├── integration_upgrade.rs          # 不动
    ├── integration_status.rs           # 修改：加一个 with-ddd 用例
    └── integration_addons_ddd.rs       # ★ 新增（6 个端到端测试）
```

**核心数据流**

maintainer 同步 ddd-run（手动；ddd-run 升级时重做）:
```
~/workshop/ddd-run/src/templates/skills/*.md      → cli/templates/ddd/skills/
~/workshop/ddd-run/src/templates/root/DOMAIN.md   → cli/templates/ddd/root/DOMAIN.md
~/workshop/ddd-run/src/templates/root/README-DDD-HARNESS.md → cli/templates/ddd/root/README-DDD-HARNESS.md
人工抽取 ~/workshop/ddd-run/src/templates/root/CLAUDE.md → cli/templates/ddd/root/claude-md-merge.md
更新 cli/Cargo.toml [package.metadata.ddd_run_version] = "<ddd-run commit short hash>"
```

end-user 用 DDD plugin:
```
superpowers-trae init --addons ddd        →  rules + skills/superpowers + skills/ddd + AGENTS.md(含 DDD 段) + DOMAIN.md + README-DDD-HARNESS.md
superpowers-trae upgrade --addons ddd     →  刷新 superpowers + 刷新 ddd skills + README；DOMAIN.md 跳过；AGENTS.md 重拼
superpowers-trae upgrade                  →  仅刷 superpowers，DDD 文件不动
superpowers-trae status                   →  Addons: ddd ✓ 3/3 / DOMAIN.md / README
```

**关键设计点**

- DDD 装到独立 namespace `.trae/skills/ddd/`（与 superpowers 平级）—— 与 v0.1 设计一致，按 plugin 隔离生命周期
- DOMAIN.md 是 **用户数据**，install-once 语义——存在则跳过，不备份不覆盖
- AGENTS.md 由 binary 在 init/upgrade 时**动态拼接**——基础 superpowers DIRECTIVE + skill index + project_rules.md 正文 + 已装 addon 的 segment（DDD 段从 `claude-md-merge.md` 嵌入）
- v0.1 的不带 `--addons` 行为完全不变——v0.1 集成测试 100% 仍通过
- `Addon` trait 真实现：v0.2 一个具体 plugin（ddd），但 trait 设计支持未来多 plugin（test-stories / kernel-trace 等）

## 4. 命令面与行为

v0.1 的 `init` / `upgrade` / `status` 三命令面**完全保留**，命令名/flag 不变。仅以下行为扩展：

### 4.1 `init --addons ddd`

预检不变（与 v0.1 同——dir 存在/已是目录、未初始化）。`addons::resolve_addons(addons_list)` 把 `["ddd"]` 解析成 `Vec<Box<dyn Addon>>`；未知名字立即 stderr + exit 1（v0.1 已有此行为，v0.2 真实装载 ddd）。

写入清单（按顺序）：

1. `.trae/rules/project_rules.md` ← v0.1 一致
2. `.trae/skills/superpowers/<14 dirs>/...` ← v0.1 一致
3. **新增：addon 驱动的写入**——遍历 `addon_list`，每个 addon 调 `install(dir, backup_existing=false, &mut session)`：
   - `DddAddon::install`:
     - 3 个 skill：`.trae/skills/ddd/<name>/SKILL.md`（来自 `templates/ddd/skills/<name>.md`，文件名做 stem 转换）
     - `README-DDD-HARNESS.md`（项目根）
     - `DOMAIN.md`（项目根，install-once：不存在才写，存在则打印 `ℹ DOMAIN.md preserved (user-managed).`）
4. `AGENTS.md` ← 动态拼接（含 addon segment）
5. `.trae/.superpowers-install.log` ← 一行 epoch 头

末尾输出维持 v0.1 格式：`✓ Initialized superpowers methodology in <dir>` + 调用方式提示。

### 4.2 `upgrade --addons ddd`

预检：`.trae/rules/project_rules.md` 必须存在（v0.1 已有）。

写入清单同 4.1，**关键差异**：所有 addon `install()` 的 `backup_existing` 参数为 `!no_backup`（默认 true，`--no-backup` 时 false）。
- 3 个 ddd skill 文件：备份 + 覆盖（用户改过的会被备份成 `.bak.<时间戳>`）
- README-DDD-HARNESS.md：备份 + 覆盖
- DOMAIN.md：**仍然跳过**（install-once 语义无视 backup_existing；这是 DDD 的特殊语义，硬编码在 `DddAddon::install` 中）

不带 `--addons` 时 `addon_list` 为空，循环 0 次，DDD 文件原状不动。

### 4.3 `upgrade`（不带 `--addons`）行为澄清

**核心兼容性约束**：v0.1 用户没装 DDD 跑 `upgrade` 必须等价 v0.1。
**新增约束**：用户装了 DDD 后跑 `upgrade`（不带 flag），DDD 文件不变。

实现保证：`addon_list` 来自 CLI flag 解析；不带 flag → 空 vec → 不调任何 `install()`。

### 4.4 `status`

新增 "Addons" 段（仅在检测到任一 addon 装入时打印；无 addon 时输出 v0.1 的 `Addons: none`）。

完整输出例：

```
superpowers-trae v0.2.0 (embedded superpowers 5.0.7, ddd plugin v0.2.0)
Project: /home/xiaojin/Documents/trae_projects/test-superpowers
─────────────────────────────────────────────────────────
✓ project_rules.md         (.trae/rules/project_rules.md, 3.3 KB)
✓ superpowers skills       14 / 14 expected
✓ AGENTS.md                (project root, 9.8 KB)
ℹ Last init/upgrade        epoch 1777368326

Addons:
  ✓ ddd
    - skills:    3 / 3 (storm/model/spec)
    - DOMAIN.md  (project root, user-managed)
    - README-DDD-HARNESS.md (project root)
```

部分装（用户删了 DOMAIN.md，或 ddd skill 缺失）→ `⚠ ddd (incomplete)` 标注 + 缺失项 MISSING + exit 1。

### 4.5 错误处理

| 失败点 | 处理 |
|---|---|
| `--addons unknown` | stderr "Unknown addon: 'X' (supported: 'ddd')" + exit 1（v0.1 已有，v0.2 信息更新）|
| addon `install()` 任一步写入失败 | rollback 已写文件 + 还原备份 + exit 1（rollback session 已支持，DDD addon 复用）|
| DDD skills 嵌入资源损坏 | cargo build 时 `include_dir!` / `include_str!` 校验路径；运行时不应发生 |
| `upgrade --addons ddd` 到无 DDD 的项目 | OK——append 行为，把 DDD 装上去（语义等同 "init --force --addons ddd" 的 ddd 部分）|

## 5. Addon trait 与实现

### 5.1 `addons/mod.rs`（修改）

```rust
//! Addon plugin scaffolding.

use anyhow::{bail, Result};
use std::path::Path;
use crate::rollback::InstallSession;

pub mod ddd;

pub trait Addon {
    fn name(&self) -> &str;
    
    /// Install/refresh in target dir. Uses session for rollback safety.
    fn install(&self, dir: &Path, backup_existing: bool, session: &mut InstallSession) -> Result<()>;
    
    /// Optional AGENTS.md segment (static, embedded at compile time).
    /// Returned text is appended verbatim after the project_rules body.
    fn agents_md_segment(&self) -> Option<&'static str> {
        None
    }
}

/// Validate `--addons` names and return concrete addon objects.
/// Empty input → empty vec. Unknown name → error.
pub fn resolve_addons(addons: &[String]) -> Result<Vec<Box<dyn Addon>>> {
    let mut out: Vec<Box<dyn Addon>> = Vec::new();
    for a in addons {
        match a.as_str() {
            "ddd" => out.push(Box::new(ddd::DddAddon)),
            other => bail!("Unknown addon: '{}' (supported: 'ddd')", other),
        }
    }
    Ok(out)
}
```

`handle_addons`（v0.1 的薄包装）保留为 alias `pub fn handle_addons(addons: &[String]) -> Result<()> { resolve_addons(addons).map(|_| ()) }`——保证 v0.1 测试不破。

### 5.2 `addons/ddd.rs`（新增）

```rust
//! DDD plugin: 3 skills + DOMAIN.md (install-once) + README-DDD-HARNESS.md.

use anyhow::Result;
use include_dir::{include_dir, Dir};
use std::path::Path;
use crate::addons::Addon;
use crate::rollback::InstallSession;

static DDD_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/ddd/skills");
const ROOT_DOMAIN_MD: &[u8] = include_bytes!("../../templates/ddd/root/DOMAIN.md");
const ROOT_README: &[u8] = include_bytes!("../../templates/ddd/root/README-DDD-HARNESS.md");
const AGENTS_MD_SEGMENT: &str = include_str!("../../templates/ddd/root/claude-md-merge.md");

pub struct DddAddon;

impl Addon for DddAddon {
    fn name(&self) -> &str { "ddd" }

    fn install(&self, dir: &Path, backup_existing: bool, session: &mut InstallSession) -> Result<()> {
        // 1. Skills: each <name>.md → <dir>/.trae/skills/ddd/<name>/SKILL.md
        for entry in DDD_SKILLS.files() {
            let stem = entry.path().file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid skill filename: {:?}", entry.path()))?;
            let target = dir.join(".trae/skills/ddd").join(stem).join("SKILL.md");
            session.write(&target, entry.contents(), backup_existing)?;
        }

        // 2. README-DDD-HARNESS.md (project root)
        session.write(&dir.join("README-DDD-HARNESS.md"), ROOT_README, backup_existing)?;

        // 3. DOMAIN.md (install-once, hardcoded backup=false)
        let domain_target = dir.join("DOMAIN.md");
        if !domain_target.exists() {
            session.write(&domain_target, ROOT_DOMAIN_MD, false)?;
        } else {
            println!("ℹ DOMAIN.md preserved (user-managed).");
        }

        Ok(())
    }

    fn agents_md_segment(&self) -> Option<&'static str> {
        Some(AGENTS_MD_SEGMENT)
    }
}
```

### 5.3 `init::run_with_options` 改造（修改）

按 §4.1 给出的清单顺序：rules → skills/superpowers → addon install → AGENTS.md → log。把 addon install 放在 AGENTS.md 之前——这样 AGENTS.md 写出时所引用的 ddd skill 路径已经在文件系统中存在：

```rust
// Phase 1: rules/project_rules.md (unchanged from v0.1)
// Phase 2: skills/superpowers/ recursive (unchanged from v0.1)

// Phase 3: install addons (writes .trae/skills/ddd/, DOMAIN.md, README-DDD-HARNESS.md)
let addon_list = addons::resolve_addons(addons_list)?;
for addon in &addon_list {
    if let Err(e) = addon.install(dir, backup_existing, &mut session) {
        session.rollback();
        return Err(e.context(format!("addon '{}' install failed", addon.name())));
    }
}

// Phase 4: AGENTS.md (dynamically rendered, including addon segments)
let skills_meta = collect_skill_frontmatters()?;
let addon_refs: Vec<&dyn Addon> = addon_list.iter().map(|b| b.as_ref()).collect();
let agents_md_body = agents_md::render_agents_md(project_rules_body, &skills_meta, &addon_refs);
session.write(&dir.join("AGENTS.md"), agents_md_body.as_bytes(), backup_existing)?;

// Phase 5: install log (unchanged from v0.1)
```

注：addon install 在 AGENTS.md 之前——AGENTS.md 引用的所有 ddd skill 路径在写出 AGENTS.md 时已存在文件系统上。无前向引用。

## 6. AGENTS.md 渲染

### 6.1 `agents_md::render_agents_md` 签名扩展

v0.1：

```rust
pub fn render_agents_md(project_rules_body: &str, skills: &[Frontmatter]) -> String
```

v0.2：

```rust
pub fn render_agents_md(
    project_rules_body: &str,
    skills: &[Frontmatter],
    addons: &[&dyn Addon],
) -> String
```

### 6.2 拼接逻辑

```rust
pub fn render_agents_md(
    project_rules_body: &str,
    skills: &[Frontmatter],
    addons: &[&dyn Addon],
) -> String {
    let skill_index = skills.iter()
        .map(|s| format!("- {} — {}", s.name, s.description))
        .collect::<Vec<_>>()
        .join("\n");

    let mut out = format!(
        r#"# AGENTS DIRECTIVE — superpowers methodology
...（v0.1 模板原样）...
{project_rules_body}"#,
        skill_index = skill_index,
        project_rules_body = project_rules_body,
    );

    for addon in addons {
        if let Some(segment) = addon.agents_md_segment() {
            out.push('\n');
            out.push_str(segment);
        }
    }

    out
}
```

### 6.3 `claude-md-merge.md` 内容

由实施 task 工程师从 ddd-run CLAUDE.md 抽取，预期约 60-80 行。覆盖：

- DDD 工作流三步及调用方式（`/ddd-storm` / `use ddd-storm to ...` 等触发）
- Ubiquitous Language 强制（必须用 DOMAIN.md 定义的术语）
- 技术栈未确定不准实现（DOMAIN.md 技术栈段填完前禁止 brainstorming 之外的实现）
- 跨聚合事务禁止
- 领域模型行为优先（禁 anemic）

具体文本由实施 task 决定（避免 spec 锁死可能误抽的措辞）；约束是"取自 ddd-run CLAUDE.md 的核心约束"。

## 7. 测试

### 7.1 单元测试（新增）

`addons::tests`：
- `resolve_empty_ok` — 空输入返回空 vec
- `resolve_ddd_returns_one` — `["ddd"]` 返回长度 1 的 vec
- `resolve_unknown_errors` — `["bogus"]` 返回 Err

`agents_md::tests`：
- `render_agents_md_appends_addon_segments` — addon 返回 "DDD SEGMENT"，输出含
- `render_agents_md_empty_addons_unchanged` — 空 addons → 与 v0.1 输出一致

### 7.2 集成测试（`tests/integration_addons_ddd.rs`，新增 6 个）

```rust
#[test] fn init_addons_ddd_writes_ddd_skills() { ... }       // .trae/skills/ddd/{ddd-storm,ddd-model,ddd-spec}/SKILL.md 都存在
#[test] fn init_addons_ddd_writes_domain_md() { ... }        // DOMAIN.md 存在并含模板内容标识
#[test] fn init_addons_ddd_writes_readme_harness() { ... }   // README-DDD-HARNESS.md 存在
#[test] fn init_addons_ddd_extends_agents_md() { ... }       // AGENTS.md 含 "DDD methodology" 字符串
#[test] fn upgrade_addons_ddd_preserves_user_domain_md() { ... }  // 改 DOMAIN.md → upgrade --addons ddd → 内容不变
#[test] fn upgrade_without_addons_keeps_existing_ddd_files() { ... }  // 装了 ddd → upgrade（无 flag）→ ddd 文件不动
```

### 7.3 status 集成测试（修改现有文件）

`tests/integration_status.rs` 加一条 `status_with_ddd_reports_addon`。

### 7.4 e2e-smoke

`Makefile` 的 `e2e-smoke` 增加 `--addons ddd` 路径覆盖（见 §10）。

### 7.5 v0.1 测试**全部保留通过**

不修改任何现有 v0.1 测试文件——它们覆盖"不带 --addons"路径，在 v0.2 中行为完全不变。

## 8. 不做（明确边界）

- DDD plugin 卸载（`remove ddd`）—— 用户可手工 `rm -rf .trae/skills/ddd DOMAIN.md README-DDD-HARNESS.md`，AGENTS.md 下次 upgrade 自动收缩
- 多 plugin 并存（v0.2 只 ddd）
- `ddd-run` 反向同步 / 自动同步—— 由 maintainer 在 ddd-run 升级时手动重做 `cp` 步骤
- 创建 `docs/ddd/` / `docs/specs/` 工作目录——这是 ddd-storm/ddd-spec 输出位置；用户首次跑 ddd-storm 时由 Trae 自然创建即可
- 把整段 ddd-run CLAUDE.md 并入 AGENTS.md——只摘核心约束；用户自填的"项目定位"/"技术栈"等字段由 DOMAIN.md 处理
- DDD plugin 的独立版本号——和 cli 共用 v0.2.0（v0.3 真有需要时再拆）

## 9. 已知风险与缓解

| 风险 | 缓解 |
|---|---|
| `claude-md-merge.md` 抽取偏差 → AGENTS.md 段措辞不准 | 实施 task 工程师对照 ddd-run CLAUDE.md 抽；review 阶段人工核对；v0.2.x patch 修订 |
| ddd-run 升级（DDD skill 内容变） → 我们要重新同步 | Cargo.toml metadata 记 ddd_run_version commit；README 写明同步流程；v0.2.x patch 重新打 release |
| 用户已装 DDD 但删了 DOMAIN.md → status 报 incomplete + 用户可能误以为坏了 | status 输出明确提示 "run upgrade --addons ddd to restore template" |
| AGENTS.md 体积膨胀（v0.1 约 9 KB，加 DDD 段约 12 KB） | 仍远小于 6 KB 硬上限的关注点不存在——AGENTS.md 没硬上限（不是 user_rules.md），Trae 设置 toggle 控制是否加载 |
| `Addon::install` 的 install-once 逻辑（DOMAIN.md）绕过 rollback session 备份机制 | 文档显式说明 install-once 文件的 backup_existing 参数被 ignore；测试覆盖 |
| addon install 中途失败 → 部分 ddd 文件已写、AGENTS.md 已含 ddd 段 | rollback session 已记录所有写入；session.rollback() 反向清理；exit 1 |

## 10. 验收标准

- `make test` 全套通过（python 12 + Rust 单测 24+ + 集成测试 含新增 6 个 + 现有 status 一个新用例）
- `make e2e-smoke` 改造后通过（覆盖 `init --addons ddd` → status → `upgrade --addons ddd` → status）：

```makefile
e2e-smoke: cli-build
	@TMP=$$(mktemp -d) && \
	$(CLI_BIN) init --dir "$$TMP" --addons ddd && \
	$(CLI_BIN) status --dir "$$TMP" && \
	$(CLI_BIN) upgrade --dir "$$TMP" --addons ddd && \
	$(CLI_BIN) status --dir "$$TMP" && \
	test -f "$$TMP/DOMAIN.md" && \
	test -f "$$TMP/.trae/skills/ddd/ddd-storm/SKILL.md" && \
	test -f "$$TMP/README-DDD-HARNESS.md" && \
	rm -rf "$$TMP" && \
	echo "✓ e2e smoke passed (with ddd)"
```

- `cargo install --path cli` 后 `superpowers-trae --version` 输出 `0.2.0`
- 在干净临时目录跑 `superpowers-trae init --addons ddd` 后：
  - `.trae/skills/ddd/{ddd-storm,ddd-model,ddd-spec}/SKILL.md` 全部存在
  - `DOMAIN.md` 存在并是 ddd-run 模板内容
  - `README-DDD-HARNESS.md` 存在
  - `AGENTS.md` 含 "DDD methodology" 段
- 在已装 DDD 项目跑 `upgrade`（无 `--addons`）后，DDD 文件 mtime 不变（不动）
- 在用户改过 DOMAIN.md 的项目跑 `upgrade --addons ddd` 后，DOMAIN.md 内容跟改的一样
- 真机：在 test-superpowers 项目跑 `superpowers-trae upgrade --addons ddd`，提示词里说"使用 ddd-storm 帮我梳理 X 业务"，预期 Trae Builder 主动 Read `.trae/skills/ddd/ddd-storm/SKILL.md` 并按 ddd-storm SKILL 流程走

## 11. 子项目顺序

| 子项目 | 范围 | 与本设计的关系 |
|---|---|---|
| 1 | Rust CLI v0.1.0：init/upgrade/status + base superpowers + ddd 占位 | ✅ 已完成（[2026-04-28 spec](2026-04-28-superpowers-trae-cli-design.md)） |
| 2（本） | DDD plugin v0.2.0：addons/ddd 真实现 + AGENTS.md 段 + status 扩展 | 本 spec |
| 3 | 文档收尾：spec/README v0.2 总结、贡献指南、v0.3 roadmap | sub-project 2 完成后做 |

---

## 附录 A：跟 v0.1 的兼容性矩阵

| v0.1 行为 | v0.2 是否变 | 说明 |
|---|---|---|
| `init`（无 flag） | 不变 | 仅装 superpowers，AGENTS.md 不含 addon 段 |
| `init --force` | 不变 | 等同 v0.1，仍仅装 superpowers |
| `init --addons ddd` | **变**（v0.1 stub 提示，v0.2 真装） | 主功能 |
| `init --addons unknown` | 不变 | 仍 exit 1 |
| `upgrade` | **微变**（行为不变，但消息可能略调） | DDD 文件不动 |
| `upgrade --addons ddd` | **变**（v0.1 stub，v0.2 真装/刷新） | 主功能 |
| `status` | **变**（多了 Addons 段） | DDD 装入时显示；未装时输出 "Addons: none"（与 v0.1 一致） |
| version 字符串 | 变（`0.1.0` → `0.2.0`） | metadata 也更新 |

v0.1 用户从 `cargo install` 升 v0.2 后，**未跑 `upgrade --addons ddd` 前**，所有命令行为与 v0.1 等价（只 `--version` 和 `status` 顶头输出版本字符串变了）。
