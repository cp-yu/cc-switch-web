# OpenClaw Runtime Specification

## Purpose

`cap.workspace.openclaw` 定义 OpenClaw 默认模型、环境、工具与健康检查管理行为。

## Requirements

### Requirement: OpenClaw Runtime Configuration

系统 SHALL 支持读取与更新 OpenClaw 的默认模型、模型目录、agents 默认值、环境变量和工具配置。

#### Scenario: Updating OpenClaw defaults

- **WHEN** 用户修改 OpenClaw 的默认模型或 agents 默认值
- **THEN** 系统持久化对应配置
- **AND** 后续读取返回更新后的结果

### Requirement: OpenClaw Health Inspection

系统 SHALL 支持扫描 OpenClaw 运行时配置健康状态，并暴露警告信息。

#### Scenario: Scanning OpenClaw health

- **WHEN** 用户打开 OpenClaw 运行时相关页面或主动触发健康扫描
- **THEN** 系统返回配置健康警告列表
- **AND** 保留足够的信息帮助定位问题
