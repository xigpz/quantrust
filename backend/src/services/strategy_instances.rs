use crate::models::strategy_config::{
    StartStrategyRequest, StrategyInstance, StrategySignal, StrategyStatus,
};
use crate::models::Candle;
use crate::services::strategy_executor::StrategyExecutor;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Global strategy instance manager
pub struct StrategyManager {
    instances: RwLock<HashMap<String, StrategyInstance>>,
    executor: StrategyExecutor,
}

impl StrategyManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            instances: RwLock::new(HashMap::new()),
            executor: StrategyExecutor::new(),
        })
    }

    /// Start a live strategy instance
    pub async fn start(&self, req: StartStrategyRequest) -> Result<String> {
        let instance_id = Uuid::new_v4().to_string();

        let instance = StrategyInstance {
            instance_id: instance_id.clone(),
            strategy_id: req.strategy_id,
            config: req.config,
            status: StrategyStatus::Running,
            positions: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);

        Ok(instance_id)
    }

    /// Stop a running strategy
    pub async fn stop(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.status = StrategyStatus::Stopped;
            instance.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    /// List all running strategies
    pub async fn list(&self) -> Vec<StrategyInstance> {
        let instances = self.instances.read().await;
        instances
            .values()
            .filter(|i| i.status == StrategyStatus::Running)
            .cloned()
            .collect()
    }

    /// Get instance by ID
    pub async fn get(&self, instance_id: &str) -> Option<StrategyInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id).cloned()
    }

    /// Generate signal for a running strategy with latest data
    pub async fn generate_signal(
        &self,
        instance_id: &str,
        candles: &[Candle],
    ) -> Result<StrategySignal> {
        let config = {
            let instances = self.instances.read().await;
            let instance = instances
                .get(instance_id)
                .context("Strategy instance not found")?;

            if instance.status != StrategyStatus::Running {
                anyhow::bail!("Strategy is not running");
            }
            instance.config.clone()
        };

        self.executor.generate_signal(&config, candles)
    }

    /// Update positions for a strategy instance
    pub async fn update_position(&self, instance_id: &str, symbol: &str, quantity: f64) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            if quantity > 0.0 {
                instance.positions.insert(symbol.to_string(), quantity);
            } else {
                instance.positions.remove(symbol);
            }
            instance.updated_at = chrono::Utc::now();
        }
        Ok(())
    }
}

impl Default for StrategyManager {
    fn default() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            executor: StrategyExecutor::new(),
        }
    }
}
