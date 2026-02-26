# QuantRust - A股量化交易平台

<p align="center">
  <strong>🚀 高性能 Rust 后端 + React 前端的开源量化交易工具</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Backend-Rust-orange?style=flat-square" />
  <img src="https://img.shields.io/badge/Frontend-React%2019-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/Database-SQLite-green?style=flat-square" />
  <img src="https://img.shields.io/badge/Data-东方财富API-red?style=flat-square" />
  <img src="https://img.shields.io/badge/License-MIT-yellow?style=flat-square" />
</p>

---

## 功能特性

| 模块 | 功能 | 状态 |
|------|------|------|
| 📊 市场总览 | 大盘指数、涨跌统计、热门板块 | ✅ |
| 🔥 热点监测 | 热度评分、成交额排名、资金流入 | ✅ |
| ⚡ 异动检测 | 量能突增、急速拉升/下跌、涨跌停 | ✅ |
| 📈 板块行情 | 行业板块涨跌、领涨股、涨跌家数 | ✅ |
| 💰 资金流向 | 主力/超大/大/中/小单净流入 | ✅ |
| 🔴 涨停监控 | 涨停股列表、封单量、换手率 | ✅ |
| ⭐ 自选股 | 自定义股票关注列表 | ✅ |
| 🧪 策略回测 | 双均线策略、净值曲线、KPI分析 | ✅ |
| 🔌 WebSocket | 实时数据推送 | ✅ |
| 🎨 Demo模式 | 无后端时自动展示模拟数据 | ✅ |

## 快速开始

### 方式一：Docker 一键部署（推荐）

```bash
git clone https://github.com/xigpz/quantrust.git
cd quantrust
docker-compose up -d
```

访问 http://localhost:3000 即可使用。

### 方式二：手动编译

**启动后端：**

```bash
cd backend
cargo build --release
RUST_LOG=quantrust_server=info cargo run --release
# 后端运行在 http://localhost:8080
```

**启动前端（新终端）：**

```bash
cd frontend
pnpm install
pnpm dev
# 前端运行在 http://localhost:3000
```

> 详细安装指南请参阅 [本地运行文档](./docs/LOCAL_SETUP.md)

## 技术栈

### 后端 (Rust)

| 组件 | 技术 | 说明 |
|------|------|------|
| Web 框架 | Axum + Tokio | 异步高性能 HTTP 服务 |
| 数据库 | SQLite (rusqlite) | 零配置嵌入式数据库 |
| 数据源 | 东方财富 API | 免费实时 A 股行情 |
| 序列化 | Serde + serde_json | 高效 JSON 处理 |
| WebSocket | tokio-tungstenite | 实时数据推送 |

### 前端 (React)

| 组件 | 技术 | 说明 |
|------|------|------|
| 框架 | React 19 + TypeScript | 类型安全的 UI 开发 |
| 样式 | Tailwind CSS 4 | 原子化 CSS |
| 图表 | Recharts | 数据可视化 |
| UI 库 | shadcn/ui | 高质量组件库 |
| 路由 | Wouter | 轻量级路由 |

## 项目结构

```
quantrust/
├── backend/                 # Rust 后端
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── src/
│       ├── main.rs          # 入口 + 定时扫描
│       ├── api/             # HTTP API 路由
│       │   ├── mod.rs
│       │   └── routes.rs    # RESTful 端点
│       ├── data/            # 数据源适配器
│       │   ├── mod.rs
│       │   ├── eastmoney.rs # 东方财富 API
│       │   └── provider.rs  # 数据提供者抽象
│       ├── db/              # 数据库
│       │   └── mod.rs       # SQLite 初始化
│       ├── models/          # 数据模型
│       │   ├── stock.rs     # 股票行情
│       │   ├── alert.rs     # 告警
│       │   └── strategy.rs  # 策略
│       ├── services/        # 业务逻辑
│       │   ├── anomaly.rs   # 异动检测引擎
│       │   ├── hot_stocks.rs# 热点排名引擎
│       │   ├── scanner.rs   # 市场扫描器
│       │   └── backtest.rs  # 回测引擎
│       └── ws/              # WebSocket
│           └── mod.rs
├── frontend/                # React 前端
│   └── client/src/
│       ├── pages/
│       │   └── Dashboard.tsx
│       ├── components/
│       │   ├── MarketBar.tsx
│       │   ├── Sidebar.tsx
│       │   └── panels/      # 功能面板
│       └── hooks/
│           ├── useMarketData.ts
│           └── mockData.ts
├── docs/                    # 设计文档
│   ├── DESIGN_PROPOSAL.md   # 设计方案总纲
│   ├── product_requirements.md
│   ├── backend_architecture.md
│   ├── api_design.md
│   ├── database_schema.md
│   ├── development_roadmap.md
│   └── LOCAL_SETUP.md       # 本地运行指南
├── docker-compose.yml
└── README.md
```

## API 接口

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/health` | GET | 健康检查 |
| `/api/market/overview` | GET | 市场概览 |
| `/api/quotes` | GET | 全市场行情 |
| `/api/hot-stocks` | GET | 热点股票 |
| `/api/anomalies` | GET | 异动检测 |
| `/api/sectors` | GET | 板块行情 |
| `/api/money-flow` | GET | 资金流向 |
| `/api/limit-up` | GET | 涨停监控 |
| `/api/candles/:symbol` | GET | K线数据 |
| `/api/backtest` | POST | 运行回测 |
| `/api/search` | GET | 股票搜索 |
| `/ws` | WebSocket | 实时推送 |

## 设计文档

本项目包含完整的设计文档，覆盖产品、架构、数据库、API 等各个维度：

- [设计方案总纲](./docs/DESIGN_PROPOSAL.md)
- [产品需求文档](./docs/product_requirements.md)
- [用户故事与用例](./docs/user_stories_and_use_cases.md)
- [后端架构设计](./docs/backend_architecture.md)
- [API 设计](./docs/api_design.md)
- [数据库 Schema](./docs/database_schema.md)
- [数据流与监控方案](./docs/dataflow_monitoring.md)
- [开发路线图](./docs/development_roadmap.md)
- [本地运行指南](./docs/LOCAL_SETUP.md)

## 许可证

MIT License

---

> **免责声明**: 本项目仅供学习和研究使用，不构成任何投资建议。量化交易存在风险，请谨慎使用。
