# Skill Management Specification

## Purpose

`cap.assets.skills` 定义技能的安装、发现、导入与按应用启用控制行为。
它确保技能既能被统一管理，也能从外部应用接入。


## Requirements

### Requirement: Managed Skill Installation

系统 SHALL 支持安装与卸载受管技能，并记录技能与应用之间的关联状态。

#### Scenario: Installing a managed skill

- **WHEN** 用户安装一个技能到受支持应用
- **THEN** 系统完成技能落盘与元数据登记
- **AND** 后续查询可返回该技能的安装状态

### Requirement: Skill Discovery And Import

系统 SHALL 支持发现可用技能源，并导入已有应用中的技能到统一管理面。

#### Scenario: Discovering and importing skills

- **WHEN** 用户触发技能发现或从应用导入技能
- **THEN** 系统返回可安装或可接管的技能集合
- **AND** 已导入的技能能够进入统一管理流程
