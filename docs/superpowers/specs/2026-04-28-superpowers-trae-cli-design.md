# superpowers-trae CLI（Rust v0.1.0）：设计文档

- 日期：2026-04-28
- 状态：Draft（待用户复审）
- 子项目：sub-project 1 of 3（参 README "项目演进路线图"）
- 上一版（v0）：shell 三件套 `scripts/{sync-upstream,build,install}.sh`
- 下一版（v1.x / v0.2.x）：sub-project 2 — DDD plugin 集成（`--addons ddd` 在本设计里仅占位）

## 1. 背景与问题

`cc-superpower-to-trae` 仓库已具备一条可工作的转换流水线（python `transform.py`/`render.py`/`build_helpers.py` + shell `sync-upstream.sh`/`build.sh`/`install.sh`），把 Superpowers 5.0.7 转换成 Trae IDE 可装产物 `dist/user/`。

但末端 `install.sh` 的真机实测暴露了三个痛点：

1. **`--user` 模式无效**：Trae IDE 的 Personal Rules 在 UI 输入、不在文件系统，写到 `~/.trae/` 的 user_rules.md 被 Trae 完全忽略。Custom Agents 在服务端账号绑定（SQLite 只缓存 `currentAgentData_<user_id>`，143 key 全扫无 agent list），脚本无法批量装，必须 UI 创建。
2. **缺独立可分发二进制**：当前需要 clone 整个仓库才能跑 `install.sh`。给同事或换机器都要带上 200MB+ 的 `upstream/` 副本。

**关键行为发现**（实测）：Trae 内置 **Builder Agent** 在用户提示词里出现"**superpowers**"触发词时，会主动 Read `.trae/rules/project_rules.md` + 相关 SKILL.md（实测 Read 了 6 个文件）然后按 brainstorming/writing-plans/test-driven-development 等 skill 流程执行；不出现触发词则默认 helpful-bot 直接给方案。

这意味着：**file-based 安装就足够让方法论上线**（项目 rules + skills + AGENTS.md），用户只需在提示词里说"使用 superpowers ..."。**Custom Agents 不再是必需**——它是可选便利（`@brainstorm` 直触发，省"superpowers"一词）；想用的人参 `docs/superpowers/trae-agents-setup.md` 在 Trae UI 手动创建即可，不进 CLI 流程。

参考项目 `~/workshop/ddd-run`（Rust + clap，3 个 ddd skill 嵌入到二进制里）证明 Rust CLI + 编译时嵌入是分发痛点的最佳解。

本设计实现 `superpowers-trae` Rust CLI v0.1.0：用户两条路径任选其一拿到 binary（`cargo install` 或预编译 release），在目标项目跑 `superpowers-trae init` 即可一键完成所有 file-based 安装。Custom Agents 创建保持可选 + 仓库内文档支持（不进 CLI）。

## 2. 目标与非目标

**目标（v0.1.0）**

1. Rust CLI binary `superpowers-trae` 提供 3 个子命令：`init` / `upgrade` / `status`。
2. binary 自包含——`include_dir!()` 编译时把 `dist/user/` 整树嵌进去。运行时不依赖文件系统外部资源。
3. `init` 一键写入：
   - `<dir>/.trae/rules/project_rules.md`
   - `<dir>/.trae/skills/superpowers/<14 个 skill 子目录>/...`
   - `<dir>/AGENTS.md`（动态生成：DIRECTIVE 强约束头 + skill index + project_rules.md 冗余）
   - `<dir>/.trae/.superpowers-install.log`
4. `init` 末尾打印简短成功消息 + 调用方式提示（说"使用 superpowers"触发；可选地参 repo 文档创建 Custom Agent）。
5. `upgrade` 默认备份 + 覆盖；`--no-backup` 跳过备份。
6. `status` 列出安装状态、彩色 OK/MISSING 标记、退出码反映状态。
7. `--addons ddd` flag 在 `init` / `upgrade` 上**仅占位**：打印 "not yet implemented" 后继续。未知 addon 名字直接报错退出 1。
8. 现有 python/shell maintainer pipeline 全部保留——Rust binary 只是 dist 的"装到目标项目"那一段。
9. 三平台 binary：`x86_64-linux` / `aarch64-apple-darwin` / `x86_64-apple-darwin`。
10. CI（GitHub Actions）打 `v*` tag 时自动 build 三平台 + 上传 GitHub Releases。

