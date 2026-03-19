# Prompt Management Specification

## Purpose

`cap.assets.prompts` 定义提示词的集中管理、启用控制与文件导入行为。
它覆盖统一提示词目录、启用状态和文件导入这三条主路径。


## Requirements

### Requirement: Managed Prompt Catalog

系统 SHALL 维护统一的提示词目录，以支持创建、更新、删除和启用提示词。

#### Scenario: Managing stored prompts

- **WHEN** 用户新增、编辑或删除提示词
- **THEN** 系统持久化对应变更
- **AND** 后续读取返回更新后的提示词集合

### Requirement: Prompt File Import

系统 SHALL 支持从文件导入提示词内容，以减少手工复制错误。

#### Scenario: Importing a prompt from file

- **WHEN** 用户选择提示词文件进行导入
- **THEN** 系统读取文件内容并生成可管理的提示词条目
- **AND** 导入后的内容可继续被启用或编辑
