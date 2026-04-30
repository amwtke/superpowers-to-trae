# superpowers-trae one-line installer for Windows.
# Requires: PowerShell 5.1+ (Win10 built-in) or PowerShell 7+.
# Usage:
#   iwr -useb https://raw.githubusercontent.com/amwtke/superpowers-to-trae/main/install.ps1 | iex
#
# Optional env vars:
#   SUPERPOWERS_INSTALL_DIR    target dir (default: $env:USERPROFILE\bin)
#   SUPERPOWERS_VERSION        release tag like v0.2.1 (default: latest)

$ErrorActionPreference = 'Stop'
$Repo = 'amwtke/superpowers-to-trae'

# Ensure TLS 1.2 on older Win10 builds where it isn't the default
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

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

# 4. Download + extract (clean up temp on failure)
$tmp = Join-Path $env:TEMP "superpowers-trae-$([guid]::NewGuid()).zip"
Write-Host "Downloading $url ..."
try {
    Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
    Expand-Archive -Force -Path $tmp -DestinationPath $dest
} finally {
    if (Test-Path $tmp) { Remove-Item $tmp -ErrorAction SilentlyContinue }
}

# 5. PATH (User scope persistent + current session)
$pathChanged = $false
$user = [Environment]::GetEnvironmentVariable('Path', 'User')
$destNorm = $dest.TrimEnd('\')
$alreadyOnPath = $false
if ($user) {
    foreach ($entry in ($user -split ';')) {
        if ($entry -and ($entry.TrimEnd('\') -ieq $destNorm)) {
            $alreadyOnPath = $true
            break
        }
    }
}
if (-not $alreadyOnPath) {
    $newPath = if ($user) { "$user;$dest" } else { $dest }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $pathChanged = $true
}
$env:Path = "$env:Path;$dest"

# 6. Verify + report
$exe = Join-Path $dest 'superpowers-trae.exe'
if (-not (Test-Path $exe)) {
    Write-Error "extracted archive does not contain superpowers-trae.exe at $exe"
    exit 1
}
try {
    $verOut = & $exe --version 2>&1
    if ($LASTEXITCODE -ne 0) { throw "exit code $LASTEXITCODE" }
} catch {
    Write-Error "binary failed to launch: $_  (arch mismatch? missing VC++ runtime? try https://aka.ms/vs/17/release/vc_redist.x64.exe)"
    exit 1
}
Write-Host "✓ $verOut installed at $exe" -ForegroundColor Green

if ($pathChanged) {
    Write-Host "PATH updated (User scope) — open a new terminal for it to take effect" -ForegroundColor Yellow
}
