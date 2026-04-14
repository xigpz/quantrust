use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::models::stock::Candle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBacktestKpis {
    pub total_return: f64,
    pub annual_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub win_rate: f64,
    pub profit_loss_ratio: f64,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBacktestTrade {
    pub timestamp: String,
    pub symbol: String,
    pub direction: String,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonEquityPoint {
    pub timestamp: String,
    pub equity: f64,
    pub benchmark: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBacktestOutput {
    pub kpis: PythonBacktestKpis,
    pub trades: Vec<PythonBacktestTrade>,
    pub equity_curve: Vec<PythonEquityPoint>,
}

#[derive(Debug, Serialize)]
struct PythonRunnerInput<'a> {
    code: &'a str,
    symbol: &'a str,
    initial_capital: f64,
    commission_rate: f64,
    candles: &'a [Candle],
}

const PYTHON_RUNNER: &str = r#"
import json
import math
import sys

try:
    import pandas as pd
except Exception as e:
    print(json.dumps({"ok": False, "error": f"缺少 pandas 依赖: {e}"}))
    sys.exit(0)

def safe_float(v, default=0.0):
    try:
        return float(v)
    except Exception:
        return default

def make_kpis(initial_capital, equity_curve, trades):
    final_equity = equity_curve[-1]["equity"] if equity_curve else initial_capital
    total_return = ((final_equity - initial_capital) / initial_capital * 100.0) if initial_capital else 0.0

    peak = initial_capital
    max_drawdown = 0.0
    daily_returns = []
    prev_eq = None
    for p in equity_curve:
        eq = p["equity"]
        if eq > peak:
            peak = eq
        dd = ((peak - eq) / peak * 100.0) if peak > 0 else 0.0
        if dd > max_drawdown:
            max_drawdown = dd
        if prev_eq and prev_eq > 0:
            daily_returns.append((eq - prev_eq) / prev_eq)
        prev_eq = eq

    sell_trades = [t for t in trades if t["direction"] == "SELL"]
    wins = [t for t in sell_trades if t.get("pnl", 0.0) > 0]
    losses = [t for t in sell_trades if t.get("pnl", 0.0) <= 0]
    total_trades = len(sell_trades)
    winning_trades = len(wins)
    losing_trades = len(losses)
    win_rate = (winning_trades / total_trades * 100.0) if total_trades else 0.0

    avg_win = sum(t.get("pnl", 0.0) for t in wins) / winning_trades if winning_trades else 0.0
    avg_loss = sum(abs(t.get("pnl", 0.0)) for t in losses) / losing_trades if losing_trades else 1.0
    profit_loss_ratio = (avg_win / avg_loss) if avg_loss > 0 else 0.0

    trading_days = max(len(equity_curve), 1)
    annual_return = total_return / trading_days * 252.0

    if len(daily_returns) > 1:
        avg = sum(daily_returns) / len(daily_returns)
        variance = sum((r - avg) ** 2 for r in daily_returns) / (len(daily_returns) - 1)
        std = math.sqrt(max(variance, 1e-12))
    else:
        avg = 0.0
        std = 1.0
    sharpe_ratio = ((avg - 0.0001) / std * math.sqrt(252.0)) if std > 0 else 0.0

    downside = [r for r in daily_returns if r < 0]
    if len(downside) > 1:
        downside_var = sum((r ** 2) for r in downside) / (len(downside) - 1)
        downside_std = math.sqrt(max(downside_var, 1e-12))
    else:
        downside_std = 1.0
    sortino_ratio = ((avg - 0.0001) / downside_std * math.sqrt(252.0)) if downside_std > 0 else 0.0

    return {
        "total_return": total_return,
        "annual_return": annual_return,
        "max_drawdown": max_drawdown,
        "sharpe_ratio": sharpe_ratio,
        "sortino_ratio": sortino_ratio,
        "win_rate": win_rate,
        "profit_loss_ratio": profit_loss_ratio,
        "total_trades": total_trades,
        "winning_trades": winning_trades,
        "losing_trades": losing_trades,
    }

def main():
    raw = sys.stdin.read()
    payload = json.loads(raw)
    code = payload["code"]
    symbol = payload["symbol"]
    initial_capital = safe_float(payload.get("initial_capital", 100000.0), 100000.0)
    commission_rate = safe_float(payload.get("commission_rate", 0.0003), 0.0003)
    candles = payload.get("candles", [])
    if len(candles) < 2:
        print(json.dumps({"ok": False, "error": "K线数据不足"}))
        return

    closes = [safe_float(c.get("close", 0.0)) for c in candles]
    volumes = [safe_float(c.get("volume", 0.0)) for c in candles]

    ctx = type("Context", (), {})()
    ctx.symbols = [symbol]

    state = {
        "cash": initial_capital,
        "positions": {},
        "avg_cost": {},
        "trades": [],
        "equity_curve": [],
        "last_price": closes[0],
    }

    def current_equity():
        pos_val = 0.0
        for s, q in state["positions"].items():
            if q > 0:
                pos_val += q * state["last_price"]
        return state["cash"] + pos_val

    def execute_order(target_symbol, qty):
        qty = float(qty)
        if abs(qty) < 1e-6:
            return
        price = state["last_price"]
        ts = current_ts
        old_pos = state["positions"].get(target_symbol, 0.0)
        old_avg = state["avg_cost"].get(target_symbol, 0.0)

        if qty > 0:
            commission = qty * price * commission_rate
            cost = qty * price + commission
            if cost > state["cash"]:
                affordable = max((state["cash"] / (price * (1.0 + commission_rate))), 0.0)
                qty = math.floor(affordable)
                if qty <= 0:
                    return
                cost = qty * price + qty * price * commission_rate
                commission = qty * price * commission_rate
            state["cash"] -= cost
            new_pos = old_pos + qty
            state["positions"][target_symbol] = new_pos
            if new_pos > 0:
                state["avg_cost"][target_symbol] = ((old_pos * old_avg) + (qty * price) + commission) / new_pos
            state["trades"].append({
                "timestamp": ts,
                "symbol": target_symbol,
                "direction": "BUY",
                "price": price,
                "quantity": qty,
                "commission": commission,
                "pnl": 0.0,
            })
        else:
            sell_qty = min(abs(qty), old_pos)
            if sell_qty <= 0:
                return
            commission = sell_qty * price * commission_rate
            revenue = sell_qty * price - commission
            state["cash"] += revenue
            new_pos = old_pos - sell_qty
            state["positions"][target_symbol] = new_pos
            pnl = (price - old_avg) * sell_qty - commission
            if new_pos <= 0:
                state["avg_cost"][target_symbol] = 0.0
            state["trades"].append({
                "timestamp": ts,
                "symbol": target_symbol,
                "direction": "SELL",
                "price": price,
                "quantity": sell_qty,
                "commission": commission,
                "pnl": pnl,
            })

    def order(target_symbol, quantity):
        execute_order(target_symbol, quantity)

    def order_target_percent(target_symbol, pct):
        pct = max(min(float(pct), 1.0), 0.0)
        equity = current_equity()
        price = state["last_price"]
        current_qty = state["positions"].get(target_symbol, 0.0)
        current_value = current_qty * price
        target_value = equity * pct
        diff_value = target_value - current_value
        diff_qty = math.floor(abs(diff_value) / max(price, 1e-9))
        if diff_qty <= 0:
            return
        execute_order(target_symbol, diff_qty if diff_value > 0 else -diff_qty)

    def order_target(target_symbol, target_qty):
        target_qty = max(float(target_qty), 0.0)
        cur_qty = state["positions"].get(target_symbol, 0.0)
        diff = target_qty - cur_qty
        execute_order(target_symbol, diff)

    def get_index_stocks(_index_code):
        return [symbol]

    def get_all_securities():
        return pd.DataFrame({"code": [symbol]})

    def get_fundamentals(_symbol, *fields):
        data = {}
        for f in fields:
            if f == "pe_ratio":
                data[f] = [15.0]
            elif f == "pb_ratio":
                data[f] = [1.8]
            elif f == "roe":
                data[f] = [12.0]
            else:
                data[f] = [0.0]
        return pd.DataFrame(data)

    class DataProxy:
        def __init__(self, index):
            self.index = index

        def history(self, _symbol, fields, count, _freq):
            start = max(0, self.index - int(count) + 1)
            c_slice = closes[start:self.index + 1]
            v_slice = volumes[start:self.index + 1]
            if isinstance(fields, list):
                frame = {}
                for f in fields:
                    if f == "close":
                        frame["close"] = c_slice
                    elif f == "volume":
                        frame["volume"] = v_slice
                return pd.DataFrame(frame)
            if fields == "close":
                return pd.Series(c_slice)
            if fields == "volume":
                return pd.Series(v_slice)
            return pd.Series([])

        def current(self, _symbol, field):
            if field == "close":
                return closes[self.index]
            if field == "volume":
                return volumes[self.index]
            return 0.0

    safe_builtins = {
        "abs": abs, "all": all, "any": any, "bool": bool, "dict": dict, "enumerate": enumerate,
        "float": float, "int": int, "len": len, "list": list, "max": max, "min": min,
        "pow": pow, "range": range, "round": round, "set": set, "str": str, "sum": sum, "tuple": tuple, "zip": zip
    }

    g = {
        "__builtins__": safe_builtins,
        "pd": pd,
        "order": order,
        "order_target": order_target,
        "order_target_percent": order_target_percent,
        "get_index_stocks": get_index_stocks,
        "get_all_securities": get_all_securities,
        "get_fundamentals": get_fundamentals,
    }

    try:
        exec(code, g)
    except Exception as e:
        print(json.dumps({"ok": False, "error": f"策略代码执行失败: {e}"}))
        return

    init_fn = g.get("init")
    handle_bar_fn = g.get("handle_bar")
    if not callable(handle_bar_fn):
        print(json.dumps({"ok": False, "error": "策略缺少 handle_bar(context, data) 函数"}))
        return

    try:
        if callable(init_fn):
            init_fn(ctx)
    except Exception as e:
        print(json.dumps({"ok": False, "error": f"init 执行失败: {e}"}))
        return

    global current_ts
    current_ts = candles[0].get("timestamp", "")
    initial_price = max(closes[0], 1e-9)
    for i, c in enumerate(candles):
        state["last_price"] = max(closes[i], 1e-9)
        current_ts = c.get("timestamp", "")
        data = DataProxy(i)
        try:
            handle_bar_fn(ctx, data)
        except Exception as e:
            print(json.dumps({"ok": False, "error": f"handle_bar 执行失败({current_ts}): {e}"}))
            return
        equity = current_equity()
        benchmark = initial_capital * (state["last_price"] / initial_price)
        state["equity_curve"].append({
            "timestamp": current_ts,
            "equity": equity,
            "benchmark": benchmark,
        })

    kpis = make_kpis(initial_capital, state["equity_curve"], state["trades"])
    print(json.dumps({
        "ok": True,
        "data": {
            "kpis": kpis,
            "trades": state["trades"],
            "equity_curve": state["equity_curve"],
        }
    }))

if __name__ == "__main__":
    main()
"#;

#[derive(Debug, Deserialize)]
struct PythonRunnerResponse {
    ok: bool,
    data: Option<PythonBacktestOutput>,
    error: Option<String>,
}

pub async fn run_python_backtest(
    code: &str,
    symbol: &str,
    candles: &[Candle],
    initial_capital: f64,
    commission_rate: f64,
) -> Result<PythonBacktestOutput> {
    let payload = PythonRunnerInput {
        code,
        symbol,
        initial_capital,
        commission_rate,
        candles,
    };
    let input_json = serde_json::to_string(&payload).context("序列化 Python 输入失败")?;

    let mut cmd = Command::new("python");
    cmd.kill_on_drop(true)
        .arg("-c")
        .arg(PYTHON_RUNNER)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().context("启动 Python 进程失败，请确认已安装 python")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_json.as_bytes())
            .await
            .context("写入 Python 输入失败")?;
        stdin.shutdown().await.ok();
    }

    let out = timeout(Duration::from_secs(15), child.wait_with_output())
        .await
        .map_err(|_| anyhow!("Python 策略执行超时（>15s）"))?
        .context("等待 Python 执行失败")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(anyhow!("Python 执行失败: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: PythonRunnerResponse =
        serde_json::from_str(stdout.trim()).context("解析 Python 回测输出失败")?;
    if !parsed.ok {
        return Err(anyhow!(
            "{}",
            parsed.error.unwrap_or_else(|| "Python 回测失败".to_string())
        ));
    }
    parsed
        .data
        .ok_or_else(|| anyhow!("Python 回测未返回结果"))
}
