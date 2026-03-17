export interface DailyCandle {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  turnover: number;
}

export type KLinePeriod = '1d' | '1w' | '1M' | '5m' | '15m' | '30m' | '60m';

export interface DailyKChartModel {
  categories: string[];
  candleValues: Array<[number, number, number, number]>;
  volumes: Array<{ value: number; itemStyle: { color: string } }>;
  ma5: Array<number | null>;
  ma10: Array<number | null>;
  ma20: Array<number | null>;
  ma30: Array<number | null>;
  maxVolume: number;
  minPrice: number;
  maxPrice: number;
  defaultStartIndex: number;
  defaultEndIndex: number;
}

const UP_COLOR = '#d84a4a';
const DOWN_COLOR = '#2ba272';
const MA5_COLOR = '#f5c542';
const MA10_COLOR = '#5aa9ff';
const MA20_COLOR = '#b07cff';
const MA30_COLOR = '#8dc859';
const DEFAULT_VISIBLE_DAYS = 58;

function formatDateLabel(timestamp: string, period: KLinePeriod) {
  const date = timestamp.slice(0, 10);
  const [, month, day] = date.split('-');

  if (period === '5m' || period === '15m' || period === '30m' || period === '60m') {
    const time = timestamp.slice(11, 16);
    return `${month}-${day}\n${time}`;
  }

  return `${month}-${day}`;
}

function buildMovingAverage(data: DailyCandle[], period: number) {
  return data.map((_, index) => {
    if (index + 1 < period) {
      return null;
    }

    const slice = data.slice(index + 1 - period, index + 1);
    const total = slice.reduce((sum, item) => sum + item.close, 0);
    return Number((total / period).toFixed(2));
  });
}

export function buildDailyKChartModel(candles: DailyCandle[], period: KLinePeriod = '1d'): DailyKChartModel {
  const categories = candles.map((item) => formatDateLabel(item.timestamp, period));
  const candleValues = candles.map((item) => [item.open, item.close, item.low, item.high] as [number, number, number, number]);
  const volumes = candles.map((item) => ({
    value: item.volume,
    itemStyle: {
      color: item.close >= item.open ? UP_COLOR : DOWN_COLOR,
    },
  }));
  const ma5 = buildMovingAverage(candles, 5);
  const ma10 = buildMovingAverage(candles, 10);
  const ma20 = buildMovingAverage(candles, 20);
  const ma30 = buildMovingAverage(candles, 30);
  const priceValues = candles.flatMap((item) => [item.low, item.high]);
  const minPrice = priceValues.length ? Math.min(...priceValues) * 0.985 : 0;
  const maxPrice = priceValues.length ? Math.max(...priceValues) * 1.015 : 100;
  const maxVolume = volumes.length ? Math.max(...volumes.map((item) => item.value)) : 0;
  const defaultEndIndex = candles.length ? candles.length - 1 : 0;
  const defaultStartIndex = Math.max(0, candles.length - DEFAULT_VISIBLE_DAYS);

  return {
    categories,
    candleValues,
    volumes,
    ma5,
    ma10,
    ma20,
    ma30,
    maxVolume,
    minPrice,
    maxPrice,
    defaultStartIndex,
    defaultEndIndex,
  };
}

export function buildDailyKOption(model: DailyKChartModel) {
  return {
    animation: false,
    legend: {
      top: 0,
      left: 8,
      itemWidth: 10,
      itemHeight: 6,
      textStyle: {
        color: '#94a3b8',
        fontSize: 10,
      },
      data: ['MA5', 'MA10', 'MA20', 'MA30'],
    },
    axisPointer: {
      link: [{ xAxisIndex: [0, 1] }],
      label: {
        backgroundColor: '#475569',
      },
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross',
      },
      backgroundColor: 'rgba(15, 23, 42, 0.94)',
      borderWidth: 0,
      textStyle: {
        color: '#e2e8f0',
        fontSize: 11,
      },
    },
    grid: [
      { left: 56, right: 20, top: 24, height: 220 },
      { left: 56, right: 20, top: 268, height: 72 },
    ],
    xAxis: [
      {
        type: 'category',
        data: model.categories,
        boundaryGap: true,
        axisLine: { lineStyle: { color: '#cbd5e1' } },
        axisTick: { show: false },
        axisLabel: { show: false },
        splitLine: { show: false },
        min: 'dataMin',
        max: 'dataMax',
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
        min: 'dataMin',
        max: 'dataMax',
      },
    ],
    yAxis: [
      {
        scale: true,
        position: 'right',
        min: model.minPrice,
        max: model.maxPrice,
        splitNumber: 5,
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: '#64748b',
          fontSize: 10,
          formatter: (value: number) => value.toFixed(2),
        },
        splitLine: {
          lineStyle: {
            color: 'rgba(148, 163, 184, 0.18)',
          },
        },
      },
      {
        gridIndex: 1,
        scale: true,
        position: 'right',
        min: 0,
        max: model.maxVolume || 1,
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: '#94a3b8',
          fontSize: 9,
        },
        splitLine: { show: false },
      },
    ],
    dataZoom: [
      {
        type: 'inside',
        xAxisIndex: [0, 1],
        startValue: model.defaultStartIndex,
        endValue: model.defaultEndIndex,
        zoomOnMouseWheel: true,
        moveOnMouseMove: true,
        moveOnMouseWheel: true,
      },
      {
        type: 'slider',
        xAxisIndex: [0, 1],
        bottom: 0,
        height: 24,
        borderColor: '#dbe4ee',
        fillerColor: 'rgba(96, 165, 250, 0.12)',
        handleStyle: {
          color: '#94a3b8',
        },
        backgroundColor: 'rgba(148, 163, 184, 0.08)',
        dataBackground: {
          lineStyle: { color: '#94a3b8' },
          areaStyle: { color: 'rgba(148, 163, 184, 0.12)' },
        },
        startValue: model.defaultStartIndex,
        endValue: model.defaultEndIndex,
        realtime: true,
        zoomLock: false,
      },
    ],
    series: [
      {
        name: '日K',
        type: 'candlestick',
        data: model.candleValues,
        itemStyle: {
          color: UP_COLOR,
          color0: DOWN_COLOR,
          borderColor: UP_COLOR,
          borderColor0: DOWN_COLOR,
        },
      },
      {
        name: 'MA5',
        type: 'line',
        data: model.ma5,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1.1, color: MA5_COLOR },
      },
      {
        name: 'MA10',
        type: 'line',
        data: model.ma10,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1.1, color: MA10_COLOR },
      },
      {
        name: 'MA20',
        type: 'line',
        data: model.ma20,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1.1, color: MA20_COLOR },
      },
      {
        name: 'MA30',
        type: 'line',
        data: model.ma30,
        smooth: true,
        showSymbol: false,
        lineStyle: { width: 1.1, color: MA30_COLOR },
      },
      {
        name: '成交量',
        type: 'bar',
        xAxisIndex: 1,
        yAxisIndex: 1,
        data: model.volumes,
        barWidth: '52%',
      },
    ],
  };
}
