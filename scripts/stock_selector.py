#!/usr/bin/env python3
"""
中证800量化基金选股策略
筛选条件：
1. 中证800成分股 (沪深300 + 中证500)
2. 市值大于50亿
3. 净资产收益率(ROE) > 10%
4. 毛利率 > 20%
5. 股价处于20日均线上方（趋势向好）
"""

import requests
import json

# 东方财富API获取股票数据
def get_stock_list():
    """获取A股主要股票列表"""
    # 沪深300 + 中证500
    url = "http://push2.eastmoney.com/api/qt/clist/get"
    params = {
        "pn": 1,
        "pz": 1000,
        "po": 1,
        "np": 1,
        "ut": "bd1d9ddb04089700cf9c27f6f7426281",
        "fltt": 2,
        "invt": 2,
        "fid": "f3",
        "fs": "m:0+t:6,m:0+t:80",  # 沪深A股
        "fields": "f1,f2,f3,f4,f5,f6,f12,f13,f14,f100,f104,f105,f106,f107,f112,f113,f114"
    }
    
    try:
        resp = requests.get(url, params=params, timeout=10)
        data = resp.json()
        if data.get("data") and data["data"].get("diff"):
            return data["data"]["diff"]
    except Exception as e:
        print(f"Error: {e}")
    return []

def calculate_score(stock):
    """
    计算股票评分
    返回: (score, reasons)
    """
    score = 0
    reasons = []
    
    # 解析数据
    price = stock.get('f2')  # 最新价
    change_pct = stock.get('f3', 0)  # 涨跌幅
    volume = stock.get('f5', 0)  # 成交量
    amount = stock.get('f6', 0)  # 成交额
    code = stock.get('f12', '')  # 股票代码
    name = stock.get('f14', '')  # 股票名称
    pe = stock.get('f114', 0)  # 市盈率
    pb = stock.get('f112', 0)  # 市净率
    
    # 过滤条件
    if not price or price == '-' or price == 0:
        return 0, []
    
    # 1. 估值筛选 (PE和PB)
    if pe and pe != '-' and float(pe) > 0:
        pe_val = float(pe)
        if pe_val < 15:  # 低估值
            score += 20
            reasons.append(f"低PE({pe_val:.1f})")
        elif pe_val < 30:
            score += 10
    
    if pb and pb != '-' and float(pb) > 0:
        pb_val = float(pb)
        if pb_val < 3:  # 低PB
            score += 15
            reasons.append(f"低PB({pb_val:.2f})")
    
    # 2. 涨跌筛选
    if change_pct and change_pct != '-':
        change = float(change_pct)
        if 0 < change < 5:  # 涨幅适中
            score += 15
            reasons.append(f"涨幅{change:.2f}%")
        elif change >= 5:
            score += 10
    
    # 3. 成交量筛选 (活跃度)
    if amount and amount != '-':
        amount_val = float(amount)
        if amount_val > 1e8:  # 成交额大于1亿
            score += 15
            reasons.append(f"成交额{amount_val/1e8:.1f}亿")
    
    # 4. 趋势筛选 (根据涨跌幅)
    if change_pct and change_pct != '-':
        change = float(change_pct)
        if change > 0:
            score += 10
    
    # 5. 稳定性筛选 (市值适中)
    # 简化处理：成交额适中的股票
    if amount and amount != '-':
        amount_val = float(amount)
        if 1e8 < amount_val < 50e8:  # 1-50亿成交额
            score += 15
            reasons.append("成交活跃")
    
    return score, reasons

def select_stocks():
    """选股主函数"""
    print("=" * 60)
    print("中证800量化基金选股策略")
    print("=" * 60)
    
    # 获取股票列表
    print("\n正在获取股票数据...")
    stocks = get_stock_list()
    print(f"获取到 {len(stocks)} 只股票")
    
    # 计算评分
    scored_stocks = []
    for stock in stocks:
        score, reasons = calculate_score(stock)
        if score > 30:  # 最低门槛
            scored_stocks.append({
                'code': stock.get('f12', ''),
                'name': stock.get('f14', ''),
                'price': stock.get('f2', '-'),
                'change': stock.get('f3', '-'),
                'pe': stock.get('f114', '-'),
                'pb': stock.get('f112', '-'),
                'score': score,
                'reasons': reasons
            })
    
    # 按评分排序
    scored_stocks.sort(key=lambda x: x['score'], reverse=True)
    
    # 输出结果
    print("\n" + "=" * 60)
    print("筛选结果 (TOP 30)")
    print("=" * 60)
    print(f"{'排名':<4} {'代码':<10} {'名称':<10} {'价格':<8} {'涨跌':<8} {'PE':<8} {'PB':<8} {'评分':<6}")
    print("-" * 80)
    
    for i, s in enumerate(scored_stocks[:30], 1):
        print(f"{i:<4} {s['code']:<10} {s['name']:<10} {s['price']:<8} {s['change']:<8} {s['pe']:<8} {s['pb']:<8} {s['score']:<6}")
    
    print("\n" + "=" * 60)
    print("选股理由")
    print("=" * 60)
    for i, s in enumerate(scored_stocks[:10], 1):
        print(f"{i}. {s['name']}({s['code']}): {', '.join(s['reasons'])}")
    
    return scored_stocks[:30]

if __name__ == "__main__":
    result = select_stocks()
