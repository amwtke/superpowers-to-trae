# superpowers-trae CLI v0.1.0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 `superpowers-trae` Rust CLI v0.1.0：用户 `cargo install` 或下载预编译 binary 后跑 `superpowers-trae init` 即可在目标项目把 Superpowers 方法论 file-based 安装好（rules + 14 skills + AGENTS.md），三平台 release（linux x86_64 + macOS Apple Silicon + macOS Intel）。

**Architecture:** Cargo 项目位于 `cli/`，与现有 python pipeline 共存；`include_dir!()` 编译时把 `dist/user/` 整树打进二进制；3 个命令 init / upgrade / status；DDD plugin 作为 `--addons ddd` flag 占位但不实现（v0.2 落地）。

**Tech Stack:** Rust 1.75+；clap derive；anyhow；colored；include_dir 0.7；assert_cmd / predicates / tempfile（dev）。Makefile 编排 maintainer 工作流；GitHub Actions 三平台 release。

**Spec:** [`docs/superpowers/specs/2026-04-28-superpowers-trae-cli-design.md`](../specs/2026-04-28-superpowers-trae-cli-design.md)

---

## File Structure

| 路径 | 责任 |
|---|---|
| `cli/Cargo.toml` | crate 元数据、依赖、release profile |
| `cli/Cargo.lock` | 依赖锁文件（cargo 自动生成，入 git）|
| `cli/README.md` | crate 自述 + 安装/使用速查 |
| `cli/src/main.rs` | clap 入口 + 子命令分发 |
| `cli/src/embed.rs` | `include_dir!("$CARGO_MANIFEST_DIR/../dist/user")` 集中点 |
| `cli/src/agents_md.rs` | `parse_skill_frontmatter` + `render_agents_md`（DIRECTIVE 头 + skill index + project_rules 复制） |
| `cli/src/rollback.rs` | `RollbackAction` enum + `write_file_with_rollback` + 倒序 unwind |
| `cli/src/addons/mod.rs` | `Addon` trait（v1 占位）+ `handle_addons`（ddd stub） |
| `cli/src/commands/mod.rs` | 子命令 module 声明 |
| `cli/src/commands/init.rs` | preflight + 主流程 + end output |
| `cli/src/commands/upgrade.rs` | 与 init 共享主流程，preflight 反向，备份默认开 |
| `cli/src/commands/status.rs` | 文件存在性检查 + 彩色报告 |
| `cli/tests/integration_init.rs` | 端到端 init |
| `cli/tests/integration_upgrade.rs` | 端到端 upgrade + 备份验证 |
| `cli/tests/integration_status.rs` | 端到端 status + 退出码 |
| `Makefile`（repo 根）| `make all / test / e2e-smoke` 编排 |
| `.github/workflows/release.yml` | tag 触发三平台 build + GitHub Releases |
| `README.md`（repo 根，修改）| 加入 Rust CLI 安装段；保留 maintainer 段 |

---

## Task 1: Cargo skeleton + clap 入口（命令骨架）

**Files:**
- Create: `cli/Cargo.toml`
- Create: `cli/src/main.rs`
- Create: `cli/.gitignore`
- Create: `cli/README.md`（占位）

- [ ] **Step 1: 创建 cli/ 目录与 Cargo.toml**

`cli/Cargo.toml`：

```toml
[package]
name = "superpowers-trae"
version = "0.1.0"
edition = "2021"
authors = ["xiaojin <amwtke@gmail.com>"]
description = "One-shot installer for the Superpowers methodology in Trae IDE projects"
license = "MIT"
readme = "README.md"
repository = "https://github.com/amwtke/cc-superpower-to-trae"

[package.metadata.upstream]
superpowers_version = "5.0.7"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
colored = "2.1"
include_dir = "0.7"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 2: 写 cli/.gitignore**

```
target/
*.rs.bk
```

- [ ] **Step 3: 写 cli/README.md 占位**

```markdown
# superpowers-trae

Rust CLI to install Superpowers methodology into Trae IDE projects.

See repo root README and [design spec](../docs/superpowers/specs/2026-04-28-superpowers-trae-cli-design.md) for details.

> Full README written by Task 16 of the implementation plan.
```

- [ ] **Step 4: 写 cli/src/main.rs（clap 骨架，3 子命令仅 stub println）**

```rust
//! superpowers-trae: One-shot installer for the Superpowers methodology in Trae IDE projects.

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "superpowers-trae", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// First-time install of Superpowers methodology into a project
    Init {
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(short, long)]
        force: bool,
        #[arg(long, value_delimiter = ',')]
        addons: Vec<String>,
    },
    /// Refresh an already-installed project to the binary's embedded Superpowers version
    Upgrade {
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(long)]
        no_backup: bool,
        #[arg(long, value_delimiter = ',')]
        addons: Vec<String>,
    },
    /// Check installation status of a project
    Status {
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { dir, force, addons } => {
            println!("init: dir={} force={} addons={:?}", dir, force, addons);
        }
        Commands::Upgrade { dir, no_backup, addons } => {
            println!("upgrade: dir={} no_backup={} addons={:?}", dir, no_backup, addons);
        }
        Commands::Status { dir } => {
            println!("status: dir={}", dir);
        }
    }
    Ok(())
}
```

- [ ] **Step 5: 验证编译 + clap 帮助输出**

```bash
cd cli && cargo build
./target/debug/superpowers-trae --help
./target/debug/superpowers-trae init --help
./target/debug/superpowers-trae --version
```

Expected:
- 编译成功（warning 可有，error 不可）
- `--help` 列出 init / upgrade / status
- `init --help` 列出 `--dir`、`--force`、`--addons`
- `--version` 输出 `superpowers-trae 0.1.0`

- [ ] **Step 6: Commit**

```bash
git add cli/Cargo.toml cli/Cargo.lock cli/.gitignore cli/README.md cli/src/main.rs
git commit -m "feat(cli): Cargo skeleton + clap 子命令骨架"
```

---

## Task 2: embed.rs — include_dir!() of dist/user/

**Files:**
- Create: `cli/src/embed.rs`
- Modify: `cli/src/main.rs`（添加 `mod embed;`）

- [ ] **Step 1: 写 cli/src/embed.rs**

```rust
//! Embedded Superpowers content baked into the binary at compile time.
//!
//! `DIST_USER` is the entire `../dist/user/` tree from the project root.
//! At runtime it provides read-only access to skill files, rules, and references.

use include_dir::{include_dir, Dir};

