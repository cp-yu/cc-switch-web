# 常用命令（运行 / 构建 / 测试）

## 运行
- 开发模式（桌面 Tauri 应用）：`pnpm dev` （等价于 `pnpm tauri dev`）
- 仅前端渲染进程（本地）：`pnpm dev:renderer`
- Web 版开发模式：`pnpm dev:web`
- 仅 Rust 后端服务器（`crates/server`）：`pnpm dev:server`（相当于 `cargo run --manifest-path crates/server/Cargo.toml`）

## 构建
- 桌面应用构建（Tauri 打包）：`pnpm build`（等价于 `pnpm tauri build`）
- 仅前端构建：`pnpm build:renderer`
- Web 版静态构建：`pnpm build:web`

## 代码质量
- TypeScript 类型检查：`pnpm typecheck`
- 自动格式化：`pnpm format`
- 格式检查（不改动代码）：`pnpm format:check`

## 测试
- 前端单元测试（一次性）：`pnpm test:unit`
- 前端单元测试（监听模式）：`pnpm test:unit:watch`
- Rust 侧测试（如需）：在对应 crate 目录执行 `cargo test`（例如 `crates/core`、`crates/server`）。