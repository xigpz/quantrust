# EastMoney Standard Intraday Chart Design

**Date:** 2026-03-16

**Goal:** Rebuild the stock detail intraday chart so it matches EastMoney's standard stock timeshare chart behavior, including `分时`, `5日`, and `日线` mode separation.

## Scope

- Replace the current pseudo-intraday line in the stock detail modal.
- Add a dedicated intraday data flow for EastMoney `trends2/get`.
- Support `分时` and `5日` with EastMoney-style rendering and interactions.
- Keep `日线` on the existing candle API path.

## Current Problems

- Backend currently maps `trends2/get` rows into the `Candle` model as if they were OHLC rows.
- Frontend reuses the candle chart path and only renders a simplified line plus volume view.
- The current implementation cannot accurately represent EastMoney intraday semantics such as average price, pre-close baseline, or five-day continuous minute data.

## Architecture

The intraday chart must be separated from the K-line chart path.

- Backend adds a dedicated intraday endpoint:
  - `GET /api/intraday/{symbol}?range=1d|5d`
- Frontend adds a dedicated `IntradayChart` component implemented with `ECharts`.
- Stock detail modal exposes three chart modes:
  - `分时`
  - `5日`
  - `日线`
- `分时` and `5日` use the new intraday endpoint and chart component.
- `日线` keeps using `/api/candles/{symbol}` and the existing candle flow.

This split avoids mixing timeshare data semantics with OHLC candles and keeps the K-line path stable.

## Backend Contract

The backend returns a dedicated intraday payload instead of `Candle[]`.

```ts
type IntradayResponse = {
  symbol: string;
  name: string;
  range: '1d' | '5d';
  preClose: number;
  points: Array<{
    timestamp: string;
    price: number;
    avgPrice: number;
    volume: number;
    turnover: number;
    changePct?: number;
    change?: number;
  }>;
};
```

### EastMoney Mapping

Map EastMoney `trends2/get` fields by documented meaning:

- `f51` -> `timestamp`
- `f52` -> `price`
- `f53` -> `avgPrice`
- `f54` -> `volume`
- `f55` -> `turnover`
- `f56` -> `changePct`
- `f57` -> `change`
- `f58` -> `preClose`

### Range Rules

- `range=1d` -> `ndays=1`
- `range=5d` -> `ndays=5`

### Backend Behavior

- Preserve raw trend points and return them in ascending timestamp order.
- Extract `preClose` once and expose it at response level.
- Return empty `points` on malformed or empty trend data instead of failing the modal.
- Keep multi-day points continuous so the frontend can control day segmentation and hover behavior.

## Frontend Chart Design

Use a dedicated `IntradayChart` component backed by `ECharts`.

### Layout

- Upper grid: price chart
  - price line
  - average price line
  - pre-close baseline
  - left Y-axis for price
  - right Y-axis for change percent
- Lower grid: volume bars

### Modes

- `分时` uses a single trading day.
- `5日` uses five-day minute data in one continuous chart.
- `日线` continues to use the existing candle chart.

### Interaction

- Shared crosshair across price and volume areas.
- Tooltip shows:
  - time
  - price
  - average price
  - change
  - change percent
  - volume
  - turnover
- `分时` uses fixed key time ticks:
  - `09:30`
  - `10:30`
  - `11:30/13:00`
  - `14:00`
  - `15:00`
- `5日` groups the X-axis by trading day while preserving minute continuity.
- Lunch break is represented as a discontinuity in trading time, not filled with fake points.

### Visual Rules

- Volume bars use red when the current price is greater than or equal to the previous point, otherwise green.
- Price axis range is symmetric around `preClose` so left price and right percent stay aligned.
- Keep the visual style clean and information-dense rather than decorative.

## Error Handling

- If intraday fetch fails, the stock detail modal shows an inline empty state for the selected intraday mode.
- Fast symbol or mode switching must avoid stale data flashing into the chart.
- Chart initialization and teardown must not leak instances when the dialog closes or switches mode.

## Testing

### Backend

- Add parser tests for `ndays=1` and `ndays=5` payloads.
- Verify JSONP stripping.
- Verify field mapping for `price`, `avgPrice`, `volume`, `turnover`, and `preClose`.
- Verify ascending ordering and empty-data handling.

### Frontend

- Add tests for the intraday data-to-chart transform logic.
- Verify axis labels and mode switching behavior.
- Verify loading, empty, and error states in the stock detail modal.

## Non-Goals

- No attempt to force intraday data through the existing `Candle` model.
- No redesign of the daily K-line path in this task.
- No expansion into additional EastMoney subviews such as tick-by-tick deals in this iteration.
