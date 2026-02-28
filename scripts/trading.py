#!/usr/bin/env python3
"""
券商实盘对接模块
支持富途、老虎、平安等券商API
"""

import requests
import json
import hashlib
import time
from abc import ABC, abstractmethod
from typing import List, Optional, Dict
from dataclasses import dataclass
from enum import Enum


class Direction(Enum):
    BUY = "BUY"
    SELL = "SELL"


class OrderType(Enum):
    MARKET = "MARKET"
    LIMIT = "LIMIT"


class OrderStatus(Enum):
    PENDING = "PENDING"
    PARTIAL = "PARTIAL"
    FILLED = "FILLED"
    CANCELLED = "CANCELLED"
    REJECTED = "REJECTED"


@dataclass
class Order:
    order_id: str
    symbol: str
    name: str
    direction: Direction
    order_type: OrderType
    price: float
    quantity: float
    filled_quantity: float
    status: OrderStatus
    create_time: str
    update_time: str


@dataclass
class Position:
    symbol: str
    name: str
    quantity: float
    avg_cost: float
    current_price: float
    market_value: float
    profit_loss: float
    profit_loss_pct: float


@dataclass
class Account:
    cash: float
    market_value: float
    total_assets: float
    positions: List[Position]


class BrokerAPI(ABC):
    """券商API抽象类"""
    
    @abstractmethod
    def buy(self, symbol: str, price: float, quantity: int) -> Order:
        pass
    
    @abstractmethod
    def sell(self, symbol: str, price: float, quantity: int) -> Order:
        pass
    
    @abstractmethod
    def cancel_order(self, order_id: str) -> bool:
        pass
    
    @abstractmethod
    def get_order(self, order_id: str) -> Order:
        pass
    
    @abstractmethod
    def get_positions(self) -> List[Position]:
        pass
    
    @abstractmethod
    def get_account(self) -> Account:
        pass


class FutuAPI(BrokerAPI):
    """富途证券API"""
    
    def __init__(self, config: Dict):
        self.config = config
        self.host = config.get("host", "openapi.futuin.com")
        self.app_id = config.get("app_id", "")
        self.license = config.get("license", "")
        self.session = requests.Session()
    
    def _request(self, method: str, endpoint: str, params: dict = None) -> dict:
        """发送请求"""
        url = f"https://{self.host}/{endpoint}"
        
        headers = {
            "Content-Type": "application/json",
            "app_id": self.app_id,
            "license": self.license,
        }
        
        if method == "GET":
            resp = self.session.get(url, params=params, headers=headers)
        else:
            resp = self.session.post(url, json=params, headers=headers)
        
        return resp.json()
    
    def buy(self, symbol: str, price: float, quantity: int) -> Order:
        """买入"""
        params = {
            "symbol": symbol,
            "price": price,
            "quantity": quantity,
            "direction": "BUY",
            "order_type": "LIMIT",
        }
        
        result = self._request("POST", "v1/order/place", params)
        
        return Order(
            order_id=result.get("order_id", ""),
            symbol=symbol,
            name="",
            direction=Direction.BUY,
            order_type=OrderType.LIMIT,
            price=price,
            quantity=quantity,
            filled_quantity=0,
            status=OrderStatus.PENDING,
            create_time=time.strftime("%Y-%m-%d %H:%M:%S"),
            update_time=time.strftime("%Y-%m-%d %H:%M:%S"),
        )
    
    def sell(self, symbol: str, price: float, quantity: int) -> Order:
        """卖出"""
        params = {
            "symbol": symbol,
            "price": price,
            "quantity": quantity,
            "direction": "SELL",
            "order_type": "LIMIT",
        }
        
        result = self._request("POST", "v1/order/place", params)
        
        return Order(
            order_id=result.get("order_id", ""),
            symbol=symbol,
            name="",
            direction=Direction.SELL,
            order_type=OrderType.LIMIT,
            price=price,
            quantity=quantity,
            filled_quantity=0,
            status=OrderStatus.PENDING,
            create_time=time.strftime("%Y-%m-%d %H:%M:%S"),
            update_time=time.strftime("%Y-%m-%d %H:%M:%S"),
        )
    
    def cancel_order(self, order_id: str) -> bool:
        """撤单"""
        result = self._request("POST", "v1/order/cancel", {"order_id": order_id})
        return result.get("code") == 0
    
    def get_order(self, order_id: str) -> Order:
        """查询订单"""
        result = self._request("GET", "v1/order/info", {"order_id": order_id})
        data = result.get("data", {})
        
        return Order(
            order_id=order_id,
            symbol=data.get("symbol", ""),
            name=data.get("name", ""),
            direction=Direction.BUY if data.get("direction") == "BUY" else Direction.SELL,
            order_type=OrderType.MARKET if data.get("order_type") == "MARKET" else OrderType.LIMIT,
            price=data.get("price", 0),
            quantity=data.get("quantity", 0),
            filled_quantity=data.get("filled_quantity", 0),
            status=OrderStatus.PENDING,
            create_time=data.get("create_time", ""),
            update_time=data.get("update_time", ""),
        )
    
    def get_positions(self) -> List[Position]:
        """查询持仓"""
        result = self._request("GET", "v1/position/list")
        positions = []
        
        for data in result.get("data", []):
            positions.append(Position(
                symbol=data.get("symbol", ""),
                name=data.get("name", ""),
                quantity=data.get("quantity", 0),
                avg_cost=data.get("avg_cost", 0),
                current_price=data.get("current_price", 0),
                market_value=data.get("market_value", 0),
                profit_loss=data.get("profit_loss", 0),
                profit_loss_pct=data.get("profit_loss_pct", 0),
            ))
        
        return positions
    
    def get_account(self) -> Account:
        """查询账户"""
        result = self._request("GET", "v1/account/info")
        data = result.get("data", {})
        
        return Account(
            cash=data.get("cash", 0),
            market_value=data.get("market_value", 0),
            total_assets=data.get("total_assets", 0),
            positions=self.get_positions(),
        )


