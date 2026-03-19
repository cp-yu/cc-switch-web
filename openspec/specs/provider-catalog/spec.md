# Provider Catalog Specification

## Purpose

`cap.providers.catalog` 定义多应用供应商目录的增删改查与排序行为。
它约束各应用供应商集合的稳定存储与可预期展示方式。


## Requirements

### Requirement: Per-App Provider Catalog

系统 SHALL 按应用维护供应商目录，支持列出、添加、更新、删除和排序供应商配置。

#### Scenario: Managing providers for one app

- **WHEN** 用户在某个应用下增删改查供应商
- **THEN** 系统仅修改该应用所属的供应商集合
- **AND** 返回更新后的目录状态

### Requirement: Stable Provider Ordering

系统 SHALL 允许显式更新供应商排序，以支持稳定的 UI 展示和切换顺序。

#### Scenario: Reordering providers

- **WHEN** 用户调整供应商顺序
- **THEN** 系统持久化新的排序结果
- **AND** 后续读取保持相同顺序
