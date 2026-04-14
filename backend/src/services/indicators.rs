use crate::models::Candle;
use anyhow::{Context, Result};

/// Calculate Simple Moving Average
pub fn sma(candles: &[Candle], period: usize, field: &str) -> Result<Vec<f64>> {
    if candles.len() < period {
        return Err(anyhow::anyhow!("not enough candles: need {}, got {}", period, candles.len()));
    }

    let values: Vec<f64> = candles.iter().map(|c| get_field(c, field)).collect();
    let mut result = Vec::with_capacity(candles.len());

    for i in 0..candles.len() {
        if i < period - 1 {
            result.push(f64::NAN);
        } else {
            let sum: f64 = values[i.saturating_sub(period - 1)..=i].iter().sum();
            result.push(sum / period as f64);
        }
    }

    Ok(result)
}

/// Calculate Exponential Moving Average
pub fn ema(candles: &[Candle], period: usize, field: &str) -> Result<Vec<f64>> {
    if candles.len() < period {
        return Err(anyhow::anyhow!("not enough candles for EMA"));
    }

    let values: Vec<f64> = candles.iter().map(|c| get_field(c, field)).collect();
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(candles.len());

    // First EMA value is SMA
    let first_ema = values[..period].iter().sum::<f64>() / period as f64;
    result.push(f64::NAN);

    for i in period..values.len() {
        if i == period {
            // First EMA after SMA
            let ema = (values[i] - first_ema) * multiplier + first_ema;
            result.push(ema);
        } else {
            let prev_ema = result[i - 1];
            let ema = (values[i] - prev_ema) * multiplier + prev_ema;
            result.push(ema);
        }
    }

    // Backfill the leading NANs properly
    for i in 1..std::cmp::min(period, result.len()) {
        result[i] = f64::NAN;
    }

    Ok(result)
}