pub static DIST_USER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist/user");
```

- [ ] **Step 2: 在 main.rs 顶部加 `mod embed;`**

在 `cli/src/main.rs` 的 `use` 语句下方加：

```rust
mod embed;
```

- [ ] **Step 3: 写一个临时验证（之后会删，仅本任务用）**

把 `main()` 里 init 分支临时改为：

```rust
        Commands::Init { dir, force, addons } => {
            let _ = (dir, force, addons);
            // Temp verification: print embedded structure
            println!("Embedded files at dist/user/:");
            for entry in embed::DIST_USER.find("**/*").unwrap() {
                if entry.as_file().is_some() {
                    println!("  {}", entry.path().display());
                }
            }
        }
```

- [ ] **Step 4: 跑验证**

```bash
cd cli && cargo build && ./target/debug/superpowers-trae init
```

Expected: 列出 `dist/user/` 下所有文件（应当看到 `rules/user_rules.md`、`skills/superpowers/brainstorming/SKILL.md` 等几十个路径）。

- [ ] **Step 5: 还原 main.rs init 分支为占位 println**

把 init 分支改回 Task 1 末尾的：

```rust
        Commands::Init { dir, force, addons } => {
            println!("init: dir={} force={} addons={:?}", dir, force, addons);
        }
```

（保留 `mod embed;`——后续任务会真用。）

- [ ] **Step 6: Commit**

```bash
git add cli/src/embed.rs cli/src/main.rs
git commit -m "feat(cli): embed dist/user/ via include_dir!()"
```

---

## Task 3: agents_md.rs - parse_skill_frontmatter (TDD)

**Files:**
- Create: `cli/src/agents_md.rs`
- Modify: `cli/src/main.rs`（添加 `mod agents_md;`）

- [ ] **Step 1: 写失败的单元测试（在 agents_md.rs 末尾）**

`cli/src/agents_md.rs`：

```rust
//! AGENTS.md generation: combines DIRECTIVE header, skill index, and project_rules.md body.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_skill_frontmatter_extracts_name_and_description() {
        let text = "---\nname: brainstorming\ndescription: Use when starting any creative work\n---\n\nbody\n";
        let fm = parse_skill_frontmatter(text).unwrap();
        assert_eq!(fm.name, "brainstorming");
        assert!(fm.description.contains("creative work"));
    }

    #[test]
    fn parse_skill_frontmatter_strips_yaml_quotes() {
        let text = "---\nname: x\ndescription: \"quoted value\"\n---\nbody\n";
        let fm = parse_skill_frontmatter(text).unwrap();
        assert_eq!(fm.description, "quoted value");
    }

    #[test]
    fn parse_skill_frontmatter_handles_no_trailing_newline() {
        let text = "---\nname: tail\ndescription: no newline\n---";
        let fm = parse_skill_frontmatter(text).unwrap();
        assert_eq!(fm.name, "tail");
        assert_eq!(fm.description, "no newline");
    }

    #[test]
    fn parse_skill_frontmatter_missing_returns_err() {
        let text = "no frontmatter here\nfoo: bar\n";
        assert!(parse_skill_frontmatter(text).is_err());
    }
}
```

- [ ] **Step 2: 在 main.rs 加 `mod agents_md;`**

- [ ] **Step 3: 跑测试看它失败（函数还没定义）**

```bash
cd cli && cargo test agents_md
```

Expected: `error[E0425]: cannot find function 'parse_skill_frontmatter'`

- [ ] **Step 4: 实现 parse_skill_frontmatter**

在 `agents_md.rs` 的 `#[cfg(test)] mod tests` 之前加：

```rust
use anyhow::{anyhow, Result};

pub struct Frontmatter {
    pub name: String,
    pub description: String,
}

/// Parse YAML-ish frontmatter (simple `key: value` lines between `---` delimiters).
/// Strips surrounding double or single quotes from values.
/// Tolerates absence of trailing newline after closing `---`.
pub fn parse_skill_frontmatter(text: &str) -> Result<Frontmatter> {
    let bytes = text.as_bytes();
    if !text.starts_with("---\n") {
        return Err(anyhow!("no frontmatter delimiter at start"));
    }
    // Find the closing "---" — line at start, followed by \n or end of string.
    let body_start = 4; // after first "---\n"
    let close = (body_start..bytes.len() - 2)
        .find(|&i| {
            (i == body_start || bytes[i - 1] == b'\n')
                && bytes[i..].starts_with(b"---")
                && (i + 3 == bytes.len() || bytes[i + 3] == b'\n')
        })
        .ok_or_else(|| anyhow!("no closing frontmatter delimiter"))?;

    let header = &text[body_start..close];
    let mut name = None;
    let mut description = None;
    for line in header.lines() {
        if let Some((k, v)) = line.split_once(':') {
            let value = strip_quotes(v.trim());
            match k.trim() {
                "name" => name = Some(value.to_string()),
                "description" => description = Some(value.to_string()),
                _ => {}
            }
        }
    }
    Ok(Frontmatter {
        name: name.ok_or_else(|| anyhow!("missing 'name' in frontmatter"))?,
        description: description.ok_or_else(|| anyhow!("missing 'description' in frontmatter"))?,
    })
}

fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if first == last && (first == b'"' || first == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}
```

- [ ] **Step 5: 跑测试看它通过**

```bash
cd cli && cargo test agents_md
```

Expected: 4 个测试全 PASS。

- [ ] **Step 6: Commit**

```bash
git add cli/src/agents_md.rs cli/src/main.rs
git commit -m "feat(cli): agents_md::parse_skill_frontmatter + tests"
```

---

## Task 4: agents_md.rs - render_agents_md (TDD)

**Files:**
- Modify: `cli/src/agents_md.rs`

- [ ] **Step 1: 在测试 mod 内追加失败测试**

```rust
    #[test]
    fn render_agents_md_includes_directive_header() {
        let out = render_agents_md("rule body\n", &[]);
        assert!(out.contains("# AGENTS DIRECTIVE — superpowers methodology"));
        assert!(out.contains("you MUST `Read`"));
    }

    #[test]
    fn render_agents_md_includes_skill_index() {
        let skills = vec![
            Frontmatter { name: "alpha".into(), description: "Do alpha".into() },
            Frontmatter { name: "beta".into(), description: "Do beta".into() },
        ];
        let out = render_agents_md("rule body\n", &skills);
        assert!(out.contains("- alpha — Do alpha"));
        assert!(out.contains("- beta — Do beta"));
    }

    #[test]
    fn render_agents_md_appends_project_rules_body() {
        let out = render_agents_md("PROJECT RULES BODY\n", &[]);
        assert!(out.contains("PROJECT RULES BODY"));
        // Body should appear AFTER the DIRECTIVE section (separated by ---)
        let directive_pos = out.find("AGENTS DIRECTIVE").unwrap();
        let body_pos = out.find("PROJECT RULES BODY").unwrap();
        assert!(directive_pos < body_pos);
    }
```

