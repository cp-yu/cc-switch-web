## MODIFIED Requirements

### Requirement: Embedded SPA Delivery

系统 SHALL 在 Web 运行时中交付嵌入式前端资源，在用户可见界面渲染前完成前端引导依赖初始化，并为 SPA 路由提供回退行为。

#### Scenario: Serving the web UI

- **WHEN** 用户访问 Web 运行时根路径或任意前端路由
- **THEN** 系统返回嵌入的前端资源或 `index.html`
- **AND** 非 API 路由使用 SPA 回退逻辑

#### Scenario: Rendering localized UI in official web releases

- **WHEN** 用户打开通过 `build:web` 生成并由官方 Web runtime 交付的前端界面
- **THEN** 系统在渲染依赖翻译函数的用户界面前完成本地化初始化
- **AND** 用户界面不得把原始 i18n key 或插值模板作为最终文案展示
