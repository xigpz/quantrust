/**
 * 板块分时主力净流入 — 多曲线实时走势（数据由后端行情扫描聚合）
 * 右侧为固定标签列，折线连接各标签与对应曲线末端，避免末端数值接近时文字重叠。
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import ReactECharts from 'echarts-for-react';
import {
  useSectorIntradayFlow,
  type SectorIntradayFlowResponse,
} from '@/hooks/useMarketData';
import { Activity, RefreshCw } from 'lucide-react';
import type { ECharts, GraphicComponentOption } from 'echarts';

const WARM = ['#ef4444', '#f97316', '#eab308', '#fb7185', '#fdba74'];
const COOL = ['#38bdf8', '#22d3ee', '#6366f1', '#2dd4bf', '#60a5fa'];

/** 标签列占用的绘图区右侧留白（像素级），折线在 margin 内走线 */
const LABEL_COL_PX = 108;

function pickLineColor(idx: number, last: number): string {
  const pal = last >= 0 ? WARM : COOL;
  return pal[idx % pal.length];
}

/** 用 rgba 表达透明度，避免 style.opacity 过低时 ZRender 不参与命中测试、点击穿透 */
function hexToRgba(hex: string, a: number): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return hex;
  const n = parseInt(m[1], 16);
  const r = (n >> 16) & 255;
  const g = (n >> 8) & 255;
  const b = n & 255;
  return `rgba(${r},${g},${b},${a})`;
}

type ZrNamed = { name?: string; parent?: ZrNamed | null };

function sectorIdxFromZrChain(raw: unknown): number | null {
  let el = raw as ZrNamed | null | undefined;
  while (el) {
    const nm = el.name;
    if (typeof nm === 'string' && /^sector-\d+$/.test(nm)) {
      const idx = Number.parseInt(nm.slice('sector-'.length), 10);
      return Number.isNaN(idx) ? null : idx;
    }
    el = el.parent ?? undefined;
  }
  return null;
}

function zrTargetFromChartParams(p: Record<string, unknown>): unknown {
  const ev = p.event;
  if (!ev || typeof ev !== 'object') return undefined;
  const o = ev as Record<string, unknown>;
  if (o.target) return o.target;
  const inner = o.event;
  if (inner && typeof inner === 'object' && (inner as Record<string, unknown>).target) {
    return (inner as Record<string, unknown>).target;
  }
  return undefined;
}

function buildSortedTimes(series: SectorIntradayFlowResponse['series']): string[] {
  const all = new Set<string>();
  for (const s of series) {
    for (const p of s.points) all.add(p.t);
  }
  return Array.from(all).sort((a, b) => a.localeCompare(b));
}

/** A股连续竞价时段：09:30-11:30、13:00-15:00（排除午休） */
function isTradingMinute(t: string): boolean {
  const m = /^(\d{2}):(\d{2})$/.exec(t);
  if (!m) return false;
  const hh = Number.parseInt(m[1], 10);
  const mm = Number.parseInt(m[2], 10);
  if (Number.isNaN(hh) || Number.isNaN(mm)) return false;
  const mins = hh * 60 + mm;
  const amStart = 9 * 60 + 30;
  const amEnd = 11 * 60 + 30;
  const pmStart = 13 * 60;
  const pmEnd = 15 * 60;
  return (mins >= amStart && mins <= amEnd) || (mins >= pmStart && mins <= pmEnd);
}

/** 方案A：把每条线转换为“相对首点变化”，首点为0，增强分钟级波动可读性 */
function toRelativeSeries(data: SectorIntradayFlowResponse): SectorIntradayFlowResponse {
  return {
    ...data,
    series: data.series.map((s) => {
      const filtered = s.points.filter((p) => isTradingMinute(p.t));
      const base = filtered[0]?.v ?? 0;
      const points = filtered.map((p) => ({ t: p.t, v: p.v - base }));
      const last = points.length ? points[points.length - 1].v : 0;
      return {
        ...s,
        points,
        last,
      };
    }),
  };
}

type OutflowAlert = {
  code: string;
  name: string;
  latestTime: string;
  latestRate: number;
  baselineRate: number;
  multiple: number;
};

type GrowthMetric = {
  code: string;
  name: string;
  delta: number;
  pct: number;
};

