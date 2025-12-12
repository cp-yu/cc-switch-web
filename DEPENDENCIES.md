# CC-Switch 依赖管理指南

## 问题说明

在新环境或干净的电脑上首次编译 CC-Switch 时，Rust 需要从源码编译大量依赖，这可能需要 10-30 分钟。这是因为：

1. **复杂依赖**：项目包含 Tauri、Axum、rquickjs 等大型 Rust 库
2. **从源码编译**：Rust 默认从源码编译所有依赖以确保安全性
3. **无预编译二进制**：不像 npm，Rust 没有中央预编译二进制仓库

## 快速解决方案

### 方案一：使用预编译脚本（推荐）

```bash
# 1. 预编译依赖（首次使用，约 5-15 分钟）
./prebuild-deps.sh

# 2. 正常构建（现在会快很多）
./build-web-release.sh
```

### 方案二：直接构建

```bash
# 直接构建，耐心等待首次编译（约 10-30 分钟）
./build-web-release.sh
```

## 编译时间分解

| 组件 | 首次编译 | 后续编译 |
|------|----------|----------|
| Tauri 核心库 | 5-10 分钟 | < 1 分钟 |
| Web 服务器 (Axum) | 2-5 分钟 | < 30 秒 |
| 前端依赖 (Node.js) | 1-3 分钟 | < 30 秒 |
| 总计 | **10-30 分钟** | **2-5 分钟** |

## 配置优化

项目已包含以下优化配置：

### 1. Cargo 配置 (`.cargo/config.toml`)
- 并行编译 (`jobs = 8`)
- 增量编译 (`CARGO_INCREMENTAL = "1"`)
- 缓存目录固定 (`CARGO_TARGET_DIR`)
- 发布版本优化 (`opt-level = "s"`)

### 2. 构建脚本优化
- 保留编译缓存
- 智能检测新环境
- 增量编译支持

## 中国大陆用户优化

如在中国大陆，可启用镜像加速：

```bash
# 编辑 .cargo/config.toml
# 取消注释以下配置：
# [source.crates-io]
# registry = "https://mirrors.ustc.edu.cn/crates.io-index"
# replace-with = 'ustc'
```

## 缓存管理

```bash
# 查看缓存大小
du -sh ~/.cargo/ .cargo-cache/

# 清理缓存（如需要）
rm -rf ~/.cargo/registry/cache
rm -rf .cargo-cache/
```

## 故障排除

### 编译失败
```bash
# 清理重新开始
cargo clean
rm -rf .cargo-cache/
./prebuild-deps.sh
```

### 内存不足
```bash
# 减少并行编译
export CARGO_BUILD_JOBS=2
./build-web-release.sh
```

### 磁盘空间不足
```bash
# 清理 target 目录
cargo clean
```

## 自动化部署

对于 CI/CD 环境，建议：

```yaml
# GitHub Actions 示例
- name: Cache Rust dependencies
  uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      .cargo-cache/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

## 性能对比

| 场景 | 优化前 | 优化后 |
|------|--------|--------|
| 新电脑首次构建 | 30-60 分钟 | 10-20 分钟 |
| 重复构建 | 10-20 分钟 | 2-5 分钟 |
| 增量构建 | 5-10 分钟 | 1-2 分钟 |