/// Calculate RSI (Relative Strength Index)
pub fn rsi(candles: &[Candle], period: usize, field: &str) -> Result<Vec<f64>> {
    if candles.len() < period + 1 {
        return Err(anyhow::anyhow!("not enough candles for RSI"));
    }

    let values: Vec<f64> = candles.iter().map(|c| get_field(c, field)).collect();
    let mut result = Vec::with_capacity(candles.len());
    result.push(f64::NAN);

    // Calculate price changes
    let mut gains = Vec::with_capacity(values.len() - 1);
    let mut losses = Vec::with_capacity(values.len() - 1);

    for i in 1..values.len() {
        let change = values[i] - values[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    // First RSI uses simple average
    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

    // Skip the first `period` values
    for _ in 0..period {
        result.push(f64::NAN);
    }

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

        if avg_loss == 0.0 {
            result.push(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            result.push(100.0 - (100.0 / (1.0 + rs)));
        }
    }

    Ok(result)
}

/// MACD result
#[derive(Debug, Clone)]
pub struct MacdResult {
    pub dif: Vec<f64>,
    pub dea: Vec<f64>,
    pub hist: Vec<f64>,
}

/// Calculate MACD (Moving Average Convergence Divergence)
pub fn macd(candles: &[Candle], fast_period: usize, slow_period: usize, signal_period: usize, field: &str) -> Result<MacdResult> {
    if candles.len() < slow_period + signal_period {
        return Err(anyhow::anyhow!("not enough candles for MACD"));
    }

    let ema_fast = ema(candles, fast_period, field)?;
    let ema_slow = ema(candles, slow_period, field)?;

    // DIF = EMA_fast - EMA_slow
    let mut dif = Vec::with_capacity(candles.len());
    for i in 0..candles.len() {
        if ema_fast[i].is_nan() || ema_slow[i].is_nan() {
            dif.push(f64::NAN);
        } else {
            dif.push(ema_fast[i] - ema_slow[i]);
        }
    }

    // DEA = EMA(DIF, signal_period)
    let dif_nonnan: Vec<f64> = dif.iter().filter(|v| !v.is_nan()).copied().collect();
    let signal_multiplier = 2.0 / (signal_period as f64 + 1.0);

    let mut dea = Vec::with_capacity(candles.len());
    let mut filled_count = 0;

    for i in 0..candles.len() {
        if dif[i].is_nan() {
            dea.push(f64::NAN);
        } else {
            filled_count += 1;
            if filled_count == 1 {
                dea.push(dif[i]); // First value
            } else if filled_count <= signal_period {
                // Use simple average for first signal_period values
                let sum: f64 = dif_nonnan[..filled_count].iter().sum();
                dea.push(sum / filled_count as f64);
            } else {
                // EMA
                let prev_dea = dea[i - 1];
                dea.push((dif[i] - prev_dea) * signal_multiplier + prev_dea);
            }
        }
    }

    // HIST = DIF - DEA
    let mut hist = Vec::with_capacity(candles.len());
    for i in 0..candles.len() {
        if dea[i].is_nan() || dif[i].is_nan() {
            hist.push(f64::NAN);
        } else {
            hist.push(dif[i] - dea[i]);
        }
    }

    Ok(MacdResult { dif, dea, hist })
}

/// Bollinger Bands result
#[derive(Debug, Clone)]
pub struct BollingerResult {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,
    pub lower: Vec<f64>,
}

/// Calculate Bollinger Bands
pub fn boll(candles: &[Candle], period: usize, std_dev: f64, field: &str) -> Result<BollingerResult> {
    let middle = sma(candles, period, field)?;
    let mut upper = Vec::with_capacity(candles.len());
    let mut lower = Vec::with_capacity(candles.len());

    let values: Vec<f64> = candles.iter().map(|c| get_field(c, field)).collect();

    for i in 0..candles.len() {
        if i < period - 1 || middle[i].is_nan() {
            upper.push(f64::NAN);
            lower.push(f64::NAN);
        } else {
            let slice = &values[i.saturating_sub(period - 1)..=i];
            let mean = middle[i];
            let variance = slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / period as f64;
            let std = variance.sqrt();
            upper.push(mean + std_dev * std);
            lower.push(mean - std_dev * std);
        }
    }

    Ok(BollingerResult { upper, middle, lower })
}

/// Get the latest value of an indicator
pub fn latest(values: &[f64]) -> f64 {
    values.iter().rev().find(|v| !v.is_nan()).copied().unwrap_or(0.0)
}

/// Get a field value from a candle
fn get_field(candle: &Candle, field: &str) -> f64 {
    match field {
        "open" => candle.open,
        "high" => candle.high,
        "low" => candle.low,
        "close" => candle.close,
        "volume" => candle.volume,
        _ => candle.close,
    }
}

/// Calculate Volume Ratio (current volume vs average of past N periods)
pub fn volume_ratio(candles: &[Candle], period: usize) -> Result<Vec<f64>> {
    if candles.len() < period + 1 {
        return Err(anyhow::anyhow!("not enough candles for volume ratio"));
    }

    let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();
    let mut result = Vec::with_capacity(candles.len());

    for i in 0..candles.len() {
        if i < period {
            result.push(f64::NAN);
        } else {
            let avg: f64 = volumes[i.saturating_sub(period)..i].iter().sum::<f64>() / period as f64;
            result.push(if avg > 0.0 { volumes[i] / avg } else { 1.0 });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Candle;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &close)| Candle {
                symbol: "TEST".to_string(),
                timestamp: format!("2024-01-{:02}", i + 1),
                open: close * 0.99,
                high: close * 1.01,
                low: close * 0.98,
                close,
                volume: 1_000_000.0,
                turnover: 0.0,
            })
            .collect()
    }

    #[test]
    fn test_sma() {
        let candles = make_candles(&[10.0, 11.0, 12.0, 13.0, 14.0]);
        let sma = sma(&candles, 3, "close").unwrap();
        assert!(sma[0].is_nan());
        assert!(sma[1].is_nan());
        assert!((sma[2] - 11.0).abs() < 0.001);
        assert!((sma[3] - 12.0).abs() < 0.001);
        assert!((sma[4] - 13.0).abs() < 0.001);
    }

    #[test]
    fn test_ema() {
        let candles = make_candles(&[10.0, 11.0, 12.0, 13.0, 14.0]);
        let ema = ema(&candles, 3, "close").unwrap();
        assert!(ema[2] > 0.0); // First valid EMA
    }

    #[test]
    fn test_rsi() {
        let candles = make_candles(&[10.0, 11.0, 12.0, 13.0, 14.0]);
        let rsi = rsi(&candles, 3, "close").unwrap();
        assert!(rsi.last().unwrap().is_finite());
    }
}
