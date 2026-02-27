#!/usr/bin/env python3
"""
行业板块热度分析
分析当前市场热点板块
"""

import requests
import json

def get_sector_data():
    """获取行业板块数据"""
    url = "http://push2.eastmoney.com/api/qt/clist/get"
    params = {
        "pn": 1,
        "pz": 100,
        "po": 1,
        "np": 1,
        "ut": "bd1d9ddb04089700cf9c27f6f7426281",
        "fltt": 2,
        "invt": 2,
        "fid": "f3",
        "fs": "m:90+t:2",  # 行业板块
        "fields": "f1,f2,f3,f4,f5,f6,f12,f13,f14,f100"
    }
    
    try:
        resp = requests.get(url, params=params, timeout=10)
        data = resp.json()
        if data.get("data") and data["data"].get("diff"):
            return data["data"]["diff"]
    except Exception as e:
        print(f"Error: {e}")
    return []

def analyze_sectors():
    """分析板块热度"""
    print("=" * 60)
    print("行业板块热度分析")
    print("=" * 60)
    
    sectors = get_sector_data()
    print(f"\n共获取 {len(sectors)} 个行业板块\n")
    
    # 按涨幅排序
    sorted_sectors = []
    for s in sectors:
        change = s.get('f3')
        if change and change != '-':
            try:
                change_val = float(change)
                sorted_sectors.append({
                    'code': s.get('f12', ''),
                    'name': s.get('f14', ''),
                    'change': change_val,
                    'amount': s.get('f6', 0)
                })
            except:
                pass
    
    sorted_sectors.sort(key=lambda x: x['change'], reverse=True)
    
    # 输出热点板块
    print("🔥 涨幅前10热门板块:")
    print("-" * 50)
    for i, s in enumerate(sorted_sectors[:10], 1):
        print(f"{i:2}. {s['name']:<12} {s['change']:>6.2f}%")
    
    print("\n💤 跌幅前10冷门板块:")
    print("-" * 50)
    for i, s in enumerate(sorted_sectors[-10:], 1):
        print(f"{i:2}. {s['name']:<12} {s['change']:>6.2f}%")
    
    return sorted_sectors

if __name__ == "__main__":
    analyze_sectors()
