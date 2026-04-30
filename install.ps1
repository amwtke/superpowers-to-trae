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
