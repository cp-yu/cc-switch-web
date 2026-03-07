#!/bin/bash

# CC-Switch Web 模式启动脚本

set -e

SCRIPT_SOURCE="${BASH_SOURCE[0]}"
while [ -L "$SCRIPT_SOURCE" ]; do
    SCRIPT_DIR="$(cd -P "$(dirname "$SCRIPT_SOURCE")" && pwd)"
    SCRIPT_SOURCE="$(readlink "$SCRIPT_SOURCE")"
    [[ "$SCRIPT_SOURCE" != /* ]] && SCRIPT_SOURCE="$SCRIPT_DIR/$SCRIPT_SOURCE"
done
PROJECT_ROOT="$(cd -P "$(dirname "$SCRIPT_SOURCE")" && pwd)"

cd "$PROJECT_ROOT"

echo "🚀 CC-Switch Web Mode Launcher"
echo "================================"
echo ""

# 检查依赖
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

if ! command -v node &> /dev/null; then
    echo "❌ Error: node not found. Please install Node.js."
    exit 1
fi

# 检查端口占用
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "⚠️  Port $1 is already in use"
        read -p "Kill the process? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            lsof -ti:$1 | xargs -r kill -9
            echo "✓ Killed process on port $1"
        else
            return 1
        fi
    fi
}

echo "📦 Checking ports..."
check_port 17666 || { echo "Backend port unavailable"; exit 1; }
check_port 3001 || { echo "Frontend port unavailable"; exit 1; }

echo ""
echo "🔨 Building backend server..."
cargo build --release --manifest-path crates/server/Cargo.toml

echo ""
echo "🎯 Starting services..."
echo ""

# 启动后端
echo "▶ Starting backend on http://localhost:17666"
cargo run --release --manifest-path crates/server/Cargo.toml > /tmp/cc-switch-backend.log 2>&1 &
BACKEND_PID=$!
echo "  Backend PID: $BACKEND_PID"

# 等待后端启动
sleep 2
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo "❌ Backend failed to start. Check logs:"
    tail -20 /tmp/cc-switch-backend.log
    exit 1
fi

# 检查后端健康
if ! curl -s http://localhost:17666/ > /dev/null; then
    echo "❌ Backend is not responding"
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi

echo "  ✓ Backend is running"
echo ""

# 启动前端
echo "▶ Starting frontend on http://localhost:3001"
npx vite --mode web --port 3001 > /tmp/cc-switch-frontend.log 2>&1 &
FRONTEND_PID=$!
echo "  Frontend PID: $FRONTEND_PID"

# 等待前端启动
sleep 3
if ! kill -0 $FRONTEND_PID 2>/dev/null; then
    echo "❌ Frontend failed to start. Check logs:"
    tail -20 /tmp/cc-switch-frontend.log
    kill $BACKEND_PID 2>/dev/null
    exit 1
fi

echo "  ✓ Frontend is running"
echo ""
echo "================================"
echo "✨ CC-Switch Web Mode is ready!"
echo ""
echo "  Frontend: http://localhost:3001"
echo "  Backend:  http://localhost:17666"
echo ""
echo "  Backend logs:  tail -f /tmp/cc-switch-backend.log"
echo "  Frontend logs: tail -f /tmp/cc-switch-frontend.log"
echo ""
echo "Press Ctrl+C to stop all services"
echo "================================"
echo ""

# 保存 PID 供停止脚本使用
echo $BACKEND_PID > /tmp/cc-switch-backend.pid
echo $FRONTEND_PID > /tmp/cc-switch-frontend.pid

# 等待中断信号
trap "echo ''; echo '🛑 Stopping services...'; kill $BACKEND_PID $FRONTEND_PID 2>/dev/null; rm -f /tmp/cc-switch-*.pid; echo '✓ Stopped'; exit 0" INT TERM

# 保持运行
wait