**非目标（留 v0.2.x / 子项目 2-3）**

- DDD plugin 真实实现（v0.1.0 仅占位）
- Windows 支持
- 写 Trae 服务端 Custom Agent（不可能——服务端 API 不公开；保留为可选手工步骤，仅在 repo 文档里说明）
- per-user marker / `agents_acknowledged` 状态追踪（已被关键发现简化掉：file-based 安装自足，无需追踪 Agent 创建状态）
- 包管理器分发（homebrew / apt / aur 等）
- `update-superpowers` 子命令（让 binary 自己拉新版 upstream，超 v1 范围）
- Trae 之外的 IDE（Cursor / Copilot CLI）支持

## 3. 顶层架构

```
cc-superpower-to-trae/                            # 仓库根
├── upstream/                                     # superpowers 原版（python pipeline 输入）
├── src/                                          # python：transform/render/build_helpers
├── scripts/                                      # shell：sync-upstream / build
├── dist/
│   ├── user/                                     # ★ Rust binary 编译时嵌入这棵树（include_dir!）
│   └── project/                                  # 现有产物，Rust CLI 不使用；保留供 deprecated shell install.sh 兼容
├── cli/                                          # ★ 新增 Rust 工程
│   ├── Cargo.toml                                # clap derive + anyhow + colored + include_dir
│   ├── Cargo.lock
│   ├── README.md                                 # crate 自述（精简版）
│   └── src/
│       ├── main.rs                               # clap 入口 + 命令分发
│       ├── lib.rs                                # 公共：log printer / paths
│       ├── embed.rs                              # include_dir! 集中点
│       ├── agents_md.rs                          # 动态拼接 AGENTS.md（DIRECTIVE 头 + skill index + project_rules.md 正文）
│       ├── addons/
│       │   └── mod.rs                            # Addon trait（v1 占位）
│       └── commands/
│           ├── mod.rs
│           ├── init.rs                           # init 命令实现
│           ├── upgrade.rs                        # upgrade 命令实现
│           └── status.rs                         # status 命令实现
├── docs/superpowers/
│   ├── specs/2026-04-28-superpowers-trae-cli-design.md  # ← 本文档
│   ├── plans/2026-04-28-superpowers-trae-cli.md         # writing-plans 阶段产出
│   ├── trae-agents-setup.md                             # 仓库内可选参考（用户自助，CLI 不再写入目标项目）
│   └── manual-smoke-test.md
├── tests/                                        # python 测试不动
│   ├── test_transform.py / test_render.py        # 5 + 7 测试
│   ├── verify-build.sh / forbidden-strings.txt
│   └── test_build_smoke.sh
├── Makefile                                      # ★ maintainer 便捷入口
└── .github/workflows/release.yml                 # ★ release 自动化
```

**核心数据流**

maintainer pipeline（更新 binary 嵌入的 superpowers 内容）:
```
scripts/sync-upstream.sh        →  upstream/ 刷新
scripts/build.sh                →  dist/user/ 重生成（python 转换器）
(cd cli && cargo build -r)      →  cli/target/release/superpowers-trae 嵌入新 dist
make e2e-smoke                  →  init/upgrade/status 全跑一遍
git tag v0.x.y && git push      →  GitHub Action 三平台 build + release
```

end-user flow:
```
cargo install --git ... 或下 release tar.gz   →  ~/.cargo/bin 或 PATH 任意位置
cd <my-project>
superpowers-trae init                           →  写 .trae/* + AGENTS.md
                                                   末尾打印一行成功 + 调用方式提示
                                                # 用户回 Trae IDE，提示词里写 "使用 superpowers ..."
                                                # 即可触发 Builder 加载 rules + skills 流程
superpowers-trae upgrade                        →  备份 + 覆盖（升级到新版 binary 嵌入的 superpowers）
superpowers-trae status                         →  彩色检查报告
```

**关键设计点**