- [ ] **Step 2: 跑测试看它失败**

```bash
cd cli && cargo test agents_md
```

Expected: `cannot find function render_agents_md`。

- [ ] **Step 3: 实现 render_agents_md**

在 `agents_md.rs` 的 `parse_skill_frontmatter` 下方追加：

```rust
/// Render the AGENTS.md content shipped to project root.
///
/// Two parts:
///   1. AGENTS DIRECTIVE header — strong imperative tied to skill loading.
///   2. Verbatim copy of project_rules.md body (double-insurance redundancy).
pub fn render_agents_md(project_rules_body: &str, skills: &[Frontmatter]) -> String {
    let skill_index = skills
        .iter()
        .map(|s| format!("- {} — {}", s.name, s.description))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# AGENTS DIRECTIVE — superpowers methodology

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

{skill_index}

## Tool name reference

For tool name mappings between Claude Code and Trae IDE, see
`.trae/skills/superpowers/references/trae-tools.md`.

---

{project_rules_body}"#,
        skill_index = skill_index,
        project_rules_body = project_rules_body,
    )
}
```

- [ ] **Step 4: 跑测试**

```bash
cd cli && cargo test agents_md
```

Expected: 7 个测试全 PASS（4 + 3 新增）。

- [ ] **Step 5: Commit**

```bash
git add cli/src/agents_md.rs
git commit -m "feat(cli): agents_md::render_agents_md + tests"
```

---

## Task 5: rollback.rs - 写文件 + 自动回滚 (TDD)

**Files:**
- Create: `cli/src/rollback.rs`
- Modify: `cli/src/main.rs`（添加 `mod rollback;`）

- [ ] **Step 1: 写失败的单元测试**

`cli/src/rollback.rs`：

```rust
//! Atomic-ish file installer with rollback on failure.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum RollbackAction {
    Created(PathBuf),
    Backed { original: PathBuf, backup: PathBuf },
}

pub struct InstallSession {
    actions: Vec<RollbackAction>,
}

impl InstallSession {
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    pub fn write(&mut self, target: &Path, bytes: &[u8], backup_existing: bool) -> Result<()> {
        if target.exists() {
            if backup_existing {
                let bak = target.with_file_name(format!(
                    "{}.bak.{}",
                    target.file_name().unwrap().to_string_lossy(),
                    timestamp(),
                ));
                fs::rename(target, &bak)?;
                self.actions.push(RollbackAction::Backed {
                    original: target.to_path_buf(),
                    backup: bak,
                });
            } else {
                // overwrite path: no backup recorded (caller responsibility)
            }
        } else {
            self.actions.push(RollbackAction::Created(target.to_path_buf()));
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, bytes)?;
        Ok(())
    }

    /// Roll back all recorded actions in reverse order. Best-effort.
    pub fn rollback(self) {
        for action in self.actions.into_iter().rev() {
            match action {
                RollbackAction::Created(p) => {
                    let _ = fs::remove_file(&p);
                }
                RollbackAction::Backed { original, backup } => {
                    let _ = fs::remove_file(&original);
                    let _ = fs::rename(&backup, &original);
                }
            }
        }
    }

    pub fn commit(self) -> Vec<RollbackAction> {
        self.actions
    }
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Simple YYYYMMDD-HHMMSS without chrono dep — derive from epoch
    // Acceptable: just use epoch seconds for uniqueness; readable timestamp not strictly needed.
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_creates_new_file_and_records_created() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("a.txt");
        let mut s = InstallSession::new();
        s.write(&path, b"hello", false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
        let acts = s.commit();
        assert_eq!(acts.len(), 1);
        matches!(&acts[0], RollbackAction::Created(p) if p == &path);
    }

    #[test]
    fn write_backups_existing_when_backup_existing_true() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("a.txt");
        fs::write(&path, b"old").unwrap();

        let mut s = InstallSession::new();
        s.write(&path, b"new", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
        let acts = s.commit();
        assert_eq!(acts.len(), 1);
        // The backup file should exist
        match &acts[0] {
            RollbackAction::Backed { backup, .. } => {
                assert!(backup.exists());
                assert_eq!(fs::read_to_string(backup).unwrap(), "old");
            }
            _ => panic!("expected Backed action"),
        }
    }

    #[test]
    fn rollback_restores_state_after_partial_install() {
        let tmp = tempdir().unwrap();
        let preexisting = tmp.path().join("a.txt");
        fs::write(&preexisting, b"original_a").unwrap();

        let new_file = tmp.path().join("b.txt");

        let mut s = InstallSession::new();
        s.write(&preexisting, b"replaced_a", true).unwrap();
        s.write(&new_file, b"created_b", false).unwrap();

        s.rollback();

        assert_eq!(fs::read_to_string(&preexisting).unwrap(), "original_a");
        assert!(!new_file.exists());
    }
}
```

- [ ] **Step 2: 在 main.rs 加 `mod rollback;`**

- [ ] **Step 3: 跑测试**

```bash
cd cli && cargo test rollback
```

Expected: 3 个测试全 PASS。

- [ ] **Step 4: Commit**

```bash
git add cli/src/rollback.rs cli/src/main.rs
git commit -m "feat(cli): rollback::InstallSession + 单测"
```

---

## Task 6: addons/mod.rs - stub trait + handle_addons (TDD)

**Files:**
- Create: `cli/src/addons/mod.rs`
- Modify: `cli/src/main.rs`（添加 `mod addons;`）

- [ ] **Step 1: 写测试**

`cli/src/addons/mod.rs`：

```rust
//! Addon plugin scaffolding (v1: stub only).
//!
//! v0.2 will wire actual implementations behind this trait.

use anyhow::{bail, Result};

pub trait Addon {
    fn name(&self) -> &str;
}

/// Validate and respond to `--addons` flag values.
/// In v0.1 only "ddd" is recognized (and is a no-op stub).
/// Unknown names error out.
pub fn handle_addons(addons: &[String]) -> Result<()> {
    for a in addons {
        match a.as_str() {
            "ddd" => {
                println!("ℹ DDD plugin not yet implemented in v1.");
                println!("  Scheduled for sub-project 2 (see roadmap).");
                println!("  Continuing with base superpowers only.");
            }
            other => {
                bail!(
                    "Unknown addon: '{}' (supported in v0.1: 'ddd' [stub])",
                    other
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_addons_ok() {
        assert!(handle_addons(&[]).is_ok());
    }

    #[test]
    fn ddd_stub_ok() {
        assert!(handle_addons(&["ddd".to_string()]).is_ok());
    }

    #[test]
    fn unknown_addon_errors() {
        let r = handle_addons(&["unknown".to_string()]);
        assert!(r.is_err());
        assert!(format!("{:?}", r.unwrap_err()).contains("Unknown addon"));
    }
}
```

