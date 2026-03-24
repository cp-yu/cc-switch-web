## Why

当前配置导入流程已经补齐了 Web runtime 的浏览器上传路径，但导出仍然停留在桌面端“先弹出保存对话框拿到本机路径，再由后端写文件”的模型上。这在 Tauri 桌面端成立，在浏览器里却没有对应能力：远程访问用户既不能替服务器选择本机保存路径，也不应该被迫回到桌面端才能拿到 SQL 备份。

## What Changes

- 为 Web runtime 的设置页增加基于浏览器下载的 SQL 导出流程，使用户可以直接从浏览器下载 CC Switch 导出的 SQL 备份。
- 增加复用现有认证会话的 Web 导出入口，使浏览器可以通过同一个 Web 端口直接下载 SQL 备份，而不依赖服务器本机文件路径或桌面保存对话框。
- 保持桌面端现有按文件路径导入/导出能力不变，同时让 Web runtime 拥有与之等价的“浏览器上传导入 + 浏览器下载导出”能力。
- 继续保持 Web runtime 在远程访问场景下的稳定 host/port 入口，并更新 Web 模式文档说明远程访问、上传与下载约束。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `settings-backup`: 配置导入导出必须同时覆盖桌面端本机文件路径流程与 Web runtime 的浏览器上传/下载流程。
- `web-runtime-delivery`: Web runtime 在远程访问场景下必须暴露可预测的单端口入口，并把 SQL 上传与下载能力纳入同一认证保护的 Web 服务面。

## Impact

- Affected code: `src/hooks/useImportExport.ts`, `src/components/settings/ImportExportSection.tsx`, `src/lib/api/settings.ts`, `src/lib/transport/*`, `src-tauri/src/commands/import_export.rs`, `src-tauri/src/database/backup.rs`, `crates/server/src/main.rs`, `crates/server/src/api/*`, `WEB_MODE.md`
- Affected systems: Web runtime 单端口服务、设置页导入导出交互、SQL 备份导入导出链路、远程访问部署方式、Web 鉴权会话
- Verification: 需要覆盖桌面端原有路径导入导出不回退、Web 本地访问上传/下载可用、远程访问配置下入口地址稳定且端口冲突时明确失败
