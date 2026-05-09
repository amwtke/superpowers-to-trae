# superpowers-to-trae

把 [Superpowers](https://github.com/obra/superpowers) 整套方法论（14 skills + 多个命令 + agents + SessionStart hook）移植到 [Trae IDE](https://www.trae.ai/) 的一键安装器。

---

## 一、用户：获取二进制 + 初始化项目

### 1.1 装二进制

```bash
# Linux + macOS
curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
```

二进制装到 `~/.local/bin/`（POSIX）或 `%USERPROFILE%\bin\`（Windows），自动加入 PATH。

可选环境变量：

| 变量 | 作用 | 默认 |
|---|---|---|
| `SUPERPOWERS_INSTALL_DIR` | 装到哪 | `~/.local/bin` / `%USERPROFILE%\bin` |
| `SUPERPOWERS_VERSION` | 装哪个版本 tag | `latest` |

<details>
<summary><b>备选：手工下 release 包 / 从源码 cargo install</b></summary>

手工：[Releases](https://github.com/amwtke/superpowers-to-trae/releases) 下载平台压缩包，解压后扔进 PATH 内目录即可。macOS 首次执行被 Gatekeeper 拦截：`xattr -d com.apple.quarantine superpowers-trae`。

源码（需 Rust 1.75+）：
```bash
cargo install --git https://github.com/amwtke/superpowers-to-trae \
  --tag v0.3.2 superpowers-trae
```

</details>

### 1.2 在你的项目里 init

```bash
cd <my-project>
superpowers-trae init
```

写入：
- `.trae/rules/project_rules.md` — Trae 项目规则（含 superpowers 元规则）
- `.trae/skills/superpowers/` — 14 个 SKILL.md
- `AGENTS.md` — 项目根，作为双保险

可选 addon：

| Addon | 来自 | 装什么 | 启用 |
|---|---|---|---|
| `ddd` | [amwtke/ddd-run](https://github.com/amwtke/ddd-run) | 3 个 DDD skill（事件风暴 → 建模 → spec）+ `DOMAIN.md`（install-once）+ `README-DDD-HARNESS.md` | `--addons ddd` |
| `bob` | [amwtke/run-bob](https://github.com/amwtke/run-bob) | 3 个 Bob 4 环 Clean Architecture skill（身份测试 → 4 环设计 → spec）+ `BOB.md`（install-once）+ `README-RUN-BOB.md` | `--addons bob` |

```bash
superpowers-trae init --addons ddd            # 首次单装 ddd
superpowers-trae init --addons bob            # 首次单装 bob
superpowers-trae init --addons ddd,bob        # 两个一起装
superpowers-trae upgrade --addons bob         # 已 init 项目追加 bob
```

`DOMAIN.md` / `BOB.md` 是 install-once 文件，已存在不覆盖。3 个 skill 与 `README-*.md` 每次 upgrade 会刷成最新（默认开 backup，加 `--no-backup` 跳过）。

### 1.3 在 Trae IDE 里调用

提示词里说"使用 superpowers ..."触发，例如：
```
使用 superpowers 头脑风暴一个订单管理系统的设计
```

可选：参考 [docs/superpowers/trae-agents-setup.md](docs/superpowers/trae-agents-setup.md) 在 Trae UI 建 Custom Agent，用 `@brainstorm` / `@write-plan` 等 @-mention 直接调用。

---

## 二、用户：升级

新版本发布时，两步搞定。

### 2.1 升级二进制

**重新跑一遍安装命令即可**（`install.sh` / `install.ps1` 自带覆盖逻辑，会拉最新 release）：

```bash
# Linux + macOS
curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
```

### 2.2 升级项目内的 skills

进到已 init 的项目根目录跑：

```bash
superpowers-trae upgrade
```

行为：
- 备份现有 `.trae/skills/superpowers/` 与 `project_rules.md`（默认开 backup，加 `--no-backup` 跳过）
- 用新 binary 内置的 dist 整体覆盖 skills 与 rules
- `AGENTS.md` 仅追加 superpowers 段，**保留你自己加的内容**
- 装了 addon 的项目要带上对应 `--addons ddd` / `--addons bob` / `--addons ddd,bob` 才会同时升级 addon skills；`DOMAIN.md` / `BOB.md` 永远不覆盖

---

## 三、维护者：发布新版本

本仓库有三条上游、三条同步流水线：

| 上游 | 维护内容 | 触发 |
|---|---|---|
| [obra/superpowers](https://github.com/obra/superpowers) | `upstream/` 镜像 + `dist/` 渲染产物 | `/update-from-upstream` |
| [amwtke/ddd-run](https://github.com/amwtke/ddd-run) | `cli/templates/ddd/` 模板 | `/sync-ddd-run` |
| [amwtke/run-bob](https://github.com/amwtke/run-bob) | `cli/templates/bob/` 模板 | `/sync-run-bob` |

三条流水线的当前上游版本都记录在 `cli/Cargo.toml` 的 `[package.metadata.upstream]` 段（`superpowers_version` / `ddd_run_version` / `run_bob_version`），是判断"是否已是最新"的 SSOT。

### 3.1 同步 superpowers 上游 — `/update-from-upstream`

由 `.claude/skills/update-from-upstream/SKILL.md` 驱动的 9 步流水线：

1. 在 Claude Code 里 `/plugin update superpowers@claude-plugins-official` + `/reload-plugins` → cache 拉新版本
2. 对比 `upstream/VERSION` 与 cache 最大版本——一致则直接终止
3. `make upgrade-superpowers`（= `sync` + `build` + `cli-build`）
4. `git diff upstream/RELEASE-NOTES.md` 找 breaking removals
5. 跟进 cli 代码（如果 cli 硬编码引用了被删的 slash command / agent）
6. `make test` 全绿
7. Bump `cli/Cargo.toml` 版本（上游 minor → cli minor，patch → patch）
8. `git commit -am "chore(upgrade): X → Y"` + `git tag vX.Y.Z` + `git push --tags`
9. 输出消费者升级通知文案（即 §二 的两条命令）

跳过 skill、手动跑也可以：

```bash
make upgrade-superpowers   # 同步 + 构建
make test                  # 全套测试
# 改 cli/Cargo.toml version
git commit -am "..." && git tag v0.x.y && git push origin main --tags
```

### 3.2 同步 ddd-run 上游 — `/sync-ddd-run`

由 `.claude/skills/sync-ddd-run/SKILL.md` 驱动的 9 步流水线：

1. `git status` 工作树必须干净
2. `git clone --depth 50 https://github.com/amwtke/ddd-run.git /tmp/ddd-run-sync`
3. 对比 `cli/Cargo.toml` 里 `ddd_run_version` 与上游 HEAD SHA——一致则直接终止
4. Diff 5 个映射 md 文件（3 个 ddd skill + `DOMAIN.md` + `README-DDD-HARNESS.md`）+ 列出上游全部 templates 文件，对比映射表找新增的非 md 文件
5. 强制覆盖 5 个映射 md（`claude-md-merge.md` 是 trae 专用，永不覆盖）
6. `make test` 全绿
7. 把 `Cargo.toml` 里 `ddd_run_version` 更新为新 SHA
8. Bump `cli/Cargo.toml` 版本（仅 md 内容增量 → patch；改 cli 代码或加新 skill → minor）
9. `git commit` + `git tag vX.Y.Z` + `git push --tags` + 输出消费者升级通知文案

**不在映射表里的上游文件 skill 不动**——例如上游的 `root/CLAUDE.md`（trae 用 `claude-md-merge.md` 替代）、`root/CleanArchitectureTest.java`（属 cli 安装逻辑改动，需单独决策）。skill 只在 Step 4 提醒。

### 3.3 同步 run-bob 上游 — `/sync-run-bob`

由 `.claude/skills/sync-run-bob/SKILL.md` 驱动的 9 步流水线（结构与 sync-ddd-run 一致）：

1. `git status` 工作树必须干净
2. `git clone --depth 50 https://github.com/amwtke/run-bob.git /tmp/run-bob-sync`
3. 对比 `cli/Cargo.toml` 里 `run_bob_version` 与上游 HEAD SHA——一致则直接终止
4. 在 `/tmp/run-bob-sync-rewrite/` 先做 `s/ARCHITECTURE\.md/BOB.md/g` 字串改写（本仓库统一用 `BOB.md` 避撞名），再 diff 5 个映射 md 文件（3 个 bob skill + `BOB.md` + `README-RUN-BOB.md`）+ 列出上游全部 templates 文件
5. 强制覆盖 5 个映射 md（已 sed 改写过）；防御性 grep 残留 `ARCHITECTURE.md` —— `claude-md-merge.md` 是 trae 专用，永不覆盖
6. `make test` 全绿
7. 把 `Cargo.toml` 里 `run_bob_version` 更新为新 SHA
8. Bump `cli/Cargo.toml` 版本（仅 md 内容增量 → patch；改 cli 代码或加新 skill → minor）
9. `git commit` + `git tag vX.Y.Z` + `git push --tags` + 输出消费者升级通知文案

**run-bob 与 ddd-run 的差异**：bob 上游里 `ARCHITECTURE.md` 在本仓库统一改名 `BOB.md`，`UseCase.java` / `TransactionalUseCaseDecorator.java` / `CleanArchitectureTest.java` 不进同步范围（属 cli 安装逻辑改动，需单独决策）。

### 发版历史

| 版本 | 主要内容 |
|---|---|
| **v0.4.0** (2026-05-09) | 加 `--addons bob`：基于 run-bob 上游的 Bob 大叔 4 环 Clean Architecture skill 集（bob-identify / bob-onion / bob-spec）+ `BOB.md`（install-once）+ `README-RUN-BOB.md`；新增 `/sync-run-bob` 维护者流水线；`status` 子命令同时报告 ddd / bob addon 状态 |
| v0.3.2 (2026-05-08) | 同步 ddd-run `8cb30e0 → a5f986e`：强化 4 环 Clean Architecture 边界——ddd-model.md 加 2 条领域层污染反模式；ddd-spec.md 加 Result/UseCase/framework 装配模板（用例层零 Spring/SLF4J）；README-DDD-HARNESS.md 加 4 环速览 + ArchUnit 守卫说明 |
| v0.3.1 | installer 装完后检测 PATH 冲突 + 残留 |
| v0.3.0 | 同步 superpowers 5.0.7 → 5.1.0；init/upgrade 自动维护 .gitignore 段 |

### 项目结构（维护者参考）

| 路径 | 说明 |
|---|---|
| `upstream/` | superpowers 原版源镜像（入 git，pipeline 输入） |
| `src/` | Python 转换器：mappings.json + transform/render/build_helpers |
| `scripts/` | sync-upstream.sh / build.sh |
| `dist/` | Python pipeline 产物，被 Rust binary 编译时嵌入 |
| `cli/` | Rust CLI 工程（`superpowers-trae` binary）；addon 模板在 `cli/templates/{ddd,bob}/` |
| `tests/` | Python 单测 + 端到端 smoke |
| `.claude/skills/update-from-upstream/` | superpowers 同步流水线 skill |
| `.claude/skills/sync-ddd-run/` | ddd-run 同步流水线 skill |
| `.claude/skills/sync-run-bob/` | run-bob 同步流水线 skill |

---

## License

本项目仅做工具链与转换器；移植的 superpowers 内容版权归原作者所有，详见 `upstream/LICENSE`。
