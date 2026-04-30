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