- [ ] **Step 2: 在 main.rs 加 `mod addons;`**

- [ ] **Step 3: 跑测试**

```bash
cd cli && cargo test addons
```

Expected: 3 个测试全 PASS。

- [ ] **Step 4: Commit**

```bash
git add cli/src/addons/mod.rs cli/src/main.rs
git commit -m "feat(cli): addons stub trait + handle_addons"
```

---

## Task 7: commands/init.rs - preflight 检查 (TDD)

**Files:**
- Create: `cli/src/commands/mod.rs`
- Create: `cli/src/commands/init.rs`
- Modify: `cli/src/main.rs`（添加 `mod commands;`）

- [ ] **Step 1: 创建 module 骨架**

`cli/src/commands/mod.rs`：

```rust
pub mod init;
```

`cli/src/commands/init.rs`：

```rust
//! `init` subcommand implementation.

use anyhow::{anyhow, Result};
use std::path::Path;

/// Pre-install validation. Returns Err if the project cannot accept install.
pub fn preflight(dir: &Path, force: bool) -> Result<()> {
    if !dir.exists() {
        return Err(anyhow!("target directory does not exist: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(anyhow!("target is not a directory: {}", dir.display()));
    }
    let rules = dir.join(".trae/rules/project_rules.md");
    if rules.exists() && !force {
        return Err(anyhow!(
            "already initialized at {} — use `superpowers-trae upgrade` (or pass --force to reinit)",
            dir.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preflight_passes_on_empty_dir() {
        let tmp = tempdir().unwrap();
        assert!(preflight(tmp.path(), false).is_ok());
    }

    #[test]
    fn preflight_fails_on_nonexistent() {
        let r = preflight(Path::new("/nonexistent/qwerty"), false);
        assert!(r.is_err());
    }

    #[test]
    fn preflight_fails_on_already_initialized_without_force() {
        let tmp = tempdir().unwrap();
        let rules = tmp.path().join(".trae/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("project_rules.md"), "x").unwrap();
        let r = preflight(tmp.path(), false);
        assert!(r.is_err());
        assert!(format!("{:?}", r.unwrap_err()).contains("already initialized"));
    }

    #[test]
    fn preflight_passes_on_already_initialized_with_force() {
        let tmp = tempdir().unwrap();
        let rules = tmp.path().join(".trae/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("project_rules.md"), "x").unwrap();
        assert!(preflight(tmp.path(), true).is_ok());
    }
}
```

- [ ] **Step 2: 在 main.rs 加 `mod commands;`**

- [ ] **Step 3: 跑测试**

```bash
cd cli && cargo test commands::init
```

Expected: 4 个测试全 PASS。

- [ ] **Step 4: Commit**

```bash
git add cli/src/commands/ cli/src/main.rs
git commit -m "feat(cli): commands::init::preflight + 单测"
```

---

## Task 8: commands/init.rs - 主流程 (写 dist + AGENTS.md + log)

**Files:**
- Modify: `cli/src/commands/init.rs`

- [ ] **Step 1: 添加 run 函数（在 preflight 下方，#[cfg(test)] 上方）**

```rust
use crate::{addons, agents_md, embed, rollback};
use include_dir::DirEntry;
use std::fs;
use std::path::PathBuf;

pub fn run(dir_arg: &str, force: bool, addons_list: &[String]) -> Result<()> {
    addons::handle_addons(addons_list)?;
    let dir = PathBuf::from(dir_arg).canonicalize().or_else(|_| {
        // If path doesn't exist yet, fall back to as-is (preflight will catch it).
        Ok::<PathBuf, anyhow::Error>(PathBuf::from(dir_arg))
    })?;
    preflight(&dir, force)?;

    let mut session = rollback::InstallSession::new();

    // Phase 1: install rules/project_rules.md (renamed from user_rules.md in dist)
    let rules_src = embed::DIST_USER
        .get_file("rules/user_rules.md")
        .ok_or_else(|| anyhow!("embedded dist missing rules/user_rules.md"))?;
    let project_rules_body = std::str::from_utf8(rules_src.contents())?;
    let rules_target = dir.join(".trae/rules/project_rules.md");
    if let Err(e) = session.write(&rules_target, rules_src.contents(), force) {
        session.rollback();
        return Err(e);
    }

    // Phase 2: install skills/ directory recursively
    let skills_dir = embed::DIST_USER
        .get_dir("skills/superpowers")
        .ok_or_else(|| anyhow!("embedded dist missing skills/superpowers"))?;
    if let Err(e) = write_embedded_dir(skills_dir, &dir.join(".trae"), force, &mut session) {
        session.rollback();
        return Err(e);
    }

    // Phase 3: AGENTS.md (dynamically rendered)
    let skills = collect_skill_frontmatters()?;
    let agents_md_body = agents_md::render_agents_md(project_rules_body, &skills);
    let agents_target = dir.join("AGENTS.md");
    if let Err(e) = session.write(&agents_target, agents_md_body.as_bytes(), force) {
        session.rollback();
        return Err(e);
    }

    // Phase 4: install log
    let log_target = dir.join(".trae/.superpowers-install.log");
    let log_body = render_install_log(&session.commit_actions_view());
    let _ = fs::create_dir_all(log_target.parent().unwrap());
    let _ = fs::write(&log_target, log_body);

    print_success(&dir);
    Ok(())
}

fn write_embedded_dir(
    src: &include_dir::Dir,
    target_root: &Path,
    backup_existing: bool,
    session: &mut rollback::InstallSession,
) -> Result<()> {
    for entry in src.entries() {
        match entry {
            DirEntry::Dir(d) => {
                write_embedded_dir(d, target_root, backup_existing, session)?;
            }
            DirEntry::File(f) => {
                let target = target_root.join(f.path());
                session.write(&target, f.contents(), backup_existing)?;
            }
        }
    }
    Ok(())
}

fn collect_skill_frontmatters() -> Result<Vec<agents_md::Frontmatter>> {
    let mut out = Vec::new();
    let dir = embed::DIST_USER
        .get_dir("skills/superpowers")
        .ok_or_else(|| anyhow!("embedded dist missing skills/superpowers"))?;
    for entry in dir.entries() {
        if let DirEntry::Dir(skill) = entry {
            // Skip `references/` — not a skill
            if skill.path().file_name().map(|n| n == "references").unwrap_or(false) {
                continue;
            }
            if let Some(skill_md) = skill.get_file(skill.path().join("SKILL.md")) {
                let text = std::str::from_utf8(skill_md.contents())?;
                let fm = agents_md::parse_skill_frontmatter(text)?;
                out.push(fm);
            }
        }
    }
    Ok(out)
}

fn render_install_log(_actions: &[rollback::RollbackAction]) -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("# install run at epoch {}\n", now)
}

fn print_success(dir: &Path) {
    use colored::Colorize;
    println!();
    println!("{} Initialized superpowers methodology in {}", "✓".green().bold(), dir.display());
    println!("  - rules:  .trae/rules/project_rules.md");
    println!("  - skills: .trae/skills/superpowers/   (14 skills)");
    println!("  - AGENTS: AGENTS.md  (project root, double-insurance)");
    println!();
    println!("To invoke methodology in Trae IDE, include \"superpowers\" in your prompt:");
    println!("  \"Use superpowers to brainstorm <X>\"");
    println!("  \"Use superpowers to plan <Y>\"");
    println!("  \"Use superpowers to debug <Z>\"");
    println!();
    println!("Optional: for @-mention shortcuts (@brainstorm / @write-plan / etc.), see");
    println!("docs/superpowers/trae-agents-setup.md in the superpowers-trae repo for");
    println!("paste-ready Custom Agent prompts.");
}
```

