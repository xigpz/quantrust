/**
 * Mock Data Generator - 模拟数据，用于前端独立演示
 * 当后端未连接时自动使用模拟数据
 */

import type {
  MarketOverview,
  StockQuote,
  HotStock,
  AnomalyStock,
  SectorInfo,
  MoneyFlow,
  IndexQuote,
  SectorIntradayFlowResponse,
} from './useMarketData';

// Seed random for consistency
function seededRandom(seed: number): () => number {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

const rand = seededRandom(42);

function randomFloat(min: number, max: number): number {
  return min + rand() * (max - min);
}

function randomInt(min: number, max: number): number {
  return Math.floor(randomFloat(min, max + 1));
}

function pick<T>(arr: T[]): T {
  return arr[randomInt(0, arr.length - 1)];
}

const stockNames = [
  '贵州茅台', '宁德时代', '隆基绿能', '比亚迪', '中国平安',
  '招商银行', '五粮液', '海天味业', '药明康德', '东方财富',
  '中信证券', '恒瑞医药', '美的集团', '格力电器', '万科A',
  '三一重工', '中国中免', '长江电力', '迈瑞医疗', '紫金矿业',
  '阳光电源', '通威股份', '中国神华', '海尔智家', '工商银行',
  '建设银行', '中国银行', '农业银行', '中国石油', '中国石化',
  '中国移动', '中国电信', '中国联通', '京东方A', '立讯精密',
  '歌尔股份', '韦尔股份', '北方华创', '中微公司', '卓胜微',
  '闻泰科技', '汇顶科技', '兆易创新', '圣邦股份', '长电科技',
  '中芯国际', '华虹半导体', '寒武纪', '科大讯飞', '海康威视',
];

const stockCodes = [
  '600519.SH', '300750.SZ', '601012.SH', '002594.SZ', '601318.SH',
  '600036.SH', '000858.SZ', '603288.SH', '603259.SH', '300059.SZ',
  '600030.SH', '600276.SH', '000333.SZ', '000651.SZ', '000002.SZ',
  '600031.SH', '601888.SH', '600900.SH', '300760.SZ', '601899.SH',
  '300274.SZ', '600438.SH', '601088.SH', '600690.SH', '601398.SH',
  '601939.SH', '601988.SH', '601288.SH', '601857.SH', '600028.SH',
  '600941.SH', '601728.SH', '600050.SH', '000725.SZ', '002475.SZ',
  '002241.SZ', '603501.SH', '002371.SZ', '688012.SH', '300782.SZ',
  '600745.SH', '603160.SH', '603986.SH', '300661.SZ', '600584.SH',
  '688981.SH', '688347.SH', '688256.SH', '002230.SZ', '002415.SZ',
];

const sectorNames = [
  '半导体', '新能源', '白酒', '银行', '医药生物',
  '人工智能', '光伏', '锂电池', '军工', '消费电子',
  '汽车', '房地产', '钢铁', '有色金属', '化工',
  '食品饮料', '传媒', '计算机', '通信', '电力设备',
];

const anomalyTypes = [
  'VolumeSpike', 'PriceSurge', 'PriceDrop', 'LimitUp', 'LimitDown',
  'LargeOrder', 'TurnoverSpike', 'GapUp', 'BreakResistance',
];

const anomalyDescs: Record<string, string[]> = {
  VolumeSpike: ['成交量较5日均量放大3.2倍', '成交量突增至近20日最高', '量能急剧放大'],
  PriceSurge: ['5分钟内涨幅超过3%', '快速拉升突破前高', '资金急速流入'],
  PriceDrop: ['5分钟内跌幅超过3%', '快速下跌跌破支撑', '大单集中抛售'],
  LimitUp: ['封板涨停，封单量大', '一字涨停，强势封板', '尾盘涨停'],
  LimitDown: ['封板跌停', '一字跌停', '尾盘跌停'],
  LargeOrder: ['出现超大买单500万+', '连续大单买入', '大单净流入异常'],
  TurnoverSpike: ['换手率突增至8.5%', '换手率创近期新高', '交投异常活跃'],
  GapUp: ['跳空高开3.5%', '缺口高开突破压力位', '高开高走'],
  BreakResistance: ['突破60日均线', '突破前期平台', '创近3个月新高'],
};

function generateStockQuote(i: number): StockQuote {
  const price = randomFloat(5, 2000);
  const changePct = randomFloat(-10, 10);
  const preClose = price / (1 + changePct / 100);
  return {
    symbol: stockCodes[i],
    name: stockNames[i],
    price: parseFloat(price.toFixed(2)),
    change: parseFloat((price - preClose).toFixed(2)),
    change_pct: parseFloat(changePct.toFixed(2)),
    open: parseFloat((preClose * (1 + randomFloat(-0.02, 0.03))).toFixed(2)),
    high: parseFloat((price * (1 + randomFloat(0, 0.03))).toFixed(2)),
    low: parseFloat((price * (1 - randomFloat(0, 0.03))).toFixed(2)),
    pre_close: parseFloat(preClose.toFixed(2)),
    volume: randomInt(100000, 50000000),
    turnover: randomFloat(1e7, 5e10),
    turnover_rate: randomFloat(0.5, 15),
    amplitude: randomFloat(1, 10),
    pe_ratio: randomFloat(-50, 200),
    total_market_cap: randomFloat(1e10, 3e12),
    circulating_market_cap: randomFloat(5e9, 2e12),
    timestamp: new Date().toISOString(),
  };
}

export function generateMockOverview(): MarketOverview {
  const shChange = randomFloat(-3, 3);
  const szChange = randomFloat(-3, 3);
  const cybChange = randomFloat(-4, 4);
  return {
    sh_index: {
      name: '上证指数',
      code: '000001.SH',
      price: parseFloat((3200 + randomFloat(-100, 100)).toFixed(2)),
      change: parseFloat((shChange * 32).toFixed(2)),
      change_pct: parseFloat(shChange.toFixed(2)),
      volume: randomInt(200e8, 500e8),
      turnover: randomFloat(3000e8, 8000e8),
    },
    sz_index: {
      name: '深证成指',
      code: '399001.SZ',
      price: parseFloat((10500 + randomFloat(-300, 300)).toFixed(2)),
      change: parseFloat((szChange * 105).toFixed(2)),
      change_pct: parseFloat(szChange.toFixed(2)),
      volume: randomInt(250e8, 600e8),
      turnover: randomFloat(4000e8, 9000e8),
    },
    cyb_index: {
      name: '创业板指',
      code: '399006.SZ',
      price: parseFloat((2100 + randomFloat(-80, 80)).toFixed(2)),
      change: parseFloat((cybChange * 21).toFixed(2)),
      change_pct: parseFloat(cybChange.toFixed(2)),
      volume: randomInt(100e8, 300e8),
      turnover: randomFloat(1500e8, 4000e8),
    },
    total_turnover: randomFloat(8000e8, 15000e8),
    up_count: randomInt(1500, 3500),
    down_count: randomInt(1000, 3000),
    flat_count: randomInt(100, 500),
    limit_up_count: randomInt(10, 80),
    limit_down_count: randomInt(5, 30),
    timestamp: new Date().toISOString(),
  };
}

export function generateMockQuotes(): StockQuote[] {
  return Array.from({ length: 50 }, (_, i) => generateStockQuote(i));
}

export function generateMockHotStocks(): HotStock[] {
  return Array.from({ length: 20 }, (_, i) => {
    const q = generateStockQuote(i);
    return {
      symbol: q.symbol,
      name: q.name,
      price: q.price,
      change_pct: q.change_pct,
      volume: q.volume,
      turnover: q.turnover,
      turnover_rate: q.turnover_rate,
      hot_score: parseFloat((100 - i * 4 + randomFloat(-5, 5)).toFixed(1)),
      hot_reason: pick(['成交额领先', '涨幅居前', '资金净流入', '板块联动', '消息刺激', '机构买入']),
      timestamp: new Date().toISOString(),
    };
  }).sort((a, b) => b.hot_score - a.hot_score);
}

export function generateMockAnomalies(): AnomalyStock[] {
  return Array.from({ length: 15 }, (_, i) => {
    const idx = randomInt(0, 49);
    const q = generateStockQuote(idx);
    const type = pick(anomalyTypes);
    const descs = anomalyDescs[type] || ['异动'];
    return {
      symbol: q.symbol,
      name: q.name,
      price: q.price,
      change_pct: type.includes('Up') || type === 'PriceSurge' || type === 'GapUp'
        ? Math.abs(q.change_pct)
        : type.includes('Down') || type === 'PriceDrop'
        ? -Math.abs(q.change_pct)
        : q.change_pct,
      anomaly_type: type,
      anomaly_score: parseFloat(randomFloat(30, 100).toFixed(1)),
      description: pick(descs),
      volume: q.volume,
      turnover_rate: q.turnover_rate,
      timestamp: new Date(Date.now() - randomInt(0, 3600000)).toISOString(),
    };
  }).sort((a, b) => b.anomaly_score - a.anomaly_score);
}

export function generateMockSectors(): SectorInfo[] {
  return sectorNames.map((name, i) => {
    const changePct = randomFloat(-5, 5);
    const mainNet = randomFloat(-12, 12);
    return {
      name,
      code: `BK${String(i + 1).padStart(4, '0')}`,
      change_pct: parseFloat(changePct.toFixed(2)),
      turnover: randomFloat(50e8, 500e8),
      leading_stock: pick(stockNames.slice(0, 20)),
      leading_stock_pct: parseFloat(randomFloat(changePct > 0 ? 3 : -3, changePct > 0 ? 10 : 3).toFixed(2)),
      stock_count: randomInt(30, 150),
      up_count: changePct > 0 ? randomInt(20, 100) : randomInt(5, 40),
      down_count: changePct < 0 ? randomInt(20, 100) : randomInt(5, 40),
      main_net_inflow: parseFloat(mainNet.toFixed(2)),
    };
  }).sort((a, b) => b.change_pct - a.change_pct);
}

const mockSectorFlowNames = [
  '新能源',
  '电力',
  '航天航空',
  '能源金属',
  '新能源车',
  '5G概念',
  'AI应用',
  '算力概念',
  '半导体',
  '消费电子',
  '银行',
  '医药',
];

export function generateMockSectorIntradayFlow(): SectorIntradayFlowResponse {
  const times = [
    '09:30',
    '10:00',
    '10:30',
    '11:00',
    '11:30',
    '13:00',
    '13:30',
    '14:00',
    '14:30',
    '15:00',
  ];
  const series = mockSectorFlowNames.map((name, i) => {
    const sign = i < 4 ? 1 : -1;
    let v = sign * randomFloat(2, 8);
    const points = times.map((t) => {
      v += randomFloat(-4, 4) * 0.35;
      return { t, v: parseFloat(v.toFixed(2)) };
    });
    const last = points[points.length - 1]?.v ?? 0;
    return {
      code: `BK${String(i + 100).padStart(4, '0')}`,
      name,
      points,
      last,
    };
  });
  const d = new Date();
  const trade_date = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  return {
    trade_date,
    updated_at: d.toISOString(),
    series,
  };
}

export function generateMockMoneyFlow(): MoneyFlow[] {
  return Array.from({ length: 20 }, (_, i) => {
    const q = generateStockQuote(i);
    const mainFlow = randomFloat(-5e8, 5e8);
    return {
      symbol: q.symbol,
      name: q.name,
      main_net_inflow: mainFlow,
      super_large_inflow: mainFlow * randomFloat(0.3, 0.6),
      large_inflow: mainFlow * randomFloat(0.2, 0.5),
      medium_inflow: randomFloat(-1e8, 1e8),
      small_inflow: randomFloat(-5e7, 5e7),
      timestamp: new Date().toISOString(),
    };
  }).sort((a, b) => b.main_net_inflow - a.main_net_inflow);
}

export function generateMockLimitUp(): StockQuote[] {
  return Array.from({ length: randomInt(15, 40) }, (_, i) => {
    const idx = randomInt(0, 49);
    const q = generateStockQuote(idx);
    q.change_pct = parseFloat(randomFloat(9.9, 20).toFixed(2));
    q.price = parseFloat((q.pre_close * (1 + q.change_pct / 100)).toFixed(2));
    q.change = parseFloat((q.price - q.pre_close).toFixed(2));
    return q;
  });
}
