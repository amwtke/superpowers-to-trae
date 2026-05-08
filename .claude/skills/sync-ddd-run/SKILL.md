---
name: sync-ddd-run
description: 维护者一键流程 — 把 ddd-run 上游 main 分支的最新模板同步到本仓库 cli/templates/ddd/ → 跑测试 → bump cli 版本 → 提交 → 打 tag → 推送，最后输出消费者升级通知文案。当用户说"同步 ddd-run"、"ddd 插件有更新同步一下"、"ddd-run 出新版了"或输入 /sync-ddd-run 时触发。**与 update-from-upstream 的区别**：那个是同步 obra/superpowers，这个是同步 amwtke/ddd-run。
---

# sync-ddd-run

把 `https://github.com/amwtke/ddd-run` main 分支的最新模板同步进 `superpowers-to-trae` 的 `cli/templates/ddd/` 目录、跑测试、bump cli 版本、发 release，最后给出消费者升级通知。

## 触发

用户说：
- "同步 ddd-run" / "ddd 插件有更新同步一下" / "ddd-run 出新版了"
- 输入 `/sync-ddd-run`

## 前置约束

- 必须在 `~/workshop/superpowers-to-trae` 仓库根（项目级 skill 默认就在）
- 工作树干净（`git status` 无未提交改动）；如果脏，先让用户决定 commit / stash
- 网络可达 `github.com`
- 任一步失败 → **立即终止流程**，不做自动回滚（保留现场让人查）

## 文件映射（SSOT）

下表是同步范围。**不在表里的文件，skill 不动**。

| ddd-run 上游 | 本仓库 | 同步策略 |
|---|---|---|
| `src/templates/skills/ddd-storm.md` | `cli/templates/ddd/skills/ddd-storm.md` | 强制覆盖 |
| `src/templates/skills/ddd-model.md` | `cli/templates/ddd/skills/ddd-model.md` | 强制覆盖 |
| `src/templates/skills/ddd-spec.md` | `cli/templates/ddd/skills/ddd-spec.md` | 强制覆盖 |
| `src/templates/root/DOMAIN.md` | `cli/templates/ddd/root/DOMAIN.md` | 强制覆盖 |
| `src/templates/root/README-DDD-HARNESS.md` | `cli/templates/ddd/root/README-DDD-HARNESS.md` | 强制覆盖 |
| `src/templates/root/CLAUDE.md` | — | **不同步**（本地 `claude-md-merge.md` 是 trae 专用合并模板，结构不同）|
| `src/templates/root/CleanArchitectureTest.java` | — | **不同步**（属 cli 安装逻辑改动，超出本 skill 范围）|

## 9 步流程

### Step 1 — 检查工作树干净

```bash
cd ~/workshop/superpowers-to-trae
git status --porcelain
```

非空 → 终止，让用户决定 commit / stash。

### Step 2 — 拉上游最新到 tmp

```bash
rm -rf /tmp/ddd-run-sync
git clone --depth 50 https://github.com/amwtke/ddd-run.git /tmp/ddd-run-sync
UPSTREAM_SHA=$(git -C /tmp/ddd-run-sync rev-parse --short HEAD)
UPSTREAM_DATE=$(git -C /tmp/ddd-run-sync log -1 --format=%ci)
echo "Upstream HEAD: $UPSTREAM_SHA ($UPSTREAM_DATE)"
```

### Step 3 — 对比上游版本，判断要不要继续

```bash
LOCAL_SHA=$(cat ~/workshop/superpowers-to-trae/upstream/DDD-VERSION 2>/dev/null || echo "none")
echo "Local recorded SHA:  $LOCAL_SHA"
echo "Upstream HEAD SHA:    $UPSTREAM_SHA"
```

判断：
- `LOCAL_SHA == UPSTREAM_SHA` → **流程终止**，告诉用户"已是最新，无需同步"。不要瞎跑下面的步骤
- `LOCAL_SHA == "none"` → 第一次跑本 skill，继续（同时本步要在 Step 7 写入 `upstream/DDD-VERSION`）
- `LOCAL_SHA != UPSTREAM_SHA` → 继续

### Step 4 — Review 5 个 md 文件 diff

对每个映射的 md 文件跑 unified diff，把摘要打印给用户：

```bash
for pair in \
  "skills/ddd-storm.md" \
  "skills/ddd-model.md" \
  "skills/ddd-spec.md" \
  "root/DOMAIN.md" \
  "root/README-DDD-HARNESS.md"; do
  echo "=== $pair ==="
  diff -q "/home/xiaojin/workshop/superpowers-to-trae/cli/templates/ddd/$pair" \
          "/tmp/ddd-run-sync/src/templates/$pair" || \
    diff -u "/home/xiaojin/workshop/superpowers-to-trae/cli/templates/ddd/$pair" \
            "/tmp/ddd-run-sync/src/templates/$pair" | head -60
done
```

**同时检查上游有没有新增超出映射表的文件**——这是为了防止映射表过时漏掉新文件：

```bash
echo "=== 上游全部 templates 文件 ==="
find /tmp/ddd-run-sync/src/templates -type f
```

把上游列表跟映射表对比，**新增的非 md 文件提醒用户**（例如又冒出来个 `*.java` 模板），由用户决定是否要扩 cli 安装逻辑——本 skill 不动。

