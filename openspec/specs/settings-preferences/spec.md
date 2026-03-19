# Settings Preferences Specification

## Purpose

`cap.settings.preferences` 定义应用设置、运行时开关与本地配置路径控制行为。

## Requirements

### Requirement: Settings Persistence

系统 SHALL 提供读取和保存应用设置的能力，并保持设置结构稳定可回读。

#### Scenario: Saving application settings

- **WHEN** 用户保存设置页中的配置项
- **THEN** 系统持久化新的设置值
- **AND** 后续读取返回相同结果

### Requirement: Runtime Toggle Management

系统 SHALL 支持管理自动启动、日志、优化器、整流器和配置目录覆盖等运行时选项。

#### Scenario: Updating runtime toggles

- **WHEN** 用户修改运行时相关设置
- **THEN** 系统更新对应配置
- **AND** 需要时返回新的运行时状态或生效路径
