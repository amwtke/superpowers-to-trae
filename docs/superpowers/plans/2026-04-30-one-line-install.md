# 一键安装脚本（install.sh / install.ps1）实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把端用户安装链路从"4 平台手工 curl + tar/zip"压成"一行 `curl … | sh` / `iwr … | iex`"，并清掉 v0.1 的 deprecated 残留。

**Architecture:** 两份 shell 入口（POSIX `install.sh` + PowerShell `install.ps1`）放仓库根，从 GitHub raw 直供；下载 release 产物到默认 user-local 目录、按需写 PATH；用 docker 黑盒冒烟测试 POSIX 路径；Windows 路径靠手工冒烟；同步删旧 `scripts/install.sh` + `dist/project/` + 文档残留；最后 bump v0.2.2 触发 release。

**Tech Stack:** POSIX shell（兼容 dash / busybox-ash / bash / zsh-as-sh）、PowerShell 5.1+ / pwsh 7+、docker（仅本地 smoke）、GitHub Releases。

**关联设计文档：** `docs/superpowers/specs/2026-04-30-one-line-install-design.md`

---

## Task 1: `tests/install-script-smoke.sh` —— 黑盒 smoke 测试

**Files:**
- Create: `tests/install-script-smoke.sh`

- [ ] **Step 1: 写 smoke 测试脚本**

```sh
#!/bin/sh
# Black-box smoke test for install.sh.
# 在干净 ubuntu:24.04 容器里跑各场景，断言用户视角行为。
#
# Requirements: docker
# Run from repo root: tests/install-script-smoke.sh

set -eu

if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: docker required" >&2
    exit 1
fi

REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
IMG="ubuntu:24.04"
PASS=0
FAIL=0

run_case() {
    name=$1
    body=$2
    printf '\n=== %s ===\n' "$name"
    if docker run --rm -v "$REPO_ROOT:/repo:ro" -w /tmp "$IMG" sh -c "$body"; then
        PASS=$((PASS + 1))
        printf '\033[32m  PASS\033[0m\n'
    else
        FAIL=$((FAIL + 1))
        printf '\033[31m  FAIL\033[0m\n'
    fi
}

# Case 1: happy path — curl 可用，默认装到 ~/.local/bin
run_case "happy path (curl, default dir)" '
    apt-get update -qq && apt-get install -qq -y curl ca-certificates >/dev/null
    sh /repo/install.sh
    test -x "$HOME/.local/bin/superpowers-trae"
    "$HOME/.local/bin/superpowers-trae" --version
'

# Case 2: SUPERPOWERS_INSTALL_DIR 覆盖默认路径
run_case "SUPERPOWERS_INSTALL_DIR override" '
    apt-get update -qq && apt-get install -qq -y curl ca-certificates >/dev/null
    SUPERPOWERS_INSTALL_DIR=/tmp/myown sh /repo/install.sh
    test -x /tmp/myown/superpowers-trae
'

# Case 3: SUPERPOWERS_VERSION 锁定旧版本
run_case "SUPERPOWERS_VERSION=v0.2.0" '
    apt-get update -qq && apt-get install -qq -y curl ca-certificates >/dev/null
    SUPERPOWERS_VERSION=v0.2.0 sh /repo/install.sh
    "$HOME/.local/bin/superpowers-trae" --version | grep -q "0.2.0"
'

# Case 4: 没 curl 时 wget 兜底
run_case "wget fallback (no curl available)" '
    apt-get update -qq && apt-get install -qq -y wget ca-certificates >/dev/null
    sh /repo/install.sh
    test -x "$HOME/.local/bin/superpowers-trae"
'

# Case 5: 重复运行不重复追加 rc 行
run_case "idempotent rc append" '
    apt-get update -qq && apt-get install -qq -y curl ca-certificates >/dev/null
    export SHELL=/bin/bash
    sh /repo/install.sh >/dev/null
    sh /repo/install.sh >/dev/null
    count=$(grep -c "superpowers-trae installer" "$HOME/.bashrc" 2>/dev/null || echo 0)
    test "$count" -le 1
'

# Case 6: curl + wget 都没有时清晰报错
run_case "neither curl nor wget => clear error" '
    out=$(sh /repo/install.sh 2>&1; true)
    echo "$out" | grep -q "neither curl nor wget"
'

printf '\n=== Summary: %d pass, %d fail ===\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ]
```

