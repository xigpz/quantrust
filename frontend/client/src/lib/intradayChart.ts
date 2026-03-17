export interface IntradayPoint {
  timestamp: string;
  price: number;
  avg_price: number;
  volume: number;
  turnover: number;
  change_pct?: number | null;
  change?: number | null;
}

export interface IntradaySeries {
  symbol: string;
  name: string;
  range: string;
  pre_close: number;
  points: IntradayPoint[];
}

export interface IntradayChartModel {
  categories: string[];
  timestamps: string[];
  prices: number[];
  avgPrices: number[];
  volumeBars: Array<{ value: number; itemStyle: { color: string } }>;
  turnoverBars: number[];
  priceMin: number;
  priceMax: number;
  preClose: number;
}

const UP_COLOR = '#d84a4a';
const DOWN_COLOR = '#2ba272';
const PRICE_COLOR = '#3b82f6';
const AVG_COLOR = '#f59e0b';

function formatIntradayLabel(timestamp: string, range: string) {
  if (!timestamp.includes(' ')) {
    return timestamp.slice(5, 10);
  }

  if (range === '5d') {
    return timestamp.slice(5, 16);
  }

  return timestamp.slice(11, 16);
}

function toIntervalValue(current: number, previous: number | null, reset: boolean) {
  if (previous === null || reset || current < previous) {
    return current;
  }

  return current - previous;
}

export function buildIntradayChartModel(series: IntradaySeries): IntradayChartModel {
  let lastVolume: number | null = null;
  let lastTurnover: number | null = null;
  let lastDate: string | null = null;

  const categories = series.points.map((point) => formatIntradayLabel(point.timestamp, series.range));
  const timestamps = series.points.map((point) => point.timestamp);
  const prices = series.points.map((point) => point.price);
  const avgPrices = series.points.map((point) => point.avg_price);
  const volumeBars = series.points.map((point, index) => {
    const datePart = point.timestamp.slice(0, 10);
    const reset = lastDate !== null && lastDate !== datePart;
    const value = toIntervalValue(point.volume, lastVolume, reset);
    const previousPrice = index > 0 ? series.points[index - 1].price : point.price;

    lastVolume = point.volume;
    lastDate = datePart;

    return {
      value,
      itemStyle: {
        color: point.price >= previousPrice ? UP_COLOR : DOWN_COLOR,
      },
    };
  });
  lastDate = null;
  const turnoverBars = series.points.map((point) => {
    const datePart = point.timestamp.slice(0, 10);
    const reset = lastDate !== null && lastDate !== datePart;
    const value = toIntervalValue(point.turnover, lastTurnover, reset);

    lastTurnover = point.turnover;
    lastDate = datePart;

    return value;
  });

  const minPrice = prices.length ? Math.min(...prices, ...avgPrices) : series.pre_close;
  const maxPrice = prices.length ? Math.max(...prices, ...avgPrices) : series.pre_close;
  const radius = Math.max(Math.abs(maxPrice - series.pre_close), Math.abs(series.pre_close - minPrice), series.pre_close * 0.002);

  return {
    categories,
    timestamps,
    prices,
    avgPrices,
    volumeBars,
    turnoverBars,
    priceMin: Number((series.pre_close - radius).toFixed(2)),
    priceMax: Number((series.pre_close + radius).toFixed(2)),
    preClose: series.pre_close,
  };
}

export function buildIntradayOption(model: IntradayChartModel) {
  return {
    animation: false,
    axisPointer: {
      link: [{ xAxisIndex: [0, 1] }],
      label: { backgroundColor: '#475569' },
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'cross' },
      backgroundColor: 'rgba(15, 23, 42, 0.94)',
      borderWidth: 0,
      textStyle: { color: '#e2e8f0', fontSize: 11 },
      formatter: (params: Array<{ dataIndex: number }>) => {
        const index = params[0]?.dataIndex ?? 0;
        const price = model.prices[index] ?? 0;
        const avgPrice = model.avgPrices[index] ?? 0;
        const volume = model.volumeBars[index]?.value ?? 0;
        const turnover = model.turnoverBars[index] ?? 0;
        const change = price - model.preClose;
        const changePct = model.preClose > 0 ? (change / model.preClose) * 100 : 0;

        return [
          `<div>${model.timestamps[index] ?? ''}</div>`,
          `<div>价格 ${price.toFixed(2)}</div>`,
          `<div>均价 ${avgPrice.toFixed(2)}</div>`,
          `<div>涨跌 ${change >= 0 ? '+' : ''}${change.toFixed(2)}</div>`,
          `<div>涨幅 ${changePct >= 0 ? '+' : ''}${changePct.toFixed(2)}%</div>`,
          `<div>成交量 ${volume.toFixed(0)}</div>`,
          `<div>成交额 ${turnover.toFixed(0)}</div>`,
        ].join('');
      },
    },
    grid: [
      { left: 56, right: 56, top: 12, height: 312 },
      { left: 56, right: 56, top: 340, height: 112 },
    ],
    xAxis: [
      {
        type: 'category',
        data: model.categories,
        boundaryGap: false,
        axisLine: { lineStyle: { color: '#cbd5e1' } },
        axisTick: { show: false },
        axisLabel: { show: false },
        splitLine: { show: false },
      },
      {
        type: 'category',
        gridIndex: 1,
        data: model.categories,
        boundaryGap: true,
        axisLine: { lineStyle: { color: '#cbd5e1' } },
        axisTick: { show: false },
        axisLabel: {
          color: '#94a3b8',
          fontSize: 10,
          hideOverlap: true,
        },
        splitLine: { show: false },
      },
    ],
    yAxis: [
      {
        position: 'left',
        min: model.priceMin,
        max: model.priceMax,
        scale: true,
        splitNumber: 5,
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: '#64748b',
          fontSize: 10,
          formatter: (value: number) => value.toFixed(2),
        },
        splitLine: { lineStyle: { color: 'rgba(148, 163, 184, 0.18)' } },
      },
      {
        position: 'right',
        min: model.priceMin,
        max: model.priceMax,
        scale: true,
        splitNumber: 5,
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: '#64748b',
          fontSize: 10,
          formatter: (value: number) => {
            const pct = model.preClose > 0 ? ((value - model.preClose) / model.preClose) * 100 : 0;
            return `${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%`;
          },
        },
        splitLine: { show: false },
      },
      {
        gridIndex: 1,
        position: 'right',
        min: 0,
        max: Math.max(...model.volumeBars.map((item) => item.value), 1),
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: '#94a3b8',
          fontSize: 9,
        },
        splitLine: { show: false },
      },
    ],
    series: [
      {
        name: '价格',
        type: 'line',
        data: model.prices,
        showSymbol: false,
        smooth: false,
        lineStyle: { width: 1.5, color: PRICE_COLOR },
        markLine: {
          silent: true,
          symbol: 'none',
          lineStyle: { color: '#94a3b8', type: 'dashed', opacity: 0.6 },
          data: [{ yAxis: model.preClose }],
        },
      },
      {
        name: '均价',
        type: 'line',
        data: model.avgPrices,
        showSymbol: false,
        smooth: false,
        lineStyle: { width: 1.2, color: AVG_COLOR },
      },
      {
        name: '成交量',
        type: 'bar',
        xAxisIndex: 1,
        yAxisIndex: 2,
        data: model.volumeBars,
        barWidth: '46%',
      },
    ],
  };
}