- `cli/` 是 Cargo 项目根；和现有 python 文件互不干扰
- `include_dir!("$CARGO_MANIFEST_DIR/../dist/user")` 编译时把整棵 dist 树打进 binary（约 4-6 MB 最终大小）
- AGENTS.md 不是静态嵌入，而是 binary 在 init 时**动态拼接**：DIRECTIVE 强约束头 + 14 行 Skill Index + project_rules.md 正文复制（双保险）
- file-based 安装路径（rules + skills + AGENTS.md）已经满足"Builder 看到触发词后加载方法论"——CLI 不必处理 Custom Agent 创建状态
- maintainer pipeline（python/shell）保留——Rust 只做"读模板 → 写到目标 → 打印调用提示"

## 4. 命令面与行为

### 4.1 `superpowers-trae init [OPTIONS]`

**Flags**

| Flag | 说明 | 默认 |
|---|---|---|
| `--dir <path>` | 目标项目根 | `.`（cwd） |
| `--force` | 已装也写（带备份） | false |
| `--addons <name>...` | 启用 plugin（v1 仅 `ddd`，但占位）| 空 |

**预检（按顺序，任一失败即 stderr 报错 + exit ≠ 0）**

1. `<dir>` 存在且是目录（exit 1）
2. `<dir>/.trae/rules/project_rules.md` 不存在（除非 `--force`）—— 已存在时 stderr 提示 "already initialized, use `superpowers-trae upgrade`" + exit 1

> 注：写权限校验与只读文件系统检测**不在 preflight**——它们由实际的 `fs::write` 错误驱动 + rollback 自动清理已写文件。这避免了 preflight 中重复模拟写盘的复杂性，代价是用户看到的错误是 "Permission denied (os error 13)" 而非更友好的提前 bail。v0.1 接受此取舍。

**写入清单（按顺序）**

1. `<dir>/.trae/rules/project_rules.md` ← `DIST_USER.get_file("rules/user_rules.md")` 的内容（注意源文件名是 user_rules.md，dest 改名为 project_rules.md）
2. `<dir>/.trae/skills/superpowers/<14 dirs>/...` ← `DIST_USER.get_dir("skills/superpowers")` 整树
3. `<dir>/AGENTS.md` ← 动态拼接（4.4 节详细）
4. `<dir>/.trae/.superpowers-install.log` ← 追加每个文件的 `INSTALL: <path>` 行（带时间戳头）

`--force` 时：上面每个目标文件已存在 → 先 `mv <file> <file>.bak.<YYYYMMDD-HHMMSS>` 再写入；log 里写 `BACKUP: <bak-path>` 行。

**末尾输出**

固定打印一段成功消息 + 调用方式说明（无状态、无 marker、无可选分支）：

```
✓ Initialized superpowers methodology in <abs-dir>
  - rules:  .trae/rules/project_rules.md
  - skills: .trae/skills/superpowers/   (14 skills)
  - AGENTS: AGENTS.md  (project root, double-insurance)

To invoke methodology in Trae IDE, include "superpowers" in your prompt:
  "Use superpowers to brainstorm <X>"
  "Use superpowers to plan <Y>"
  "Use superpowers to debug <Z>"

Optional: for @-mention shortcuts (@brainstorm / @write-plan / etc.), see
docs/superpowers/trae-agents-setup.md in the superpowers-trae repo for
paste-ready Custom Agent prompts.
```

**`--addons` 处理**

- `--addons ddd`：打印 `ℹ DDD plugin not yet implemented in v1. Scheduled for sub-project 2.` + 继续（exit 0）
- `--addons <unknown>`：stderr 报错 `Unknown addon: '<x>' (supported in v0.1: 'ddd' [stub])` + exit 1

### 4.2 `superpowers-trae upgrade [OPTIONS]`

**Flags**

| Flag | 说明 | 默认 |
|---|---|---|
| `--dir <path>` | 目标项目根 | `.` |
| `--no-backup` | 跳过 `.bak.*` 备份直接覆盖 | false |
| `--addons <name>...` | 同 init | 空 |

**预检**

1. `<dir>/.trae/rules/project_rules.md` 必须存在 → 不存在时 stderr 提示 "not initialized, use `init` first" + exit 1

**行为**