class TradingEngine:
    """交易引擎 - 整合策略、风控、券商"""
    
    def __init__(self, broker: BrokerAPI, risk_config: Dict = None):
        self.broker = broker
        self.risk_config = risk_config or {
            "max_position": 0.8,
            "stop_loss": 0.05,
            "take_profit": 0.15,
        }
        self.orders = []
    
    def execute_buy(self, symbol: str, price: float, quantity: int) -> Order:
        """执行买入（带风控）"""
        # 检查仓位
        account = self.broker.get_account()
        position_ratio = account.market_value / account.total_assets if account.total_assets > 0 else 0
        
        if position_ratio > self.risk_config["max_position"]:
            raise ValueError(f"仓位超限: {position_ratio*100:.1f}%")
        
        # 执行买入
        order = self.broker.buy(symbol, price, quantity)
        self.orders.append(order)
        
        return order
    
    def execute_sell(self, symbol: str, price: float, quantity: int) -> Order:
        """执行卖出"""
        order = self.broker.sell(symbol, price, quantity)
        self.orders.append(order)
        
        return order
    
    def check_stop_loss(self, symbol: str) -> Optional[Order]:
        """检查止损"""
        positions = self.broker.get_positions()
        
        for pos in positions:
            if pos.symbol == symbol:
                loss_pct = -pos.profit_loss_pct / 100
                
                if loss_pct >= self.risk_config["stop_loss"]:
                    # 触发止损，卖出全部
                    return self.broker.sell(symbol, pos.current_price, int(pos.quantity))
        
        return None
    
    def sync_account(self) -> Account:
        """同步账户"""
        return self.broker.get_account()


def create_broker(broker_type: str, config: Dict) -> BrokerAPI:
    """工厂方法：创建券商实例"""
    brokers = {
        "futu": FutuAPI,
        # "tiger": TigerAPI,
        # "pingan": PingAnAPI,
    }
    
    if broker_type not in brokers:
        raise ValueError(f"不支持的券商: {broker_type}")
    
    return brokers[broker_type](config)


# 示例
if __name__ == "__main__":
    # 配置（需要替换为真实信息）
    config = {
        "host": "openapi.futuin.com",
        "app_id": "your_app_id",
        "license": "your_license",
    }
    
    # 创建券商接口
    broker = create_broker("futu", config)
    
    # 创建交易引擎
    engine = TradingEngine(broker)
    
    # 查询账户
    account = engine.sync_account()
    print(f"总资产: {account.total_assets:.2f}")
    print(f"持仓: {len(account.positions)}")
    
    # 示例买入
    # order = engine.execute_buy("00700", 350.0, 100)
    # print(f"下单成功: {order.order_id}")
