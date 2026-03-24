## MODIFIED Requirements

### Requirement: Config Import Export

系统 SHALL 支持导出当前配置，并支持通过桌面端本机文件路径、Web runtime 浏览器下载、桌面端本机文件路径导入与 Web runtime 浏览器上传四种自然交互路径完成 SQL 配置备份与恢复，以便在同机或跨设备场景中迁移或恢复环境。

#### Scenario: Exporting configuration to a desktop file path
- **WHEN** 桌面端用户导出当前配置
- **THEN** 系统生成可持久化的 SQL 导出文件并写入用户选择的本机路径
- **AND** 返回导出结果与文件位置

#### Scenario: Exporting configuration from the web runtime
- **WHEN** 已认证的 Web runtime 用户导出当前配置
- **THEN** 系统通过同一个 Web 服务入口返回 SQL 导出文件并触发浏览器下载
- **AND** 导出内容保持与桌面端 SQL 备份格式一致
- **AND** 流程不要求服务器本机文件路径或桌面保存对话框

#### Scenario: Importing configuration from a desktop file path
- **WHEN** 桌面端用户选择一个本机 SQL 备份文件并触发导入
- **THEN** 系统读取该文件并仅接受由 CC Switch 导出的 SQL 备份
- **AND** 系统在导入前创建安全备份并在导入后刷新后续配置读取状态

#### Scenario: Importing configuration from a web upload
- **WHEN** 已认证的 Web runtime 用户上传一个 SQL 备份文件并触发导入
- **THEN** 系统在同一个 Web 服务入口接收该文件而不要求服务器本机文件路径
- **AND** 系统仅接受由 CC Switch 导出的 SQL 备份
- **AND** 系统在导入前创建安全备份并在导入后刷新后续配置读取状态
