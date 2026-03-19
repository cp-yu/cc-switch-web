# Proxy Control Specification

## Purpose

`cap.proxy.control` 定义代理服务的启动、停止、配置与按应用接管行为。
它强调代理生命周期控制和按应用隔离的接管边界。


## Requirements

### Requirement: Proxy Lifecycle Control

系统 SHALL 支持启动和停止代理服务，并暴露当前运行状态与配置快照。

#### Scenario: Starting the proxy server

- **WHEN** 用户启动代理服务
- **THEN** 系统以当前配置启动代理进程或任务
- **AND** 状态查询显示代理为运行中

### Requirement: Per-App Takeover Control

系统 SHALL 允许为每个受支持应用单独配置代理接管状态。

#### Scenario: Enabling takeover for one app

- **WHEN** 用户为某个应用启用代理接管
- **THEN** 系统仅更新该应用的接管状态
- **AND** 不改变其他应用的接管配置