- 跟 init 相同的 4 项写入清单
- 默认每个目标已存在的文件先备份再写（log 里 `BACKUP:` + `INSTALL:` 行成对）
- `--no-backup` 跳过 `mv`，直接覆盖
- 末尾打印成功消息（不复述调用方式提示——升级时用户已经知道）

### 4.3 `superpowers-trae status [OPTIONS]`

**Flags**

| Flag | 说明 | 默认 |
|---|---|---|
| `--dir <path>` | 目标项目根 | `.` |

**输出（彩色 OK/MISSING/WARN 标记）**

```
superpowers-trae v0.1.0 (embedded superpowers 5.0.7)
Project: /home/xiaojin/Documents/trae_projects/test-superpowers
─────────────────────────────────────────────────────────
✓ project_rules.md         (.trae/rules/project_rules.md, 3.3 KB)
✓ superpowers skills       14 / 14 expected
✓ AGENTS.md                (project root, 4.1 KB)
ℹ Last init/upgrade        2026-04-28 14:32:01 (from install log)

Addons: none
```

**退出码**：所有项 OK → 0；任一 MISSING → 1。

### 4.4 AGENTS.md 内容（动态生成）

`init` / `upgrade` 时，`<dir>/AGENTS.md` 内容由两段拼接：

```markdown
# AGENTS DIRECTIVE — superpowers methodology

> **Critical**: When the user's request matches a skill's purpose listed below,
> you MUST `Read` the corresponding SKILL.md file and follow it exactly BEFORE
> producing any other output (including clarifying questions, exploration, or
> proposed solutions). Do not paraphrase from memory — skills evolve and the
> file is the source of truth.

## How to load a skill

1. Identify which skill matches the user's request (see Skill Index below).
2. Use the `Read` tool on `.trae/skills/superpowers/<skill-name>/SKILL.md`.
3. Follow that file's checklist verbatim.

## Skill Index

[14 行 - <name> — <description>，由 binary 在 init 时遍历 DIST_USER 生成]

## Tool name reference

For tool name mappings between Claude Code and Trae IDE, see
`.trae/skills/superpowers/references/trae-tools.md`.

---

[这里是 .trae/rules/project_rules.md 的完整复制 — 双保险冗余]
```

**Skill Index 的生成**：binary 启动时遍历 `DIST_USER.get_dir("skills/superpowers")`，跳过 `references/`，对每个 skill 子目录读 `SKILL.md` 解析 frontmatter（简单 `key: value` 行解析，跟 python `render.py:parse_skill_frontmatter` 等价）抽 `name` + `description`。

**为什么不直接静态 include AGENTS.md**：

- 头部 DIRECTIVE 比 project_rules.md 措辞更强（即便用户没说"superpowers"触发词，AGENTS.md 也作为另一加载路径增加方法论被采纳的机会——Trae 设置开关 "将 AGENTS.md 包含在上下文中" 默认开）
- 后半冗余 project_rules.md 内容确保两个加载点（AGENTS.md / project_rules.md）一致
- skill index 来自 dist 实时枚举，新增 skill 不需要改 Rust 代码

### 4.5 错误处理

| 失败点 | 处理 |
|---|---|
| init 检测到已初始化（无 `--force`） | stderr 提示 "use upgrade" + exit 1 |
| upgrade 未初始化 | stderr 提示 "use init first" + exit 1 |
| 写入失败（权限 / 磁盘满 / 中断） | rollback 已写入文件 + 还原已备份的 `.bak.*` + stderr 错误链 + exit 1 |
| `<dir>/.trae` 中已有用户的 superpowers 之外内容 | 不冲突字段不动；冲突字段按 init/upgrade 各自语义处理（见上）|
| `--addons <unknown>` | stderr 报错 + exit 1 |
| 嵌入资源损坏（理论上 cargo build 时验证，运行时不应发生） | panic + bug 报告提示 |

**rollback 实现**：`commands::init` 维护一个 `Vec<RollbackAction>`（含 `Created(PathBuf)` / `Backed(PathBuf, PathBuf)`）；任一步 fail 时倒序回滚 + bail anyhow 错。

## 5. 模板嵌入策略

### 5.1 `include_dir!()` 的运行模型

