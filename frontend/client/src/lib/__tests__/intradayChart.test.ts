import { describe, expect, it } from 'vitest';
import { buildIntradayChartModel, buildIntradayOption, type IntradaySeries } from '@/lib/intradayChart';

const series: IntradaySeries = {
  symbol: '001309.SZ',
  name: '德明利',
  range: '1d',
  pre_close: 320.05,
  points: [
    {
      timestamp: '2026-03-16 09:25',
      price: 351,
      avg_price: 351,
      volume: 1000,
      turnover: 351000,
      change_pct: 0,
      change: 0,
    },
    {
      timestamp: '2026-03-16 09:30',
      price: 352.06,
      avg_price: 352.06,
      volume: 3000,
      turnover: 1056180,
      change_pct: 10,
      change: 32.01,
    },
    {
      timestamp: '2026-03-16 09:31',
      price: 351.5,
      avg_price: 351.82,
      volume: 4500,
      turnover: 1583430,
      change_pct: 9.83,
      change: 31.45,
    },
  ],
};

describe('intradayChart', () => {
  it('derives interval volume and turnover from cumulative trends rows', () => {
    const model = buildIntradayChartModel(series);

    expect(model.volumeBars.map((item) => item.value)).toEqual([1000, 2000, 1500]);
    expect(model.turnoverBars).toEqual([351000, 705180, 527250]);
  });

  it('keeps premarket labels and symmetric percent bounds around pre close', () => {
    const model = buildIntradayChartModel(series);

    expect(model.categories).toEqual(['09:25', '09:30', '09:31']);
    expect(model.priceMin).toBeLessThan(series.pre_close);
    expect(model.priceMax).toBeGreaterThan(series.pre_close);
    expect(Number((series.pre_close - model.priceMin).toFixed(2))).toBe(Number((model.priceMax - series.pre_close).toFixed(2)));
  });

  it('builds an EastMoney-style option with linked price and volume areas', () => {
    const model = buildIntradayChartModel(series);
    const option = buildIntradayOption(model);

    expect(option.grid).toHaveLength(2);
    expect(option.series).toHaveLength(3);
    expect(option.tooltip.trigger).toBe('axis');
    expect(option.series[2].type).toBe('bar');
  });
});