- [ ] **Step 2: chmod + 跑一次确认会因 install.sh 不存在而 FAIL**

```bash
chmod +x tests/install-script-smoke.sh
tests/install-script-smoke.sh || true   # 期望全部 FAIL（install.sh 还没写）
```

预期：所有 case FAIL，因为 `/repo/install.sh` 不存在。

- [ ] **Step 3: 提交**

```bash
git add tests/install-script-smoke.sh
git commit -m "test(install): 加 docker 黑盒冒烟测试（先于 install.sh）"
```

---

## Task 2: `install.sh` —— POSIX 一键安装器

**Files:**
- Create: `install.sh`

- [ ] **Step 1: 写 install.sh（完整内容）**

```sh
#!/bin/sh
# superpowers-trae one-line installer for Linux + macOS.
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh
#
# Optional env vars:
#   SUPERPOWERS_INSTALL_DIR    target dir (default: $HOME/.local/bin)
#   SUPERPOWERS_VERSION        release tag like v0.2.1 (default: latest)

set -eu

REPO="amwtke/superpowers-to-trae"

red()    { printf '\033[31m%s\033[0m\n' "$1" >&2; }
green()  { printf '\033[32m%s\033[0m\n' "$1"; }
yellow() { printf '\033[33m%s\033[0m\n' "$1"; }

die() { red "ERROR: $1"; exit 1; }

# 1. Platform detection
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$OS-$ARCH" in
    linux-x86_64|linux-amd64)     ASSET="linux-x86_64.tar.gz" ;;
    darwin-arm64|darwin-aarch64)  ASSET="macos-aarch64.tar.gz" ;;
    darwin-x86_64)                ASSET="macos-x86_64.tar.gz" ;;
    *) die "unsupported platform: $OS-$ARCH (only Linux x86_64, macOS aarch64/x86_64)" ;;
esac

# 2. Pick downloader
if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl"
elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget"
else
    die "neither curl nor wget found; please install one"
fi

download_to() {
    # $1=URL  $2=outfile
    if [ "$DOWNLOADER" = "curl" ]; then
        curl -fsSL -o "$2" "$1"
    else
        wget -qO "$2" "$1"
    fi
}

# 3. Resolve config
INSTALL_DIR="${SUPERPOWERS_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${SUPERPOWERS_VERSION:-latest}"
mkdir -p "$INSTALL_DIR" || die "cannot create $INSTALL_DIR"

# 4. Build URL
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/$REPO/releases/latest/download/superpowers-trae-$ASSET"
else
    URL="https://github.com/$REPO/releases/download/$VERSION/superpowers-trae-$ASSET"
fi

# 5. Download + extract
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "Downloading $URL ..."
download_to "$URL" "$TMP/sp.tar.gz" || die "download failed: $URL"
tar xzf "$TMP/sp.tar.gz" -C "$TMP" || die "extract failed"
[ -f "$TMP/superpowers-trae" ] || die "archive does not contain superpowers-trae binary"

if command -v install >/dev/null 2>&1; then
    install -m 0755 "$TMP/superpowers-trae" "$INSTALL_DIR/superpowers-trae"
else
    cp "$TMP/superpowers-trae" "$INSTALL_DIR/superpowers-trae"
    chmod 0755 "$INSTALL_DIR/superpowers-trae"
fi

# 6. PATH check + rc append
need_rc=0
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) need_rc=1 ;;
esac

rc_modified=""
if [ "$need_rc" = "1" ]; then
    shell_name=$(basename "${SHELL:-/bin/sh}")
    case "$shell_name" in
        zsh)  rc="$HOME/.zshrc" ;;
        bash) rc="$HOME/.bashrc" ;;
        fish) rc="$HOME/.config/fish/config.fish" ;;
        *)    rc="$HOME/.profile" ;;
    esac

    if [ ! -f "$rc" ] || ! grep -F -q "$INSTALL_DIR" "$rc" 2>/dev/null; then
        mkdir -p "$(dirname "$rc")"
        if [ "$shell_name" = "fish" ]; then
            printf '\n# Added by superpowers-trae installer\nfish_add_path %s\n' "$INSTALL_DIR" >> "$rc"
        else
            printf '\n# Added by superpowers-trae installer\nexport PATH="%s:$PATH"\n' "$INSTALL_DIR" >> "$rc"
        fi
        rc_modified="$rc"
    fi
fi

# 7. Verify + report
ver_line=$("$INSTALL_DIR/superpowers-trae" --version 2>&1 | head -n1) || die "binary failed to run; arch mismatch?"
green "✓ $ver_line installed at $INSTALL_DIR/superpowers-trae"

if [ -n "$rc_modified" ]; then
    yellow "PATH updated in $rc_modified — open a new shell or run: . $rc_modified"
fi
```

