# MCP Management Specification

## Purpose

`cap.assets.mcp` 定义统一管理 MCP 服务器配置、启用状态与多应用同步的行为约束。

## Requirements

### Requirement: Unified MCP Catalog

系统 SHALL 提供统一的 MCP 服务器目录，使界面与后端命令都通过同一份受管配置读取和修改服务器定义。

#### Scenario: Listing managed MCP servers

- **WHEN** 用户读取统一 MCP 配置或打开 MCP 管理界面
- **THEN** 系统返回集中维护的 MCP 服务器列表
- **AND** 每个条目包含服务器定义、启用状态与适用应用信息

### Requirement: App-Specific MCP Sync

系统 SHALL 支持按应用启用或停用 MCP 服务器，并将变更同步到对应应用的生效配置。

#### Scenario: Toggling MCP server for an app

- **WHEN** 用户为 Claude、Codex、Gemini 或其他受支持应用切换某个 MCP 服务器的启用状态
- **THEN** 系统更新受管配置中的应用开关
- **AND** 仅把变更同步到被选中的目标应用配置
