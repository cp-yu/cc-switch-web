## Why

当前 GitHub Releases 发布的 Ubuntu 程序实际是单二进制 Web runtime，而这条 `build:web` 路径没有稳定触发前端 i18n 初始化。结果是用户打开网页界面时会直接看到原始翻译 key 或插值模板，破坏首屏可用性，也打破了 Web 与 Tauri 的运行时一致性。

## What Changes

- 将前端本地化初始化收敛到 Web 与 Tauri 都会经过的公共入口，而不是依赖某个平台分支间接导入。
- 明确 Web runtime 在交付嵌入式 SPA 时，必须在用户可见界面渲染前完成本地化引导，避免暴露原始 i18n key 或模板。
- 为 `build:web` / GitHub Web release 路径补充回归验证，覆盖这次入口初始化缺失的故障模式。

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `web-runtime-delivery`: 嵌入式 Web runtime 在交付 SPA 时必须完成前端本地化引导，保证正式 Web release 不显示原始翻译 key 或模板。

## Impact

- Affected code: `src/main.tsx`, `src/platform/bootstrap.web.ts`, `src/platform/bootstrap.tauri.ts`, `src/i18n/index.ts`
- Affected systems: `pnpm build:web` 构建产物、`cc-switch-web` GitHub release 资产、嵌入式 Web runtime 首屏渲染
- Verification: 需要覆盖 Web build 路径的本地化初始化回归，而不只是 Tauri 路径
