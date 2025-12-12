#!/bin/bash

# CC-Switch Web 版本构建脚本
# 构建一个独立的可执行文件，包含前端和后端

set -e

VERSION="0.1.0"
BUILD_DIR="release-web"
BINARY_NAME="cc-switch-web"

# Rust 编译优化设置
export CARGO_INCREMENTAL=1  # 启用增量编译
export CARGO_TARGET_DIR="$PWD/crates/server/target"  # 固定 target 目录位置

echo "╔════════════════════════════════════════════════════╗"
echo "║       CC-Switch Web Release Builder                ║"
echo "║                Version $VERSION                       ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# 检查依赖
echo "📋 Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    echo "   建议: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 检查是否为新环境（首次编译）
if [ ! -d ".cargo-cache" ]; then
    echo ""
    echo "🆕 检测到新环境，建议先运行依赖预编译："
    echo "   ./prebuild-deps.sh"
    echo ""
    echo "⏳ 继续构建可能需要较长时间（首次编译依赖）..."
    sleep 3
fi

if ! command -v node &> /dev/null; then
    echo "❌ Error: node not found. Please install Node.js."
    exit 1
fi

if ! command -v pnpm &> /dev/null; then
    echo "⚠️  pnpm not found, trying npm..."
    PKG_MGR="npm"
else
    PKG_MGR="pnpm"
fi

echo "✓ Dependencies OK (using $PKG_MGR)"
echo ""

# 清理旧构建
echo "🧹 Cleaning old builds..."
rm -rf dist/
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# 注意：不清理 target 目录以利用编译缓存

# 显示编译缓存状态
if [ -d "crates/server/target" ]; then
    CACHE_SIZE=$(du -sh crates/server/target 2>/dev/null | cut -f1)
    echo "💾 Using existing build cache (${CACHE_SIZE})"
else
    echo "🆕 No build cache found, will compile from scratch"
fi

# 安装前端依赖
echo ""
echo "📦 Installing frontend dependencies..."
$PKG_MGR install

# 构建前端
echo ""
echo "🔨 Building frontend (Web mode)..."
$PKG_MGR run build:web

if [ ! -f "dist/index.html" ]; then
    echo "❌ Frontend build failed: dist/index.html not found"
    exit 1
fi

echo "✓ Frontend built successfully"
echo "  Files in dist/:"
ls -la dist/ | head -10

# 构建后端（会嵌入 dist/ 目录的前端文件）
echo ""
echo "🔨 Building backend (Release mode with incremental compilation)..."
cargo build --release --manifest-path crates/server/Cargo.toml

BINARY_PATH="crates/server/target/release/$BINARY_NAME"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Backend build failed: $BINARY_PATH not found"
    exit 1
fi

echo "✓ Backend built successfully"

# 复制到发布目录
echo ""
echo "📁 Creating release package..."

cp "$BINARY_PATH" "$BUILD_DIR/"
chmod +x "$BUILD_DIR/$BINARY_NAME"

# 获取二进制大小
BINARY_SIZE=$(du -h "$BUILD_DIR/$BINARY_NAME" | cut -f1)

# 创建启动脚本
cat > "$BUILD_DIR/run.sh" << 'RUNEOF'
#!/bin/bash
# CC-Switch Web 启动脚本

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 默认配置
export CC_SWITCH_PORT=${CC_SWITCH_PORT:-17666}
export CC_SWITCH_HOST=${CC_SWITCH_HOST:-127.0.0.1}

./cc-switch-web "$@"
RUNEOF
chmod +x "$BUILD_DIR/run.sh"

# 创建 systemd 服务文件（可选）
cat > "$BUILD_DIR/cc-switch-web.service" << 'SVCEOF'
[Unit]
Description=CC-Switch Web Server
After=network.target

[Service]
Type=simple
User=%i
WorkingDirectory=/opt/cc-switch-web
ExecStart=/opt/cc-switch-web/cc-switch-web
Restart=on-failure
RestartSec=5
Environment=CC_SWITCH_PORT=17666
Environment=CC_SWITCH_HOST=0.0.0.0

[Install]
WantedBy=multi-user.target
SVCEOF

# 创建 README
cat > "$BUILD_DIR/README.md" << 'READMEEOF'
# CC-Switch Web Server

一个独立运行的 CC-Switch Web 版本，无需 Tauri 桌面环境。

## 快速开始

```bash
# 直接运行
./cc-switch-web

# 或使用启动脚本
./run.sh
```

然后在浏览器打开: http://localhost:17666

## 配置

通过环境变量配置：

```bash
# 修改端口
CC_SWITCH_PORT=8080 ./cc-switch-web

# 监听所有网络接口（远程访问）
CC_SWITCH_HOST=0.0.0.0 ./cc-switch-web

# 设置认证 Token（可选）
CC_SWITCH_AUTH_TOKEN=your-secret-token ./cc-switch-web
```

## 作为系统服务运行

```bash
# 复制文件到 /opt
sudo mkdir -p /opt/cc-switch-web
sudo cp cc-switch-web run.sh /opt/cc-switch-web/
sudo chmod +x /opt/cc-switch-web/*

# 安装 systemd 服务
sudo cp cc-switch-web.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable cc-switch-web
sudo systemctl start cc-switch-web

# 查看状态
sudo systemctl status cc-switch-web
```

## 数据位置

- 数据库: `~/.cc-switch/cc-switch.db`
- 设置: `~/.cc-switch/settings.json`
- Claude 配置: `~/.claude.json`

## 端口说明

- 默认端口: `17666`
- API: `http://localhost:17666/api/invoke`
- WebSocket: `ws://localhost:17666/api/ws`
- 健康检查: `http://localhost:17666/health`
READMEEOF

# 创建压缩包
echo ""
echo "📦 Creating archive..."
ARCHIVE_NAME="cc-switch-web-linux-x64-v${VERSION}.tar.gz"
tar -czf "$ARCHIVE_NAME" -C "$BUILD_DIR" .

# 最终输出
echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║              Build Complete! 🎉                    ║"
echo "╠════════════════════════════════════════════════════╣"
echo "║                                                    ║"
printf "║  📁 Release directory: %-27s ║\n" "$BUILD_DIR/"
printf "║  📦 Archive: %-38s ║\n" "$ARCHIVE_NAME"
printf "║  📊 Binary size: %-34s ║\n" "$BINARY_SIZE"
echo "║                                                    ║"
echo "╠════════════════════════════════════════════════════╣"
echo "║  To run:                                           ║"
echo "║    cd $BUILD_DIR && ./cc-switch-web                 ║"
echo "║                                                    ║"
echo "║  Or extract archive:                               ║"
echo "║    tar -xzf $ARCHIVE_NAME"
echo "║    ./cc-switch-web                                 ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# 显示发布目录内容
echo "📂 Release contents:"
ls -lah "$BUILD_DIR/"
