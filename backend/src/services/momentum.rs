use crate::models::*;
use anyhow::Result;

/// Momentum strategy based on RSI and MACD.
pub struct MomentumStrategy;

impl MomentumStrategy {
    pub fn new() -> Self {
        Self
    }

    /// Calculate RSI for the provided candle series.
    pub fn calculate_rsi(&self, candles: &[Candle], period: usize) -> Result<f64> {
        if candles.len() < period + 1 {
            return Err(anyhow::anyhow!("not enough candles"));
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..closes.len() {
            let change = closes[i] - closes[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return Ok(100.0);
        }

        let rs = avg_gain / avg_loss;
        Ok(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Calculate MACD tuple (DIF, DEA, histogram).
    pub fn calculate_macd(&self, candles: &[Candle]) -> Result<(f64, f64, f64)> {
        if candles.len() < 26 {
            return Err(anyhow::anyhow!("not enough candles for MACD"));
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let ema12 = self.ema(&closes, 12);
        let ema26 = self.ema(&closes, 26);
        let dif = ema12 - ema26;

        let mut ema_values: Vec<f64> = Vec::new();
        for i in 26..closes.len() {
            let slice = &closes[..i];
            ema_values.push(self.ema(slice, 9));
        }

        let dea = ema_values.last().copied().unwrap_or(dif);
        let hist = dif - dea;

        Ok((dif, dea, hist))
    }

    fn ema(&self, values: &[f64], period: usize) -> f64 {
        if values.len() < period {
            return 0.0;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = values.iter().take(period).sum::<f64>() / period as f64;

        for value in &values[period..] {
            ema = (*value - ema) * multiplier + ema;
        }

        ema
    }

    /// Build a simple momentum buy signal.
    pub fn buy_signal(&self, candles: &[Candle]) -> Result<MomentumSignal> {
        let rsi = self.calculate_rsi(candles, 14)?;
        let (dif, dea, hist) = self.calculate_macd(candles)?;

        let mut score = 0;
        let mut reasons = Vec::new();

        if rsi < 30.0 {
            score += 2;
            reasons.push(format!("RSI oversold: {:.1}", rsi));
        } else if rsi < 40.0 {
            score += 1;
            reasons.push(format!("RSI low: {:.1}", rsi));
        }

        if hist > 0.0 && dif > dea {
            score += 2;
            reasons.push("MACD golden cross".to_string());
        }

        if dif < 0.0 && dea < 0.0 && hist > 0.0 {
            score += 1;
            reasons.push("MACD rebound below zero".to_string());
        }

        if candles.len() >= 6 {
            let recent_vol: f64 = candles.iter().rev().take(3).map(|c| c.volume).sum();
            let prev_vol: f64 = candles.iter().rev().skip(3).take(3).map(|c| c.volume).sum();
            if recent_vol > prev_vol * 1.5 {
                score += 1;
                reasons.push("volume expansion".to_string());
            }
        }

        Ok(MomentumSignal {
            score,
            rsi,
            macd_dif: dif,
            macd_dea: dea,
            macd_hist: hist,
            reasons,
        })
    }

    /// Build a simple momentum sell signal.
    pub fn sell_signal(&self, candles: &[Candle]) -> Result<bool> {
        let rsi = self.calculate_rsi(candles, 14)?;
        let (dif, dea, hist) = self.calculate_macd(candles)?;

        let rsi_overbought = rsi > 70.0;
        let macd_dead_cross = hist < 0.0 && dif < dea;

        Ok(rsi_overbought || macd_dead_cross)
    }

    pub fn momentum_score(&self, candles: &[Candle]) -> Result<f64> {
        let signal = self.buy_signal(candles)?;
        Ok(signal.score as f64)
    }
}

impl Default for MomentumStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MomentumSignal {
    pub score: i32,
    pub rsi: f64,
    pub macd_dif: f64,
    pub macd_dea: f64,
    pub macd_hist: f64,
    pub reasons: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi() {
        let _strategy = MomentumStrategy::new();
        let _candles = vec![
            Candle {
                symbol: "TEST".to_string(),
                timestamp: "2024-01-01".to_string(),
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 104.0,
                volume: 1_000_000.0,
                turnover: 0.0,
            },
            Candle {
                symbol: "TEST".to_string(),
                timestamp: "2024-01-02".to_string(),
                open: 104.0,
                high: 108.0,
                low: 103.0,
                close: 107.0,
                volume: 1_200_000.0,
                turnover: 0.0,
            },
        ];
    }
}