- [ ] **Step 2: chmod 并跑 smoke 测试，全部应该 PASS**

```bash
chmod +x install.sh
tests/install-script-smoke.sh
```

预期：6 个 case 全 PASS。

如果有 FAIL，修 install.sh 直到通过。常见踩坑：
- macOS BSD `mktemp` vs Linux GNU 差异（脚本里只用 `-d`，OK）
- `printf '\n...' >> $rc` 在 dash 下要确认 `\n` 被解释（POSIX printf 必须支持，OK）
- shellcheck 报警：`shellcheck install.sh` 应零告警

- [ ] **Step 3: 提交**

```bash
git add install.sh
git commit -m "feat(install): 加 POSIX 一键安装脚本 install.sh

- 支持 Linux x86_64、macOS aarch64/x86_64
- curl 优先 wget 兜底；neither 时清晰报错
- 默认装到 \$HOME/.local/bin；SUPERPOWERS_INSTALL_DIR 可覆盖
- 默认装最新 release；SUPERPOWERS_VERSION 可锁定 tag
- PATH 不在则按 \$SHELL 选 rc 文件追加（zsh/bash/fish/profile）
- 重复跑幂等（grep -F 字面量去重）
- 6 case docker 黑盒 smoke 全过"
```

---

## Task 3: `install.ps1` —— PowerShell 一键安装器（Windows）

**Files:**
- Create: `install.ps1`

无法在 Linux 上自动化测试 PowerShell 脚本。脚本写完做静态 review + 在 Windows 机器上手工冒烟。

- [ ] **Step 1: 写 install.ps1（完整内容）**

```powershell
# superpowers-trae one-line installer for Windows.
# Usage:
#   iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
#
# Optional env vars:
#   SUPERPOWERS_INSTALL_DIR    target dir (default: $env:USERPROFILE\bin)
#   SUPERPOWERS_VERSION        release tag like v0.2.1 (default: latest)

$ErrorActionPreference = 'Stop'
$Repo = 'amwtke/superpowers-to-trae'

# 1. Arch check
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
if ($arch -ne [System.Runtime.InteropServices.Architecture]::X64) {
    Write-Error "unsupported arch: $arch (only X64 is supported on Windows)"
    exit 1
}

# 2. Resolve config
$dest = if ($env:SUPERPOWERS_INSTALL_DIR) { $env:SUPERPOWERS_INSTALL_DIR } else { Join-Path $env:USERPROFILE 'bin' }
$ver  = if ($env:SUPERPOWERS_VERSION)     { $env:SUPERPOWERS_VERSION }     else { 'latest' }
New-Item -ItemType Directory -Force -Path $dest | Out-Null

# 3. Build URL
if ($ver -eq 'latest') {
    $url = "https://github.com/$Repo/releases/latest/download/superpowers-trae-windows-x86_64.zip"
} else {
    $url = "https://github.com/$Repo/releases/download/$ver/superpowers-trae-windows-x86_64.zip"
}

# 4. Download + extract
$tmp = Join-Path $env:TEMP "superpowers-trae-$([guid]::NewGuid()).zip"
Write-Host "Downloading $url ..."
Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
Expand-Archive -Force -Path $tmp -DestinationPath $dest
Remove-Item $tmp

# 5. PATH（User scope，永久 + 当前会话）
$pathChanged = $false
$user = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not ($user -like "*$dest*")) {
    $newPath = if ($user) { "$user;$dest" } else { $dest }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $pathChanged = $true
}
$env:Path = "$env:Path;$dest"

# 6. Verify + report
$exe = Join-Path $dest 'superpowers-trae.exe'
$verOut = & $exe --version 2>&1
Write-Host "✓ $verOut installed at $exe" -ForegroundColor Green

if ($pathChanged) {
    Write-Host "PATH updated (User scope) — open a new terminal for it to take effect" -ForegroundColor Yellow
}
```

- [ ] **Step 2: 静态 review check**