function fmtAmount(v: number): string {
  return `${v >= 0 ? '+' : ''}${v.toFixed(2)} 亿`;
}

function fmtPct(v: number): string {
  return `${v >= 0 ? '+' : ''}${v.toFixed(2)}%`;
}

/** 检测“流出速率突然加大”板块（按最近一跳与近期基线对比） */
function detectOutflowAlerts(
  data: SectorIntradayFlowResponse,
  minStepOutflow: number,
  minMultiple: number,
): OutflowAlert[] {
  const alerts: OutflowAlert[] = [];

  for (const s of data.series) {
    const pts = s.points.filter((p) => isTradingMinute(p.t));
    if (pts.length < 4) continue;

    const rates: Array<{ t: string; v: number }> = [];
    for (let i = 1; i < pts.length; i += 1) {
      rates.push({
        t: pts[i].t,
        v: pts[i].v - pts[i - 1].v, // 单位：亿元/采样步
      });
    }
    if (rates.length < 3) continue;

    const latest = rates[rates.length - 1];
    // 仅关注“流出加大”（净流入变化为负，且绝对值扩大）
    if (latest.v >= 0) continue;

    const history = rates.slice(Math.max(0, rates.length - 7), rates.length - 1);
    const histOutflowAbs = history
      .filter((r) => r.v < 0)
      .map((r) => Math.abs(r.v));
    const baselineAbs =
      histOutflowAbs.length > 0
        ? histOutflowAbs.reduce((sum, x) => sum + x, 0) / histOutflowAbs.length
        : 0;

    const latestAbs = Math.abs(latest.v);
    const multiple = baselineAbs > 0 ? latestAbs / baselineAbs : latestAbs >= 1 ? 9.99 : 0;

    // 阈值：当前至少 -minStepOutflow 亿/步，且相对近期流出基线至少 minMultiple 倍
    if (latestAbs >= minStepOutflow && multiple >= minMultiple) {
      alerts.push({
        code: s.code,
        name: s.name,
        latestTime: latest.t,
        latestRate: latest.v,
        baselineRate: -baselineAbs,
        multiple,
      });
    }
  }

  return alerts
    .sort((a, b) => Math.abs(b.latestRate) - Math.abs(a.latestRate))
    .slice(0, 6);
}

/** 计算相对首点的增长幅度（金额 + 百分比） */
function computeGrowthMetrics(data: SectorIntradayFlowResponse): GrowthMetric[] {
  return data.series
    .map((s) => {
      const pts = s.points.filter((p) => isTradingMinute(p.t));
      if (pts.length < 2) return null;
      const base = pts[0].v;
      const last = pts[pts.length - 1].v;
      const delta = last - base;
      const denom = Math.max(Math.abs(base), 1e-6);
      const pct = (delta / denom) * 100;
      return { code: s.code, name: s.name, delta, pct };
    })
    .filter((x): x is GrowthMetric => x !== null)
    .sort((a, b) => b.pct - a.pct);
}

/** 根据全部采样点留出上下边距，避免多线挤在一条窄带里 */
function computeYExtent(series: { points: { v: number }[] }[]): { min: number; max: number } {
  let vmin = Infinity;
  let vmax = -Infinity;
  for (const s of series) {
    for (const p of s.points) {
      if (!Number.isFinite(p.v)) continue;
      vmin = Math.min(vmin, p.v);
      vmax = Math.max(vmax, p.v);
    }
  }
  if (vmin === Infinity) {
    return { min: -5, max: 5 };
  }
  const span = Math.max(vmax - vmin, Math.max(Math.abs(vmin), Math.abs(vmax)) * 0.06, 0.01);
  const pad = Math.max(span * 0.28, 0.45);
  return { min: vmin - pad, max: vmax + pad };
}