> 注：`session.commit_actions_view()` 暂时不存在——这是给 log 用的"看一眼但不消耗"接口。先在 rollback.rs 里加一个 `pub fn commit_actions_view(&self) -> &[RollbackAction] { &self.actions }`，再编译。

- [ ] **Step 2: 在 rollback.rs 添加 view 方法**

`cli/src/rollback.rs` 在 `impl InstallSession` 末尾加：

```rust
    pub fn commit_actions_view(&self) -> &[RollbackAction] {
        &self.actions
    }
```

- [ ] **Step 3: 在 main.rs 把 init 分支接到 run**

把 main.rs 里：

```rust
        Commands::Init { dir, force, addons } => {
            println!("init: dir={} force={} addons={:?}", dir, force, addons);
        }
```

替换为：

```rust
        Commands::Init { dir, force, addons } => {
            commands::init::run(&dir, force, &addons)?;
        }
```

- [ ] **Step 4: 编译 + 手工验证**

```bash
cd cli && cargo build
TMP=$(mktemp -d)
./target/debug/superpowers-trae init --dir "$TMP"
ls "$TMP/.trae/rules/" "$TMP/.trae/skills/superpowers/" | head
cat "$TMP/AGENTS.md" | head -30
rm -rf "$TMP"
```

Expected:
- 编译成功
- 末尾打印 "✓ Initialized superpowers methodology in ..." 那段
- 14 个 skill 子目录都在
- AGENTS.md 头部含 "AGENTS DIRECTIVE" + 14 行 skill index

- [ ] **Step 5: Commit**

```bash
git add cli/src/commands/init.rs cli/src/rollback.rs cli/src/main.rs
git commit -m "feat(cli): init 主流程 — 写 rules/skills/AGENTS.md/log"
```

---

## Task 9: 集成测试 init

**Files:**
- Create: `cli/tests/integration_init.rs`

- [ ] **Step 1: 写集成测试**

```rust
//! End-to-end tests for `superpowers-trae init`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn init_writes_expected_files() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(tmp.path().join(".trae/rules/project_rules.md").exists());
    assert!(tmp.path().join(".trae/skills/superpowers/brainstorming/SKILL.md").exists());
    assert!(tmp.path().join("AGENTS.md").exists());
    assert!(tmp.path().join(".trae/.superpowers-install.log").exists());
}

#[test]
fn init_refuses_already_initialized_without_force() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn init_force_creates_backup() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--force"])
        .assert()
        .success();

    let rules_dir = tmp.path().join(".trae/rules");
    let entries: Vec<_> = fs::read_dir(&rules_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(entries.iter().any(|n| n.starts_with("project_rules.md.bak.")));
}

#[test]
fn init_addons_ddd_prints_stub_and_exits_zero() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "ddd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DDD plugin not yet implemented"));
}

#[test]
fn init_addons_unknown_errors() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--addons", "bogus"])
        .assert()
        .failure();
}

#[test]
fn init_end_output_mentions_superpowers_trigger_word() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Use superpowers to"));
}

#[test]
fn agents_md_contains_directive_and_skill_index() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let body = fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(body.contains("AGENTS DIRECTIVE"));
    assert!(body.contains("- brainstorming —"));
    assert!(body.contains("- writing-plans —"));
}
```

- [ ] **Step 2: 跑测试**

```bash
cd cli && cargo test --test integration_init
```

Expected: 7 个集成测试全 PASS。

- [ ] **Step 3: Commit**

```bash
git add cli/tests/integration_init.rs
git commit -m "test(cli): init 端到端集成测试"
```

---

## Task 10: commands/upgrade.rs

**Files:**
- Create: `cli/src/commands/upgrade.rs`
- Modify: `cli/src/commands/mod.rs`
- Modify: `cli/src/main.rs`

- [ ] **Step 1: 写 upgrade.rs**

```rust
//! `upgrade` subcommand: refresh an already-initialized project to the binary's embedded version.

use crate::{addons, commands::init};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub fn run(dir_arg: &str, no_backup: bool, addons_list: &[String]) -> Result<()> {
    addons::handle_addons(addons_list)?;
    let dir = PathBuf::from(dir_arg).canonicalize()?;
    preflight(&dir)?;

    // upgrade reuses the init main flow with `force=true` semantics:
    //   - default backup_existing = true (= !no_backup)
    //   - skip the "already initialized" preflight (we already verified it IS initialized)
    init::run_with_options(&dir, /* force */ true, /* backup_existing */ !no_backup, addons_list)?;
    Ok(())
}

pub fn preflight(dir: &Path) -> Result<()> {
    let rules = dir.join(".trae/rules/project_rules.md");
    if !rules.exists() {
        return Err(anyhow!(
            "not initialized at {} — use `superpowers-trae init` first",
            dir.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preflight_fails_when_not_initialized() {
        let tmp = tempdir().unwrap();
        assert!(preflight(tmp.path()).is_err());
    }

    #[test]
    fn preflight_passes_when_initialized() {
        let tmp = tempdir().unwrap();
        let rules = tmp.path().join(".trae/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("project_rules.md"), "x").unwrap();
        assert!(preflight(tmp.path()).is_ok());
    }
}
```

- [ ] **Step 2: 重构 init.rs 让 upgrade 能复用**

