# EastMoney Standard Intraday Chart Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the stock detail modal's pseudo intraday view with an EastMoney-style `分时 / 5日 / 日线` chart flow backed by a dedicated intraday API and ECharts renderer.

**Architecture:** Split intraday data from the existing candle flow instead of forcing `trends2/get` rows into `Candle`. Add a backend `/api/intraday/{symbol}` endpoint that returns normalized EastMoney trend semantics, then render that payload in a dedicated frontend `IntradayChart` component built around a pure option-builder utility so the visual logic stays testable.

**Tech Stack:** Rust, Axum, Reqwest, Serde, React 19, TypeScript, ECharts, Vitest, pnpm

---

### Task 1: Backend intraday models and parser

**Files:**
- Create: `backend/src/models/intraday.rs`
- Modify: `backend/src/models/mod.rs`
- Modify: `backend/src/data/eastmoney.rs`
- Test: `backend/src/data/eastmoney.rs`

**Step 1: Write the failing test**

Add a parser test in `backend/src/data/eastmoney.rs` that proves `trends2/get` rows are mapped by EastMoney semantics instead of OHLC semantics.

```rust
#[test]
fn intraday_parser_maps_trends_rows_into_points() {
    let payload = r#"miniquotechart_jp0({"data":{"name":"比亚迪","trends":[
        "2026-03-16 09:30,99.67,99.70,12345,456789,0.10,0.10,99.57",
        "2026-03-16 09:31,99.80,99.76,22345,556789,0.23,0.23,99.57"
    ]}})"#;

    let series = parse_intraday_series("002594.SZ", "1d", payload).unwrap();

    assert_eq!(series.pre_close, 99.57);
    assert_eq!(series.points.len(), 2);
    assert_eq!(series.points[0].timestamp, "2026-03-16 09:30");
    assert_eq!(series.points[0].price, 99.67);
    assert_eq!(series.points[0].avg_price, 99.70);
    assert_eq!(series.points[0].volume, 12345.0);
    assert_eq!(series.points[0].turnover, 456789.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test intraday_parser_maps_trends_rows_into_points -- --nocapture`
Expected: FAIL because `parse_intraday_series` and the new intraday response types do not exist yet.

**Step 3: Write minimal implementation**

