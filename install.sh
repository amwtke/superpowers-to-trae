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
