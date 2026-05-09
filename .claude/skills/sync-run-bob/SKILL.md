---
name: sync-run-bob
description: 维护者一键流程 — 把 run-bob 上游 main 分支的最新模板同步到本仓库 cli/templates/bob/ → 自动把 ARCHITECTURE.md 字串改写为 BOB.md → 跑测试 → bump cli 版本 → 提交 → 打 tag → 推送，最后输出消费者升级通知文案。当用户说"同步 run-bob"、"bob 插件有更新同步一下"、"run-bob 出新版了"或输入 /sync-run-bob 时触发。**与 sync-ddd-run 的区别**：那个同步 ddd-run 模板 → cli/templates/ddd/，本 skill 同步 run-bob → cli/templates/bob/，并且会做 ARCHITECTURE.md → BOB.md 字串替换。
---

# sync-run-bob

把 `https://github.com/amwtke/run-bob` main 分支的最新模板同步进 `superpowers-to-trae` 的 `cli/templates/bob/` 目录、跑测试、bump cli 版本、发 release，最后给出消费者升级通知。

## 触发

用户说：
- "同步 run-bob" / "bob 插件有更新同步一下" / "run-bob 出新版了"
- 输入 `/sync-run-bob`

## 前置约束

- 必须在 `~/workshop/superpowers-to-trae` 仓库根（项目级 skill 默认就在）
- 工作树干净（`git status` 无未提交改动）；如果脏，先让用户决定 commit / stash
- 网络可达 `github.com`
- 任一步失败 → **立即终止流程**，不做自动回滚（保留现场让人查）

## 文件映射（SSOT）

下表是同步范围。**不在表里的文件，skill 不动**。
**注意**：本仓库统一把 `ARCHITECTURE.md` 重命名为 `BOB.md`（避免与项目里已有的 ARCHITECTURE.md 撞名）。所有同步进来的文件必须做 `s/ARCHITECTURE\.md/BOB.md/g` 字串替换。

| run-bob 上游 | 本仓库 | 同步策略 |
|---|---|---|
| `src/templates/skills/bob-identify.md` | `cli/templates/bob/skills/bob-identify.md` | 强制覆盖 + sed 改写 |
| `src/templates/skills/bob-onion.md` | `cli/templates/bob/skills/bob-onion.md` | 强制覆盖 + sed 改写 |
| `src/templates/skills/bob-spec.md` | `cli/templates/bob/skills/bob-spec.md` | 强制覆盖 + sed 改写 |
| `src/templates/root/ARCHITECTURE.md` | `cli/templates/bob/root/BOB.md` | 强制覆盖 + sed 改写（同时改文件名）|
| `src/templates/root/README-RUN-BOB.md` | `cli/templates/bob/root/README-RUN-BOB.md` | 强制覆盖 + sed 改写 |
| `src/templates/root/CLAUDE.md` | — | **不同步**（本地 `claude-md-merge.md` 是 trae 专用合并模板）|
| `src/templates/root/CleanArchitectureTest.java` | — | **不同步**（属 cli 安装逻辑改动，超出本 skill 范围）|
| `src/templates/root/UseCase.java` | — | **不同步**（同上）|
| `src/templates/root/TransactionalUseCaseDecorator.java` | — | **不同步**（同上）|

## 9 步流程

### Step 1 — 检查工作树干净

```bash
cd ~/workshop/superpowers-to-trae
git status --porcelain
```

非空 → 终止，让用户决定 commit / stash。

### Step 2 — 拉上游最新到 tmp

```bash
rm -rf /tmp/run-bob-sync
git clone --depth 50 https://github.com/amwtke/run-bob.git /tmp/run-bob-sync
UPSTREAM_SHA=$(git -C /tmp/run-bob-sync rev-parse --short HEAD)
UPSTREAM_DATE=$(git -C /tmp/run-bob-sync log -1 --format=%ci)
echo "Upstream HEAD: $UPSTREAM_SHA ($UPSTREAM_DATE)"
```

### Step 3 — 对比上游版本，判断要不要继续

run-bob SHA 的 SSOT 是 `cli/Cargo.toml` 的 `[package.metadata.upstream].run_bob_version` 字段（同一段还有 `superpowers_version` / `ddd_run_version`）。

```bash
LOCAL_SHA=$(grep '^run_bob_version' ~/workshop/superpowers-to-trae/cli/Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
echo "Local recorded SHA:  $LOCAL_SHA"
echo "Upstream HEAD SHA:   $UPSTREAM_SHA"
```

判断：
- `LOCAL_SHA == UPSTREAM_SHA` → **流程终止**，告诉用户"已是最新，无需同步"。不要瞎跑下面的步骤
- `LOCAL_SHA != UPSTREAM_SHA` → 继续

### Step 4 — Review 5 个 md 文件 diff

对每个映射的 md 文件先做 sed 改写，再跑 unified diff，把摘要打印给用户：

