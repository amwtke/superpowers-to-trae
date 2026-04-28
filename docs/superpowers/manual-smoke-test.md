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

1. 找到 `~/.trae/.superpowers-install.log`（用户级）或 `<project>/.trae/.superpowers-install.log`（项目级）中所有 `INSTALL:` 行
2. 删除对应文件；如有 `BACKUP:` 行，将 `.bak.*` 文件 mv 回原名
3. **预期**：恢复到安装前状态。

如以上任一验证失败，记录失败步骤与现象到 GitHub issue 或团队渠道。
