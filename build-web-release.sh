#!/bin/bash

# CC-Switch Web 构建脚本
# 产出与 GitHub Releases Linux Web 资产同名的本地二进制。

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
SOURCE_BINARY_NAME="cc-switch-web"
SOURCE_BINARY_PATH="$PROJECT_ROOT/crates/server/target/release/$SOURCE_BINARY_NAME"
RELEASE_VERSION=""
RELEASE_ASSET_NAME=""
RELEASE_ASSET_PATH=""

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

RELEASE_VERSION="${RELEASE_VERSION:-$(node -p "require('./package.json').version")}"
RELEASE_ASSET_NAME="cc-switch-web-v${RELEASE_VERSION}-linux-x86_64-ubuntu20.04"
RELEASE_ASSET_PATH="$OUTPUT_DIR/$RELEASE_ASSET_NAME"

if [[ ! -d "$PROJECT_ROOT/node_modules" ]]; then
    echo "❌ Error: node_modules not found."
    echo "   Please run \`${PACKAGE_MANAGER} install\` first."
    exit 1
fi

echo "📦 Using package manager: $PACKAGE_MANAGER"
echo "🏷️  Release version: $RELEASE_VERSION"
echo "📁 Output directory: $OUTPUT_DIR"
echo "📌 Asset name: $RELEASE_ASSET_NAME"
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

if [[ ! -x "$SOURCE_BINARY_PATH" ]]; then
    echo "❌ Error: backend build failed, binary not found at $SOURCE_BINARY_PATH"
    exit 1
fi

cp "$SOURCE_BINARY_PATH" "$RELEASE_ASSET_PATH"
chmod +x "$RELEASE_ASSET_PATH"

BINARY_SIZE="$(du -h "$RELEASE_ASSET_PATH" | cut -f1)"

echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║                 Build Complete                    ║"
echo "╠════════════════════════════════════════════════════╣"
printf "║  Output: %-40s ║\n" "$RELEASE_ASSET_PATH"
printf "║  Size:   %-40s ║\n" "$BINARY_SIZE"
echo "╠════════════════════════════════════════════════════╣"
echo "║  Run:                                              ║"
printf "║    %s%-43s ║\n" "" "$RELEASE_ASSET_PATH"
echo "╚════════════════════════════════════════════════════╝"
echo ""
echo "For Ubuntu 20.04 compatibility parity, build this script on Ubuntu 20.04 or an equivalent baseline environment."
