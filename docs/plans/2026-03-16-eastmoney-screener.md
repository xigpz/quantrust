# EastMoney Screener Workbench Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an EastMoney-style visual screener workbench with internal rule modeling, local template persistence, and EastMoney link import mapping.

**Architecture:** Introduce a backend screener domain centered on a shared `ScreenerDefinition` model, then rebuild the frontend screener panel as a visual workbench that consumes the catalog, run, template, and import APIs. Keep execution local by evaluating rules against the existing market quote cache and explicit derived metrics.

**Tech Stack:** Rust, Axum, rusqlite, React 19, TypeScript, Vite, Vitest

---

### Task 1: Define backend screener domain models

**Files:**
- Create: `backend/src/models/screener.rs`
- Modify: `backend/src/models/mod.rs`
- Test: `backend/src/models/screener.rs`

**Step 1: Write the failing test**

Add model tests in `backend/src/models/screener.rs` that deserialize a nested `ScreenerDefinition` payload and assert:

- nested `AND` and `OR` groups are preserved
- numeric and range values deserialize correctly
- import metadata preserves unsupported conditions

**Step 2: Run test to verify it fails**

Run: `cargo test screener_definition --manifest-path backend/Cargo.toml`
Expected: FAIL because `backend/src/models/screener.rs` and the new model types do not exist yet.

**Step 3: Write minimal implementation**

Create serializable structs and enums for:

- `ScreenerDefinition`
- `ScreenerGroup`
- `ScreenerCondition`
- `ScreenerSort`
- `ScreenerCatalogField`
- `ImportedConditionWarning`

Export the module from `backend/src/models/mod.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test screener_definition --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/models/screener.rs backend/src/models/mod.rs
git commit -m "feat: add screener domain models"
```

### Task 2: Add screener template persistence

**Files:**
- Modify: `backend/src/db/mod.rs`
- Test: `backend/src/db/mod.rs`

**Step 1: Write the failing test**

Add a database initialization test in `backend/src/db/mod.rs` that opens a temporary database, runs initialization, and asserts the `screener_templates` table exists.

**Step 2: Run test to verify it fails**

Run: `cargo test screener_templates_table --manifest-path backend/Cargo.toml`
Expected: FAIL because the new table is not created yet.

**Step 3: Write minimal implementation**

Extend database initialization to create:

- `screener_templates`
  - `id`
  - `user_id`
  - `name`
  - `description`
  - `definition_json`
  - `source_type`
  - `created_at`
  - `updated_at`
- `screener_runs` if retained for diagnostics

Keep JSON storage explicit instead of splitting rules into many relational tables.

**Step 4: Run test to verify it passes**

Run: `cargo test screener_templates_table --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/db/mod.rs
git commit -m "feat: add screener template persistence"
```

### Task 3: Build the screener service catalog and validator

**Files:**
- Create: `backend/src/services/screener.rs`
- Modify: `backend/src/services/mod.rs`
- Test: `backend/src/services/screener.rs`

**Step 1: Write the failing test**

Add service tests that assert:

- the catalog includes ready fields such as `price`, `change_pct`, `volume`, `turnover_rate`, `pe_ratio`, `total_market_cap`
- invalid field and operator combinations are rejected
- unavailable fields are flagged as unsupported

**Step 2: Run test to verify it fails**

Run: `cargo test screener_catalog --manifest-path backend/Cargo.toml`
Expected: FAIL because the screener service does not exist yet.

**Step 3: Write minimal implementation**

Implement a `ScreenerService` that exposes:

- a static or constructed condition catalog
- request validation against field metadata
- error objects that carry condition ids for frontend highlighting

Do not add EastMoney parsing yet in this task.

**Step 4: Run test to verify it passes**

Run: `cargo test screener_catalog --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/services/screener.rs backend/src/services/mod.rs
git commit -m "feat: add screener catalog and validation"
```

### Task 4: Implement rule execution against cached quotes

**Files:**
- Modify: `backend/src/services/screener.rs`
- Test: `backend/src/services/screener.rs`

