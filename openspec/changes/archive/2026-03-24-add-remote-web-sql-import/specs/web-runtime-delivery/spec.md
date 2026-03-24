## MODIFIED Requirements

### Requirement: API And Port Availability

系统 SHALL 暴露 HTTP 与 WebSocket API；在回环地址本机使用场景下，默认端口不可用时可选择可用端口或明确失败；在非回环地址的远程访问场景下，系统 MUST 保持配置的 host/port 入口稳定，并在请求端口不可用时明确失败而不是静默切换到其他端口。

#### Scenario: Starting on an occupied default port for local access
- **WHEN** Web runtime 使用回环地址启动且默认端口已被占用，并且自动选端口功能开启
- **THEN** 系统选择后续可用端口启动
- **AND** 输出新的访问地址供本机用户使用

#### Scenario: Starting on an occupied requested port for remote access
- **WHEN** Web runtime 使用非回环地址启动且请求端口已被占用
- **THEN** 系统启动失败并明确说明请求端口不可用
- **AND** 系统不得静默切换到其他端口

## ADDED Requirements

### Requirement: Authenticated Web SQL Import Surface

系统 SHALL 在 Web runtime 的同源 HTTP 服务面提供受认证保护的 SQL 上传入口，使远程浏览器可通过与 Web UI 相同的 host/port 发起配置导入。

#### Scenario: Uploading SQL through the web runtime
- **WHEN** 已认证用户通过 Web UI 所在的同源地址上传 SQL 备份文件
- **THEN** 系统在同一个 host/port 上接收上传请求并执行导入流程
- **AND** 未认证请求被拒绝