肉眼过一遍：
- `$ErrorActionPreference = 'Stop'` 在头部
- 所有 cmdlet 都是 PowerShell 5.1 自带（Invoke-WebRequest / Expand-Archive / New-Item / Environment）
- 没有 PS Gallery 模块依赖
- 路径用 `Join-Path` 而不是字符串拼接

- [ ] **Step 3: 提交**

```bash
git add install.ps1
git commit -m "feat(install): 加 PowerShell 一键安装脚本 install.ps1

- 支持 Windows x86_64
- 默认装到 %USERPROFILE%\bin；SUPERPOWERS_INSTALL_DIR 可覆盖
- 默认装最新 release；SUPERPOWERS_VERSION 可锁定 tag
- 写 User PATH 永久生效 + 当前会话立即生效
- 仅依赖 PowerShell 5.1+ 内置 cmdlet"
```

---

## Task 4: 删 v0.1 deprecated 包袱

**Files:**
- Delete: `scripts/install.sh`
- Delete: `dist/project/` 整个目录

- [ ] **Step 1: 删旧 install.sh**

```bash
git rm scripts/install.sh
```

- [ ] **Step 2: 删 dist/project/**

```bash
git rm -r dist/project
```

- [ ] **Step 3: 验证 cargo build 不受影响**

`cli/` 用 `include_dir!` 嵌入 `dist/user/`，不读 `dist/project/`。验证：

```bash
cd cli && cargo check && cd ..
```

预期：编译通过，无关于 `dist/project` 的错误。

- [ ] **Step 4: 跑一次完整测试链**

```bash
make test
```

预期：python 测试 + Rust 测试全过。

- [ ] **Step 5: 提交**

```bash
git commit -m "chore: 删 v0.1 deprecated install.sh + dist/project/

scripts/install.sh 是 v0.1 的 dist/user → ~/.trae 拷贝器，自 Rust CLI 接管后
头注释就标了 DEPRECATED；dist/project/ 是它专属产物。CLI 不读 dist/project，
Makefile / CI / 代码均无引用，可安全删除。

历史档案（docs/superpowers/specs/2026-04-28-* / plans/2026-04-28-*）保留作记录。"
```

---

## Task 5: 重写 README 安装段

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 替换 "1. 装 superpowers-trae binary" 段**

旧内容（"从源码 cargo install" + 4 平台 details）整段替换为：

````markdown
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
  --tag v0.2.2 superpowers-trae
```

</details>
````

- [ ] **Step 2: 删 "旧 shell 脚本（deprecated）" 整段**

定位 `## 旧 shell 脚本（deprecated）` 节（脚本删了之后这段无意义），整段连同标题、3-4 行说明、空行删除。

- [ ] **Step 3: 肉眼过一遍格式**

```bash
git diff README.md | head -200
```

确认：
- markdown 格式正常（代码块闭合、details 闭合）
- 没有遗留的"`scripts/install.sh`"字符串

```bash
grep -n "scripts/install.sh\|deprecated" README.md
```

预期：grep 无匹配（或只在与本次主题无关的地方）。

- [ ] **Step 4: 提交**

```bash
git add README.md
git commit -m "docs(readme): 安装段改一键命令；删 deprecated 旧脚本说明"
```

---

## Task 6: 更新 manual-smoke-test.md

**Files:**
- Modify: `docs/superpowers/manual-smoke-test.md`

- [ ] **Step 1: 替换两处 install.sh 引用**

打开文件，找：

```
1. `bash scripts/install.sh --user`
```

替换为：

```
1. `superpowers-trae init --dir "$HOME/sp-smoke-test"`（用一个干净空目录模拟项目根，避免污染 $HOME）
```

找：

```
1. `bash scripts/install.sh --project /path/to/test-proj`
```

替换为：

```
1. `superpowers-trae init --dir /path/to/test-proj`
```

- [ ] **Step 2: 文档其它部分通读，把"shell 脚本"语境改为"CLI"**

如果文档其它句子还在描述旧脚本行为（备份、log 等），同步精简为 CLI 行为。

- [ ] **Step 3: 提交**

```bash
git add docs/superpowers/manual-smoke-test.md
git commit -m "docs(smoke-test): 用 superpowers-trae init 替换 scripts/install.sh"
```

---

## Task 7: 更新 trae-agents-setup.md

**Files:**
- Modify: `docs/superpowers/trae-agents-setup.md`

- [ ] **Step 1: 改 line 3**

旧：

```
`scripts/install.sh` 已经把 `rules/project_rules.md` 和 `skills/superpowers/` 文件级落到项目里。但 **Trae 的 Custom Agent 跟 Trae 账号绑定、存在 ByteDance 服务器**，本地脚本装不进去——必须在 Trae IDE UI 手工创建。
```

新：

```
`superpowers-trae init` 已经把 `rules/project_rules.md` 和 `skills/superpowers/` 文件级落到项目里。但 **Trae 的 Custom Agent 跟 Trae 账号绑定、存在 ByteDance 服务器**，CLI 装不进去——必须在 Trae IDE UI 手工创建。
```

- [ ] **Step 2: 提交**

```bash
git add docs/superpowers/trae-agents-setup.md
git commit -m "docs(trae-agents): scripts/install.sh → superpowers-trae init"
```

---

## Task 8: bump v0.2.2 + 触发 release

**Files:**
- Modify: `cli/Cargo.toml`
- Modify: `cli/Cargo.lock` (auto)
- Modify: `README.md`（"从源码安装"段的 `--tag v0.2.x`）

- [ ] **Step 1: bump Cargo.toml**

```toml
# cli/Cargo.toml line 3
version = "0.2.2"
```

- [ ] **Step 2: 同步 README 里的 cargo install tag**

定位 `--tag v0.2.1 superpowers-trae` 改成 `--tag v0.2.2 superpowers-trae`。

- [ ] **Step 3: 让 cargo 同步 Cargo.lock**

```bash
cd cli && cargo check --offline 2>&1 | tail -3 && cd ..
git diff cli/Cargo.lock | head -10
```

预期：`cli/Cargo.lock` 里 `name = "superpowers-trae"` 那块版本号自动改成 `0.2.2`。

- [ ] **Step 4: 提交**

```bash
git add cli/Cargo.toml cli/Cargo.lock README.md
git commit -m "chore(version): bump to v0.2.2

切版本以触发新一轮 release，同时把 README cargo install --tag 同步到 v0.2.2。"
```

- [ ] **Step 5: push + tag + push tag**

```bash
git push origin main
git tag -a v0.2.2 -m "v0.2.2 — 一键安装脚本 install.sh / install.ps1"
git push origin v0.2.2
```

- [ ] **Step 6: 盯 release workflow**

```bash
gh run list --workflow=release.yml --limit 1
gh run watch <run-id> --exit-status
```

预期：4 个 matrix（含 macOS Intel）全 ✓，release 页 4 个 asset 齐全。

- [ ] **Step 7: 真机自测一键命令（happy path 验收）**

```bash
SUPERPOWERS_INSTALL_DIR=/tmp/sp-final-test \
    curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh
/tmp/sp-final-test/superpowers-trae --version
# 预期输出：superpowers-trae 0.2.2
rm -rf /tmp/sp-final-test
```

---

## 验收清单

- [ ] `tests/install-script-smoke.sh` 6 个 case 全过
- [ ] `install.sh` shellcheck 零告警（可选：`shellcheck install.sh`）
- [ ] `install.ps1` 在 Windows 实机跑一次成功（让 Trae Windows 用户或本地 VM 验证）
- [ ] `scripts/install.sh` + `dist/project/` 已删
- [ ] `grep -rn "scripts/install.sh" README.md docs/superpowers/manual-smoke-test.md docs/superpowers/trae-agents-setup.md` 无匹配
- [ ] release v0.2.2 4 个 asset 齐全
- [ ] 一键命令真机跑通，二进制版本号显示 0.2.2

---

## 风险与备注

- **release.yml 已在 v0.2.1 验证**：4 平台 matrix + permissions 已正确，本计划不动 release.yml。
- **macOS Intel runner 排队**：v0.2.1 时 macos-13 排了 30+ 分钟。如果 v0.2.2 触发后 macos-13 长时间 queued，先放着等；其它 3 个产物会先到位。
- **install.ps1 无 CI**：维护者机器是 Linux，无法跑 Windows 自动化。计划只做静态 review + 手工冒烟。如未来需要 CI，可加 windows-latest job 在 release.yml 之外的 workflow 里跑（follow-up）。
- **smoke 测试需要联网**：tests/install-script-smoke.sh 真实下载 GitHub release，离线环境不可用。这是有意为之——黑盒断言用户视角行为。
