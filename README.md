# cc-superpower-to-trae

把 [Superpowers](https://github.com/obra/superpowers) 整套方法论（14 skills + 3 commands + code-reviewer agent + SessionStart hook）移植到 [Trae IDE](https://www.trae.ai/)。

## 目录

- 设计文档：[docs/superpowers/specs/](docs/superpowers/specs/)
- 实施计划：[docs/superpowers/plans/](docs/superpowers/plans/)
- **Trae IDE 自定义 Agent 创建指南**（可选，提供 @-mention 快捷调用）：[docs/superpowers/trae-agents-setup.md](docs/superpowers/trae-agents-setup.md)
- 手工冒烟测试：[docs/superpowers/manual-smoke-test.md](docs/superpowers/manual-smoke-test.md)

## 快速开始（end-user）

### 1. 装 superpowers-trae binary

**一键安装（推荐）：**

```bash
# Linux + macOS
curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
```

跑完后二进制装到 `~/.local/bin/`（POSIX）或 `%USERPROFILE%\bin\`（Windows），并自动加入 PATH。如已在 PATH 里则跳过。

可选环境变量：

| 变量 | 作用 | 默认 |
|---|---|---|
| `SUPERPOWERS_INSTALL_DIR` | 装到哪 | `~/.local/bin` / `%USERPROFILE%\bin` |
| `SUPERPOWERS_VERSION` | 装哪个版本 tag | `latest` |

<details>
<summary><b>不想跑脚本？手工安装</b></summary>

从 [Releases](https://github.com/amwtke/superpowers-to-trae/releases) 下载对应平台压缩包：

- Linux x86_64：`superpowers-trae-linux-x86_64.tar.gz`
- macOS Apple Silicon：`superpowers-trae-macos-aarch64.tar.gz`
- macOS Intel：`superpowers-trae-macos-x86_64.tar.gz`
- Windows x86_64：`superpowers-trae-windows-x86_64.zip`

POSIX：`tar xzf <archive> && sudo install -m 0755 superpowers-trae /usr/local/bin/`
Windows：解压后把 `superpowers-trae.exe` 放到任一在 `%PATH%` 的目录。
macOS 首次执行被 Gatekeeper 拦截：`xattr -d com.apple.quarantine superpowers-trae`。

</details>

<details>
<summary><b>从源码安装（需要 Rust 1.75+）</b></summary>

```bash
cargo install --git https://github.com/amwtke/superpowers-to-trae \
  --tag v0.3.0 superpowers-trae
```

</details>

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

### 4. （可选）启用 DDD plugin

如果你要用 DDD（领域驱动设计）方法论先建模、再交给 superpowers 实现：

````bash
# 第一次 init 同时装 DDD
superpowers-trae init --addons ddd

# 已 init 后追加
superpowers-trae upgrade --addons ddd
````

写入额外文件：
- `.trae/skills/ddd/{ddd-storm,ddd-model,ddd-spec}/SKILL.md` — 3 个 DDD skill
- `DOMAIN.md` — 领域模型 SSOT（**只在不存在时安装**，后续 upgrade 不会覆盖你的模型）
- `README-DDD-HARNESS.md` — DDD 工作流向导
- `AGENTS.md` 末尾追加 "DDD methodology" 元规则段

工作流：
```
业务需求 → 使用 ddd-storm <描述>（事件风暴）
       → 使用 ddd-model（建领域模型，更新 DOMAIN.md）
       → 使用 ddd-spec <场景>（生成 spec）
       → 使用 superpowers brainstorming（决定技术栈）
       → superpowers writing-plans / executing-plans（TDD 实施）
```

DDD plugin 移植自 [ddd-run](https://github.com/amwtke/ddd-run)。

## Maintainer 工作流

```bash
make all        # sync upstream + build dist + cargo build
make test       # python + Rust 全套测试
make e2e-smoke  # init/upgrade/status 链路 smoke
git tag v0.2.x && git push --tags  # 触发 release workflow
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
