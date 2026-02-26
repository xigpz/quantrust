# 数据流与实时监测方案设计

## 1. 核心数据流设计

本系统的核心是一个事件驱动的异步数据流，它确保了各个服务之间的低耦合和高吞吐量。数据流的核心是NATS消息队列，所有关键状态的变更都通过发布和订阅事件来完成。

![事件驱动数据流图](diagrams/dataflow.png)

### 1.1 数据流转过程

1.  **行情摄入**: `数据摄入服务`通过WebSocket或HTTP轮询的方式从外部数据源（如AkShare）获取实时A股行情。获取到的原始数据（如Tick、K线）被封装成统一的`MarketDataEvent`事件，然后发布到NATS的`market.data`主题上。同时，数据会被异步写入ClickHouse数据库进行持久化。

2.  **策略计算**: `策略引擎`订阅`market.data`主题。当接收到新的行情事件时，它会将其分发给所有正在运行的策略实例。策略逻辑根据新的市场数据进行计算，如果满足开平仓条件，则生成`SignalEvent`（交易信号事件），并发布到`trading.signal`主题。

3.  **风险控制**: `风控引擎`订阅`trading.signal`主题。在执行任何交易之前，风控引擎会进行一系列前置检查，包括但不限于：账户资金是否充足、持仓是否超过限制、交易频率是否过高等。检查通过后，`风控引擎`会将信号转换为具体的`OrderEvent`（订单事件），发布到`trading.order`主题。

4.  **订单执行**: `交易引擎`订阅`trading.order`主题。接收到订单事件后，它会根据订单类型（模拟或实盘）将其路由到相应的执行器。对于模拟交易，它会在内存中进行撮合；对于实盘交易，它会通过券商API下单。订单执行后，`交易引擎`会发布`FillEvent`（成交事件）到`trading.fill`主题。

5.  **状态更新与推送**: 多个服务会订阅`trading.fill`主题。例如，`策略引擎`会根据成交回报更新策略的内部状态（如持仓、均价）；`WebSocket推送服务`则会将成交信息、更新后的持仓信息等实时推送到前端UI，实现界面的实时更新。

### 1.2 关键事件定义

| 事件类型 | 结构体 (简化) | 主题 (Topic) | 描述 |
| :--- | :--- | :--- | :--- |
| `MarketDataEvent` | `{ symbol, timestamp, open, high, low, close, volume }` | `market.data.{symbol}` | 市场行情数据事件 |
| `SignalEvent` | `{ strategy_id, symbol, direction, price, quantity }` | `trading.signal` | 策略产生的交易信号 |
| `OrderEvent` | `{ order_id, symbol, direction, type, price, quantity }` | `trading.order` | 经过风控检查后的可执行订单 |
| `FillEvent` | `{ order_id, fill_id, symbol, price, quantity, timestamp }` | `trading.fill` | 订单成交回报 |

## 2. 实时监测方案

为了确保系统在生产环境中的稳定性、性能和可靠性，我们需要建立一套全面的实时监测方案，遵循可观测性（Observability）的三大支柱：**Metrics（指标）、Logging（日志）和Tracing（追踪）**。

我们将采用业界主流的开源监控套件：**Prometheus + Grafana + Loki + Tempo**。

![监控系统架构图](diagrams/monitoring_architecture.png)

### 2.1 监控技术栈

-   **Prometheus**: 用于收集和存储时间序列格式的**Metrics**。每个Rust服务都会内置一个Prometheus客户端，暴露一个`/metrics`端点，由Prometheus服务器定期抓取。
-   **Loki**: 用于聚合和查询**Logs**。所有服务都会输出结构化的JSON日志，由一个统一的代理（如Promtail）收集并发送到Loki服务器。
-   **Tempo**: 用于存储和查询分布式**Traces**。我们将使用`OpenTelemetry`标准，在服务中注入追踪代码，以跟踪一个请求在多个服务之间的完整调用链。
-   **Grafana**: 作为统一的可视化平台，用于创建仪表盘（Dashboard）来展示从Prometheus、Loki和Tempo中查询到的数据。
-   **Alertmanager**: 与Prometheus集成，根据预设的告警规则（如延迟过高、错误率上升）发送告警到Email、Slack或PagerDuty等。

### 2.2 关键监控指标 (Metrics)

| 服务 | 关键指标 | 描述 |
| :--- | :--- | :--- |
| **API Gateway** | `http_requests_total` (by path, method, status) | HTTP请求总数、错误率 |
| | `http_requests_duration_seconds` | HTTP请求延迟（P95, P99） |
| | `websocket_connections_active` | 当前活跃的WebSocket连接数 |
| **消息队列** | `nats_messages_in_flight` (by subject) | 在途消息数量（队列深度） |
| | `nats_processing_time_seconds` | 消息处理延迟 |
| **策略/回测引擎** | `strategy_pnl_total` (by strategy_id) | 各策略的实时盈亏 |
| | `backtest_duration_seconds` | 回测任务执行时长 |
| **交易引擎** | `trade_execution_latency_seconds` | 从收到订单到执行完成的延迟 |
| | `broker_api_errors_total` | 券商API调用错误计数 |

### 2.3 日志与追踪

-   **结构化日志**: 所有日志都将以JSON格式输出，并包含关键字段，如`timestamp`, `level`, `service_name`, `trace_id`, `span_id`。这使得在Loki中进行高效的筛选和聚合成为可能。例如，可以轻松查询到`trace_id`为`xyz`的所有相关日志。

-   **分布式追踪**: 当一个请求进入API网关时，`OpenTelemetry`会为其生成一个唯一的`trace_id`。这个`trace_id`会通过HTTP头或消息队列的元数据在后续的所有服务调用中传递。这使得我们可以在Grafana/Tempo中查看到一个请求的完整生命周期，快速定位性能瓶瓶颈或错误发生的环节。
