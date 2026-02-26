# QuantRust: API 设计 (初稿)

**版本**: 1.0
**作者**: Manus AI
**日期**: 2026-02-26
**关联文档**: [后端系统架构设计](./backend_architecture.md)

---

## 1. 概述

本文档定义了 QuantRust 平台前后端交互的核心 RESTful API 和 WebSocket 接口。API 设计遵循 OpenAPI 3.0 规范，旨在提供清晰、一致且易于使用的接口。

**基础 URL**: `https://api.quantrust.com/v1`

**认证**: 所有需要认证的接口都必须在 HTTP Header 中包含 `Authorization: Bearer <JWT_TOKEN>`。

## 2. RESTful API

### 2.1 认证 (Auth)

- **`POST /auth/register`**: 用户注册
    - **Request Body**: `{ "email": "...", "password": "..." }`
    - **Response (201)**: `{ "message": "Registration successful, please check your email for verification." }`

- **`POST /auth/login`**: 用户登录
    - **Request Body**: `{ "email": "...", "password": "..." }`
    - **Response (200)**: `{ "access_token": "...", "user": { "id": ..., "email": "...", "role": "..." } }`

- **`GET /auth/me`**: 获取当前用户信息
    - **Response (200)**: `{ "id": ..., "email": "...", "role": "..." }`

### 2.2 策略 (Strategies)

- **`GET /strategies`**: 获取用户的所有策略列表
    - **Response (200)**: `[ { "id": ..., "name": "...", "description": "...", "updated_at": "..." } ]`

- **`POST /strategies`**: 创建一个新策略
    - **Request Body**: `{ "name": "...", "description": "...", "language": "python" }`
    - **Response (201)**: `{ "id": ..., "name": "...", ... }`

- **`GET /strategies/{strategy_id}`**: 获取单个策略的详细信息（包括最新代码）
    - **Response (200)**: `{ "id": ..., "name": "...", "versions": [ ... ], "latest_code": "..." }`

- **`PUT /strategies/{strategy_id}`**: 更新策略基本信息（名称、描述）

- **`POST /strategies/{strategy_id}/versions`**: 保存一个新版本的策略代码
    - **Request Body**: `{ "code": "...", "commit_message": "..." }`
    - **Response (201)**: `{ "version_number": ..., "created_at": "..." }`

### 2.3 回测 (Backtests)

- **`POST /backtests`**: 启动一个新的回测任务
    - **Request Body**: `{ "strategy_version_id": ..., "parameters": { ... } }`
    - **Response (202)**: `{ "backtest_id": ..., "status": "PENDING" }` (后端异步执行)

- **`GET /backtests`**: 获取历史回测报告列表
    - **Response (200)**: `[ { "id": ..., "strategy_name": "...", "kpis": { ... }, "finished_at": "..." } ]`

- **`GET /backtests/{backtest_id}`**: 获取单个回测报告的详细信息
    - **Response (200)**: `{ "id": ..., "parameters": { ... }, "kpis": { ... }, "trades": [ ... ] }`

### 2.4 模拟交易 (Paper Trading)

- **`GET /paper/accounts`**: 获取用户的模拟交易账户列表

- **`POST /paper/accounts`**: 创建一个新的模拟交易账户

- **`GET /paper/accounts/{account_id}/positions`**: 获取模拟账户的当前持仓

- **`GET /paper/accounts/{account_id}/orders`**: 获取模拟账户的订单历史

- **`POST /paper/deploy`**: 部署策略到模拟盘
    - **Request Body**: `{ "strategy_version_id": ..., "account_id": ..., "initial_balance": ... }`
    - **Response (200)**: `{ "deployment_id": ..., "status": "RUNNING" }`

- **`POST /paper/deployments/{deployment_id}/stop`**: 停止一个正在运行的模拟策略

## 3. WebSocket API

WebSocket 用于实现数据的实时推送，减少客户端轮询。

**连接地址**: `wss://ws.quantrust.com/v1?token=<JWT_TOKEN>`

### 3.1 订阅 (Subscribe)

客户端通过发送 JSON 消息来订阅不同的数据流。

- **订阅实时行情**: `{ "action": "subscribe", "channel": "market_data", "symbol": "600519.SH" }`
- **订阅账户更新**: `{ "action": "subscribe", "channel": "account_updates", "account_id": "paper_123" }`
- **订阅策略日志**: `{ "action": "subscribe", "channel": "strategy_logs", "deployment_id": "deploy_abc" }`

### 3.2 推送 (Push)

服务器向客户端推送实时数据。

- **行情推送**: `{ "channel": "market_data", "data": { "symbol": "...", "price": ..., "timestamp": ... } }`
- **持仓更新**: `{ "channel": "account_updates", "data": { "type": "position_update", "positions": [ ... ] } }`
- **订单成交**: `{ "channel": "account_updates", "data": { "type": "fill", "order": { ... } } }`
- **日志推送**: `{ "channel": "strategy_logs", "data": { "level": "INFO", "message": "...", "timestamp": ... } }`
