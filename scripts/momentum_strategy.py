#!/usr/bin/env python3
"""
动量策略选股脚本
基于RSI、MACD、成交量动量进行选股
"""

import requests
import json
from datetime import datetime, timedelta

# 东方财富API配置
BASE_URL = "http://push2.eastmoney.com"

def get_stock_kline(symbol: str, days: int = 60) -> list:
    """获取股票K线数据"""
    url = f"{BASE_URL}/api/qt/stock/kline/get"
    params = {
        "secid": f"1.{symbol}",  # 1=上海, 0=深圳
        "fields1": "f1,f2,f3,f4,f5,f6",
        "fields2": "f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61",
        "klt": "101",  # 日K
        "fqt": "1",    # 前复权
        "end": "20500101",
        "lmt": days,
    }
    try:
        r = requests.get(url, params=params, timeout=10)
        data = r.json()
        if data.get("data") and data["data"].get("klines"):
            klines = data["data"]["klines"]
            result = []
            for k in klines:
                fields = k.split(",")
                result.append({
                    "date": fields[0],
                    "open": float(fields[1]),
                    "close": float(fields[2]),
                    "high": float(fields[3]),
                    "low": float(fields[4]),
                    "volume": float(fields[5]),
                    "amount": float(fields[6]) if len(fields) > 6 else 0,
                })
            return result
    except Exception as e:
        print(f"获取{symbol}数据失败: {e}")
    return []

def calculate_rsi(prices: list, period: int = 14) -> float:
    """计算RSI指标"""
    if len(prices) < period + 1:
        return 50.0
    
    gains = []
    losses = []
    for i in range(1, len(prices)):
        change = prices[i] - prices[i-1]
        if change > 0:
            gains.append(change)
            losses.append(0)
        else:
            gains.append(0)
            losses.append(abs(change))
    
    avg_gain = sum(gains[-period:]) / period
    avg_loss = sum(losses[-period:]) / period
    
    if avg_loss == 0:
        return 100.0
    
    rs = avg_gain / avg_loss
    rsi = 100 - (100 / (1 + rs))
    return rsi

def calculate_ema(prices: list, period: int) -> float:
    """计算指数移动平均"""
    if len(prices) < period:
        return prices[-1] if prices else 0
    
    multiplier = 2 / (period + 1)
    ema = sum(prices[:period]) / period
    
    for price in prices[period:]:
        ema = (price - ema) * multiplier + ema
    
    return ema

def calculate_macd(prices: list) -> dict:
    """计算MACD指标"""
    if len(prices) < 26:
        return {"dif": 0, "dea": 0, "hist": 0}
    
    ema12 = calculate_ema(prices, 12)
    ema26 = calculate_ema(prices, 26)
    dif = ema12 - ema26
    
    # 计算DEA (9日EMA)
    macd_line = []
    for i in range(26, len(prices)):
        e = calculate_ema(prices[:i], 9)
        macd_line.append(dif - e)
    
    dea = macd_line[-1] if macd_line else dif
    hist = dif - dea
    
    return {"dif": dif, "dea": dea, "hist": hist}

def momentum_score(prices: list, volumes: list) -> dict:
    """计算动量评分"""
    if len(prices) < 30:
        return {"score": 0, "reason": "数据不足"}
    
    rsi = calculate_rsi(prices, 14)
    macd = calculate_macd(prices)
    
    score = 0
    reasons = []
    
    # RSI超卖反弹信号
    if rsi < 30:
        score += 2
        reasons.append(f"RSI超卖({rsi:.1})")
    elif rsi < 40:
        score += 1
        reasons.append(f"RSI低位({rsi:.1})")
    
    # MACD金叉
    if macd["hist"] > 0 and macd["dif"] > macd["dea"]:
        score += 2
        reasons.append("MACD金叉")
    
    # MACD零下金叉（反弹）
    if macd["dif"] < 0 and macd["dea"] < 0 and macd["hist"] > 0:
        score += 1
        reasons.append("MACD零下金叉")
    
    # 成交量放大
    if len(volumes) >= 10:
        recent_vol = sum(volumes[-3:]) / 3
        prev_vol = sum(volumes[-10:-7]) / 3
        if recent_vol > prev_vol * 1.5:
            score += 1
            reasons.append("成交量放大")
    
    # 价格上涨趋势
    if prices[-1] > prices[-5] > prices[-10]:
        score += 1
        reasons.append("上涨趋势")
    
    return {"score": score, "rsi": rsi, "macd": macd, "reasons": reasons}

def scan_momentum_stocks(symbols: list) -> list:
    """扫描动量股票"""
    results = []
    
    for symbol in symbols:
        print(f"分析 {symbol}...")
        klines = get_stock_kline(symbol, 60)
        
        if len(klines) < 30:
            continue
        
        prices = [k["close"] for k in klines]
        volumes = [k["volume"] for k in klines]
        
        momentum = momentum_score(prices, volumes)
        
        if momentum["score"] >= 2:
            results.append({
                "symbol": symbol,
                "score": momentum["score"],
                "rsi": momentum.get("rsi", 0),
                "macd": momentum.get("macd", {}),
                "reasons": momentum.get("reasons", []),
                "price": prices[-1],
                "change": (prices[-1] - prices[-2]) / prices[-2] * 100,
            })
    
    # 按评分排序
    results.sort(key=lambda x: x["score"], reverse=True)
    return results

def main():
    # 测试股票列表（实际使用时应从热门股票或自选股获取）
    test_symbols = [
        "600519",  # 贵州茅台
        "000858",  # 五粮液
        "601318",  # 平安银行
        "600036",  # 招商银行
        "000333",  # 美的集团
    ]
    
    print("=" * 50)
    print("动量策略选股扫描")
    print("=" * 50)
    
    results = scan_momentum_stocks(test_symbols)
    
    print("\n符合条件的股票：")
    print("-" * 50)
    for r in results:
        print(f"{r['symbol']} | 评分: {r['score']} | 现价: {r['price']:.2f} | 涨幅: {r['change']:+.2f}%")
        print(f"  原因: {', '.join(r['reasons'])}")
        print(f"  RSI: {r['rsi']:.1}, MACD: dif={r['macd']['dif']:.2f}, dea={r['macd']['dea']:.2f}")
        print()

if __name__ == "__main__":
    main()