function buildLeaderGraphics(
  chart: ECharts,
  data: SectorIntradayFlowResponse,
  times: string[],
  yMin: number,
  yMax: number,
  selectedIdx: number | null,
): GraphicComponentOption[] {
  if (!times.length || !data.series.length) return [];

  const pTL = chart.convertToPixel({ xAxisIndex: 0, yAxisIndex: 0 }, [times[0], yMax]);
  const pBR = chart.convertToPixel({ xAxisIndex: 0, yAxisIndex: 0 }, [times[times.length - 1], yMin]);
  if (!pTL || !pBR || pTL.some((n) => !Number.isFinite(n)) || pBR.some((n) => !Number.isFinite(n))) {
    return [];
  }

  const plotLeft = Math.min(pTL[0], pBR[0]);
  const plotRight = Math.max(pTL[0], pBR[0]);
  const plotTop = Math.min(pTL[1], pBR[1]);
  const plotBottom = Math.max(pTL[1], pBR[1]);

  const cw = chart.getWidth();
  const labelX = cw - 8;

  type Item = { idx: number; name: string; last: number; color: string; px: number; py: number };
  const items: Item[] = [];

  data.series.forEach((s, idx) => {
    if (!s.points.length) return;
    const lp = s.points[s.points.length - 1];
    const pix = chart.convertToPixel({ xAxisIndex: 0, yAxisIndex: 0 }, [lp.t, lp.v]);
    if (!pix || pix.some((n) => !Number.isFinite(n))) return;
    items.push({
      idx,
      name: s.name,
      last: s.last,
      color: pickLineColor(idx, s.last),
      px: pix[0],
      py: pix[1],
    });
  });

  items.sort((a, b) => a.py - b.py);

  const n = items.length;
  const innerPad = 8;
  const plotH = Math.max(plotBottom - plotTop - 2 * innerPad, 1);
  const slot = plotH / Math.max(n, 1);

  const graphics: GraphicComponentOption[] = [];

  const hasSel = selectedIdx !== null;

  items.forEach((it, i) => {
    const labelY = plotTop + innerPad + slot * (i + 0.5);
    const ribX = Math.min(plotRight + 8, labelX - 22);
    const active = selectedIdx === it.idx;

    let points: number[][];
    if (ribX >= it.px) {
      points = [
        [it.px, it.py],
        [ribX, it.py],
        [ribX, labelY],
        [labelX, labelY],
      ];
    } else {
      const kinkX = it.px + 14;
      points = [
        [it.px, it.py],
        [kinkX, it.py],
        [kinkX, labelY],
        [labelX, labelY],
      ];
    }

    const lineW = active ? 2.8 : 1.15;
    const lineA = hasSel ? (active ? 1 : 0.28) : 0.92;
    const textFillA = hasSel ? (active ? 1 : 0.42) : 1;
    const textBg = active ? 'rgba(30, 41, 59, 0.98)' : 'rgba(15, 23, 42, 0.9)';
    const borderW = active ? 1.6 : 0.5;
    const fontSize = active ? 12 : 11;

    graphics.push({
      type: 'group',
      name: `sector-${it.idx}`,
      silent: false,
      zlevel: 1,
      z: active ? 20 : hasSel ? 2 : 4,
      children: [
        {
          type: 'polyline',
          name: `sector-${it.idx}`,
          shape: { points },
          style: {
            stroke: hexToRgba(it.color, lineA),
            lineWidth: lineW,
            fill: 'none' as const,
          },
          silent: false,
          z: 1,
        },
        {
          type: 'text',
          name: `sector-${it.idx}`,
          style: {
            text: `${it.name} ${it.last >= 0 ? '+' : ''}${it.last.toFixed(1)}`,
            fill: hexToRgba(it.color, textFillA),
            fontSize,
            fontWeight: 600,
            fontFamily: 'ui-sans-serif, system-ui, sans-serif',
            textAlign: 'right',
            textVerticalAlign: 'middle',
            x: labelX,
            y: labelY,
            backgroundColor: textBg,
            padding: active ? [4, 8] : [3, 7],
            borderRadius: 4,
            borderColor: hexToRgba(it.color, hasSel && !active ? 0.4 : 1),
            borderWidth: borderW,
            shadowBlur: active ? 12 : 0,
            shadowColor: active ? it.color : 'transparent',
          },
          silent: false,
          z: 2,
        },
      ],
    });
  });

  return graphics;
}

