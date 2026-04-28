# cc-superpower-to-trae

把 [Superpowers](https://github.com/obra/superpowers) 整套方法论（14 skills + 3 commands + code-reviewer agent + SessionStart hook）移植到 [Trae IDE](https://www.trae.ai/)。

## 目录

- 设计文档：[docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md](docs/superpowers/specs/2026-04-28-superpowers-to-trae-design.md)
- 实施计划：[docs/superpowers/plans/2026-04-28-superpowers-to-trae.md](docs/superpowers/plans/2026-04-28-superpowers-to-trae.md)
- **Trae IDE 自定义 Agent 创建指南**：[docs/superpowers/trae-agents-setup.md](docs/superpowers/trae-agents-setup.md) ← `bash scripts/install.sh` 之后必看，把 4 个 Agent 在 Trae UI 里创建出来
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

两种方式都会在目标位置已有同名文件时自动备份为 `<file>.bak.YYYYMMDD-HHMMSS`，并写日志到 `~/.trae/.superpowers-install.log`（用户级）或 `<project>/.trae/.superpowers-install.log`（项目级）。

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
| `src/` | 转换知识：mappings.json + 模板 + transform.py / render.py / build_helpers.py |
| `scripts/` | sync-upstream.sh / build.sh / install.sh |
| `dist/` | 构建产物（入 git） |
| `tests/` | 单测 + 端到端 fixture smoke |
| `docs/superpowers/` | 设计文档、实施计划、手工测试步骤 |

## License

本项目仅做工具链与转换器；移植的 superpowers 内容版权归原作者所有，详见 `upstream/LICENSE`。
