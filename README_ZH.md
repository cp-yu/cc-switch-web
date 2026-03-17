<div align="center">

# CC Switch Fork

[English](README.md) | 中文 | [日本語](README_JA.md)

当前仓库的 GitHub Releases 仅正式发布 Web 运行版本。

</div>

## 项目定位

这个 fork 主要用于本地定制和持续开发，当前代码库提供：

- 管理 Claude Code、Codex、Gemini、OpenCode、OpenClaw 的配置
- 管理 MCP、Prompts、Skills、代理服务、故障转移和使用量功能
- 作为正式发布物的单二进制 Web 运行时
- 保留在仓库中的 Tauri 桌面代码，仅用于本地开发

## 界面预览

|                  主界面                   |                  添加供应商                  |
| :---------------------------------------: | :------------------------------------------: |
| ![主界面](assets/screenshots/main-zh.png) | ![添加供应商](assets/screenshots/add-zh.png) |

## 正式发布资产

GitHub Releases 仅发布 Web 运行版本。

| 平台 | 资产名 | 运行方式 |
| --- | --- | --- |
| Windows x86_64 | `cc-switch-web-v{version}-windows-x86_64.exe` | `./cc-switch-web-v{version}-windows-x86_64.exe` |
| macOS x86_64 | `cc-switch-web-v{version}-macos-x86_64` | `chmod +x ./cc-switch-web-v{version}-macos-x86_64 && ./cc-switch-web-v{version}-macos-x86_64` |
| Linux x86_64 | `cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` | `chmod +x ./cc-switch-web-v{version}-linux-x86_64-ubuntu20.04 && ./cc-switch-web-v{version}-linux-x86_64-ubuntu20.04` |

### 默认运行参数

- 访问地址：`http://127.0.0.1:17666`
- 自定义端口：`CC_SWITCH_PORT=8080`
- 自定义监听地址：`CC_SWITCH_HOST=0.0.0.0`
- Linux 兼容基线：Ubuntu 20.04+

### 平台说明

- Windows：直接在 PowerShell 或命令提示符中运行 `.exe`。
- macOS：二进制未签名，若被 Gatekeeper 阻止，需要先移除隔离属性。
- Linux：正式资产在 Ubuntu 20.04 上构建，用来显式保证最低兼容基线。

```bash
xattr -d com.apple.quarantine ./cc-switch-web-v{version}-macos-x86_64
```

## 本地开发

### 环境要求

- Node.js 18+
- pnpm 8+ 或 npm
- Rust 1.85+
- Tauri CLI 2.8+，仅桌面端本地开发时需要

### 常用命令

```bash
# 安装依赖
pnpm install

# Web 开发
pnpm dev:server
pnpm dev:web

# 类型检查
pnpm typecheck

# 前端单元测试
pnpm test:unit

# 构建嵌入式 Web 前端
pnpm build:web
```

### 本地启动 Web

```bash
./start-web.sh
```

启动后访问：

```text
http://localhost:17666
```

停止服务：

```bash
./stop-web.sh
```

运行时文件默认写入 `./.run/web/`：

- 日志：`backend.log`
- PID：`backend.pid`

如需改到其他目录：

```bash
CC_SWITCH_RUNTIME_DIR=/tmp/cc-switch-web ./start-web.sh
```

### 手动构建 Web 版本

```bash
pnpm build:web
cargo build --release --manifest-path crates/server/Cargo.toml
./crates/server/target/release/cc-switch-web
```

### 本地构建 Linux 发布同名资产

```bash
./build-web-release.sh
```

该脚本会产出 `release-web/cc-switch-web-v{version}-linux-x86_64-ubuntu20.04`。

## 技术栈

- 前端：React 18、TypeScript、Vite、TailwindCSS、TanStack Query
- 后端：Tauri 2、Rust、tokio、serde
- 测试：vitest、MSW、@testing-library/react

## 项目结构

```text
src/                 前端代码
src-tauri/           Tauri 桌面端后端
crates/server/       Web 服务端
crates/core/         共用核心逻辑
tests/               前端测试
assets/              截图等资源
docs/                补充文档
```

## 许可证

见 [LICENSE](LICENSE)。
