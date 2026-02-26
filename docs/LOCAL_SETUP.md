# QuantRust 本地运行指南

本文档详细说明如何在本地环境中搭建和运行 QuantRust A股量化交易平台。

---

## 目录

- [环境要求](#环境要求)
- [方式一：Docker 一键部署（推荐）](#方式一docker-一键部署推荐)
- [方式二：手动编译运行](#方式二手动编译运行)
- [验证运行](#验证运行)
- [配置说明](#配置说明)
- [常见问题](#常见问题)

---

## 环境要求

### Docker 部署（推荐）

| 依赖 | 最低版本 | 安装指南 |
|------|---------|---------|
| Docker | 20.10+ | [docs.docker.com](https://docs.docker.com/get-docker/) |
| Docker Compose | 2.0+ | Docker Desktop 自带 |

### 手动编译

| 依赖 | 最低版本 | 安装指南 |
|------|---------|---------|
| Rust | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| pnpm | 8+ | `npm install -g pnpm` |
| build-essential | — | `sudo apt install build-essential pkg-config libssl-dev` (Linux) |

---

## 方式一：Docker 一键部署（推荐）

### 1. 克隆仓库

```bash
git clone https://github.com/xigpz/quantrust.git
cd quantrust
```

### 2. 启动服务

```bash
docker-compose up -d
```

首次构建需要较长时间（Rust 编译约 5-10 分钟），后续启动会使用缓存。

### 3. 访问应用

- **前端界面**: http://localhost:3000
- **后端 API**: http://localhost:8080
- **健康检查**: http://localhost:8080/api/health

### 4. 查看日志

```bash
# 查看所有服务日志
docker-compose logs -f

# 仅查看后端日志
docker-compose logs -f backend

# 仅查看前端日志
docker-compose logs -f frontend
```

### 5. 停止服务

```bash
docker-compose down

# 如需清除数据
docker-compose down -v
```

---

## 方式二：手动编译运行

### 1. 克隆仓库

```bash
git clone https://github.com/xigpz/quantrust.git
cd quantrust
```

### 2. 启动后端

```bash
cd backend

# 编译（首次约 3-5 分钟）
cargo build --release

# 运行
RUST_LOG=quantrust_server=info cargo run --release
```

后端将在 `http://localhost:8080` 启动。

### 3. 启动前端（新终端窗口）

```bash
cd frontend

# 安装依赖
pnpm install

# 开发模式运行
pnpm dev
```

前端将在 `http://localhost:3000` 启动。

### 4. 生产构建前端

```bash
cd frontend
pnpm build

# 使用 Node.js 服务
pnpm start
```

---

## 验证运行

### 检查后端健康状态

```bash
curl http://localhost:8080/api/health
# 预期返回: {"success":true,"data":"ok","message":""}
```

### 检查市场数据

```bash
# 市场概览
curl http://localhost:8080/api/market/overview | python3 -m json.tool

# 热点股票
curl http://localhost:8080/api/hot-stocks | python3 -m json.tool

# 异动检测
curl http://localhost:8080/api/anomalies | python3 -m json.tool

# 板块行情
curl http://localhost:8080/api/sectors | python3 -m json.tool
```

### 测试 WebSocket

```bash
# 使用 websocat 工具（需安装）
websocat ws://localhost:8080/ws
```

---

## 配置说明

### 后端环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `RUST_LOG` | `quantrust_server=info` | 日志级别 |
| `DATABASE_URL` | `quantrust.db` | SQLite 数据库路径 |
| `SERVER_HOST` | `0.0.0.0` | 监听地址 |
| `SERVER_PORT` | `8080` | 监听端口 |

### 前端环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `VITE_API_BASE` | `（空）` | 后端 API 地址（留空则使用 Vite 代理） |

> **本地开发无需设置** `VITE_API_BASE`，Vite 已配置代理自动转发。
> 生产部署时才需要在 `.env.local` 中设置：
> ```env
> VITE_API_BASE=http://your-backend-host:8080
> ```

---

## 数据说明

### 数据源

QuantRust 使用**东方财富公开 API** 获取实时行情数据，包括：

- 沪深A股实时行情
- 大盘指数（上证、深证、创业板）
- 板块行情
- 资金流向
- 个股K线数据

### 交易时段

- **开盘时间**: 周一至周五 09:30 - 15:00
- **数据刷新**: 交易时段每 15 秒自动刷新
- **非交易时段**: 显示最近一个交易日的收盘数据

### Demo 模式

当后端未启动时，前端会自动切换到 **Demo 模式**，使用模拟数据展示界面功能。启动后端后会自动切换为实时数据。

---

## 常见问题

### Q: 编译 Rust 后端时报错 `linker 'cc' not found`

**A:** 安装 C 编译工具链：

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# macOS
xcode-select --install

# Fedora
sudo dnf install gcc openssl-devel
```

### Q: 前端页面空白，控制台报 CORS 错误

**A:** 确保后端已启动。前端 Vite 开发服务器已配置 API 代理（`/api/*` → `localhost:8080`），本地开发不会有 CORS 问题。

### Q: 非交易时段数据是否正常？

**A:** 正常。东方财富 API 在非交易时段返回最新收盘价数据，所有功能均可正常使用。前端底部状态栏会显示 "Demo 模式" 仅当后端未启动时才出现。

### Q: Docker 构建很慢

**A:** Rust 首次编译需要下载和编译大量依赖。建议：
1. 确保网络畅通（可配置 Rust 镜像源）
2. 分配足够的 Docker 内存（至少 4GB）
3. 后续构建会利用缓存，速度会快很多

### Q: 如何配置 Rust 镜像源加速编译？

**A:** 在 `~/.cargo/config.toml` 中添加：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
```

---

## 项目结构

```
quantrust/
├── backend/                # Rust 后端
│   ├── Cargo.toml         # Rust 依赖配置
│   ├── Dockerfile         # 后端 Docker 镜像
│   └── src/
│       ├── main.rs        # 入口文件
│       ├── api/           # HTTP API 路由
│       ├── data/          # 数据源适配器
│       ├── db/            # 数据库初始化
│       ├── models/        # 数据模型
│       ├── services/      # 业务逻辑
│       └── ws/            # WebSocket
├── frontend/              # React 前端
│   ├── package.json
│   ├── Dockerfile
│   └── client/src/
│       ├── pages/         # 页面组件
│       ├── components/    # UI 组件
│       └── hooks/         # 数据 hooks
├── docs/                  # 设计文档
├── docker-compose.yml     # Docker 编排
└── README.md
```
