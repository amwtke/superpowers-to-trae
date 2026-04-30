# 一键安装脚本（install.sh / install.ps1）：设计文档

- 日期：2026-04-30
- 状态：Draft（待用户复审）
- 子项目：v0.2.x 端用户安装链路打磨
- 关联：`v0.2.1` release 已产出 4 平台二进制（Linux x86_64 / macOS aarch64 / macOS x86_64 / Windows x86_64）

## 1. 背景与问题

v0.2.1 release matrix 已经产出 4 个平台的预编译二进制，README 也写了每平台具体的 `curl` + `tar` / PowerShell + `Expand-Archive` 步骤（约 80 行可折叠 details）。但端用户体验仍然多步：

- 选平台 → 复制对应代码块 → 跑 4-6 行命令 → 自己保证目标目录在 PATH

主流 Rust / 工具链生态（rustup、bun、deno、cargo-binstall）都已经收敛成 **一行 `curl … | sh`**。本设计把这一行带进 `superpowers-trae`：用户跑一条命令，binary 就装好且 PATH 就绪，下一步直接 `superpowers-trae init` 进项目。

同时清掉两块 v0.1 历史包袱（已被 Rust CLI 取代）：
- `scripts/install.sh`（旧 dist/user → ~/.trae 拷贝器，自己头注释就标 DEPRECATED）
- `dist/project/`（仅旧脚本使用，CLI 不读）

## 2. 目标与非目标

**目标**

1. POSIX `install.sh` + Windows `install.ps1`，覆盖 Linux x86_64、macOS aarch64/x86_64、Windows x86_64 共 4 个 release artifact。
2. 一行命令完成：检测平台 → 下对应包 → 解压 → 装到默认路径 → 必要时把路径加入 PATH。
3. 默认 user-local（无 sudo）：POSIX `~/.local/bin`、Windows `%USERPROFILE%\bin`。
4. 环境变量覆盖：`SUPERPOWERS_INSTALL_DIR`（装到哪）、`SUPERPOWERS_VERSION`（装哪个 tag，默认 `latest`）。
5. README 端用户安装段从 80 行压到 ~15 行（一键命令 + 可折叠的手工步骤兜底）。
6. 删 `scripts/install.sh` + `dist/project/` + 文档里相关引用。

**非目标**

- sha256 / GPG 校验（release 暂未发布 checksum 产物；HTTPS + GitHub releases 在当前威胁模型可接受；列入 follow-up）
- Linux ARM64 / Windows ARM64 / 其它非 release matrix 平台（无产物可装）
- 卸载脚本（删一个文件 + 一行 PATH 太简单，不值得脚本化）
- 自动 init 当前目录（设计阶段已明确选 "A：只装 binary"）
- 验证升级路径中的版本兼容性（升级是覆盖；用户需要老版本走 `SUPERPOWERS_VERSION=v0.2.0`）

## 3. 公共契约（用户视角）

### 3.1 安装命令

```bash
# Linux + macOS
curl -fsSL https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.sh | sh

# Windows (PowerShell 5.1+ / pwsh)
iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
```

### 3.2 行为约定

跑完成功后：
- 二进制位置：`$INSTALL_DIR/superpowers-trae`（POSIX）或 `$INSTALL_DIR\superpowers-trae.exe`（Windows）
- 屏幕标准输出：`✓ superpowers-trae <version> installed at <path>`，再加一行版本信息
- 如果改过 PATH（rc 文件 / 用户环境变量），多打印一行 `Open a new shell or run: source <rc>` / `Open a new terminal for PATH to take effect`
- exit code 0

失败：
- 不识别的平台/架构 → stderr 红字 + exit 1
- 没 `curl` 且没 `wget`（POSIX）→ stderr 红字 + exit 1
- 网络下载失败（HTTP 非 2xx / 连接错误）→ 透传错误 + exit 非零
- 解压失败 / 写入目标目录失败 → 透传错误 + exit 非零

### 3.3 环境变量

| 变量 | 作用 | 默认 |
|---|---|---|
| `SUPERPOWERS_INSTALL_DIR` | 装到哪 | POSIX `~/.local/bin` / Windows `%USERPROFILE%\bin` |
| `SUPERPOWERS_VERSION` | 装哪个版本 tag（如 `v0.2.1`） | `latest` |

环境变量在脚本里只读一次，不进 rc 文件、不持久化。

## 4. install.sh（POSIX shell）

### 4.1 兼容性目标

- Shebang `#!/bin/sh` + POSIX 严格语法（不用 `[[ ]]`、`local`、数组）
- 在 bash / dash / busybox-ash / zsh-as-sh 都能跑
- macOS 默认 `bash 3.2` 或 `zsh` 也覆盖
- `set -e`；定义 `die()` 输出红字到 stderr 后 exit 1

### 4.2 步骤流

