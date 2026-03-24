## Context

当前 `cap.settings.backup` 的 Web 侧只补齐了“浏览器上传导入”，但导出仍沿用桌面端模型：前端调用 `save_file_dialog` 获取本机路径，再调用 `export_config_to_file(filePath)` 让后端写文件。这条链路默认假设“触发导出的 UI”和“真正写文件的进程”在同一台机器上。

这个假设在 Web runtime 中同样不成立。浏览器能做的是接收响应并触发下载，而不是替服务器选择磁盘路径。远程电脑即使能打开设置页，也不应该知道或控制服务器本机路径。既然导入已经拆成桌面路径与 Web 上传两条自然路径，导出也应该遵循同一个原则：桌面端继续写本机路径，Web runtime 直接下载 SQL 文件。

与此同时，`crates/server/src/main.rs` 已经把非回环 host 下的端口行为收紧为稳定入口契约。新增浏览器导出后，这个单端口契约不只承载 Web UI、JSON-RPC 与上传，还必须承载受认证保护的下载能力。

## Goals / Non-Goals

**Goals:**

- 让 Web runtime 支持浏览器直接下载 CC Switch 导出的 SQL 文件。
- 保持桌面端现有路径导入导出能力，避免为迁就 Web 把桌面端也重写一遍。
- 保持 Web runtime 的浏览器上传导入能力，并让上传/下载都复用同一个认证与单端口服务面。
- 继续维持远程访问场景下稳定可预期的 host/port 入口，不在非回环地址上静默漂移。
- 复用现有 SQL 导出格式、导入校验、安全备份与导入后同步语义，不引入第二套备份语义。

**Non-Goals:**

- 不把整个设置页传输层重写成通用文件流框架。
- 不改变 SQL 导出格式，也不放宽“只接受 CC Switch 导出的 SQL”校验。
- 不默认把 Web runtime 暴露到公网；是否绑定非回环地址仍由部署者显式决定。
- 不额外支持浏览器自定义保存路径；下载后的落盘位置由浏览器和用户代理自己决定。

## Decisions

### Decision: 前端把“桌面路径导入导出”和“Web 上传下载”拆成两条显式路径

设置页不再假装所有平台都能产出或消费 `filePath: string`。桌面端继续走文件对话框和本机路径读写；Web runtime 则分别使用浏览器文件选择上传和浏览器下载。

Rationale:

- 浏览器没有服务器本机路径，这不是兼容层能抹平的差异。
- 桌面与 Web 各自使用自然能力，状态管理、提示文案和测试边界都更清楚。

Alternatives considered:

- 继续沿用 `filePath: string`，让 Web 端伪造路径或依赖浏览器返回假路径。
  Rejected，因为这在语义上就是错的，导入导出最终都会退回到不可实现的服务器本地读写。
- 把所有平台都改成统一上传/下载字节流。
  Rejected，因为桌面端本来就有稳定的本地文件访问能力，强行统一只会增加复杂度。

### Decision: Web 导入导出都走同源 HTTP 文件入口，而不是把 SQL 内容塞进通用 JSON-RPC invoke

Web runtime 为导入提供受认证保护的 HTTP 上传入口，为导出提供受认证保护的 HTTP 下载入口，都挂在现有 Web 服务同一 host/port 下。导出响应直接返回 SQL 文件内容和下载头，导入继续复用现有数据库导入逻辑。

Rationale:

- 文件上传和下载本来就是 HTTP 的原生能力，和浏览器交互、响应头、文件名、大小限制天然匹配。
- 避免把大 SQL 文本或 base64 内容塞进 JSON-RPC/WebSocket 负载，减少前后端都要为“大文件命令”兜底的额外复杂度。
- 仍然复用同一个 Web 服务、同一个 Cookie 认证域，不引入第二个端口。

Alternatives considered:

- 新增 `export_config_content()` 之类的通用命令，再由前端自己拼 Blob 下载。
  Rejected，因为命令面会开始承担大文件传输语义，接口边界不干净，而且响应头和下载文件名也要额外补洞。
- 让 Web 端继续提示“只支持桌面导出”。
  Rejected，因为这会让远程 Web runtime 的备份闭环断掉，和已经补齐的浏览器导入能力不对称。

### Decision: 远程访问语义绑定到显式 host 配置，非回环地址下禁止自动换端口

当 Web runtime 使用非回环地址启动时，系统必须把 `CC_SWITCH_HOST` 与 `CC_SWITCH_PORT` 视为稳定入口契约；若请求端口不可用，则直接失败。保留回环地址下的自动选端口行为，用于本机开发和临时调试。

Rationale:

- 远程客户端只能依赖一个可事先告知的地址。端口自动漂移在单机开发里是便利，在多机访问里是故障源。
- 上传和下载都复用这个地址后，稳定入口的重要性只会更高，不会更低。

Alternatives considered:

- 保持所有场景下默认自动换端口。
  Rejected，因为远程用户会连错地址，上传和下载入口都会失效。
- 默认把 host 改成 `0.0.0.0`。
  Rejected，因为这会把安全边界默认放宽，超出本 change 必要范围。

## Risks / Trade-offs

- [Risk] 新增专用上传/下载入口会让 Web runtime 的 API 面出现两条“非 JSON-RPC”例外。 → Mitigation: 仅为 SQL 文件导入导出保留这组 HTTP 路径，其余业务命令继续走现有 invoke/WS 面。
- [Risk] 浏览器下载文件名和保存位置受用户代理控制。 → Mitigation: 服务端提供明确的 `Content-Disposition` 文件名建议，但不试图接管浏览器本地保存路径。
- [Risk] 远程访问下固定端口失败会改变一部分用户对“自动找可用端口”的预期。 → Mitigation: 仅在非回环 host 场景收紧；本机默认行为不变，并在日志和文档里明确失败原因。
- [Risk] SQL 文件上传/下载可能较大，更容易暴露请求体限制或超时问题。 → Mitigation: 上传继续设置明确大小上限；下载复用现有 SQL 导出逻辑，并提供直接文件响应而不是额外编码层。
- [Risk] 远程 SQL 导入导出本身都是高权限操作。 → Mitigation: 上传与下载入口都必须复用现有 Web 登录会话校验，未认证请求直接拒绝。

## Migration Plan

1. 现有桌面端导入导出流程保持不变，升级后无需迁移已有本地使用方式。
2. Web runtime 新增浏览器导出下载能力后，远程使用者不再需要回到桌面端创建 SQL 备份。
3. 需要远程上传或下载 SQL 的部署场景，显式设置非回环 `CC_SWITCH_HOST`、固定 `CC_SWITCH_PORT`，并建议同时启用 `~/.cc-switch/web-auth.json` 认证。
4. 若远程场景下端口被占用，服务启动直接失败，由部署者释放端口或改配新端口，不再静默改口。
5. 如需回滚到旧行为，只需恢复回环 host 或重新允许本机自动端口模式；SQL 文件格式与数据库内容不需要额外迁移。

## Open Questions

None.