```rust
// cli/src/embed.rs
use include_dir::{include_dir, Dir};

pub static DIST_USER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist/user");
```

- 编译时校验路径存在；漏文件 cargo build 直接挂
- 运行时 `DIST_USER.get_file("rules/user_rules.md")` 返回 `&File`，可读 `.contents()` 字节
- `DIST_USER.get_dir("skills/superpowers")` 可枚举所有子目录与文件
- 二进制体积：dist/user/ 当前约 250 KB（gzip 后约 100 KB）；最终 binary 4-6 MB（Rust 静态链接基础 + 嵌入资源 + lto 优化）

### 5.2 写入到目标的 helper

```rust
// cli/src/commands/init.rs（节选）
fn write_dist_recursive(dir: &Dir, target_root: &Path, rollback: &mut Vec<RollbackAction>)
    -> Result<()>
{
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(d) => write_dist_recursive(d, target_root, rollback)?,
            DirEntry::File(f) => write_file_with_rollback(
                target_root.join(f.path()), f.contents(), rollback,
            )?,
        }
    }
    Ok(())
}

fn write_file_with_rollback(target: PathBuf, bytes: &[u8], rollback: &mut Vec<RollbackAction>)
    -> Result<()>
{
    if target.exists() {
        let bak = target.with_extension(format!("bak.{}", timestamp()));
        fs::rename(&target, &bak)?;
        rollback.push(RollbackAction::Backed(target.clone(), bak));
    } else {
        rollback.push(RollbackAction::Created(target.clone()));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, bytes)?;
    Ok(())
}
```

## 6. 测试策略

### 6.1 Rust 单元测试（`cargo test`）

`cli/src/` 各模块的纯逻辑（不写盘）：

- `commands::init` 的预检函数（路径校验，不真写）
- `parse_skill_frontmatter`（抽 name+description，含无尾换行 + YAML 引号边界）
- `agents_md::render`（拼 DIRECTIVE 头 + skill index + project_rules 正文）

### 6.2 Rust 集成测试（`cli/tests/`）

跑 binary 在 `tempfile::tempdir()` 上：

```rust
// cli/tests/integration_init.rs
#[test]
fn init_writes_expected_files() { ... }

#[test]
fn init_refuses_already_initialized_without_force() { ... }

#[test]
fn init_force_creates_backup() { ... }

#[test]
fn upgrade_creates_backup_by_default() { ... }

#[test]
fn upgrade_no_backup_skips_backup() { ... }

#[test]
fn status_reports_complete_install() { ... }

#[test]
fn status_reports_missing_files() { ... }

#[test]
fn addons_ddd_prints_stub_warning_and_exits_zero() { ... }

#[test]
fn addons_unknown_errors_exit_1() { ... }

#[test]
fn init_end_output_mentions_superpowers_trigger_word() { ... }
```

dev-dependencies：`assert_cmd`、`predicates`、`tempfile`。

### 6.3 端到端冒烟（Makefile）

```makefile
e2e-smoke:
    @TMP=$$(mktemp -d) && \
    $(CARGO_BIN) init --dir "$$TMP" && \
    $(CARGO_BIN) status --dir "$$TMP" && \
    $(CARGO_BIN) upgrade --dir "$$TMP" && \
    $(CARGO_BIN) status --dir "$$TMP" && \
    rm -rf "$$TMP" && \
    echo "✓ e2e smoke passed"
```

`make e2e-smoke` 跑完 init → status → upgrade → status；CI release workflow 也跑。

### 6.4 现有 python 测试不变

- `tests/test_transform.py`（5 测试）/ `tests/test_render.py`（7 测试）保留
- `tests/verify-build.sh` / `tests/test_build_smoke.sh` 保留
- 测的是"upstream → dist"转换流水线，跟 Rust CLI 解耦

`make test` 一并跑 python + Rust 全套。

## 7. Maintainer 工作流（Makefile）

