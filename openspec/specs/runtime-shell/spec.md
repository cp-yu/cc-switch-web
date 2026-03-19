# Runtime Shell Specification

## Purpose

`cap.runtime.shell` 定义前端运行时外壳的顶层导航、视图切换与宿主页面行为。
它约束顶层导航、当前应用状态和主要视图恢复行为。


## Requirements

### Requirement: Top-Level View Routing

系统 SHALL 提供稳定的顶层视图切换，使供应商、设置、技能、会话和工作区等页面能在统一外壳下导航。

#### Scenario: Switching between top-level views

- **WHEN** 用户在主界面切换顶层视图
- **THEN** 系统加载对应页面内容
- **AND** 保留当前应用与页面状态的必要上下文

### Requirement: Active App Shell State

系统 SHALL 保持当前应用选择与主要视图状态，以支持跨页面的一致交互。

#### Scenario: Restoring last app and view

- **WHEN** 用户重新进入应用或刷新运行时
- **THEN** 系统恢复最近一次保存的应用与视图选择
- **AND** 在不可用时回退到有效默认值