**Step 1: Write the failing test**

Add execution tests that use sample `StockQuote` data and assert:

- flat `AND` rules filter correctly
- nested `OR` groups filter correctly
- sort order is applied after filtering
- selected columns are projected into result rows

**Step 2: Run test to verify it fails**

Run: `cargo test screener_execution --manifest-path backend/Cargo.toml`
Expected: FAIL because execution behavior is not implemented yet.

**Step 3: Write minimal implementation**

Implement:

- rule-tree evaluation against `StockQuote`
- derived value helpers for first-pass fields
- result row generation independent from raw `StockQuote`
- sorting and limit handling

Keep the execution path pure and testable so routes stay thin.

**Step 4: Run test to verify it passes**

Run: `cargo test screener_execution --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/services/screener.rs
git commit -m "feat: add screener execution engine"
```

### Task 5: Add EastMoney link import mapping

**Files:**
- Modify: `backend/src/services/screener.rs`
- Test: `backend/src/services/screener.rs`

**Step 1: Write the failing test**

Add import-mapping tests that assert:

- a recognized EastMoney screener URL returns a `ScreenerDefinition`
- supported conditions are mapped into local fields
- unsupported conditions are collected into warnings
- malformed URLs return structured import errors

**Step 2: Run test to verify it fails**

Run: `cargo test eastmoney_import --manifest-path backend/Cargo.toml`
Expected: FAIL because import mapping is not implemented yet.

**Step 3: Write minimal implementation**

Add a parser and mapper that:

- extracts condition payload from the EastMoney URL
- maps known keys into local catalog fields
- returns `importMeta` with warning details for unsupported conditions

Do not promise full EastMoney field coverage; support only the first approved compatibility set.

**Step 4: Run test to verify it passes**

Run: `cargo test eastmoney_import --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/services/screener.rs
git commit -m "feat: add EastMoney screener import mapping"
```

### Task 6: Expose screener APIs through Axum routes

**Files:**
- Modify: `backend/src/api/routes.rs`
- Test: `backend/src/api/routes.rs`

**Step 1: Write the failing test**

Add route-level tests for:

- `GET /api/screener/catalog`
- `POST /api/screener/run`
- `POST /api/screener/import-eastmoney`
- template CRUD routes

Assert both success payloads and validation failure payloads.

**Step 2: Run test to verify it fails**

Run: `cargo test screener_routes --manifest-path backend/Cargo.toml`
Expected: FAIL because the new routes do not exist yet.

**Step 3: Write minimal implementation**

Refactor the old `/api/screener` behavior into the new route set and wire:

- catalog endpoint
- run endpoint
- import endpoint
- template list/create/update/delete endpoints

Keep handlers thin by delegating validation and execution to `ScreenerService`.

**Step 4: Run test to verify it passes**

Run: `cargo test screener_routes --manifest-path backend/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/api/routes.rs
git commit -m "feat: add screener API routes"
```

### Task 7: Add frontend screener API client types

**Files:**
- Modify: `frontend/client/src/hooks/useMarketData.ts`
- Create: `frontend/client/src/lib/screener.ts`
- Test: `frontend/client/src/lib/__tests__/screener.test.ts`

**Step 1: Write the failing test**

Add frontend tests that assert:

- run request payloads serialize correctly
- import response diagnostics normalize correctly
- template payloads preserve group logic and columns

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run client/src/lib/__tests__/screener.test.ts`
Expected: FAIL because the screener client utilities do not exist yet.

**Step 3: Write minimal implementation**

Create shared TypeScript types and small API helpers for:

- catalog fetch
- run request
- import request
- template CRUD

Keep UI state mapping out of this file; it should stay transport-focused.

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run client/src/lib/__tests__/screener.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add frontend/client/src/hooks/useMarketData.ts frontend/client/src/lib/screener.ts frontend/client/src/lib/__tests__/screener.test.ts
git commit -m "feat: add frontend screener client helpers"
```

### Task 8: Rebuild the screener panel as a visual workbench

