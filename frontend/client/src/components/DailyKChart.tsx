import { useMemo } from 'react';
import ReactECharts from 'echarts-for-react';
import { buildDailyKChartModel, buildDailyKOption, type DailyCandle, type KLinePeriod } from '@/lib/dailyKChart';

interface DailyKChartProps {
  candles: DailyCandle[];
  period?: KLinePeriod;
  height?: number;
}

export default function DailyKChart({ candles, period = '1d', height = 370 }: DailyKChartProps) {
  const model = useMemo(() => buildDailyKChartModel(candles, period), [candles, period]);
  const option = useMemo(() => buildDailyKOption(model), [model]);

  return (
    <div className="rounded-lg border border-border bg-card/60 px-2 py-2">
      <ReactECharts option={option} notMerge lazyUpdate style={{ width: '100%', height }} />
    </div>
  );
}