```bash
TMP_REWRITE=/tmp/run-bob-sync-rewrite
rm -rf $TMP_REWRITE && mkdir -p $TMP_REWRITE/skills $TMP_REWRITE/root

for f in bob-identify.md bob-onion.md bob-spec.md; do
  sed 's/ARCHITECTURE\.md/BOB.md/g' /tmp/run-bob-sync/src/templates/skills/$f \
    > $TMP_REWRITE/skills/$f
done
sed 's/ARCHITECTURE\.md/BOB.md/g' /tmp/run-bob-sync/src/templates/root/README-RUN-BOB.md \
  > $TMP_REWRITE/root/README-RUN-BOB.md
sed 's/ARCHITECTURE\.md/BOB.md/g' /tmp/run-bob-sync/src/templates/root/ARCHITECTURE.md \
  > $TMP_REWRITE/root/BOB.md

for pair in \
  "skills/bob-identify.md" \
  "skills/bob-onion.md" \
  "skills/bob-spec.md" \
  "root/BOB.md" \
  "root/README-RUN-BOB.md"; do
  echo "=== $pair ==="
  diff -q "/home/xiaojin/workshop/superpowers-to-trae/cli/templates/bob/$pair" \
          "$TMP_REWRITE/$pair" || \
    diff -u "/home/xiaojin/workshop/superpowers-to-trae/cli/templates/bob/$pair" \
            "$TMP_REWRITE/$pair" | head -60
done
```

**同时检查上游有没有新增超出映射表的文件**——这是为了防止映射表过时漏掉新文件：

```bash
echo "=== 上游全部 templates 文件 ==="
find /tmp/run-bob-sync/src/templates -type f
```

把上游列表跟映射表对比，**新增的非 md 文件提醒用户**（例如又冒出来个新 `*.java` 模板），由用户决定是否要扩 cli 安装逻辑——本 skill 不动。

### Step 5 — 强制覆盖 5 个 md 文件（已 sed 改写）

```bash
cd ~/workshop/superpowers-to-trae

cp $TMP_REWRITE/skills/bob-identify.md cli/templates/bob/skills/
cp $TMP_REWRITE/skills/bob-onion.md    cli/templates/bob/skills/
cp $TMP_REWRITE/skills/bob-spec.md     cli/templates/bob/skills/
cp $TMP_REWRITE/root/BOB.md            cli/templates/bob/root/
cp $TMP_REWRITE/root/README-RUN-BOB.md cli/templates/bob/root/

# 防御性检查：确认没有任何残留 ARCHITECTURE.md 字串
echo "--- 检查残留 ARCHITECTURE.md 字串（应当只有零结果） ---"
grep -rn 'ARCHITECTURE\.md' cli/templates/bob/ || echo "(clean)"

git status --short cli/templates/bob/
```

如果 grep 命中任何 `ARCHITECTURE.md` → **停下来**，sed 替换可能没覆盖所有引用形式（如带反斜杠的 markdown link），手工修复后再继续。

### Step 6 — 跑全套测试

```bash
make test
```

74+ 项必须全绿。失败 → 修，不继续装。常见失败：

| 失败 | 原因 / 处理 |
|---|---|
| frontmatter `name` 字段校验失败 | 上游某个 skill 改了 frontmatter，cli 校验规则不接受 → 看 `cli/src/templates.rs` 调一下 |
| bob_install_creates_skills_and_root_files 失败 | 上游加了 / 删了 skill → 改 `cli/src/addons/bob.rs` 测试断言 |
| 其它 hard-coded 路径校验失败 | 上游新增了硬编码路径 → 看 cli 是否需要相应更新 |

### Step 7 — 更新 Cargo.toml 里的 run_bob_version

把 `cli/Cargo.toml` 的 `[package.metadata.upstream].run_bob_version = "<旧 SHA>"` 改成 `"<新 UPSTREAM_SHA>"`。这一步与 Step 8 的 version bump 一起改 Cargo.toml，可一次性完成。

### Step 8 — Bump cli 版本

读 `cli/Cargo.toml` 当前版本，按上游变更影响判断：

| 上游变化范围 | cli bump |
|---|---|
| 仅文档措辞改动（README / 注释） | patch (0.4.0 → 0.4.1) |
| skill 内容增量（加了规则 / 反模式 / 示例） | patch (0.4.0 → 0.4.1) |
| skill frontmatter 改名 / 删了 skill / 加了新 skill | minor (0.4.0 → 0.5.0) |
| 上游新增非 md 文件且决定移植 → 改 cli 代码 | minor (0.4.0 → 0.5.0) |

### Step 9 — Commit + Tag + Push

```bash
cd ~/workshop/superpowers-to-trae
git add -A
git commit -m "$(cat <<EOF
chore(bob): sync run-bob upstream → $UPSTREAM_SHA

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
🎉 superpowers-to-trae 已升级到 **v<新 cli 版本>**（同步上游 run-bob `<UPSTREAM_SHA>`）。

## 升级二进制（消费者跑）

```bash
# Linux + macOS + Windows (bash/zsh/git-bash)
cargo install --git https://github.com/amwtke/superpowers-to-trae.git superpowers-trae --force
```

## 升级项目内的 bob 插件

二进制装好后，进到自己的项目根目录跑：

```bash
superpowers-trae upgrade --addons bob
```

`BOB.md` 是 install-once，不会被覆盖；3 个 bob skill + README-RUN-BOB.md 会刷成最新。

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
| Step 5 sed 残留 `ARCHITECTURE.md` | 停下，让用户手工修（可能是带反斜杠/链接的边界情况）|
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
| **sync-run-bob**（本 skill）| 维护者：同步 run-bob 上游 | `cli/templates/bob/` + 发版 |
| sync-ddd-run | 维护者：同步 ddd-run 上游 | `cli/templates/ddd/` + 发版 |
| update-from-upstream | 维护者：同步 obra/superpowers 上游 | `upstream/` + `dist/` + 发版 |
| install-superpowers-trae | 消费者：拉 binary | `~/.cargo/bin/` |
