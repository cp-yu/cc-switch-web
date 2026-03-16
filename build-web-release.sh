#!/bin/bash

# CC-Switch Web 构建脚本
# 仅负责构建并产出 Web 版本二进制，不处理额外发布打包流程

set -euo pipefail

SCRIPT_SOURCE="${BASH_SOURCE[0]}"
while [ -L "$SCRIPT_SOURCE" ]; do
    SCRIPT_DIR="$(cd -P "$(dirname "$SCRIPT_SOURCE")" && pwd)"
    SCRIPT_SOURCE="$(readlink "$SCRIPT_SOURCE")"
    [[ "$SCRIPT_SOURCE" != /* ]] && SCRIPT_SOURCE="$SCRIPT_DIR/$SCRIPT_SOURCE"
done
PROJECT_ROOT="$(cd -P "$(dirname "$SCRIPT_SOURCE")" && pwd)"

cd "$PROJECT_ROOT"

OUTPUT_DIR="${WEB_RELEASE_DIR:-$PROJECT_ROOT/release-web}"
BINARY_NAME="cc-switch-web"
BINARY_PATH="$PROJECT_ROOT/crates/server/target/release/$BINARY_NAME"

export CARGO_INCREMENTAL=1
export CARGO_TARGET_DIR="$PROJECT_ROOT/crates/server/target"

require_command() {
    local cmd="$1"
    local message="$2"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "❌ Error: $message"
        exit 1
    fi
}

resolve_package_manager() {
    if command -v pnpm >/dev/null 2>&1; then
        PACKAGE_MANAGER="pnpm"
        return
    fi

    if command -v npm >/dev/null 2>&1; then
        PACKAGE_MANAGER="npm"
        return
    fi

    echo "❌ Error: pnpm or npm not found."
    exit 1
}

run_web_build() {
    if [[ "$PACKAGE_MANAGER" == "pnpm" ]]; then
        pnpm build:web
    else
        npm run build:web
    fi
}

echo "╔════════════════════════════════════════════════════╗"
echo "║          CC-Switch Web Builder                    ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

require_command cargo "cargo not found. Please install Rust."
require_command node "node not found. Please install Node.js."
resolve_package_manager

if [[ ! -d "$PROJECT_ROOT/node_modules" ]]; then
    echo "❌ Error: node_modules not found."
    echo "   Please run \`${PACKAGE_MANAGER} install\` first."
    exit 1
fi

echo "📦 Using package manager: $PACKAGE_MANAGER"
echo "📁 Output directory: $OUTPUT_DIR"
echo ""

echo "🧹 Preparing output directory..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "🎨 Building frontend assets..."
run_web_build

if [[ ! -f "$PROJECT_ROOT/dist/index.html" ]]; then
    echo "❌ Error: frontend build failed, dist/index.html not found."
    exit 1
fi

echo ""
echo "🔨 Building backend binary..."
cargo build --release --manifest-path "$PROJECT_ROOT/crates/server/Cargo.toml"

if [[ ! -x "$BINARY_PATH" ]]; then
    echo "❌ Error: backend build failed, binary not found at $BINARY_PATH"
    exit 1
fi

cp "$BINARY_PATH" "$OUTPUT_DIR/$BINARY_NAME"
chmod +x "$OUTPUT_DIR/$BINARY_NAME"

BINARY_SIZE="$(du -h "$OUTPUT_DIR/$BINARY_NAME" | cut -f1)"

echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║                 Build Complete                    ║"
echo "╠════════════════════════════════════════════════════╣"
printf "║  Output: %-40s ║\n" "$OUTPUT_DIR/$BINARY_NAME"
printf "║  Size:   %-40s ║\n" "$BINARY_SIZE"
echo "╠════════════════════════════════════════════════════╣"
echo "║  Run:                                              ║"
printf "║    %s%-43s ║\n" "" "$OUTPUT_DIR/$BINARY_NAME"
echo "╚════════════════════════════════════════════════════╝"
