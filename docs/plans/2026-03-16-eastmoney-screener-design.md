# EastMoney Screener Workbench Design

**Date:** 2026-03-16

**Goal:** Build an EastMoney-style stock screener workbench inside Quantrust with a visual rule builder, local template persistence, and EastMoney screener link import mapping.

## Scope

- Replace the current simple screener panel with a visual workbench.
- Introduce a unified internal screener definition model for visual editing, execution, persistence, and import mapping.
- Add backend APIs for screener catalog, execution, template CRUD, and EastMoney link import.
- Reuse the existing quote cache as the primary execution data source for the first version.
- Support high-compatibility import of EastMoney screener links, with explicit reporting for unsupported conditions.

## Product Direction

The target is not a cosmetic clone of EastMoney's page. The target is a durable local screener system that preserves EastMoney's usage model:

- visual condition building
- grouped logical conditions
- common condition shortcuts
- result sorting and column selection
- reusable saved templates
- importing EastMoney screener links into local rules

Primary usage is visual. Script-first authoring is not part of this iteration.

## Current Problems

- The current screener panel only supports a few numeric filters and a fixed result table.
- The backend `/api/screener` path only filters cached quotes with a flat request model.
- There is no condition catalog, no grouped rule tree, no template storage, and no import flow.
- Existing strategy selection and screener selection are separate concepts and cannot share saved definitions.
- The system cannot accept an EastMoney screener link and convert it into an internal template.

## Architecture

The new screener system should be built around one internal model: `ScreenerDefinition`.

`ScreenerDefinition` is the single source of truth for:

- frontend visual editing
- backend validation
- rule execution
- template persistence
- EastMoney import output

The architecture has four layers:

1. `Condition Catalog`
   - Defines which fields can be screened.
   - Declares category, type, supported operators, source, and compatibility metadata.

2. `Rule Tree`
   - Represents user-selected conditions as nested groups.
   - Supports `AND` and `OR` groups plus leaf conditions.

3. `Execution Engine`
   - Validates rules.
   - Resolves direct quote fields and derived fields.
   - Filters and sorts cached stock data.

4. `Persistence and Import`
   - Saves local screener templates.
   - Imports EastMoney links into internal definitions plus warning metadata.

This design keeps the frontend flexible while preventing the EastMoney import flow from becoming the actual internal model.

## Internal Data Model

### Screener Definition

The internal shape should be conceptually equivalent to:

```ts
type ScreenerDefinition = {
  name?: string;
  description?: string;
  logic: ScreenerGroup;
  sorts: ScreenerSort[];
  columns: string[];
  source?: 'manual' | 'eastmoney_import';
  importMeta?: {
    originalUrl?: string;
    importedConditions: number;
    unsupportedConditions: Array<{
      key: string;
      reason: string;
      raw?: string;
    }>;
  };
};
```

### Rule Tree

```ts
type ScreenerGroup = {
  id: string;
  operator: 'AND' | 'OR';
  children: Array<ScreenerGroup | ScreenerCondition>;
};

type ScreenerCondition = {
  id: string;
  field: string;
  operator: string;
  value: string | number | boolean | string[] | [number, number];
};
```

### Condition Catalog Metadata

Each catalog entry must describe:

- `field`
- `label`
- `category`
- `valueType`
- `operators`
- `dataSource`
- `eastmoneyCompatible`
- `status`: `ready | derived | unavailable`

This allows the frontend to render valid editors and the backend to reject unsupported combinations early.

## Compatibility Strategy

High compatibility with EastMoney means:

- matching the overall visual workflow
- supporting a large subset of common conditions
- accepting EastMoney links as import input
- preserving unsupported conditions as explicit warnings

It does not mean promising identical results for all EastMoney strategies on day one.

### Compatibility Tiers

`P0` direct quote fields from current cache:

- latest price
- change amount and change percent
- volume
- turnover
- turnover rate
- amplitude
- PE
- total market cap
- circulating market cap
- symbol and name
- limit-up or limit-down style derived flags

`P1` derived from current quote cache and local computations:

- simple price buckets
- relative intraday position
- approximate volume ratio based on cached context
- short-window ranking metrics
- simple moving-average style fields once candle support is wired into the engine

`P2` deferred until additional data sources exist:

- real ROE
- revenue growth
- profit growth
- full PB and PS correctness
- institutional flow refinements
- advanced chip-distribution or financial-quality fields

`P2` fields should appear in the catalog only when they can be backed by real data, not placeholders.

## EastMoney Import Design

Import should be handled by a dedicated API, not folded into general screener execution.

### Import Behavior

