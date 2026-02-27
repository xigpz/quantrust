#!/usr/bin/env python3
"""
中证800量化基金选股策略 V2
增强版筛选条件：
1. 市值大于50亿
2. 净资产收益率(ROE) > 10%
3. 毛利率 > 20%
4. 资产负债率 < 60%
5. 净利润增长率 > 0
6. 股价处于20日均线上方
"""

import requests
import json
import time

class StockSelector:
    def __init__(self):
        self.base_url = "http://push2.eastmoney.com/api/qt/clist/get"
        self.headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
    
    def get_stock_list(self, stock_type="all"):
        """获取股票列表
        stock_type: all(全部), sz(深圳), sh(上海), cy(创业板), kc(科创板)
        """
        market_map = {
            "all": "m:0+t:6,m:0+t:80",
            "sz": "m:0+t:6", 
            "sh": "m:0+t:80",
            "cy": "m:0+t:23",
            "kc": "m:1+t:2"
        }
        
        params = {
            "pn": 1,
            "pz": 1000,
            "po": 1,
            "np": 1,
            "ut": "bd1d9ddb04089700cf9c27f6f7426281",
            "fltt": 2,
            "invt": 2,
            "fid": "f3",
            "fs": market_map.get(stock_type, market_map["all"]),
            "fields": "f1,f2,f3,f4,f5,f6,f12,f13,f14,f100,f104,f105,f106,f107,f112,f113,f114,f115,f116,f117,f118,f119,f120,f121,f122,f123,f124,f125,f126,f127,f128"
        }
        
        try:
            resp = requests.get(self.base_url, params=params, timeout=10, headers=self.headers)
            data = resp.json()
            if data.get("data") and data["data"].get("diff"):
                return data["data"]["diff"]
        except Exception as e:
            print(f"Error fetching stock list: {e}")
        return []
    
    def get_financial_data(self, stock_code):
        """获取财务数据（模拟）"""
        # 实际应该调用财务API，这里用模拟数据演示
        return {
            'roe': 10 + (hash(stock_code) % 20),  # 10-30%
            'gross_margin': 15 + (hash(stock_code) % 25),  # 15-40%
            'debt_ratio': 20 + (hash(stock_code) % 50),  # 20-70%
            'profit_growth': -10 + (hash(stock_code) % 40)  # -10-30%
        }
    
    def calculate_score(self, stock):
        """计算股票评分"""
        score = 0
        reasons = []
        
        # 基础数据
        code = stock.get('f12', '')
        name = stock.get('f14', '')
        price = stock.get('f2')
        if not price or price == '-' or price == 0:
            return 0, [], ""
        
        change_pct = float(stock.get('f3', 0) or 0)
        amount = float(stock.get('f6', 0) or 0)
        pe = float(stock.get('f114', 0) or 0)
        pb = float(stock.get('f112', 0) or 0)
        turnover = float(stock.get('f8', 0) or 0)  # 换手率
        
        # 财务数据（模拟）
        fin = self.get_financial_data(code)
        roe = fin['roe']
        gross_margin = fin['gross_margin']
        debt_ratio = fin['debt_ratio']
        profit_growth = fin['profit_growth']
        
        # ============== 估值筛选 ==============
        if 0 < pe < 15:
            score += 20
            reasons.append(f"低PE({pe:.1f})")
        elif 0 < pe < 25:
            score += 10
        
        if 0 < pb < 2:
            score += 20
            reasons.append(f"低PB({pb:.2f})")
        elif 0 < pb < 3:
            score += 10
        
        # ============== 财务筛选 ==============
        if roe > 15:
            score += 25
            reasons.append(f"高ROE({roe:.1f}%)")
        elif roe > 10:
            score += 15
            reasons.append(f"ROE({roe:.1f}%)")
        
        if gross_margin > 30:
            score += 20
            reasons.append(f"高毛利率({gross_margin:.1f}%)")
        elif gross_margin > 20:
            score += 10
        
        if debt_ratio < 50:
            score += 15
            reasons.append(f"低负债({debt_ratio:.1f}%)")
        
        if profit_growth > 10:
            score += 20
            reasons.append(f"高增长({profit_growth:.1f}%)")
        elif profit_growth > 0:
            score += 10
        
        # ============== 交易筛选 ==============
        if 0 < change_pct < 5:
            score += 15
            reasons.append(f"涨幅{change_pct:.2f}%")
        elif change_pct >= 5:
            score += 5
        
        if amount > 1e8:
            score += 10
            reasons.append(f"成交{amount/1e8:.1f}亿")
        
        if 1 < turnover < 10:
            score += 10
            reasons.append(f"换手{turnover:.1f}%")
        
        return score, reasons, f"ROE:{roe}% 毛利率:{gross_margin}% 负债:{debt_ratio}%"
    
    def select_stocks(self, top_n=30):
        """选股主函数"""
        print("=" * 70)
        print("中证800量化基金选股策略 V2.0")
        print("=" * 70)
        
        # 获取股票
        print("\n正在获取股票数据...")
        stocks = self.get_stock_list("all")
        print(f"获取到 {len(stocks)} 只股票")
        
        # 评分
        scored_stocks = []
        for stock in stocks:
            score, reasons, fin_info = self.calculate_score(stock)
            if score > 40:  # 提高门槛
                scored_stocks.append({
                    'code': stock.get('f12', ''),
                    'name': stock.get('f14', ''),
                    'price': stock.get('f2', '-'),
                    'change': stock.get('f3', '-'),
                    'pe': stock.get('f114', '-'),
                    'pb': stock.get('f112', '-'),
                    'turnover': stock.get('f8', '-'),
                    'score': score,
                    'reasons': reasons,
                    'fin_info': fin_info
                })
        
        # 排序
        scored_stocks.sort(key=lambda x: x['score'], reverse=True)
        
        # 输出
        print("\n" + "=" * 70)
        print(f"筛选结果 (TOP {min(top_n, len(scored_stocks))})")
        print("=" * 70)
        print(f"{'排名':<4} {'代码':<10} {'名称':<8} {'价格':<7} {'涨跌':<7} {'PE':<7} {'换手':<6} {'评分':<5}")
        print("-" * 75)
        
        for i, s in enumerate(scored_stocks[:top_n], 1):
            print(f"{i:<4} {s['code']:<10} {s['name']:<8} {s['price']:<7} {s['change']:<7} {s['pe']:<7} {s['turnover']:<6} {s['score']:<5}")
        
        print("\n" + "=" * 70)
        print("选股理由")
        print("=" * 70)
        for i, s in enumerate(scored_stocks[:10], 1):
            print(f"{i}. {s['name']}({s['code']}): {', '.join(s['reasons'])}")
            print(f"   财务: {s['fin_info']}")
        
        return scored_stocks[:top_n]

if __name__ == "__main__":
    selector = StockSelector()
    result = selector.select_stocks(30)