```makefile
.PHONY: all sync build cli-build cli-install cli-test python-test test e2e-smoke clean upgrade-superpowers

all: sync build cli-build

sync:
    bash scripts/sync-upstream.sh

build:
    bash scripts/build.sh

cli-build:
    cd cli && cargo build --release

cli-install:
    cd cli && cargo install --path .

cli-test:
    cd cli && cargo test

python-test:
    python3 -m unittest discover tests

test: python-test cli-test

e2e-smoke: cli-build
    @TMP=$$(mktemp -d) && \
    cli/target/release/superpowers-trae init --dir "$$TMP" && \
    cli/target/release/superpowers-trae status --dir "$$TMP" && \
    cli/target/release/superpowers-trae upgrade --dir "$$TMP" && \
    rm -rf "$$TMP" && \
    echo "✓ e2e smoke passed"

upgrade-superpowers: sync build cli-build
    @echo "✓ Maintainer pipeline done. Review with: git diff dist/ && git diff cli/"

clean:
    rm -rf dist/
    cd cli && cargo clean
```

## 8. 交付与 Release 流程

### 8.1 安装方式

**A：从源码（cargo install）**

```bash
cargo install --git https://github.com/<your-gh-username>/cc-superpower-to-trae \
  --tag v0.1.0 superpowers-trae
```

落到 `~/.cargo/bin/superpowers-trae`。需要 Rust 1.75+。

**B：预编译二进制（GitHub Releases）**

```bash
# Linux x86_64
curl -L https://github.com/<your-gh-username>/cc-superpower-to-trae/releases/download/v0.1.0/superpowers-trae-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv superpowers-trae /usr/local/bin/

# macOS Apple Silicon
curl -L .../superpowers-trae-aarch64-apple-darwin.tar.gz | tar xz
mv superpowers-trae ~/.local/bin/

# macOS Intel
curl -L .../superpowers-trae-x86_64-apple-darwin.tar.gz | tar xz
mv superpowers-trae ~/.local/bin/
```

无 Rust 依赖，下载即用。

### 8.2 GitHub Actions Release Workflow

`.github/workflows/release.yml` 在 `git push --tags`（tag 形如 `v*`）时触发：

```yaml
name: release
on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-apple-darwin
            os: macos-14
          - target: x86_64-apple-darwin
            os: macos-13
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with: { python-version: "3.11" }
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Maintainer pipeline (sync + build dist)
        run: |
          bash scripts/sync-upstream.sh /path/to/local-superpowers-tarball || true
          bash scripts/build.sh
      - name: cargo build
        run: cd cli && cargo build --release --target ${{ matrix.target }}
      - name: Package
        run: |
          tar czf superpowers-trae-${{ matrix.target }}.tar.gz \
            -C cli/target/${{ matrix.target }}/release superpowers-trae \
            -C ../../../ LICENSE README.md
      - uses: softprops/action-gh-release@v2
        with:
          files: superpowers-trae-${{ matrix.target }}.tar.gz
```

> 注：CI 跑 `sync-upstream.sh` 时本地无 superpowers cache，所以 maintainer 应先把 dist 的最新版 commit 到 git，CI 直接用仓库内已有的 `dist/user/`。release workflow 不重新跑 sync。需要在 workflow 里加判断：如 `dist/user/` 已存在则跳过 sync 步骤。

### 8.3 Cargo.toml release profile

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

跟 ddd-run 同款。预期 binary 体积 4-6 MB（含嵌入 dist）。

### 8.4 版本号

- v0.1.0：本设计落地完成，sub-project 1 验收
- v0.1.x：bug fix / 嵌入版本升级（superpowers 5.0.7 → 5.0.8 不破 API 时）
- v0.2.0：sub-project 2 落地（DDD plugin 真实现）
- `Cargo.toml` 的 `[package.metadata.upstream]` 记录嵌入的 superpowers 版本号，`status` 命令输出会展示

## 9. 验收标准

- `make all && make test && make e2e-smoke` 全部通过
- 在 macOS（Apple Silicon + Intel）+ Linux（x86_64）三平台上 cargo build 成功
- `cargo install --path cli` 后 `superpowers-trae --version` 输出 `0.1.0` 与 `superpowers 5.0.7`
- 在干净临时目录跑 `superpowers-trae init`，验证：
  - `.trae/rules/project_rules.md` 内容跟 `dist/user/rules/user_rules.md` 一致
  - `.trae/skills/superpowers/` 下 14 个 skill 子目录齐全
  - `AGENTS.md` 头部含 "AGENTS DIRECTIVE" + 14 行 Skill Index + project_rules.md 正文复制
  - 末尾打印成功消息 + "use superpowers ..." 调用方式提示
