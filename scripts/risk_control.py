#!/usr/bin/env python3
"""
仓位管理 & 风控脚本
用于实盘或模拟交易的风险控制
"""

import json
from datetime import datetime
from typing import Dict, List, Optional

class RiskManager:
    """风险管理器"""
    
    def __init__(self, config: dict = None):
        # 默认配置
        self.config = config or {
            "max_position_ratio": 0.8,       # 最大仓位 80%
            "max_single_position": 0.2,     # 单股最大 20%
            "stop_loss_ratio": 0.05,        # 5% 止损
            "take_profit_ratio": 0.15,      # 15% 止盈
            "max_drawdown_threshold": 0.15, # 15% 最大回撤
            "enabled": True,
        }
        
    def can_buy(self, current_position: float, new_amount: float, 
                total_capital: float, single_stock_value: float) -> tuple:
        """检查是否可以买入"""
        if not self.config["enabled"]:
            return True, "风控已关闭"
        
        # 总仓位检查
        total_ratio = (current_position + new_amount) / total_capital
        if total_ratio > self.config["max_position_ratio"]:
            return False, f"总仓位超限: {total_ratio*100:.1f}%"
        
        # 单股仓位检查
        single_ratio = single_stock_value / total_capital
        if single_ratio > self.config["max_single_position"]:
            return False, f"单股仓位超限: {single_ratio*100:.1f}%"
        
        return True, "可以买入"
    
    def check_stop_loss(self, entry_price: float, current_price: float) -> tuple:
        """检查是否触发止损"""
        loss = (entry_price - current_price) / entry_price
        if loss >= self.config["stop_loss_ratio"]:
            return True, f"触发止损: 亏损 {loss*100:.1f}%"
        return False, ""
    
    def check_take_profit(self, entry_price: float, current_price: float) -> tuple:
        """检查是否触发止盈"""
        profit = (current_price - entry_price) / entry_price
        if profit >= self.config["take_profit_ratio"]:
            return True, f"触发止盈: 盈利 {profit*100:.1f}%"
        return False, ""
    
    def calculate_position_size(self, capital: float, price: float, 
                               risk_per_trade: float = 0.02) -> int:
        """根据风险计算仓位"""
        max_loss = capital * risk_per_trade
        stop_loss_price = price * (1 - self.config["stop_loss_ratio"])
        price_risk = price - stop_loss_price
        
        if price_risk <= 0:
            return 0
            
        shares = int(max_loss / price_risk)
        max_shares = int(capital * self.config["max_single_position"] / price)
        
        return min(shares, max_shares)
    
    def generate_signal(self, position: float, entry_price: float, 
                       current_price: float, capital: float) -> dict:
        """生成交易信号"""
        signals = []
        
        # 止损检查
        sl_triggered, sl_msg = self.check_stop_loss(entry_price, current_price)
        if sl_triggered:
            signals.append({"action": "SELL", "reason": sl_msg})
        
        # 止盈检查
        tp_triggered, tp_msg = self.check_take_profit(entry_price, current_price)
        if tp_triggered:
            signals.append({"action": "SELL", "reason": tp_msg})
        
        if not signals:
            signals.append({"action": "HOLD", "reason": "未触发风控条件"})
        
        return {
            "timestamp": datetime.now().isoformat(),
            "entry_price": entry_price,
            "current_price": current_price,
            "pnl_ratio": (current_price - entry_price) / entry_price,
            "signals": signals
        }