把 `init::run` 改为薄包装，提取核心到 `run_with_options`：

```rust
pub fn run(dir_arg: &str, force: bool, addons_list: &[String]) -> Result<()> {
    addons::handle_addons(addons_list)?;
    let dir = PathBuf::from(dir_arg).canonicalize().or_else(|_| {
        Ok::<PathBuf, anyhow::Error>(PathBuf::from(dir_arg))
    })?;
    preflight(&dir, force)?;
    run_with_options(&dir, force, /* backup_existing */ force, addons_list)
}

pub fn run_with_options(
    dir: &Path,
    _force: bool,
    backup_existing: bool,
    _addons_list: &[String],
) -> Result<()> {
    let mut session = rollback::InstallSession::new();

    let rules_src = embed::DIST_USER
        .get_file("rules/user_rules.md")
        .ok_or_else(|| anyhow!("embedded dist missing rules/user_rules.md"))?;
    let project_rules_body = std::str::from_utf8(rules_src.contents())?;
    let rules_target = dir.join(".trae/rules/project_rules.md");
    if let Err(e) = session.write(&rules_target, rules_src.contents(), backup_existing) {
        session.rollback();
        return Err(e);
    }

    let skills_dir = embed::DIST_USER
        .get_dir("skills/superpowers")
        .ok_or_else(|| anyhow!("embedded dist missing skills/superpowers"))?;
    if let Err(e) = write_embedded_dir(skills_dir, &dir.join(".trae"), backup_existing, &mut session) {
        session.rollback();
        return Err(e);
    }

    let skills = collect_skill_frontmatters()?;
    let agents_md_body = agents_md::render_agents_md(project_rules_body, &skills);
    let agents_target = dir.join("AGENTS.md");
    if let Err(e) = session.write(&agents_target, agents_md_body.as_bytes(), backup_existing) {
        session.rollback();
        return Err(e);
    }

    let log_target = dir.join(".trae/.superpowers-install.log");
    let log_body = render_install_log(session.commit_actions_view());
    let _ = std::fs::create_dir_all(log_target.parent().unwrap());
    let _ = std::fs::write(&log_target, log_body);

    print_success(dir);
    Ok(())
}
```

（`run` 只剩薄壳；`run_with_options` 接收 `backup_existing` 参数，不调 preflight。）

- [ ] **Step 3: 在 commands/mod.rs 加 `pub mod upgrade;`**

- [ ] **Step 4: 在 main.rs 接通 upgrade 分支**

把：

```rust
        Commands::Upgrade { dir, no_backup, addons } => {
            println!("upgrade: dir={} no_backup={} addons={:?}", dir, no_backup, addons);
        }
```

改为：

```rust
        Commands::Upgrade { dir, no_backup, addons } => {
            commands::upgrade::run(&dir, no_backup, &addons)?;
        }
```

- [ ] **Step 5: 跑测试**

```bash
cd cli && cargo test
```

Expected: 全部 PASS（rollback 3 + agents_md 7 + addons 3 + commands::init 4 + commands::upgrade 2 + integration_init 7 = 26 测试）。

- [ ] **Step 6: Commit**

```bash
git add cli/src/commands/ cli/src/main.rs
git commit -m "feat(cli): upgrade 命令 + 共享 run_with_options"
```

---

## Task 11: 集成测试 upgrade

**Files:**
- Create: `cli/tests/integration_upgrade.rs`

- [ ] **Step 1: 写测试**

```rust
//! End-to-end tests for `superpowers-trae upgrade`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn upgrade_refuses_uninitialized_project() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn upgrade_creates_backup_by_default() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Modify a file so we can detect the backup
    let target = tmp.path().join(".trae/skills/superpowers/brainstorming/SKILL.md");
    fs::write(&target, "USER MODIFIED").unwrap();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Backup file should exist
    let parent = target.parent().unwrap();
    let entries: Vec<_> = fs::read_dir(parent)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(entries.iter().any(|n| n.starts_with("SKILL.md.bak.")));
    // Restored content should be from embedded dist (not "USER MODIFIED")
    assert_ne!(fs::read_to_string(&target).unwrap(), "USER MODIFIED");
}

#[test]
fn upgrade_no_backup_skips_backup() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["upgrade", "--dir", tmp.path().to_str().unwrap(), "--no-backup"])
        .assert()
        .success();

    // No .bak.* files should exist anywhere
    let bak_count = walkdir_count_baks(tmp.path());
    assert_eq!(bak_count, 0);
}

fn walkdir_count_baks(root: &std::path::Path) -> usize {
    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        if let Ok(rd) = fs::read_dir(&p) {
            for e in rd.flatten() {
                let path = e.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.contains(".bak."))
                    .unwrap_or(false)
                {
                    count += 1;
                }
            }
        }
    }
    count
}
```

- [ ] **Step 2: 加 walkdir crate 到 dev-deps**

不需要——上面手写了 walkdir 等价实现。`Cargo.toml` 不变。

- [ ] **Step 3: 跑测试**

```bash
cd cli && cargo test --test integration_upgrade
```

Expected: 3 个测试全 PASS。

- [ ] **Step 4: Commit**

```bash
git add cli/tests/integration_upgrade.rs
git commit -m "test(cli): upgrade 端到端集成测试"
```

---

## Task 12: commands/status.rs

**Files:**
- Create: `cli/src/commands/status.rs`
- Modify: `cli/src/commands/mod.rs`
- Modify: `cli/src/main.rs`

- [ ] **Step 1: 写 status.rs**

