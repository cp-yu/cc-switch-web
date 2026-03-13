#!/bin/bash

# CC-Switch Web 模式停止脚本

set -euo pipefail

SCRIPT_SOURCE="${BASH_SOURCE[0]}"
while [ -L "$SCRIPT_SOURCE" ]; do
    SCRIPT_DIR="$(cd -P "$(dirname "$SCRIPT_SOURCE")" && pwd)"
    SCRIPT_SOURCE="$(readlink "$SCRIPT_SOURCE")"
    [[ "$SCRIPT_SOURCE" != /* ]] && SCRIPT_SOURCE="$SCRIPT_DIR/$SCRIPT_SOURCE"
done
PROJECT_ROOT="$(cd -P "$(dirname "$SCRIPT_SOURCE")" && pwd)"

RUNTIME_DIR="${CC_SWITCH_RUNTIME_DIR:-$PROJECT_ROOT/.run/web}"
BACKEND_PID_FILE="$RUNTIME_DIR/backend.pid"
FRONTEND_PID_FILE="$RUNTIME_DIR/frontend.pid"
BACKEND_PORT="${CC_SWITCH_PORT:-17666}"
FRONTEND_PORT="${CC_SWITCH_WEB_PORT:-3001}"

is_pid_running() {
    local pid="${1:-}"
    [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

read_pid_file() {
    local pid_file="$1"
    [[ -f "$pid_file" ]] || return 1
    local pid
    pid="$(<"$pid_file")"
    [[ "$pid" =~ ^[0-9]+$ ]] || return 1
    printf '%s\n' "$pid"
}

stop_pid() {
    local pid="$1"
    local name="$2"
    local waited=0

    if ! is_pid_running "$pid"; then
        return 0
    fi

    kill "$pid" 2>/dev/null || true
    while is_pid_running "$pid" && (( waited < 10 )); do
        sleep 1
        waited=$((waited + 1))
    done

    if is_pid_running "$pid"; then
        kill -9 "$pid" 2>/dev/null || true
    fi

    echo "✓ Stopped $name (PID: $pid)"
}

find_port_pids() {
    local port="$1"

    if command -v lsof >/dev/null 2>&1; then
        lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true
        return
    fi

    if command -v fuser >/dev/null 2>&1; then
        fuser "$port/tcp" 2>/dev/null | tr ' ' '\n' | sed '/^$/d' || true
    fi
}

stop_from_pid_file() {
    local pid_file="$1"
    local name="$2"
    local pid=""

    pid="$(read_pid_file "$pid_file" 2>/dev/null || true)"
    if [[ -n "$pid" ]]; then
        stop_pid "$pid" "$name"
    fi
    rm -f "$pid_file"
}

echo "🛑 Stopping CC-Switch Web Mode..."

stop_from_pid_file "$FRONTEND_PID_FILE" "frontend"
stop_from_pid_file "$BACKEND_PID_FILE" "backend"

for pid in $(find_port_pids "$FRONTEND_PORT"); do
    if is_pid_running "$pid"; then
        stop_pid "$pid" "frontend(port $FRONTEND_PORT)"
    fi
done

for pid in $(find_port_pids "$BACKEND_PORT"); do
    if is_pid_running "$pid"; then
        stop_pid "$pid" "backend(port $BACKEND_PORT)"
    fi
done

rm -f "$FRONTEND_PID_FILE" "$BACKEND_PID_FILE"

echo "✓ All services stopped"