Create `backend/src/models/intraday.rs` with dedicated response types and export them from `backend/src/models/mod.rs`.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayPoint {
    pub timestamp: String,
    pub price: f64,
    pub avg_price: f64,
    pub volume: f64,
    pub turnover: f64,
    pub change_pct: Option<f64>,
    pub change: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradaySeries {
    pub symbol: String,
    pub name: String,
    pub range: String,
    pub pre_close: f64,
    pub points: Vec<IntradayPoint>,
}
```

In `backend/src/data/eastmoney.rs`, add `parse_intraday_series` that:

```rust
fn parse_intraday_series(symbol: &str, range: &str, payload: &str) -> Result<IntradaySeries> {
    let parsed: Value = serde_json::from_str(strip_jsonp_wrapper(payload))?;
    let data = parsed.get("data").cloned().unwrap_or(Value::Null);
    let name = data.get("name").and_then(|v| v.as_str()).unwrap_or(symbol).to_string();

    let mut points = data
        .get("trends")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|row| row.as_str())
        .filter_map(|row| {
            let parts: Vec<&str> = row.split(',').collect();
            if parts.len() < 8 {
                return None;
            }
            Some(IntradayPoint {
                timestamp: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                avg_price: parts[2].parse().ok()?,
                volume: parts[3].parse().unwrap_or(0.0),
                turnover: parts[4].parse().unwrap_or(0.0),
                change_pct: parts[5].parse().ok(),
                change: parts[6].parse().ok(),
            })
        })
        .collect::<Vec<_>>();

    points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let pre_close = points.first().and_then(|_| {
        data.get("trends")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.first())
            .and_then(|row| row.as_str())
            .and_then(|row| row.split(',').nth(7))
            .and_then(|v| v.parse().ok())
    }).unwrap_or(0.0);

    Ok(IntradaySeries { symbol: symbol.to_string(), name, range: range.to_string(), pre_close, points })
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test intraday_parser_maps_trends_rows_into_points -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add backend/src/models/intraday.rs backend/src/models/mod.rs backend/src/data/eastmoney.rs
git commit -m "feat: add EastMoney intraday parser"
```

### Task 2: Backend intraday fetcher and API route

**Files:**
- Modify: `backend/src/data/eastmoney.rs`
- Modify: `backend/src/data/provider.rs`
- Modify: `backend/src/api/routes.rs`
- Test: `backend/src/data/eastmoney.rs`

**Step 1: Write the failing test**

Add a helper test proving range normalization and endpoint URL selection are correct.

```rust
#[test]
fn intraday_range_maps_to_expected_ndays() {
    assert_eq!(intraday_ndays("1d"), 1);
    assert_eq!(intraday_ndays("5d"), 5);
    assert_eq!(intraday_ndays("unexpected"), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test intraday_range_maps_to_expected_ndays -- --nocapture`
Expected: FAIL because `intraday_ndays` does not exist yet.

**Step 3: Write minimal implementation**

In `backend/src/data/eastmoney.rs`, add a dedicated fetcher:

```rust
fn intraday_ndays(range: &str) -> u32 {
    match range {
        "5d" => 5,
        _ => 1,
    }
}

pub async fn get_intraday(&self, symbol: &str, range: &str) -> Result<IntradaySeries> {
    let (market, code) = parse_symbol(symbol);
    let secid = format!("{}.{}", market, code);
    let ndays = intraday_ndays(range);
    let url = format!(
        "https://push2.eastmoney.com/api/qt/stock/trends2/get?fields1=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13,f14,f17&fields2=f51,f52,f53,f54,f55,f56,f57,f58&dect=1&mpi=1000&ut=fa5fd1943c7b386f172d6893dbfba10b&secid={}&ndays={}&iscr=0&iscca=0&cb=miniquotechart_jp0",
        secid,
        ndays,
    );
    let text = self.client.get(&url).header("Referer", "https://quote.eastmoney.com/").send().await?.text().await?;
    parse_intraday_series(symbol, range, &text)
}
```

In `backend/src/data/provider.rs`, expose:

```rust
pub async fn get_intraday(&self, symbol: &str, range: &str) -> Result<IntradaySeries> {
    self.api.get_intraday(symbol, range).await
}
```

In `backend/src/api/routes.rs`, add:

```rust
#[derive(Deserialize)]
pub struct IntradayParams {
    pub range: Option<String>,
}

async fn get_intraday(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<IntradayParams>,
) -> Json<ApiResponse<Option<IntradaySeries>>> {
    let range = params.range.unwrap_or_else(|| "1d".to_string());
    match state.provider.get_intraday(&symbol, &range).await {
        Ok(series) => ok_response(Some(series)),
        Err(e) => {
            tracing::warn!("Failed to get intraday data for {}: {}", symbol, e);
            ok_response(None)
        }
    }
}
```

Also register `.route("/api/intraday/{symbol}", get(get_intraday))` in `create_router`.

**Step 4: Run test to verify it passes**

Run: `cargo test intraday_ -- --nocapture`
Expected: PASS for the new backend intraday tests.

**Step 5: Commit**

```bash
git add backend/src/data/eastmoney.rs backend/src/data/provider.rs backend/src/api/routes.rs
git commit -m "feat: expose intraday api"
```

### Task 3: Frontend intraday transform and ECharts option builder

**Files:**
- Modify: `frontend/package.json`
- Create: `frontend/client/src/lib/intradayChart.ts`
- Create: `frontend/client/src/lib/__tests__/intradayChart.test.ts`
- Test: `frontend/client/src/lib/__tests__/intradayChart.test.ts`

**Step 1: Write the failing test**

Create a pure-data test for both `分时` and `5日` behavior.

```ts
import { describe, expect, it } from 'vitest';
import { buildIntradayChartModel, buildIntradayOption } from '../intradayChart';

const series = {
  symbol: '002594.SZ',
  name: '比亚迪',
  range: '1d',
  preClose: 99.57,
  points: [
    { timestamp: '2026-03-16 09:30', price: 99.67, avgPrice: 99.70, volume: 12345, turnover: 456789, changePct: 0.1, change: 0.1 },
    { timestamp: '2026-03-16 09:31', price: 99.80, avgPrice: 99.76, volume: 22345, turnover: 556789, changePct: 0.23, change: 0.23 },
  ],
} as const;

describe('buildIntradayChartModel', () => {
  it('builds aligned percent range and volume colors', () => {
    const model = buildIntradayChartModel(series);
    expect(model.priceAxis.min).toBeLessThan(99.57);
    expect(model.priceAxis.max).toBeGreaterThan(99.57);
    expect(model.volumeColors).toEqual(['#ef4444', '#ef4444']);
  });

  it('creates an echarts option with 2 line series and 1 bar series', () => {
    const option = buildIntradayOption(series);
    expect(option.series).toHaveLength(3);
    expect(option.xAxis).toHaveLength(2);
    expect(option.yAxis).toHaveLength(3);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir frontend exec vitest run client/src/lib/__tests__/intradayChart.test.ts`
Expected: FAIL because the module and functions do not exist yet.

**Step 3: Write minimal implementation**

Add the chart dependencies first:

```bash
pnpm --dir frontend add echarts echarts-for-react
```

Then create `frontend/client/src/lib/intradayChart.ts` with a pure transform and option builder.

```ts
export function buildIntradayChartModel(series: IntradaySeries) {
  const prices = series.points.map((point) => point.price);
  const maxDelta = Math.max(...prices.map((price) => Math.abs(price - series.preClose)), 0.01);
  const min = series.preClose - maxDelta;
  const max = series.preClose + maxDelta;

  return {
    labels: series.points.map((point) => point.timestamp),
    prices,
    avgPrices: series.points.map((point) => point.avgPrice),
    volumes: series.points.map((point) => point.volume),
    volumeColors: series.points.map((point, index, all) =>
      index === 0 || point.price >= all[index - 1].price ? '#ef4444' : '#22c55e'
    ),
    priceAxis: { min, max },
    percentAxis: {
      min: ((min - series.preClose) / series.preClose) * 100,
      max: ((max - series.preClose) / series.preClose) * 100,
    },
  };
}
```

Build `buildIntradayOption` on top of that model and keep all chart math out of the component.

**Step 4: Run test to verify it passes**

Run: `pnpm --dir frontend exec vitest run client/src/lib/__tests__/intradayChart.test.ts`
Expected: PASS.

Run: `pnpm --dir frontend check`
Expected: PASS.

**Step 5: Commit**

```bash
git add frontend/package.json frontend/pnpm-lock.yaml frontend/client/src/lib/intradayChart.ts frontend/client/src/lib/__tests__/intradayChart.test.ts
git commit -m "feat: add intraday chart model and options"
```

### Task 4: Frontend intraday chart component and modal integration

**Files:**
- Create: `frontend/client/src/components/IntradayChart.tsx`
- Create: `frontend/client/src/lib/stockDetailChartMode.ts`
- Create: `frontend/client/src/lib/__tests__/stockDetailChartMode.test.ts`
- Modify: `frontend/client/src/components/StockDetailModal.tsx`
- Test: `frontend/client/src/lib/__tests__/stockDetailChartMode.test.ts`

**Step 1: Write the failing test**

Create a pure helper test for mode-to-request mapping so the modal's fetch split is covered without a DOM test harness.

```ts
import { describe, expect, it } from 'vitest';
import { getStockDetailChartRequest } from '../stockDetailChartMode';

describe('getStockDetailChartRequest', () => {
  it('maps intraday modes to the dedicated API and day candles to candle API', () => {
    expect(getStockDetailChartRequest('002594.SZ', 'intraday')).toEqual('/api/intraday/002594.SZ?range=1d');
    expect(getStockDetailChartRequest('002594.SZ', '5d')).toEqual('/api/intraday/002594.SZ?range=5d');
    expect(getStockDetailChartRequest('002594.SZ', '1d')).toEqual('/api/candles/002594.SZ?period=1d&count=90');
  });
});
```

**Step 2: Run test to verify it fails**

Run: `pnpm --dir frontend exec vitest run client/src/lib/__tests__/stockDetailChartMode.test.ts`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

Create `frontend/client/src/lib/stockDetailChartMode.ts`:

```ts
export type StockDetailChartMode = 'intraday' | '5d' | '1d';

export function getStockDetailChartRequest(symbol: string, mode: StockDetailChartMode) {
  if (mode === 'intraday') return `/api/intraday/${symbol}?range=1d`;
  if (mode === '5d') return `/api/intraday/${symbol}?range=5d`;
  return `/api/candles/${symbol}?period=1d&count=90`;
}
```

Create `frontend/client/src/components/IntradayChart.tsx` as a thin wrapper around `ReactECharts`:

```tsx
import ReactECharts from 'echarts-for-react';
import { buildIntradayOption } from '@/lib/intradayChart';

export default function IntradayChart({ data }: { data: IntradaySeries }) {
  return <ReactECharts option={buildIntradayOption(data)} style={{ height: 320, width: '100%' }} notMerge lazyUpdate />;
}
```

Update `frontend/client/src/components/StockDetailModal.tsx` to:

- replace the current `period` state with `mode: 'intraday' | '5d' | '1d'`
- fetch `/api/intraday/{symbol}` for `intraday` and `5d`
- keep `/api/candles/{symbol}` for `1d`
- render `IntradayChart` for intraday modes
- keep the existing candle chart for `1d`
- cancel stale requests during rapid symbol or mode changes

**Step 4: Run test to verify it passes**

Run: `pnpm --dir frontend exec vitest run client/src/lib/__tests__/stockDetailChartMode.test.ts client/src/lib/__tests__/intradayChart.test.ts`
Expected: PASS.

Run: `pnpm --dir frontend check`
Expected: PASS.

**Step 5: Commit**

```bash
git add frontend/client/src/components/IntradayChart.tsx frontend/client/src/lib/stockDetailChartMode.ts frontend/client/src/lib/__tests__/stockDetailChartMode.test.ts frontend/client/src/components/StockDetailModal.tsx
git commit -m "feat: integrate EastMoney intraday chart"
```

### Task 5: End-to-end verification and cleanup

**Files:**
- Modify: `docs/plans/2026-03-16-eastmoney-standard-intraday-design.md`
- Modify: `docs/plans/2026-03-16-eastmoney-standard-intraday.md`

**Step 1: Run backend verification**

Run: `cargo test intraday_ -- --nocapture`
Expected: PASS for all new backend intraday tests.

**Step 2: Run frontend verification**

Run: `pnpm --dir frontend exec vitest run client/src/lib/__tests__/intradayChart.test.ts client/src/lib/__tests__/stockDetailChartMode.test.ts`
Expected: PASS.

Run: `pnpm --dir frontend check`
Expected: PASS.

**Step 3: Manual verification**

Open the stock detail modal and verify:

- `分时` shows price line, average line, aligned percent axis, and volume bars.
- `5日` loads multi-day minute data with date grouping.
- `日线` still uses the old candle path.
- Fast mode switching does not flash stale data.
- Empty or failed intraday responses show a stable empty state.

**Step 4: Update plan notes**

Add short implementation notes to this plan doc describing any deviations discovered during execution.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-16-eastmoney-standard-intraday-design.md docs/plans/2026-03-16-eastmoney-standard-intraday.md
git commit -m "docs: finalize EastMoney intraday implementation notes"
```