class PositionManager:
    """仓位管理器"""
    
    def __init__(self, initial_capital: float = 100000):
        self.initial_capital = initial_capital
        self.cash = initial_capital
        self.positions: Dict[str, dict] = {}  # {symbol: {qty, avg_price, ...}}
        self.history: List[dict] = []
        
    def buy(self, symbol: str, price: float, qty: int, risk_manager: RiskManager) -> tuple:
        """买入"""
        cost = price * qty
        
        # 风控检查
        current_position = self.get_total_position_value()
        allowed, reason = risk_manager.can_buy(
            current_position, cost, self.cash + current_position, cost
        )
        
        if not allowed:
            return False, reason
        
        if cost > self.cash:
            return False, f"资金不足: 需要 {cost:.2f}, 可用 {self.cash:.2f}"
        
        # 执行买入
        if symbol in self.positions:
            old_qty = self.positions[symbol]["qty"]
            old_avg = self.positions[symbol]["avg_price"]
            new_qty = old_qty + qty
            new_avg = (old_avg * old_qty + price * qty) / new_qty
            self.positions[symbol] = {"qty": new_qty, "avg_price": new_avg}
        else:
            self.positions[symbol] = {"qty": qty, "avg_price": price}
        
        self.cash -= cost
        self.history.append({
            "time": datetime.now().isoformat(),
            "action": "BUY", "symbol": symbol, "price": price, "qty": qty
        })
        
        return True, f"买入 {symbol} x {qty} @ {price:.2f}"
    
    def sell(self, symbol: str, price: float, qty: int = None) -> tuple:
        """卖出"""
        if symbol not in self.positions:
            return False, f"没有 {symbol} 持仓"
        
        pos = self.positions[symbol]
        sell_qty = qty if qty else pos["qty"]
        
        if sell_qty > pos["qty"]:
            return False, f"持仓不足: {pos['qty']}"
        
        revenue = price * sell_qty
        self.cash += revenue
        
        if sell_qty == pos["qty"]:
            del self.positions[symbol]
        else:
            pos["qty"] -= sell_qty
        
        self.history.append({
            "time": datetime.now().isoformat(),
            "action": "SELL", "symbol": symbol, "price": price, "qty": sell_qty
        })
        
        return True, f"卖出 {symbol} x {sell_qty} @ {price:.2f}"
    
    def get_total_position_value(self) -> float:
        """获取持仓总市值"""
        total = 0
        for symbol, pos in self.positions.items():
            total += pos["qty"] * pos["avg_price"]
        return total
    
    def get_portfolio_value(self, current_prices: dict) -> float:
        """获取组合当前价值"""
        position_value = 0
        for symbol, pos in self.positions.items():
            if symbol in current_prices:
                position_value += pos["qty"] * current_prices[symbol]
            else:
                position_value += pos["qty"] * pos["avg_price"]
        return self.cash + position_value
    
    def get_status(self) -> dict:
        """获取账户状态"""
        return {
            "cash": self.cash,
            "position_value": self.get_total_position_value(),
            "positions": self.positions,
            "positions_count": len(self.positions)
        }
    
    def calculate_drawdown(self, equity_curve: List[float]) -> float:
        """计算最大回撤"""
        if not equity_curve:
            return 0
        
        peak = equity_curve[0]
        max_dd = 0
        
        for e in equity_curve:
            if e > peak:
                peak = e
            dd = (peak - e) / peak
            if dd > max_dd:
                max_dd = dd
        
        return max_dd


def demo():
    """演示"""
    print("=" * 50)
    print("仓位管理 & 风控演示")
    print("=" * 50)
    
    # 初始化
    risk_mgr = RiskManager()
    position_mgr = PositionManager(100000)
    
    print(f"\n初始资金: {position_mgr.initial_capital:.2f}")
    print(f"风控配置: {risk_mgr.config}")
    
    # 模拟买入
    print("\n--- 买入测试 ---")
    ok, msg = position_mgr.buy("600519", 1800, 100, risk_mgr)
    print(f"买入贵州茅台: {msg}")
    
    ok, msg = position_mgr.buy("000858", 150, 1000, risk_mgr)
    print(f"买入五粮液: {msg}")
    
    # 账户状态
    print("\n--- 账户状态 ---")
    status = position_mgr.get_status()
    print(f"现金: {status['cash']:.2f}")
    print(f"持仓市值: {status['position_value']:.2f}")
    print(f"持仓: {status['positions']}")
    
    # 模拟价格变化和风控检查
    print("\n--- 风控信号检测 ---")
    signals = risk_mgr.generate_signal(
        position=status_mgr.positions["600519"]["qty"] * 1800,
        entry_price=1800,
        current_price=1700,  # 下跌 5.5%
        capital=100000
    )
    print(f"贵州茅台信号: {signals}")
    
    print("\n完成!")


if __name__ == "__main__":
    demo()