```
1. 平台检测
   OS=$(uname -s | tr '[:upper:]' '[:lower:]')
   ARCH=$(uname -m)
   匹配表（不在表里 → die）：
     linux  + x86_64                  → linux-x86_64.tar.gz
     darwin + arm64 | aarch64         → macos-aarch64.tar.gz
     darwin + x86_64                  → macos-x86_64.tar.gz

2. 下载工具选择
   优先 curl -fsSL；缺失则 wget -qO；都缺 → die

3. 解析配置
   INSTALL_DIR=${SUPERPOWERS_INSTALL_DIR:-$HOME/.local/bin}
   VERSION=${SUPERPOWERS_VERSION:-latest}
   mkdir -p "$INSTALL_DIR"

4. 构造 URL
   if VERSION = latest:
     URL=https://github.com/amwtke/superpowers-to-trae/releases/latest/download/superpowers-trae-<asset>
   else:
     URL=https://github.com/amwtke/superpowers-to-trae/releases/download/<VERSION>/superpowers-trae-<asset>

5. 下载 + 解压
   TMP=$(mktemp -d)
   trap 'rm -rf "$TMP"' EXIT
   <downloader> "$URL" > "$TMP/sp.tar.gz"  (或 -o)
   tar xzf "$TMP/sp.tar.gz" -C "$TMP"
   install -m 0755 "$TMP/superpowers-trae" "$INSTALL_DIR/superpowers-trae"

6. PATH 检查 + 必要时写 rc
   case ":$PATH:" in
     *":$INSTALL_DIR:"*) need_rc=0 ;;
     *)                  need_rc=1 ;;
   esac
   if need_rc:
     检测 $SHELL basename：
       zsh   → "$HOME/.zshrc"        export 行：export PATH="$INSTALL_DIR:$PATH"
       bash  → "$HOME/.bashrc"       同上
       fish  → "$HOME/.config/fish/config.fish"  fish_add_path "$INSTALL_DIR"
       其它  → 默认 "$HOME/.profile" export 行
     如果 rc 已经包含 INSTALL_DIR 字面量（grep -F -q）→ skip 不重复追加
     否则 append 一行（前面 echo 一个 newline 隔开）

7. 验证 + 打印
   "$INSTALL_DIR/superpowers-trae" --version 读出版本号
   绿字打印 "✓ superpowers-trae <version> installed at $INSTALL_DIR/superpowers-trae"
   如果 need_rc=1，多打印一行 "Open a new shell or run: source <rc>"
```

### 4.3 边界与错误

| 场景 | 行为 |
|---|---|
| `uname -m` 返回 `aarch64` 但 OS=linux | die：unsupported（无 linux-aarch64 release） |
| `mkdir -p $INSTALL_DIR` 失败（权限） | die：cannot create install dir |
| 下载 HTTP 404（错版本号） | curl/wget 自身返回非 0 → set -e 触发退出 + 透传 stderr |
| `tar xzf` 失败（包损坏） | tar 报错 + 退出 |
| `install` 命令缺失（极少见） | fallback：`cp` + `chmod 0755` |
| rc 文件不存在 | 直接 `>>` 追加，shell 会创建 |
| 不识别 $SHELL（exotic） | 写 `$HOME/.profile`；用户重开 shell 后大多数登录 shell 会读 |

## 5. install.ps1（Windows PowerShell）

### 5.1 兼容性目标

- Windows PowerShell 5.1（Win10+ 内置）+ pwsh 7+
- `$ErrorActionPreference = 'Stop'`
- 不依赖 PowerShell Gallery 模块；只用 built-in cmdlets

### 5.2 步骤流

```
1. arch 检测
   $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
   $arch -ne 'X64' → throw（无 windows-aarch64 release）

2. 解析配置
   $dest = if ($env:SUPERPOWERS_INSTALL_DIR) { ... } else { "$env:USERPROFILE\bin" }
   $ver = if ($env:SUPERPOWERS_VERSION) { ... } else { 'latest' }
   New-Item -ItemType Directory -Force -Path $dest | Out-Null

3. 构造 URL
   if $ver = 'latest':
     $base = 'releases/latest/download'
   else:
     $base = "releases/download/$ver"
   $url = "https://github.com/amwtke/superpowers-to-trae/$base/superpowers-trae-windows-x86_64.zip"

4. 下载 + 解压
   $tmp = Join-Path $env:TEMP "sp-$(New-Guid).zip"
   Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
   Expand-Archive -Force -Path $tmp -DestinationPath $dest
   Remove-Item $tmp

5. PATH（用户级，永久）
   $user = [Environment]::GetEnvironmentVariable('Path','User')
   if ($user -notlike "*$dest*") {
     [Environment]::SetEnvironmentVariable('Path', "$user;$dest", 'User')
     $pathChanged = $true
   }
   $env:Path = "$env:Path;$dest"   # 当前会话立即生效

6. 验证 + 打印
   $ver = & "$dest\superpowers-trae.exe" --version
   Write-Host "✓ superpowers-trae $ver installed at $dest\superpowers-trae.exe" -ForegroundColor Green
   if ($pathChanged) {
     Write-Host "Open a new terminal for PATH to take effect" -ForegroundColor Yellow
   }
```

