## Context

当前前端入口 `src/main.tsx` 不直接导入 `@/i18n`。Tauri 构建之所以正常，是因为平台别名解析到 `src/platform/bootstrap.tauri.ts`，该文件顺手导入了 `@/i18n`；而 Web 构建解析到 `src/platform/bootstrap.web.ts`，这里没有对应导入。结果是官方 GitHub release 的 Web runtime 能加载页面，但 `useTranslation()` 消费到的是未初始化状态，用户看到的是原始 key/template。

这不是翻译资源缺包问题。`src/i18n/index.ts` 通过静态 `import` 直接把 `en/ja/zh` JSON 编译进前端 bundle，问题仅在于 Web release 路径没有执行这个模块。

## Goals / Non-Goals

**Goals:**

- 让 Web 与 Tauri 两条前端构建路径都无条件执行同一份 i18n 初始化。
- 保证正式 Web release 首屏不会显示原始翻译 key 或插值模板。
- 保持现有语言选择逻辑、翻译资源结构和平台分支职责不变，避免顺手重构一圈。

**Non-Goals:**

- 不重写翻译资源、key 命名或语言回退策略。
- 不引入新的 i18n 依赖、远程语言包或服务端语言协商。
- 不借机重构整个前端 bootstrap 架构，只修复这条入口不对称。

## Decisions

### Decision: 把 i18n 初始化放到公共前端入口

把 `@/i18n` 放到 `src/main.tsx` 或等价的公共入口层显式导入，让所有 build mode 都经过同一初始化路径。

Rationale:

- 这是最小、最直接的修复。i18n 是前端渲染前置依赖，本来就不该寄生在某个平台特有 bootstrap 上。
- 这样能消除 “Tauri 正常只是因为副作用导入碰巧存在” 这种脆弱耦合。

Alternatives considered:

- 在 `bootstrap.web.ts` 里补一个 `@/i18n` 导入。
  Rejected，因为这会把相同副作用复制到两个平台文件里，继续保留入口不对称。
- 在 `App.tsx` 内部懒加载 i18n。
  Rejected，因为壳层组件已经开始渲染后再补初始化，本质上是在和渲染时序赌运气。

### Decision: 把回归验证绑在 Web build 路径上

验证必须覆盖 `build:web` 或 Web runtime 路径，而不是只看 Tauri 开发态。

Rationale:

- 这次回归就是 release 资产特有路径触发的。
- 如果只验证通用组件或 Tauri 入口，等于什么也没防住。

Alternatives considered:

- 仅靠人工跑 GitHub release 资产验证。
  Rejected，因为代价高，而且无法稳定防回归。

## Risks / Trade-offs

- [Risk] 入口提前导入 i18n 可能暴露原先被平台分支遮蔽的初始化顺序问题。 → Mitigation: 保持 `src/i18n/index.ts` 仅负责资源注册和语言解析，不掺入平台 API。
- [Risk] 测试如果只断言某个具体中文文案，后续文案修改会导致脆弱失败。 → Mitigation: 优先断言“不出现原始 key/template”，必要时配合稳定 key 的存在性验证。
- [Risk] Tauri 路径可能出现重复导入。 → Mitigation: 保持初始化模块幂等，只保留公共入口导入，移除平台分支里的隐式职责。
