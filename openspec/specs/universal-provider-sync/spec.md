# Universal Provider Sync Specification

## Purpose

`cap.providers.universal` 定义通用供应商定义与应用专用配置之间的同步行为。

## Requirements

### Requirement: Universal Provider Registry

系统 SHALL 提供一套独立于具体应用的通用供应商定义，用于复用公共连接与模型配置。

#### Scenario: Managing universal providers

- **WHEN** 用户创建或更新通用供应商
- **THEN** 系统持久化该通用定义
- **AND** 允许后续按应用进行同步

### Requirement: Sync To App Providers

系统 SHALL 支持把通用供应商同步到具体应用的供应商目录，并生成对应应用可用的配置。

#### Scenario: Syncing a universal provider

- **WHEN** 用户将通用供应商同步到某个应用
- **THEN** 系统生成或更新该应用下的对应供应商项
- **AND** 保留应用所需的标识与关联信息