### 5.3 边界与错误

| 场景 | 行为 |
|---|---|
| 32 位 Windows / ARM64 | throw：unsupported arch |
| 没网 / DNS 失败 | Invoke-WebRequest 抛异常，$ErrorActionPreference 终止 |
| zip 损坏 | Expand-Archive 报错 |
| 用户 PATH 已含 $dest | 跳过修改，仅当前会话 append |
| `.exe` 被杀软拦 | OS 层面问题，无法在脚本里兜底；用户自行解决 |

## 6. 清理清单

### 6.1 删除

- `scripts/install.sh`（v0.1 deprecated 安装器）
- `dist/project/` 整个目录（旧脚本专属产物，CLI 不读）

### 6.2 文档更新

- **`README.md`**
  - "1. 装 superpowers-trae binary" 段重写：顶部 2 行一键命令；其下保留可折叠 `<details>` "手工安装"作为 fallback（保留当前 4 平台 curl + tar/zip 的具体步骤）；"从源码 cargo install" 留作第三选项
  - 删 line 99-105 整段 "旧 shell 脚本（deprecated）"
- **`docs/superpowers/manual-smoke-test.md`**
  - line 12 `bash scripts/install.sh --user` → `superpowers-trae init --dir <tmp>` 等价命令（同时验证整个手工冒烟流程）
  - line 24 `bash scripts/install.sh --project /path/to/test-proj` → `superpowers-trae init --dir /path/to/test-proj`
- **`docs/superpowers/trae-agents-setup.md`** line 3 引用 `scripts/install.sh` 的一句改写为 `superpowers-trae init`
- **历史 specs/plans 不动**：`docs/superpowers/specs/2026-04-28-*.md` / `docs/superpowers/plans/2026-04-28-*.md` 是历史档案

### 6.3 不动

- `Makefile`（不引用 install.sh）
- `scripts/sync-upstream.sh` / `scripts/build.sh`（maintainer pipeline 核心）
- `.github/workflows/release.yml`（与 install 链路无关）

## 7. 测试方案

| 场景 | 验证方式 | 自动化 |
|---|---|---|
| install.sh on Linux x86_64 | `tests/install-script-smoke.sh` 在 ubuntu docker 跑一键命令 | 是 |
| 不识别平台 | 同脚本里改 `OS`/`ARCH` env var 跑 | 是 |
| `SUPERPOWERS_INSTALL_DIR` | 设非默认路径跑 | 是 |
| `SUPERPOWERS_VERSION=v0.2.0` | 装老版本 | 是 |
| 重复跑（PATH 不重复追加） | 跑两次 grep rc 行数 | 是 |
| 没 curl 没 wget | minimal docker image 跑 | 是 |
| install.sh on macOS aarch64 | 本地手工（维护者机器是 Linux，外部用户 / Trae 用户验证） | 否 |
| install.sh on macOS x86_64 | 同上 | 否 |
| install.ps1 on Windows | 本地或 Trae Windows 用户冒烟 | 否 |

`tests/install-script-smoke.sh` 在 maintainer 机器上跑，CI 接入留作 follow-up（GitHub Actions 跑 docker 即可，但当前规模没必要）。

## 8. 实现顺序（高层）

1. 写 `install.sh` + 本地 docker smoke 测通
2. 写 `install.ps1`（无法本地测 → 写完代码评审 + 在 Windows 机器手工冒烟）
3. 写 `tests/install-script-smoke.sh`
4. 删 `scripts/install.sh` + `dist/project/`
5. 更新 README / manual-smoke-test / trae-agents-setup
6. 提一个 commit 全套上 main，打 v0.2.2 tag 触发 release

## 9. 风险与权衡

| 风险 | 缓解 |
|---|---|
| `curl ... \| sh` 模式被批评不安全（执行任意远程代码） | 内容来自 GitHub raw（HTTPS + GitHub 可信链），代码已入 git review；用户可用浏览器查看后再跑；提供"手工安装"fallback |
| macOS aarch64 / Windows 上脚本有 bug 但维护者无法本地验证 | 列入冒烟测试矩阵交给 Trae 用户验；脚本逻辑短（< 100 行）便于 review |
| 用户 rc 文件已经被多个工具污染、追加再加一行嫌乱 | grep 字面量去重；只在不在 PATH 时才追加 |
| GitHub raw URL 命中流量配额 | install 脚本只读 ~3KB 源码，不会触发；下载主体走 `releases/download/` 不算配额 |
