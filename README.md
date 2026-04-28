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
