# Web Runtime Delivery Specification

## Purpose

`cap.runtime.web` 定义嵌入式 Web 运行时的静态资源交付、API 暴露与端口选择行为。

## Requirements

### Requirement: Embedded SPA Delivery

系统 SHALL 在 Web 运行时中交付嵌入式前端资源，并为 SPA 路由提供回退行为。

#### Scenario: Serving the web UI

- **WHEN** 用户访问 Web 运行时根路径或任意前端路由
- **THEN** 系统返回嵌入的前端资源或 `index.html`
- **AND** 非 API 路由使用 SPA 回退逻辑

### Requirement: API And Port Availability

系统 SHALL 暴露 HTTP 与 WebSocket API，并在默认端口不可用时选择可用端口或明确失败。

#### Scenario: Starting on an occupied default port

- **WHEN** 默认端口已被占用且自动选端口功能开启
- **THEN** 系统选择后续可用端口启动
- **AND** 输出新的访问地址供用户使用
