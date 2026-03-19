# Specs Index

## Purpose

这份索引把主 specs 的目录命名与正式 OPSX capability ID 对齐，避免后续在 proposal、delta spec、archive 或 sync 流程里靠记忆做映射。

## Naming Contract

- `openspec/specs/<spec-slug>/spec.md` 是主 spec 的稳定落盘路径。
- `cap.*` 是正式架构图里的 capability 标识，作为跨文档引用的权威名字。
- `dom.*` 表示 capability 所属的架构边界；需要先理解边界时先看 domain，再看 spec 细节。
- 新 change 里的 delta specs 应复用这里的 `spec-slug`，不要随手再造同义目录名。
- 如果 capability intent 与 spec 内容发生实质偏移，优先同步主 spec 和 OPSX，而不是额外加一层别名。

## Capability Map

| Spec Slug | Capability ID | Domain | Primary Code Refs |
| --- | --- | --- | --- |
| `mcp-management` | `cap.assets.mcp` | `dom.assets` | `src-tauri/src/commands/mcp.rs:55`, `src-tauri/src/commands/mcp.rs:164` |
| `prompt-management` | `cap.assets.prompts` | `dom.assets` | `src-tauri/src/commands/prompt.rs:12` |
| `skill-management` | `cap.assets.skills` | `dom.assets` | `src-tauri/src/commands/skill.rs:32`, `src-tauri/src/commands/skill.rs:98` |
| `provider-catalog` | `cap.providers.catalog` | `dom.providers` | `src-tauri/src/commands/provider.rs:14`, `src-tauri/src/services/provider/mod.rs:133` |
| `provider-switching` | `cap.providers.switch` | `dom.providers` | `src-tauri/src/commands/provider.rs:72`, `src-tauri/src/commands/provider.rs:90` |
| `universal-provider-sync` | `cap.providers.universal` | `dom.providers` | `src-tauri/src/commands/provider.rs:278` |
| `proxy-control` | `cap.proxy.control` | `dom.proxy` | `src-tauri/src/commands/proxy.rs:12` |
| `proxy-resilience` | `cap.proxy.resilience` | `dom.proxy` | `src-tauri/src/commands/proxy.rs:251`, `src-tauri/src/commands/proxy.rs:367` |
| `usage-observability` | `cap.proxy.usage` | `dom.proxy` | `src-tauri/src/commands/usage.rs:15`, `src-tauri/src/commands/proxy.rs:266` |
| `runtime-shell` | `cap.runtime.shell` | `dom.runtime` | `src/App.tsx:1` |
| `runtime-command-surface` | `cap.runtime.commands` | `dom.runtime` | `src-tauri/src/lib.rs:481` |
| `web-runtime-delivery` | `cap.runtime.web` | `dom.runtime` | `crates/server/src/main.rs:111` |
| `settings-preferences` | `cap.settings.preferences` | `dom.settings` | `src-tauri/src/commands/settings.rs:17` |
| `settings-backup` | `cap.settings.backup` | `dom.settings` | `src-tauri/src/commands/import_export.rs:21`, `src-tauri/src/commands/import_export.rs:128` |
| `settings-sync` | `cap.settings.sync` | `dom.settings` | `src-tauri/src/commands/webdav_sync.rs:85`, `src-tauri/src/commands/deeplink.rs:10` |
| `workspace-memory` | `cap.workspace.memory` | `dom.workspace` | `src-tauri/src/commands/workspace.rs:60`, `src-tauri/src/commands/workspace.rs:310` |
| `session-management` | `cap.workspace.sessions` | `dom.workspace` | `src-tauri/src/commands/session_manager.rs:6` |
| `openclaw-runtime` | `cap.workspace.openclaw` | `dom.workspace` | `src-tauri/src/commands/openclaw.rs:42`, `src-tauri/src/commands/openclaw.rs:83`, `src-tauri/src/commands/openclaw.rs:102` |
