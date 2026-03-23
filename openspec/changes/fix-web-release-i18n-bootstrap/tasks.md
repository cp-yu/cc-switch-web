## 1. Bootstrap Alignment

- [x] 1.1 将前端 i18n 初始化移动到 Web 与 Tauri 共用的入口层，消除对 `bootstrap.tauri.ts` 副作用导入的依赖
- [x] 1.2 清理平台 bootstrap 中仅用于顺带触发 i18n 的隐式耦合，保持平台文件只负责平台差异逻辑

## 2. Regression Coverage

- [x] 2.1 为 Web build/runtime 路径补充回归验证，确认用户界面不会显示原始 i18n key 或插值模板
- [x] 2.2 运行并记录与本变更相关的最小验证集合，覆盖至少一个 Web 路径和一个现有正常路径
