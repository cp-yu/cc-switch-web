# 项目概览

- 名称：cc-switch
- 目标：统一管理 Claude Code / Codex / Gemini CLI 的供应商配置、MCP 服务器、Skills 扩展和系统提示词，并提供一键切换、速度测试、导入导出等能力。
- 主要技术栈：
  - 前端：TypeScript + React + Vite + Tailwind CSS
  - 桌面端：Tauri 2（Rust + WebView）
  - 后端 / 核心：Rust（`src-tauri`、`crates/core`、`crates/server` 等）
- 持久化：SQLite + JSON 双层架构（配置/供应商等同步到 SQLite，设备级数据留在 JSON）。
- 目录结构（核心部分）：
  - `src/`：前端 React 应用
  - `src-tauri/`：Tauri 主进程、命令与服务（Rust）
  - `crates/core/`：核心业务逻辑库（Rust）
  - `crates/server/`：独立服务器 / API 层（Rust）
  - `assets/`、`docs/` 等：静态资源与文档。