- Accept a full EastMoney screener URL.
- Parse known query segments and encoded condition identifiers.
- Map recognized conditions into local catalog fields and operators.
- Return a generated `ScreenerDefinition`.
- Return import diagnostics:
  - imported condition count
  - unsupported condition count
  - unsupported condition details

### Import Principle

EastMoney is an external rule representation. Quantrust should import it into local rules, not execute it remotely or bind its runtime to EastMoney page semantics.

This keeps the local screener stable even if EastMoney UI parameters change.

## Backend Design

### New API Surface

- `GET /api/screener/catalog`
- `POST /api/screener/run`
- `POST /api/screener/import-eastmoney`
- `GET /api/screener/templates`
- `POST /api/screener/templates`
- `PUT /api/screener/templates/{id}`
- `DELETE /api/screener/templates/{id}`

The existing simple `/api/screener` route should either be retired or reimplemented as a thin compatibility wrapper over the new execution engine.

### Execution Flow

1. Receive `ScreenerDefinition`.
2. Validate fields, operators, and value types against the catalog.
3. Build derived values for fields marked `derived`.
4. Evaluate the rule tree against cached quotes.
5. Apply sorting and result-column projection.
6. Return paginated result rows plus metadata.

### Execution Data Source

The first version should execute primarily against `ScannerCache.all_quotes` from [scanner.rs](/F:/my-cursor-project/quantrust/backend/src/services/scanner.rs). This keeps execution fast and independent from per-request external fetches.

Derived metrics should be computed in a dedicated screener service layer rather than scattered inside route handlers.

### Persistence

Add dedicated tables rather than overloading the existing strategy tables:

- `screener_templates`
- `screener_runs` optional but recommended for recent runs and import diagnostics

Template records should store JSON payloads for:

- rule tree
- sort definitions
- result columns
- import metadata

## Frontend Workbench Design

The current [ScreenerPanel.tsx](/F:/my-cursor-project/quantrust/frontend/client/src/components/panels/ScreenerPanel.tsx) should become a full workbench rather than a flat filter form.

### Layout

- top toolbar
  - run
  - clear
  - save template
  - import EastMoney link
- left condition builder
  - rule groups
  - condition cards
  - `AND` or `OR` toggles
  - category browser
  - common-condition shortcuts
- right result area
  - result count
  - sort selectors
  - column manager
  - result table
- side drawer or tabbed area for templates
  - my templates
  - imported templates
  - recent runs

### Interaction Rules

- adding a field should present only valid operators
- changing a field should reset invalid operator or value state
- invalid conditions should block execution and be highlighted inline
- imported EastMoney links should open an import report before execution
- result rows should preserve current stock-detail open behavior and watchlist actions

### Relationship to Strategy Panel

This workbench does not replace the existing strategy panel immediately. It creates a reusable screener definition format that later strategy features can reference.

That allows gradual convergence instead of forcing the current strategy panel to change in the same iteration.

## Error Handling

### Import Errors

- malformed URL
- unsupported EastMoney parameter layout
- recognized link with partially unsupported conditions

The frontend must show an import report instead of silently dropping conditions.

### Validation Errors

- unknown field
- unsupported operator
- invalid value type
- incomplete grouped logic

These errors should be returned with enough condition identity to map back to a specific UI card.

### Data Availability Errors

Unavailable data-backed fields must not be fake-calculated. They should remain disabled or marked unsupported until implemented.

## Testing Strategy

### Backend

- unit tests for catalog validation
- unit tests for nested rule-tree evaluation
- unit tests for derived-field computation
- unit tests for EastMoney import mapping
- route tests for catalog, run, import, and template CRUD

### Frontend

- component tests for condition-card editing and group logic
- tests for import-report rendering
- tests for result-column selection and sort updates
- tests for template load and save workflows

### Manual Verification

Use several real EastMoney screener links and verify:

- import succeeds where expected
- unsupported conditions are reported explicitly
- local condition structure matches user intent
- result counts are directionally reasonable for supported fields

Exact stock-for-stock parity is not required for first release. Semantic compatibility and explicit diagnostics are required.

## Non-Goals

- no script-first screener DSL in this iteration
- no promise of full EastMoney field coverage on day one
- no fake financial-factor support without actual data sources
- no merge of all strategy features into the screener panel in the first version

## Success Criteria

- A user can visually build and run complex grouped screeners.
- A user can save and reload screener templates locally.
- A user can paste an EastMoney screener link and receive a local template plus diagnostics.
- Unsupported imported conditions are visible and actionable.
- The screener is extensible through catalog additions instead of ad hoc frontend and backend patches.
