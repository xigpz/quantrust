import { describe, expect, it } from 'vitest';
import { STOCK_DETAIL_CHART_MODES } from '@/lib/stockDetailChartMode';

describe('stockDetailChartMode', () => {
  it('keeps EastMoney-style period ordering', () => {
    expect(STOCK_DETAIL_CHART_MODES.map((item) => item.label)).toEqual([
      '盘前',
      '分时',
      '5日',
      '日K',
      '周K',
      '月K',
      '5分',
      '15分',
      '30分',
      '60分',
    ]);
  });

  it('maps intraday and kline requests to the expected api params', () => {
    const fiveDay = STOCK_DETAIL_CHART_MODES.find((item) => item.id === 'fiveDay');
    const dayK = STOCK_DETAIL_CHART_MODES.find((item) => item.id === 'dayK');
    const m30 = STOCK_DETAIL_CHART_MODES.find((item) => item.id === 'm30');

    expect(fiveDay).toMatchObject({ kind: 'intraday', intradayRange: '5d' });
    expect(dayK).toMatchObject({ kind: 'kline', candlePeriod: '1d', candleCount: 240 });
    expect(m30).toMatchObject({ kind: 'kline', candlePeriod: '30m', candleCount: 240 });
  });
});