```rust
//! `status` subcommand: report installation completeness.

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_SKILL_COUNT: usize = 14;

pub fn run(dir_arg: &str) -> Result<()> {
    let dir = PathBuf::from(dir_arg).canonicalize()?;

    let version = env!("CARGO_PKG_VERSION");
    let upstream_version = "5.0.7"; // matches Cargo.toml [package.metadata.upstream]
    println!("superpowers-trae v{} (embedded superpowers {})", version, upstream_version);
    println!("Project: {}", dir.display());
    println!("─────────────────────────────────────────────────────────");

    let mut all_ok = true;

    let rules = dir.join(".trae/rules/project_rules.md");
    if rules.exists() {
        let size = fs::metadata(&rules).map(|m| m.len()).unwrap_or(0);
        println!("{} project_rules.md         (.trae/rules/, {} bytes)", "✓".green().bold(), size);
    } else {
        println!("{} project_rules.md         MISSING", "✗".red().bold());
        all_ok = false;
    }

    let skills_root = dir.join(".trae/skills/superpowers");
    let skill_count = count_skill_dirs(&skills_root);
    if skill_count == EXPECTED_SKILL_COUNT {
        println!("{} superpowers skills       {} / {} expected", "✓".green().bold(), skill_count, EXPECTED_SKILL_COUNT);
    } else {
        println!("{} superpowers skills       {} / {} expected", "✗".red().bold(), skill_count, EXPECTED_SKILL_COUNT);
        all_ok = false;
    }

    let agents_md = dir.join("AGENTS.md");
    if agents_md.exists() {
        let size = fs::metadata(&agents_md).map(|m| m.len()).unwrap_or(0);
        println!("{} AGENTS.md                (project root, {} bytes)", "✓".green().bold(), size);
    } else {
        println!("{} AGENTS.md                MISSING", "✗".red().bold());
        all_ok = false;
    }

    let log = dir.join(".trae/.superpowers-install.log");
    if log.exists() {
        if let Ok(meta) = fs::metadata(&log) {
            if let Ok(modified) = meta.modified() {
                let secs = modified.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                println!("{} Last init/upgrade        epoch {} (from install log mtime)", "ℹ".blue().bold(), secs);
            }
        }
    }

    println!();
    println!("Addons: none");

    if !all_ok {
        anyhow::bail!("status: one or more required files missing");
    }
    Ok(())
}

fn count_skill_dirs(root: &Path) -> usize {
    if !root.is_dir() {
        return 0;
    }
    fs::read_dir(root)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_dir()
                        && e.file_name().to_str().map(|n| n != "references").unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}
```

- [ ] **Step 2: commands/mod.rs 加 `pub mod status;`**

- [ ] **Step 3: main.rs 接通 status 分支**

```rust
        Commands::Status { dir } => {
            commands::status::run(&dir)?;
        }
```

- [ ] **Step 4: 编译 + 手工验证**

```bash
cd cli && cargo build
TMP=$(mktemp -d)
./target/debug/superpowers-trae init --dir "$TMP"
./target/debug/superpowers-trae status --dir "$TMP"
rm -rf "$TMP"
```

Expected: status 输出三行 `✓` 行 + Addons: none + 退出码 0。

- [ ] **Step 5: Commit**

```bash
git add cli/src/commands/ cli/src/main.rs
git commit -m "feat(cli): status 命令实现"
```

---

## Task 13: 集成测试 status

**Files:**
- Create: `cli/tests/integration_status.rs`

- [ ] **Step 1: 写测试**

```rust
//! End-to-end tests for `superpowers-trae status`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cli() -> Command {
    Command::cargo_bin("superpowers-trae").unwrap()
}

#[test]
fn status_reports_complete_install() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("project_rules.md"))
        .stdout(predicate::str::contains("14 / 14 expected"))
        .stdout(predicate::str::contains("AGENTS.md"));
}

#[test]
fn status_fails_when_uninstalled() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn status_fails_when_skills_partially_missing() {
    let tmp = tempdir().unwrap();
    cli()
        .args(["init", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Delete one skill dir
    fs::remove_dir_all(tmp.path().join(".trae/skills/superpowers/brainstorming")).unwrap();

    cli()
        .args(["status", "--dir", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stdout(predicate::str::contains("13 / 14 expected"));
}
```

- [ ] **Step 2: 跑测试**

```bash
cd cli && cargo test --test integration_status
```

Expected: 3 个测试全 PASS。

- [ ] **Step 3: Commit**

```bash
git add cli/tests/integration_status.rs
git commit -m "test(cli): status 端到端集成测试"
```

---

## Task 14: Makefile（maintainer 工作流）

**Files:**
- Create: `Makefile`（仓库根）

- [ ] **Step 1: 写 Makefile**

```makefile
.PHONY: all sync build cli-build cli-install cli-test python-test test e2e-smoke clean upgrade-superpowers

CLI_BIN := cli/target/release/superpowers-trae

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
	$(CLI_BIN) init --dir "$$TMP" && \
	$(CLI_BIN) status --dir "$$TMP" && \
	$(CLI_BIN) upgrade --dir "$$TMP" && \
	$(CLI_BIN) status --dir "$$TMP" && \
	rm -rf "$$TMP" && \
	echo "✓ e2e smoke passed"

upgrade-superpowers: sync build cli-build
	@echo "✓ Maintainer pipeline done. Review with: git diff dist/ && git diff cli/"

clean:
	rm -rf dist/
	cd cli && cargo clean
```

- [ ] **Step 2: 验证 make 入口**

```bash
make test         # python + Rust 全部
make e2e-smoke    # init/status/upgrade/status 链路
```

Expected: 两个目标都 PASS。

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "feat: Makefile maintainer 工作流"
```

---

## Task 15: GitHub Actions release workflow

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: 写 workflow**

```yaml
name: release

on:
  push:
    tags: ["v*"]

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            archive_suffix: linux-x86_64
          - target: aarch64-apple-darwin
            os: macos-14
            archive_suffix: macos-aarch64
          - target: x86_64-apple-darwin
            os: macos-13
            archive_suffix: macos-x86_64

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Verify dist/user is committed
        run: |
          test -f dist/user/rules/user_rules.md || {
            echo "ERROR: dist/user/ not in repo. Run 'make all' locally and commit before tagging." >&2
            exit 1
          }

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: cargo build
        working-directory: cli
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package
        run: |
          ARCHIVE="superpowers-trae-${{ matrix.archive_suffix }}.tar.gz"
          tar czf "$ARCHIVE" \
            -C cli/target/${{ matrix.target }}/release superpowers-trae \
            -C ../../../../ LICENSE README.md
          echo "ARCHIVE=$ARCHIVE" >> $GITHUB_ENV

      - uses: softprops/action-gh-release@v2
        with:
          files: ${{ env.ARCHIVE }}
```

- [ ] **Step 2: 本地 dry-run 一下 yaml 语法**

```bash
# 用 yamllint 或简单 python -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
```

Expected: 无错误。

- [ ] **Step 3: 验证 LICENSE 文件存在（packaging 需要）**

```bash
ls LICENSE 2>/dev/null && echo "OK" || echo "MISSING - need to create one"
```

如果 MISSING：写一个 MIT LICENSE 到仓库根。

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: GitHub Actions release workflow（三平台）"
```

---

## Task 16: 仓库根 README + crate README + AGENTS.md 链接

**Files:**
- Modify: `README.md`（仓库根，加 Rust CLI 安装段）
- Modify: `cli/README.md`（写完整 crate 自述）

- [ ] **Step 1: 改写仓库根 README.md**

读现有 README.md，把当前的"## 快速开始"段（包含 sync-upstream / build / install --user / install --project 的旧 shell 流程描述）整体替换为新内容。用 Write 工具直接覆盖整个 README.md，新内容完整版如下（用 `cat <<'OUTER_EOF' > README.md` heredoc 写入更稳）：

