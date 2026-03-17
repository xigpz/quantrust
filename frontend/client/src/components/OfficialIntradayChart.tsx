import { useMemo } from 'react';
import ReactECharts from 'echarts-for-react';
import { buildIntradayChartModel, buildIntradayOption, type IntradaySeries } from '@/lib/intradayChart';

interface OfficialIntradayChartProps {
  series: IntradaySeries;
}

export default function OfficialIntradayChart({ series }: OfficialIntradayChartProps) {
  const model = useMemo(() => buildIntradayChartModel(series), [series]);
  const option = useMemo(() => buildIntradayOption(model), [model]);

  return (
    <div className="rounded-xl border border-border bg-card/70 px-2 py-2">
      <ReactECharts option={option} notMerge lazyUpdate style={{ width: '100%', height: 470 }} />
    </div>
  );
}
