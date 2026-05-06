---
name: update-from-upstream
description: 维护者一键升级流程 — 把 obra/superpowers 上游新版本同步到本仓库 → 构建 → 测试 → bump cli 版本 → 提交 → 打 tag → 推送，最后输出消费者升级通知文案。当用户说"发新版"、"升级到上游 X.Y.Z"、"superpowers 官方有更新同步一下"、"release 一个新版本"、"做一次 upstream 升级"或输入 /update-from-upstream 时触发。**不要与 install-superpowers-trae 混淆**——后者是消费者拉 binary，本 skill 是维护者发版。
---

# update-from-upstream

把 `obra/superpowers` 上游新版本同步进 `superpowers-to-trae` 这个 fork、生成 dist、bump cli 版本、发 release，最后给出消费者升级通知。

## 触发

用户说：
- "发新版" / "release 一个新版本" / "做一次 upstream 升级"
- "升级到 superpowers 5.X.Y" / "superpowers 出新版了，同步一下"
- 输入 `/update-from-upstream`

## 前置约束

- 必须在 `~/workshop/superpowers-to-trae` 仓库根（项目级 skill 默认就在）
- 工作树干净（`git status` 无未提交改动）；如果脏，先让用户决定 commit / stash
- Claude Code 已安装 superpowers plugin（cache 在 `~/.claude/plugins/cache/claude-plugins-official/superpowers/`）
- 任一步失败 → **立即终止流程**，不做自动回滚（保留现场让人查）

## 9 步流程

### Step 1 — 让 Claude Code 拉最新 plugin

提示用户在 Claude Code 里跑：
```
/plugin update superpowers@claude-plugins-official
/reload-plugins
```
Claude 自己**不能**调这两个 slash command（它们是 Claude Code 内置 cli 命令，不是 skill）。等用户确认更新完，再继续。

### Step 2 — 检查 cache 里是否真的有新版本

```bash
echo "=== 仓库当前版本 ==="
cat ~/workshop/superpowers-to-trae/upstream/VERSION

echo "=== cache 里的所有版本（取最大者作为目标版本）==="
ls -1 ~/.claude/plugins/cache/claude-plugins-official/superpowers/ | sort -V
```

判断：
- cache 最大版本 == upstream/VERSION → **流程终止，告诉用户"没有新版本可升"**，不要瞎跑下面步骤
- cache 最大版本 > upstream/VERSION → 继续

### Step 3 — 一键 sync + build + cli-build

```bash
cd ~/workshop/superpowers-to-trae
make upgrade-superpowers
```
等价于 `make sync` + `make build` + `make cli-build`。注意它**不跑 cargo install**，只是 release 编译到 `cli/target/release/`。

观察输出里的 `verify-build` 警告：
- `phrase_replacements with 0 matches` → 某条移植规则在新版上游里失效了，记下警告条目，**继续**（非 fatal），但 Step 5 review 时要顺手看一眼是不是要更新映射

### Step 4 — Review 真实差异

```bash
git diff upstream/VERSION                     # 确认 5.X.Y → 5.X'.Y' 的具体跨度
git diff upstream/RELEASE-NOTES.md | head -100  # 看上游 changelog
git diff --stat upstream/                      # 看哪些文件变了
git diff --stat dist/                          # 看渲染产物有没有意外漂移
```

**重点找 breaking 变更**：
- `Removals` 段落（删了什么 slash command / agent / skill）
- 重命名的文件 / 改了 frontmatter `name` 字段的 skill
- `hooks-cursor.json` / `package.json` 改动

### Step 5 — 跟进 cli 代码

如果上游删了某个被 cli 硬编码引用的东西（典型：删 slash command、删命名 agent、改 skill 名字），cli 要同步改。

定位查询：
```bash
# 上游删的 slash command 名字 / agent 名字 / 路径，在 cli 源码里搜
cd cli && grep -r "<被删的名字>" src/ tests/
```

如果有命中 → 改代码并跟着写测试；没命中 → 跳过本步。

### Step 6 — 跑全套测试

```bash
cd ~/workshop/superpowers-to-trae && make test
```

47+ 项必须全绿。失败 → 修，不继续装。

### Step 7 — Bump cli 版本

读 `cli/Cargo.toml` 当前版本，按上游跨度同步：

| 上游跨度 | cli bump |
|---|---|
| patch (5.0.7 → 5.0.8) | patch (0.2.2 → 0.2.3) |
| minor (5.0.7 → 5.1.0) | minor (0.2.2 → 0.3.0) |
| major (5.x → 6.x) | minor (0.x.y → 0.(x+1).0)，cli 仍在 0.x 阶段不轻易跳 1.0 |

```bash
# 编辑 cli/Cargo.toml 的 version 字段
# 同时检查这两处文档里有没有硬编码版本号需要同步:
grep -nE 'v?[0-9]+\.[0-9]+\.[0-9]+' README.md cli/README.md | grep -i 'tag\|install\|cargo' | head
```

### Step 8 — Commit + Tag + Push

```bash
cd ~/workshop/superpowers-to-trae
git add -A
git commit -m "$(cat <<'EOF'
chore(upgrade): superpowers <旧版本> → <新版本>

主要变更（从 upstream/RELEASE-NOTES.md 摘）:
- <breaking 1>
- <breaking 2>
- <其它>

cli bump: <旧 cli 版本> → <新 cli 版本>

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git tag v<新 cli 版本>     # 例如 v0.3.0
git push origin main --tags
```

GitHub Actions 会基于 tag 自动出 release（如果配了的话）。如未配，提示用户去 GitHub UI 手动发 release。

### Step 9 — 通知消费者升级（输出文案给用户）

把以下 markdown 文案**原样输出**到对话里，告诉用户"复制这段发给订阅你的人"：

```markdown
🎉 superpowers-to-trae 已升级到 **v<新 cli 版本>**（同步上游 superpowers <新上游版本>）。

## 升级二进制（消费者跑）

```bash
# Linux + macOS
curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
```

## 升级项目内的 superpowers 组件

二进制装好后，进到自己的项目根目录跑：

```bash
superpowers-trae upgrade
```

## 主要变更

<这里粘 Step 4 摘的 breaking 变更要点>
```

## 失败处理

| 失败点 | 行为 |
|---|---|
| Step 2 发现没有新版本 | 终止流程，告诉用户当前已是最新，不做空 commit |
| Step 3 `make build` 警告 phrase_replacements 0 matches | 不阻断，但 Step 5 必须跟进映射规则更新 |
| Step 5 cli 改完 Step 6 测试挂 | 把失败用例贴出来，停下让用户判断（修代码 vs. 改测试 vs. 退回上一版） |
| Step 7 Cargo.toml 改完 cargo build 报错 | 通常是 cli 引用了被删的 upstream 文件路径，回 Step 5 |
| Step 8 push 被拒（无权限 / 网络） | 原样输出 git 错误，不要 force push，让用户处理 |

## 不做

- 不自动 stash / reset --hard——脏工作树留给用户决定
- 不跳过测试——发版必须全绿
- 不强制改 cli 代码——上游删的东西如果 cli 没引用就不动
- 不修改 git 配置 / 不改远端 / 不强 push
- 不替消费者执行 install——本 skill 只产出通知文案

## 与 install-superpowers-trae 的区别

| 这个 skill | install-superpowers-trae |
|---|---|
| 维护者用，发版流水线 | 消费者用，拉 binary |
| 在仓库根跑，改 upstream/dist/cli | 在任意目录跑，只动 ~/.cargo/bin/ |
| 推 git tag、出 release | 不碰 git |