```bash
cat > README.md <<'OUTER_EOF'
# cc-superpower-to-trae

把 [Superpowers](https://github.com/obra/superpowers) 整套方法论（14 skills + 3 commands + code-reviewer agent + SessionStart hook）移植到 [Trae IDE](https://www.trae.ai/)。

## 目录

- 设计文档：[docs/superpowers/specs/](docs/superpowers/specs/)
- 实施计划：[docs/superpowers/plans/](docs/superpowers/plans/)
- **Trae IDE 自定义 Agent 创建指南**（可选，提供 @-mention 快捷调用）：[docs/superpowers/trae-agents-setup.md](docs/superpowers/trae-agents-setup.md)
- 手工冒烟测试：[docs/superpowers/manual-smoke-test.md](docs/superpowers/manual-smoke-test.md)

## 快速开始（end-user）

### 1. 装 superpowers-trae binary

**从源码（需要 Rust 1.75+）：**

```bash
cargo install --git https://github.com/amwtke/cc-superpower-to-trae \
  --tag v0.1.0 superpowers-trae
```

**预编译二进制：** 在 [Releases](https://github.com/amwtke/cc-superpower-to-trae/releases) 选对应平台 tar.gz 解压到 PATH 上（Linux x86_64 / macOS Apple Silicon / macOS Intel）。

### 2. 在你的项目里 init

```bash
cd <my-project>
superpowers-trae init
```

写入：
- `.trae/rules/project_rules.md` — Trae 项目规则（含 superpowers 元规则）
- `.trae/skills/superpowers/` — 14 个 SKILL.md 子目录
- `AGENTS.md` — 项目根，作为 project_rules 的双保险

### 3. 在 Trae IDE 里调用

提示词里说"使用 superpowers ..."触发：

```
使用 superpowers 头脑风暴一个订单管理系统的设计
```

或可选地参 [docs/superpowers/trae-agents-setup.md](docs/superpowers/trae-agents-setup.md) 在 Trae UI 创建 Custom Agent，用 `@brainstorm` / `@write-plan` / `@code-reviewer` 等 @-mention 调用（无需触发词）。

## Maintainer 工作流

```bash
make all        # sync upstream + build dist + cargo build
make test       # python + Rust 全套测试
make e2e-smoke  # init/upgrade/status 链路 smoke
git tag v0.1.x && git push --tags  # 触发 release workflow
```

## 项目结构

| 路径 | 说明 |
|---|---|
| `upstream/` | superpowers 原版源（入 git，maintainer pipeline 输入） |
| `src/` | python 转换器：mappings.json + transform.py / render.py / build_helpers.py |
| `scripts/` | shell maintainer 入口：sync-upstream.sh / build.sh |
| `dist/` | python pipeline 产物，被 Rust binary 编译时嵌入 |
| `cli/` | Rust CLI 工程（superpowers-trae binary） |
| `tests/` | python 单测 + 端到端 fixture smoke |
| `docs/superpowers/` | 设计文档、实施计划、手工测试步骤 |

## License

本项目仅做工具链与转换器；移植的 superpowers 内容版权归原作者所有，详见 `upstream/LICENSE`。
OUTER_EOF
```

- [ ] **Step 2: 写 cli/README.md（crate 自述）**

```bash
cat > cli/README.md <<'OUTER_EOF'
# superpowers-trae

One-shot installer for the [Superpowers](https://github.com/obra/superpowers) methodology in [Trae IDE](https://www.trae.ai/) projects.

## Install

```bash
cargo install --git https://github.com/amwtke/cc-superpower-to-trae \
  --tag v0.1.0 superpowers-trae
```

Or download a pre-built binary from [Releases](https://github.com/amwtke/cc-superpower-to-trae/releases).

## Usage

```bash
superpowers-trae init                     # first-time install in current dir
superpowers-trae init --dir /path/to/proj # in another dir
superpowers-trae init --force             # overwrite existing
superpowers-trae upgrade                  # refresh to embedded version (with backup)
superpowers-trae upgrade --no-backup      # skip backup
superpowers-trae status                   # check install state
```

## Invocation in Trae

Tell the Builder agent:

> Use superpowers to brainstorm <X>

Optional: create 4 Custom Agents (@brainstorm / @write-plan / @execute-plan / @code-reviewer) for `@`-mention invocation. See `docs/superpowers/trae-agents-setup.md` in the parent repo.

## License

MIT
OUTER_EOF
```

- [ ] **Step 3: 验证两个 README**

```bash
head -30 README.md
head -30 cli/README.md
# 确认无 OUTER_EOF 残留
grep "OUTER_EOF" README.md cli/README.md && echo "BAD" || echo "OK"
```

Expected: head 输出新内容；grep 不命中（heredoc 终止符正确剥离）。

- [ ] **Step 4: Commit**

```bash
git add README.md cli/README.md
git commit -m "docs: README 加 superpowers-trae CLI 使用与 maintainer 段"
```

---

## Self-Review

完成所有 16 个 Task 后做端到端：

- [ ] 全套测试 + smoke：

```bash
make test
make e2e-smoke
```

Expected：全部 PASS。

- [ ] 把当前 spec 跟 plan 中的产物逐一勾对：

  - [x] §2 目标 (1) 三命令 init / upgrade / status ✔（Task 1, 8, 10, 12）
  - [x] §2 目标 (2) include_dir 嵌入 dist/user ✔（Task 2）
  - [x] §2 目标 (3) init 写 4 项 + log ✔（Task 8）
  - [x] §2 目标 (4) 末尾打印调用方式 ✔（Task 8 print_success）
  - [x] §2 目标 (5) upgrade 默认备份 + --no-backup ✔（Task 10）
  - [x] §2 目标 (6) status OK/MISSING + 退出码 ✔（Task 12）
  - [x] §2 目标 (7) --addons ddd stub ✔（Task 6）
  - [x] §2 目标 (8) 现有 python pipeline 保留 ✔（Task 14 Makefile）
  - [x] §2 目标 (9) 三平台 binary ✔（Task 15）
  - [x] §2 目标 (10) GitHub Actions tag 触发 ✔（Task 15）

如有 mappings.json 或 dist/ 在升级中漂移，按 §11 风险表的缓解走。

---

## 实施前注意

按 subagent-driven-development skill 要求：**先建 worktree** 再开始任务执行。Branch 名建议 `feat/cli-v0-1-0`。

```bash
git worktree add .worktrees/feat-cli-v0-1-0 -b feat/cli-v0-1-0
cd .worktrees/feat-cli-v0-1-0
```