- `upgrade` 跑后 `.bak.<时间戳>` 文件正确生成；`--no-backup` 不生成
- `init --addons ddd` 退出码 0 + 打印 stub 提示
- `init --addons unknown` 退出码 1
- 真机：在 test-superpowers 项目跑 `superpowers-trae init`，提示词里说"使用 superpowers 创建一个简单的订单管理系统"；预期 Builder Agent 主动 Read project_rules.md + brainstorming SKILL.md 并按 brainstorming 流程逐步问澄清问题（已在 v0 阶段实测验证此机制）

## 10. 不做（明确边界）

- DDD plugin 真实现（`--addons ddd` 仅 stub；sub-project 2 处理）
- Windows 支持（v1 三平台限 Linux + macOS）
- 写 Trae 服务端 Custom Agent（API 不公开；属可选手工步骤，仅 repo 文档支持）
- per-user marker 状态追踪（关键发现简化掉了，不需要）
- 包管理器分发（homebrew / apt / aur 等）
- "binary 自动从远程拉新版 upstream"——这等价 v2 阶段的"on-demand sync"，超 v1 范围
- Cursor / Copilot CLI / Codex 等 Trae 之外的 IDE 支持

## 11. 已知风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| `include_dir!` 嵌入路径在 cargo workspace 下解析问题 | cargo build 失败 | 用 `$CARGO_MANIFEST_DIR/../dist/user` 锚定相对路径；CI 矩阵覆盖 ubuntu + macOS 验证 |
| GitHub Actions 跑 maintainer pipeline 时无本地 superpowers cache | release workflow 失败 | release 前 maintainer 先在本地 `make all` 并 commit 最新 `dist/`；CI 跳过 sync 步骤直接用仓库内 dist |
| Trae IDE 升级后改变 AGENTS.md / project_rules.md 加载行为 | 方法论触发失效 | project_rules.md + AGENTS.md 双保险互为冗余；`status` 不依赖 Trae 内部存储 |
| 用户忘了说"使用 superpowers"触发词 | Builder 默认 helpful-bot 模式，不进入方法论流程 | `init` 末尾打印调用方式提示；AGENTS.md 头部 DIRECTIVE 强约束作为补救路径 |
| binary 体积过大（>10 MB） | 用户下载体感差 | release profile + strip + lto 已开；如仍大可考虑 gzip 压缩资源在 binary 内运行时解压 |
| Trae IDE 改 AGENTS.md 加载行为或废弃此约定 | 双保险失效 | 文档显式说明这是 best-effort；project_rules.md 仍是基础保障 |

## 12. 子项目顺序

| 子项目 | 范围 | 与本设计的关系 |
|---|---|---|
| 1 (本) | Rust CLI v0.1.0：init/upgrade/status + base superpowers + ddd 占位 | 本 spec |
| 2 | DDD plugin 真实现：把 ddd-run 的 3 个 skill 移植进 binary，`--addons ddd` 真生效 | 用本设计预留的 `addons/` trait 与 stub 接口 |
| 3 | 文档收尾：spec/README 写清 v2 架构、roadmap、贡献指南 | 在 sub-project 2 完成后做最终 README + spec 大改 |

---

## 附录 A：跟现有 shell 脚本的关系

| 现有脚本 | 处理方式（v0.1.0） |
|---|---|
| `scripts/sync-upstream.sh` | 保留，maintainer 用 |
| `scripts/build.sh` | 保留，maintainer 用 |
| `scripts/install.sh` | **保留但标 deprecated**——README 加提示 "v0.1+ users should prefer `superpowers-trae init`"；shell 脚本继续工作以兼容 |
| `tests/verify-build.sh` | 保留，python pipeline 自检 |
| `tests/test_build_smoke.sh` | 保留 |

`scripts/install.sh` 不立即删除——保留至 v0.2.0 / sub-project 3 阶段决定，给已经在用 shell 流程的 maintainer 平滑过渡。
