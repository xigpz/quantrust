import { describe, expect, it } from 'vitest';
import { buildDailyKChartModel, buildDailyKOption } from '@/lib/dailyKChart';

function buildSampleCandles(count: number) {
  return Array.from({ length: count }, (_, index) => {
    const day = String(index + 1).padStart(2, '0');
    const open = 100 + index * 0.6;
    const close = open + (index % 2 === 0 ? 1.2 : -0.8);
    const high = Math.max(open, close) + 1.1;
    const low = Math.min(open, close) - 0.9;

    return {
      timestamp: `2026-03-${day}`,
      open,
      high,
      low,
      close,
      volume: 100000 + index * 2500,
      turnover: 9000000 + index * 123456,
    };
  });
}

describe('dailyKChart', () => {
  it('builds candlestick and volume series from daily candles', () => {
    const model = buildDailyKChartModel(buildSampleCandles(8));

    expect(model.categories[0]).toBe('03-01');
    expect(model.candleValues[0]).toEqual([100, 101.2, 99.1, 102.3]);
    expect(model.volumes[0]).toEqual({
      value: 100000,
      itemStyle: { color: '#d84a4a' },
    });
  });

  it('defaults data zoom to the latest window and supports dragging historical range', () => {
    const model = buildDailyKChartModel(buildSampleCandles(120));
    const option = buildDailyKOption(model);
    const insideZoom = option.dataZoom[0];
    const sliderZoom = option.dataZoom[1];

    expect(insideZoom.type).toBe('inside');
    expect(sliderZoom.type).toBe('slider');
    expect(sliderZoom.startValue).toBe(62);
    expect(sliderZoom.endValue).toBe(119);
    expect(sliderZoom.realtime).toBe(true);
    expect(sliderZoom.zoomLock).toBe(false);
  });

  it('shows full history when there are fewer candles than the default window', () => {
    const model = buildDailyKChartModel(buildSampleCandles(20));
    const option = buildDailyKOption(model);
    const sliderZoom = option.dataZoom[1];

    expect(sliderZoom.startValue).toBe(0);
    expect(sliderZoom.endValue).toBe(19);
  });
});
