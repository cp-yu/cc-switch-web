# Provider Switching Specification

## Purpose

`cap.providers.switch` 定义活动供应商切换与默认配置导入行为。
它覆盖活动目标切换以及从 live 配置纳管默认值的过程。


## Requirements

### Requirement: Active Provider Switching

系统 SHALL 支持切换每个应用的当前供应商，并在成功后暴露新的活动目标。

#### Scenario: Switching the current provider

- **WHEN** 用户将某个应用切换到另一个已存在的供应商
- **THEN** 系统把该供应商标记为当前活动项
- **AND** 后续查询返回新的当前供应商标识

### Requirement: Default Config Import

系统 SHALL 支持把应用当前生效的 live 配置导入为受管默认供应商，以便纳入统一管理。

#### Scenario: Importing live defaults

- **WHEN** 用户触发默认配置导入
- **THEN** 系统读取当前 live 配置
- **AND** 将其转换为可管理的供应商条目