**Files:**
- Modify: `frontend/client/src/components/panels/ScreenerPanel.tsx`
- Create: `frontend/client/src/components/panels/screener/ConditionBuilder.tsx`
- Create: `frontend/client/src/components/panels/screener/ConditionGroup.tsx`
- Create: `frontend/client/src/components/panels/screener/ConditionCard.tsx`
- Create: `frontend/client/src/components/panels/screener/ScreenerResultsTable.tsx`
- Create: `frontend/client/src/components/panels/screener/ScreenerTemplateDrawer.tsx`
- Test: `frontend/client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx`

**Step 1: Write the failing test**

Add component tests that assert:

- conditions can be added and removed
- group logic toggles between `AND` and `OR`
- invalid conditions block run action
- imported warning counts render in the UI

**Step 2: Run test to verify it fails**

Run: `pnpm exec vitest run client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx`
Expected: FAIL because the new workbench components do not exist yet.

**Step 3: Write minimal implementation**

Replace the existing flat form with:

- toolbar actions
- visual rule builder
- result table with configurable columns
- template drawer
- EastMoney import modal or inline entry point

Preserve current stock row click behavior and watchlist integration.

**Step 4: Run test to verify it passes**

Run: `pnpm exec vitest run client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add frontend/client/src/components/panels/ScreenerPanel.tsx frontend/client/src/components/panels/screener frontend/client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx
git commit -m "feat: rebuild screener panel as workbench"
```

### Task 9: Verify end-to-end behavior and clean up compatibility edges

**Files:**
- Modify: `backend/src/api/routes.rs`
- Modify: `backend/src/services/screener.rs`
- Modify: `frontend/client/src/components/panels/ScreenerPanel.tsx`
- Test: `backend/src/services/screener.rs`
- Test: `frontend/client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx`
- Test: `frontend/client/src/lib/__tests__/screener.test.ts`

**Step 1: Write the failing test**

Add or extend tests to cover:

- partially supported EastMoney imports
- template reload preserving nested rules
- execution errors mapping back to specific condition cards

**Step 2: Run test to verify it fails**

Run: `cargo test screener --manifest-path backend/Cargo.toml`
Expected: FAIL until compatibility edge cases are handled.

Run: `pnpm exec vitest run client/src/lib/__tests__/screener.test.ts client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx`
Expected: FAIL until UI and client handling are aligned.

**Step 3: Write minimal implementation**

Tighten:

- import diagnostics
- validation payload shape
- template normalization
- compatibility wrapper behavior for legacy screener calls if retained

**Step 4: Run test to verify it passes**

Run: `cargo test screener --manifest-path backend/Cargo.toml`
Expected: PASS

Run: `pnpm exec vitest run client/src/lib/__tests__/screener.test.ts client/src/components/panels/screener/__tests__/ConditionBuilder.test.tsx`
Expected: PASS

Run: `pnpm check`
Expected: PASS

**Step 5: Commit**

```bash
git add backend/src/api/routes.rs backend/src/services/screener.rs frontend/client/src/components/panels/ScreenerPanel.tsx frontend/client/src/components/panels/screener frontend/client/src/lib/screener.ts frontend/client/src/lib/__tests__/screener.test.ts
git commit -m "feat: finalize EastMoney screener workbench"
```

### Task 10: Manual verification against real EastMoney links

**Files:**
- Modify: `docs/plans/2026-03-16-eastmoney-screener.md`

**Step 1: Prepare verification cases**

Collect 3 to 5 representative EastMoney screener URLs:

- mostly supported basic-condition link
- mixed supported and unsupported link
- malformed or stale link

**Step 2: Run verification**

Run backend and frontend locally, then verify:

- import report appears
- supported conditions land in the builder
- unsupported conditions are visible
- execution returns plausible results
- saved templates reload exactly

**Step 3: Record outcomes**

Append a short verification note to this plan with:

- tested URLs
- supported condition counts
- unsupported condition counts
- notable mismatches

**Step 4: Commit**

```bash
git add docs/plans/2026-03-16-eastmoney-screener.md
git commit -m "docs: record screener verification results"
```
