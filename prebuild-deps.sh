#!/bin/bash

# CC-Switch 依赖预编译脚本
# 用于新环境或 CI/CD 环境快速预编译常用依赖

set -e

echo "╔════════════════════════════════════════════════════╗"
echo "║     CC-Switch Dependencies Pre-Compiler            ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# Rust 编译优化
export CARGO_INCREMENTAL=1
export CARGO_TARGET_DIR="$PWD/.prebuild-cache"

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    echo "   建议: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "🔧 Rust 版本信息:"
rustc --version
cargo --version
echo ""

# 创建预编译缓存目录
mkdir -p .prebuild-cache

echo "📦 开始预编译核心依赖..."

# 1. 预编译 Tauri 相关依赖（最耗时）
echo ""
echo "🚀 预编译 Tauri 核心依赖..."
timeout 300 cargo fetch --manifest-path src-tauri/Cargo.toml
timeout 600 cargo build --manifest-path src-tauri/Cargo.toml --release --bins --target-dir .prebuild-cache/tauri || {
    echo "⚠️ Tauri 预编译超时，将在实际构建时继续"
}

# 2. 预编译 Web 服务器依赖
echo ""
echo "🌐 预编译 Web 服务器依赖..."
timeout 300 cargo fetch --manifest-path crates/server/Cargo.toml
timeout 600 cargo build --manifest-path crates/server/Cargo.toml --release --target-dir .prebuild-cache/server || {
    echo "⚠️ Server 预编译超时，将在实际构建时继续"
}

# 3. 清理编译产物，只保留缓存
echo ""
echo "🧹 清理编译产物，保留依赖缓存..."
rm -rf .prebuild-cache/*/release
rm -rf .prebuild-cache/*/debug

# 显示缓存大小
if [ -d ".prebuild-cache" ]; then
    CACHE_SIZE=$(du -sh .prebuild-cache 2>/dev/null | cut -f1)
    echo "✅ 依赖缓存完成 (${CACHE_SIZE})"
else
    echo "✅ 依赖下载完成"
fi

echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║                   完成！🎉                         ║"
echo "╠════════════════════════════════════════════════════╣"
echo "║                                                    ║"
echo "║  现在可以运行 ./build-web-release.sh              ║"
echo "║  编译速度将大幅提升                               ║"
echo "║                                                    ║"
echo "╚════════════════════════════════════════════════════╝"