export default function SectorIntradayFlowPanel() {
  const { data, loading, error, refetch, isDemo } = useSectorIntradayFlow();
  const chartRef = useRef<ReactECharts>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const clickAttachedInst = useRef<ECharts | null>(null);
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);
  const [minStepOutflow, setMinStepOutflow] = useState(0.5);
  const [minMultiple, setMinMultiple] = useState(1.8);
  const outflowAlerts = useMemo(
    () => (data ? detectOutflowAlerts(data, minStepOutflow, minMultiple) : []),
    [data?.updated_at, data?.trade_date, data, minStepOutflow, minMultiple],
  );
  const growthMetrics = useMemo(
    () => (data ? computeGrowthMetrics(data).slice(0, 12) : []),
    [data?.updated_at, data?.trade_date, data],
  );

  const onChartClick = useCallback((p: Record<string, unknown>) => {
    const fromGraphic = sectorIdxFromZrChain(zrTargetFromChartParams(p));
    if (fromGraphic != null) {
      setSelectedIdx((cur) => (cur === fromGraphic ? null : fromGraphic));
      return;
    }

    if (p.componentType === 'series' && typeof p.seriesIndex === 'number') {
      const idx = p.seriesIndex;
      setSelectedIdx((cur) => (cur === idx ? null : idx));
      return;
    }
    if (p.componentType === 'graphic' && typeof p.name === 'string' && /^sector-\d+$/.test(p.name)) {
      const idx = Number.parseInt((p.name as string).slice('sector-'.length), 10);
      if (!Number.isNaN(idx)) {
        setSelectedIdx((cur) => (cur === idx ? null : idx));
      }
      return;
    }
    setSelectedIdx(null);
  }, []);

  const ensureChartClickListener = useCallback(() => {
    const inst = chartRef.current?.getEchartsInstance?.() ?? null;
    if (!inst) return;
    if (clickAttachedInst.current === inst) return;
    if (clickAttachedInst.current) {
      clickAttachedInst.current.off('click', onChartClick);
    }
    clickAttachedInst.current = inst;
    inst.on('click', onChartClick);
  }, [onChartClick]);

  useEffect(() => {
    return () => {
      if (clickAttachedInst.current) {
        clickAttachedInst.current.off('click', onChartClick);
        clickAttachedInst.current = null;
      }
    };
  }, [onChartClick]);

  const option = useMemo(() => {
    if (loading && !data) {
      return {
        backgroundColor: 'transparent',
        graphic: [] as GraphicComponentOption[],
        title: {
          text: '加载中…',
          left: 'center',
          top: 'middle',
          textStyle: { color: '#888', fontSize: 13 },
        },
      };
    }
    if (error && !data?.series?.length) {
      return {
        backgroundColor: 'transparent',
        graphic: [] as GraphicComponentOption[],
        title: {
          text: `加载失败：${error}`,
          left: 'center',
          top: 'middle',
          textStyle: { color: '#f87171', fontSize: 13 },
        },
      };
    }
    if (!data?.series?.length) {
      return {
        backgroundColor: 'transparent',
        graphic: [] as GraphicComponentOption[],
        title: {
          text: '暂无分时数据（等待后端完成几次行情扫描后自动出现曲线）',
          left: 'center',
          top: 'middle',
          textStyle: { color: '#888', fontSize: 13 },
        },
      };
    }

    const relativeData = toRelativeSeries(data);
    const times = buildSortedTimes(relativeData.series);
    const { min: yMin, max: yMax } = computeYExtent(relativeData.series);
    const hasSel = selectedIdx !== null;

    const series = relativeData.series.map((s, idx) => {
      const positive = s.last >= 0;
      const pal = positive ? WARM : COOL;
      const color = pal[idx % pal.length];
      const map = new Map(s.points.map((p) => [p.t, p.v]));
      const linedata = times.map((t) => {
        const y = map.get(t);
        return y === undefined ? null : ([t, y] as [string, number]);
      });

      const active = selectedIdx === idx;
      const lineWidth = active ? 3.2 : hasSel ? 1.25 : 1.6;
      const alpha = hasSel ? (active ? 1 : 0.14) : 1;
      const lineColor = hexToRgba(color, alpha);

      return {
        name: s.name,
        type: 'line' as const,
        smooth: 0.35,
        showSymbol: false,
        lineStyle: { width: lineWidth, color: lineColor },
        itemStyle: { color: lineColor },
        connectNulls: true,
        z: active ? 18 : hasSel ? 2 : 6,
        emphasis: { disabled: true },
        data: linedata,
      };
    });

    return {
      backgroundColor: 'transparent',
      graphic: [] as GraphicComponentOption[],
      grid: {
        left: 52,
        right: LABEL_COL_PX,
        top: 28,
        bottom: 32,
        containLabel: true,
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
        valueFormatter: (v: number) => fmtAmount(Number(v)),
      },
      xAxis: {
        type: 'category',
        data: times,
        boundaryGap: false,
        axisLabel: { color: '#94a3b8', fontSize: 10 },
        axisLine: { lineStyle: { color: 'rgba(148,163,184,0.25)' } },
        splitLine: { show: true, lineStyle: { color: 'rgba(148,163,184,0.12)' } },
      },
      yAxis: {
        type: 'value',
        min: yMin,
        max: yMax,
        splitNumber: 6,
        name: '较首点变化(亿)',
        nameTextStyle: { color: '#94a3b8', fontSize: 10 },
        axisLabel: {
          color: '#94a3b8',
          fontSize: 10,
          formatter: (v: number) => Number(v).toFixed(2),
        },
        splitLine: { lineStyle: { color: 'rgba(148,163,184,0.12)' } },
      },
      series,
    };
  }, [data, selectedIdx, loading, error]);

  const applyLeaders = useCallback(() => {
    const inst = chartRef.current?.getEchartsInstance?.() ?? null;
    if (!inst) return;
    if (!data?.series?.length) {
      inst.setOption({ graphic: [] }, { replaceMerge: ['graphic'] });
      return;
    }
    const relativeData = toRelativeSeries(data);
    const times = buildSortedTimes(relativeData.series);
    const { min: yMin, max: yMax } = computeYExtent(relativeData.series);
    const g = buildLeaderGraphics(inst, relativeData, times, yMin, yMax, selectedIdx);
    inst.setOption({ graphic: g }, { replaceMerge: ['graphic'] });
  }, [data, selectedIdx]);

  useEffect(() => {
    const t = requestAnimationFrame(() => {
      applyLeaders();
      ensureChartClickListener();
    });
    return () => cancelAnimationFrame(t);
  }, [applyLeaders, ensureChartClickListener]);

  useEffect(() => {
    const el = wrapRef.current;
    if (!el || typeof ResizeObserver === 'undefined') return;
    const ro = new ResizeObserver(() => {
      requestAnimationFrame(() => {
        applyLeaders();
        ensureChartClickListener();
      });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [applyLeaders, ensureChartClickListener]);

  useEffect(() => {
    setSelectedIdx(null);
  }, [data?.updated_at, data?.trade_date]);

  const showChartSubtitle =
    Boolean(data?.trade_date) && (data?.series?.length ?? 0) > 0;

  return (
    <div className="flex flex-col min-h-[480px]">
      <header className="sticky top-0 z-10 border-b border-border bg-card px-4 pt-3 pb-4">
        <div className="flex items-center justify-between gap-2">
          <div className="flex flex-wrap items-center gap-2">
            <Activity className="w-4 h-4 shrink-0 text-rose-400" />
            <h2 className="text-sm font-semibold leading-none">板块分时走势</h2>
            <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
              {data?.series?.length ?? 0} 条
            </span>
            {isDemo && (
              <span
                className="text-[10px] text-muted-foreground border border-border px-1.5 py-0.5 rounded"
                title="已开启 VITE_USE_MOCK_FALLBACK 或接口失败回退时的本地假数据。默认仅使用后端真实数据。"
              >
                模拟数据
              </span>
            )}
          </div>
          <button
            type="button"
            onClick={() => void refetch()}
            className="shrink-0 text-muted-foreground hover:text-foreground transition-colors"
            aria-label="刷新"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>
        {showChartSubtitle && data?.trade_date && (
          <p className="mt-3 text-xs font-medium text-muted-foreground tracking-tight">
            {data.trade_date} 分时资金流
          </p>
        )}
      </header>

      <div ref={wrapRef} className="flex-1 px-1 pt-3 pb-2 min-h-[400px]">
        <ReactECharts
          ref={chartRef}
          option={option}
          notMerge
          lazyUpdate
          style={{ width: '100%', height: 460 }}
          onEvents={{
            finished: () => {
              requestAnimationFrame(() => {
                applyLeaders();
                ensureChartClickListener();
              });
            },
          }}
        />
      </div>

      <div className="px-4 pb-2">
        <div className="flex flex-wrap items-center justify-between gap-2 mb-1.5">
          <div className="text-[11px] text-muted-foreground">流出速率提醒（最近一跳）</div>
          <div className="flex items-center gap-2 text-[10px] text-muted-foreground">
            <label className="flex items-center gap-1">
              <span>最小流出</span>
              <input
                type="number"
                min={0.1}
                step={0.1}
                value={minStepOutflow}
                onChange={(e) => {
                  const v = Number.parseFloat(e.target.value);
                  if (Number.isFinite(v)) setMinStepOutflow(Math.max(0.1, v));
                }}
                className="w-14 h-6 px-1 rounded border border-border bg-background text-foreground"
              />
              <span>亿/步</span>
            </label>
            <label className="flex items-center gap-1">
              <span>放大倍数</span>
              <input
                type="number"
                min={1.1}
                step={0.1}
                value={minMultiple}
                onChange={(e) => {
                  const v = Number.parseFloat(e.target.value);
                  if (Number.isFinite(v)) setMinMultiple(Math.max(1.1, v));
                }}
                className="w-12 h-6 px-1 rounded border border-border bg-background text-foreground"
              />
              <span>x</span>
            </label>
          </div>
        </div>
        {outflowAlerts.length === 0 ? (
          <div className="text-[10px] text-muted-foreground/70 border border-border rounded px-2 py-1.5">
            暂无“流出突然加大”板块（阈值：单步流出≥{minStepOutflow.toFixed(1)}亿，且较近期基线≥{minMultiple.toFixed(1)}倍）。
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-1.5">
            {outflowAlerts.map((a) => (
              <div key={a.code} className="border border-red-900/50 bg-red-950/25 rounded px-2 py-1.5">
                <div className="flex items-center justify-between text-[11px]">
                  <span className="text-foreground">{a.name}</span>
                  <span className="text-red-400 font-medium">
                    {a.latestRate.toFixed(2)} 亿/步
                  </span>
                </div>
                <div className="mt-0.5 text-[10px] text-muted-foreground">
                  {a.latestTime} · 较近期流出基线放大 {a.multiple.toFixed(1)}x（基线 {a.baselineRate.toFixed(2)}）
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="px-4 pb-2">
        <div className="text-[11px] text-muted-foreground mb-1.5">增长幅度分布（较首点）</div>
        {growthMetrics.length === 0 ? (
          <div className="text-[10px] text-muted-foreground/70 border border-border rounded px-2 py-1.5">
            暂无可用于计算增长幅度的数据点。
          </div>
        ) : (
          <div className="space-y-1.5">
            {(() => {
              const maxAbsPct = Math.max(...growthMetrics.map((g) => Math.abs(g.pct)), 0.01);
              return growthMetrics.map((g) => {
                const width = `${Math.max(6, (Math.abs(g.pct) / maxAbsPct) * 100)}%`;
                const positive = g.pct >= 0;
                return (
                  <div key={g.code} className="border border-border rounded px-2 py-1.5">
                    <div className="flex items-center justify-between text-[11px] mb-1">
                      <span className="text-foreground">{g.name}</span>
                      <span className={positive ? 'text-red-400 font-medium' : 'text-cyan-400 font-medium'}>
                        {fmtPct(g.pct)} · {fmtAmount(g.delta)}
                      </span>
                    </div>
                    <div className="h-1.5 bg-muted/40 rounded overflow-hidden">
                      <div
                        className={`h-full ${positive ? 'bg-red-400/80' : 'bg-cyan-400/80'}`}
                        style={{ width }}
                      />
                    </div>
                  </div>
                );
              });
            })()}
          </div>
        )}
      </div>

      <p className="text-[10px] text-muted-foreground px-4 pb-3 leading-relaxed">
        说明：首次开盘后，系统在几次全市场扫描内锁定「主力净流入绝对值靠前」的若干板块，并在每次扫描时追加数据点；横轴为北京时间（自动排除午间休市 11:30-13:00），纵轴显示“相对首个采样点的变化值（亿元）”，用于观察分时波动。右侧为标签列，折线连接各板块末端与对应标签；点击标签、折线或曲线可高亮该板块（再点一次取消），点击空白处取消选中。
        {isDemo && (
          <span className="block mt-1.5 text-foreground/70">
            当前为本地模拟曲线（已设置 VITE_USE_MOCK_FALLBACK=true 或接口失败回退）。需要真实数据时请启动后端并关闭该开关。
          </span>
        )}
      </p>
    </div>
  );
}
