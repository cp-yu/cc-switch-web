# 任务完成后的建议检查项

每次在本项目中完成开发任务后，建议按以下顺序自检：

1. **编译 / 构建检查**
   - 前端 Web / Tauri 是否能正常启动：`pnpm dev` 或 `pnpm dev:web`。
   - 如改动 Rust 代码（`src-tauri` / `crates/*`），在对应 crate 运行一次 `cargo build` 或 `cargo test` 确保无编译错误。

2. **测试**
   - 若改动前端逻辑或 hooks：运行 `pnpm test:unit`，必要时补充 / 更新相关测试用例。
   - 若改动 Rust 核心逻辑 / API：在对应 crate 目录下运行 `cargo test`（如有测试），或新增针对性的单元测试。

3. **类型与格式**
   - 运行 `pnpm typecheck` 确保 TypeScript 类型正确。
   - 运行 `pnpm format` 或至少 `pnpm format:check` 保持代码风格一致。

4. **手动验证**
   - 按改动功能点做最小化的手动验证（例如：新增 / 更新供应商、切换供应商、调用相关 API 等）。
   - 如涉及多平台行为（托盘菜单、系统托盘等），至少在当前开发平台手动点击验证一次。

5. **文档 / 备注**
   - 若引入新运行方式、环境变量或调试技巧，更新到合适的文档或记忆（如 `suggested_commands.md` 等），方便后续复用。