### Step 5 — 强制覆盖 5 个 md 文件

```bash
cd ~/workshop/superpowers-to-trae

cp /tmp/ddd-run-sync/src/templates/skills/ddd-storm.md cli/templates/ddd/skills/
cp /tmp/ddd-run-sync/src/templates/skills/ddd-model.md cli/templates/ddd/skills/
cp /tmp/ddd-run-sync/src/templates/skills/ddd-spec.md  cli/templates/ddd/skills/
cp /tmp/ddd-run-sync/src/templates/root/DOMAIN.md      cli/templates/ddd/root/
cp /tmp/ddd-run-sync/src/templates/root/README-DDD-HARNESS.md cli/templates/ddd/root/

git status --short cli/templates/ddd/
```

### Step 6 — 跑全套测试

```bash
make test
```

47+ 项必须全绿。失败 → 修，不继续装。常见失败：

| 失败 | 原因 / 处理 |
|---|---|
| frontmatter `name` 字段校验失败 | 上游某个 skill 改了 frontmatter，cli 校验规则不接受 → 看 `cli/src/templates.rs` 调一下 |
| 路径硬编码失败 | 上游引用了 `.claude/` 之类路径，cli 的 phrase_replacements 没覆盖 → 看 `cli/src/phrase_replace.rs` 加规则 |

### Step 7 — 更新 upstream/DDD-VERSION

```bash
cat > upstream/DDD-VERSION <<EOF
$UPSTREAM_SHA
EOF
```

（一行 commit SHA 短码，作为下次 sync 的对比基准。）

### Step 8 — Bump cli 版本

读 `cli/Cargo.toml` 当前版本，按上游变更影响判断：

| 上游变化范围 | cli bump |
|---|---|
| 仅文档措辞改动（README / 注释） | patch (0.3.1 → 0.3.2) |
| skill 内容增量（加了规则 / 反模式 / 示例） | patch (0.3.1 → 0.3.2) |
| skill frontmatter 改名 / 删了 skill / 加了新 skill | minor (0.3.1 → 0.4.0) |
| 上游新增非 md 文件且决定移植 → 改 cli 代码 | minor (0.3.1 → 0.4.0) |

```bash
# 编辑 cli/Cargo.toml 的 version 字段
```

### Step 9 — Commit + Tag + Push

```bash
cd ~/workshop/superpowers-to-trae
git add -A
git commit -m "$(cat <<EOF
chore(ddd): sync ddd-run upstream → $UPSTREAM_SHA

主要变更（从 Step 4 diff 摘）:
- <要点 1>
- <要点 2>

cli bump: <旧版> → <新版>

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git tag v<新 cli 版本>
git push origin main --tags
```

GitHub Actions 会基于 tag 自动出 release（如已配）。如未配，提示用户去 GitHub UI 手动发 release。

### Step 10 — 输出消费者升级通知

把以下 markdown 文案**原样输出**到对话里，告诉用户"复制这段发给订阅你的人"：

````markdown
🎉 superpowers-to-trae 已升级到 **v<新 cli 版本>**（同步上游 ddd-run `<UPSTREAM_SHA>`）。

## 升级二进制（消费者跑）

```bash
# Linux + macOS + Windows (bash/zsh/git-bash)
cargo install --git https://github.com/amwtke/superpowers-to-trae.git superpowers-trae --force
```

## 升级项目内的 ddd 插件

二进制装好后，进到自己的项目根目录跑：

```bash
superpowers-trae upgrade --addons ddd
```

`DOMAIN.md` 是 install-once，不会被覆盖；3 个 ddd skill + README-DDD-HARNESS.md 会刷成最新。

## 主要变更

<这里粘 Step 4 diff 摘的要点>
````

## 失败处理

| 失败点 | 行为 |
|---|---|
| Step 1 工作树脏 | 终止，让用户先 commit / stash |
| Step 2 git clone 网络失败 | 原文输出错误，让用户检查代理 / GitHub 可达性 |
| Step 3 SHA 一致 | 终止流程，告诉用户"已是最新"，不做空 commit |
| Step 4 上游有新增非映射文件 | 不阻断；汇报给用户由其决定是否扩映射 |
| Step 6 测试挂 | 把失败用例贴出来，停下让用户判断（修 cli 代码 vs. 退回上一版） |
| Step 9 push 被拒（无权限 / 网络）| 原样输出 git 错误，不要 force push，让用户处理 |

## 不做

- 不自动 stash / reset --hard——脏工作树留给用户决定
- 不跳过测试——发版必须全绿
- 不动 `claude-md-merge.md`——这是 trae 专用文件
- 不自动移植上游新增的非 md 模板（如 Java 文件）——属 cli 安装逻辑变更
- 不修改 git 配置 / 不改远端 / 不强 push
- 不替消费者执行 install——本 skill 只产出通知文案

## 与其它 skill 的区别

| skill | 用途 | 操作对象 |
|---|---|---|
| **sync-ddd-run**（本 skill）| 维护者：同步 ddd-run 上游 | `cli/templates/ddd/` + 发版 |
| update-from-upstream | 维护者：同步 obra/superpowers 上游 | `upstream/` + `dist/` + 发版 |
| install-superpowers-trae | 消费者：拉 binary | `~/.cargo/bin/` |
