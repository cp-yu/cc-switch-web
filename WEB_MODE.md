# CC-Switch Web 模式使用指南

CC-Switch 现在支持 Web 模式，通过 WebSocket + JSON-RPC 2.0 协议访问后端服务。

## 架构

```
┌─────────────────┐    WebSocket     ┌──────────────────┐
│  Web Frontend   │ ←──────────────→ │  Rust Backend    │
│  (localhost:3001) │   JSON-RPC 2.0   │  (localhost:17666)│
└─────────────────┘                  └──────────────────┘
                                              │
                                              ↓
                                     ~/.cc-switch/cc-switch.db
```

## 快速开始

### 一键启动

```bash
./start-web.sh
```

服务启动后访问：**http://localhost:3001**

### 停止服务

```bash
./stop-web.sh
```

或在运行 `start-web.sh` 的终端按 `Ctrl+C`

## 手动启动（开发模式）

### 1. 启动后端

```bash
# 开发模式
cargo run --manifest-path crates/server/Cargo.toml

# 或生产模式
cargo run --release --manifest-path crates/server/Cargo.toml
```

后端将在 `http://localhost:17666` 启动

### 2. 启动前端

```bash
npx vite --mode web --port 3001
```

前端将在 `http://localhost:3001` 启动

## 使用 package.json 脚本

```bash
# 启动后端
npm run dev:server
# 或
pnpm dev:server

# 启动前端
npm run dev:web
# 或
pnpm dev:web
```

## 环境变量配置

Web 模式使用 `.env.web` 配置：

```env
# Web 模式 - 使用 WebSocket 连接后端服务器
VITE_CC_SWITCH_MODE=ws
VITE_CC_SWITCH_API_BASE=/api
```

## 密码保护

Web 模式支持可选的密码保护，防止未授权访问。

### 启用密码保护

1. **生成密码哈希**：

   ```bash
   # 使用 Python (推荐)
   python3 -c "import bcrypt; print(bcrypt.hashpw(b'your-password', bcrypt.gensalt(12)).decode())"

   # 或使用 htpasswd
   htpasswd -nbBC 12 "" "your-password" | tr -d ':\n' | sed 's/$2y/$2b/'
   ```

2. **创建配置文件** `~/.cc-switch/web-auth.json`：

   ```json
   {
     "password_hash": "$2b$12$xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
   }
   ```

3. **重启服务器** - 认证自动启用

### 禁用密码保护

删除或重命名配置文件即可：

```bash
rm ~/.cc-switch/web-auth.json
# 或
mv ~/.cc-switch/web-auth.json ~/.cc-switch/web-auth.json.disabled
```

### 修改密码

重新生成哈希并更新配置文件：

```bash
# 生成新哈希
python3 -c "import bcrypt; print(bcrypt.hashpw(b'new-password', bcrypt.gensalt(12)).decode())"

# 编辑配置文件
nano ~/.cc-switch/web-auth.json
```

### 安全特性

| 特性 | 说明 |
|------|------|
| 密码存储 | bcrypt 哈希 (cost=12)，不可逆 |
| 会话管理 | HttpOnly Cookie，7天有效期 |
| Cookie 安全 | SameSite=Strict，防止 CSRF |
| 会话清理 | 每小时自动清理过期会话 |
| 向后兼容 | 无配置文件时认证禁用 |

### 注意事项

- 密码只能通过编辑配置文件修改，不提供 UI 修改入口
- 配置文件中只存储哈希，原始密码不会被保存
- 关闭浏览器后 Cookie 仍然有效（7天内无需重新登录）
- 清除浏览器 Cookie 后需要重新登录

## 端口说明

- **前端**: `3001` - Web 界面
- **后端**: `17666` - JSON-RPC API 服务器
  - HTTP: `http://localhost:17666/api/invoke`
  - WebSocket: `ws://localhost:17666/api/ws`

## 数据共享

Web 模式与桌面模式共享相同的数据：

- **数据库**: `~/.cc-switch/cc-switch.db`
- **配置**: `~/.cc-switch/settings.json`
- **Skills**: `~/.claude/skills/`
- **MCP 配置**: `~/.claude.json`

## 支持的功能

Web 模式支持 70+ API 命令：

- ✅ Provider 管理（增删改查、切换）
- ✅ Settings 配置
- ✅ MCP 服务器管理
- ✅ Prompt 提示词管理
- ✅ Skill 技能管理
- ✅ DeepLink 导入
- ✅ 环境变量管理
- ✅ 密码保护（可选）
- ❌ 系统托盘（仅桌面端）
- ❌ 自动启动（仅桌面端）
- ❌ 文件对话框（仅桌面端）

## 故障排查

### 后端无法启动

```bash
# 检查端口占用
lsof -i:17666

# 查看日志
tail -f /tmp/cc-switch-backend.log
```

### 前端无法连接后端

1. 确认后端已启动：`curl http://localhost:17666/`
2. 检查代理配置：`vite.config.mts` 中的 proxy 设置
3. 查看浏览器控制台是否有错误

### 数据不同步

Web 模式和桌面模式使用相同的数据库文件，应该是同步的。如果出现不一致：

```bash
# 检查数据库文件
ls -la ~/.cc-switch/cc-switch.db

# 备份数据库
cp ~/.cc-switch/cc-switch.db ~/.cc-switch/cc-switch.db.backup
```

## 开发说明

### 目录结构

```
cc-switch/
├── crates/
│   ├── core/           # 共享核心库（桌面+Web）
│   └── server/         # Web 服务器
├── src/                # 前端代码
│   └── lib/
│       └── transport/  # 传输层抽象（Tauri/HTTP/WebSocket）
├── src-tauri/          # Tauri 桌面端
├── start-web.sh        # Web 启动脚本
└── stop-web.sh         # Web 停止脚本
```

### 添加新的 API 命令

1. 在 `crates/core/src/lib.rs` 添加函数
2. 在 `crates/server/src/api/dispatch.rs` 添加命令分发
3. 前端通过 `@/lib/transport` 自动适配

## 生产部署

### 构建优化版本

```bash
# 构建后端
cargo build --release --manifest-path crates/server/Cargo.toml

# 构建前端
npm run build:web
# 或
pnpm build:web
```

构建产物：
- 后端: `crates/server/target/release/cc-switch-server`
- 前端: `dist/`

### 部署到服务器

```bash
# 1. 复制后端二进制
scp crates/server/target/release/cc-switch-server user@server:/opt/cc-switch/

# 2. 复制前端静态文件
scp -r dist/* user@server:/var/www/cc-switch/

# 3. 配置 Nginx 反向代理
# 参考 docs/nginx.conf.example
```

## 技术栈

- **前端**: React + TypeScript + Vite
- **后端**: Rust + Axum + WebSocket
- **协议**: JSON-RPC 2.0
- **数据库**: SQLite (共享 `~/.cc-switch/cc-switch.db`)

## 许可证

MIT
