use crate::models::strategy_config::{StrategyConfig, StrategySignal, StrategyStatus};
use crate::services::strategy_executor::StrategyExecutor;
use crate::services::virtual_trading;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// 策略交易管理器 - 在后台运行策略并执行虚拟交易
pub struct StrategyTradingManager {
    instances: RwLock<HashMap<String, TradingInstance>>,
    signal_txs: RwLock<HashMap<String, broadcast::Sender<StrategySignal>>>,
    executor: StrategyExecutor,
}

#[derive(Clone)]
pub struct TradingInstance {
    pub instance_id: String,
    pub strategy_id: String,
    pub strategy_name: String,
    pub config: StrategyConfig,
    pub status: StrategyStatus,
    pub positions: HashMap<String, f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl StrategyTradingManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            instances: RwLock::new(HashMap::new()),
            signal_txs: RwLock::new(HashMap::new()),
            executor: StrategyExecutor::new(),
        })
    }

    /// 启动策略交易
    pub async fn start(&self, config: StrategyConfig) -> Result<String> {
        let instance_id = Uuid::new_v4().to_string();
        let strategy_name = config.name.clone();

        let instance = TradingInstance {
            instance_id: instance_id.clone(),
            strategy_id: config.id.clone(),
            strategy_name,
            config,
            status: StrategyStatus::Running,
            positions: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        let (tx, _rx) = broadcast::channel(100);

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);

        let mut signal_txs = self.signal_txs.write().await;
        signal_txs.insert(instance_id.clone(), tx);

        Ok(instance_id)
    }

    /// 停止策略交易
    pub async fn stop(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.status = StrategyStatus::Stopped;
        }
        Ok(())
    }

    /// 获取所有运行中的实例
    pub async fn list_running(&self) -> Vec<TradingInstance> {
        let instances = self.instances.read().await;
        instances
            .values()
            .filter(|i| i.status == StrategyStatus::Running)
            .cloned()
            .collect()
    }

    /// 获取所有实例
    pub async fn list_all(&self) -> Vec<TradingInstance> {
        let instances = self.instances.read().await;
        instances.values().cloned().collect()
    }

    /// 获取实例配置
    pub async fn get_instance(&self, instance_id: &str) -> Option<TradingInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id).cloned()
    }

    /// 生成交易信号并执行
    pub async fn tick(&self, instance_id: &str, price: f64) -> Result<Option<StrategySignal>> {
        let config = {
            let instances = self.instances.read().await;
            let instance = instances.get(instance_id).context("Instance not found")?;
            if instance.status != StrategyStatus::Running {
                anyhow::bail!("Strategy not running");
            }
            instance.config.clone()
        };

        // 使用单根K线生成信号
        let candles = vec![crate::models::Candle {
            symbol: config.symbols.first().unwrap_or(&"000001".to_string()).clone(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0.0,
            turnover: 0.0,
        }];

        let signal = self.executor.generate_signal(&config, &candles)?;

        // 如果是买入或卖出信号，自动执行虚拟交易
        if signal.action != crate::models::strategy_config::SignalAction::Hold {
            let symbol = signal.symbol.as_str();
            let quantity = ((100000.0 * 0.95) / price).floor() * 100.0; // 固定金额买100股整数

            match signal.action {
                crate::models::strategy_config::SignalAction::Buy => {
                    let _ = virtual_trading::buy_stock(
                        symbol,
                        symbol, // name, would need lookup
                        price,
                        quantity,
                        &signal.reason,
                    );
                }
                crate::models::strategy_config::SignalAction::Sell => {
                    // 获取当前持仓
                    if let Ok(positions) = virtual_trading::get_positions() {
                        if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
                            let _ = virtual_trading::sell_stock(
                                symbol,
                                price,
                                pos.quantity,
                                &signal.reason,
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Some(signal))
    }

    /// 获取信号订阅通道
    pub async fn subscribe(&self, instance_id: &str) -> Option<broadcast::Receiver<StrategySignal>> {
        let signal_txs = self.signal_txs.read().await;
        signal_txs.get(instance_id).map(|tx| tx.subscribe())
    }

    /// 广播信号到所有订阅者
    pub async fn broadcast_signal(&self, instance_id: &str, signal: &StrategySignal) {
        let signal_txs = self.signal_txs.read().await;
        if let Some(tx) = signal_txs.get(instance_id) {
            let _ = tx.send(signal.clone());
        }
    }
}

impl Default for StrategyTradingManager {
    fn default() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            signal_txs: RwLock::new(HashMap::new()),
            executor: StrategyExecutor::new(),
        }
    }
}
