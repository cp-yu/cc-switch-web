# Settings Sync Specification

## Purpose

`cap.settings.sync` 定义通过 WebDAV 与深链路进行配置同步和转移的行为。
它覆盖远端同步与深链配置导入两种配置迁移路径。


## Requirements

### Requirement: WebDAV Configuration Sync

系统 SHALL 支持测试 WebDAV 连接、保存同步设置并执行上传下载操作。

#### Scenario: Uploading configuration through WebDAV

- **WHEN** 用户在启用 WebDAV 同步后触发上传
- **THEN** 系统把当前配置推送到远端存储
- **AND** 返回同步结果与错误信息（如有）

### Requirement: Deep Link Import

系统 SHALL 支持解析深链并把其中的配置负载导入到统一管理面。

#### Scenario: Importing configuration from a deep link

- **WHEN** 用户打开受支持的配置深链
- **THEN** 系统解析深链中的导入请求
- **AND** 将可导入内容合并到本地配置流程中
