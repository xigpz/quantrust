import { API_BASE, type ApiResponse } from "@/hooks/useMarketData";

export type ScreenerLogic = "AND" | "OR";
export type ScreenerOperator = ">" | ">=" | "<" | "<=" | "=" | "between" | "in" | "contains";
export type ScreenerSource = "manual" | "eastmoney_import";
export type ScreenerSortDirection = "asc" | "desc";
export type ScreenerValue = number | string | boolean | string[] | number[] | [number, number];

export interface ImportedConditionWarning {
  key: string;
  reason: string;
  raw?: string;
}

export interface ScreenerImportMeta {
  originalUrl?: string;
  importedConditions: number;
  unsupportedConditions: ImportedConditionWarning[];
}

export interface ScreenerCondition {
  id: string;
  field: string;
  operator: ScreenerOperator;
  value: ScreenerValue;
}

export interface ScreenerGroup {
  id: string;
  operator: ScreenerLogic;
  children: ScreenerNode[];
}

export type ScreenerNode = ScreenerGroup | ScreenerCondition;

export interface ScreenerSort {
  field: string;
  direction: ScreenerSortDirection;
}

export interface ScreenerDefinition {
  name?: string;
  description?: string;
  logic: ScreenerGroup;
  sorts: ScreenerSort[];
  columns: string[];
  source?: ScreenerSource;
  importMeta?: ScreenerImportMeta;
}

export interface ScreenerCatalogField {
  field: string;
  label: string;
  category: string;
  valueType: "number" | "range" | "enum" | "boolean" | "text";
  operators: ScreenerOperator[];
  dataSource: string;
  eastmoneyCompatible: boolean;
  status: "ready" | "derived" | "unavailable";
}

export interface ScreenerResultRow {
  [key: string]: string | number | boolean | string[] | number[] | null | undefined;
}

export interface ScreenerExecutionResult {
  total_count: number;
  rows: ScreenerResultRow[];
}

export interface ScreenerTemplateRecord {
  id: string;
  name: string;
  description?: string;
  definition: ScreenerDefinition;
  source_type: string;
  created_at: string;
  updated_at: string;
}

export interface ScreenerTemplateInput {
  name: string;
  description?: string;
  definition: ScreenerDefinition;
  sourceType?: string;
}

export interface ScreenerRunPayload {
  definition: ScreenerDefinition;
  limit?: number;
}

export interface ScreenerTemplatePayload {
  name: string;
  description?: string;
  definition: ScreenerDefinition;
  source_type: string;
}

type FetchLike = (input: RequestInfo | URL, init?: RequestInit) => Promise<{ json(): Promise<ApiResponse<any>> }>;

function normalizeImportMeta(importMeta?: Partial<ScreenerImportMeta>): ScreenerImportMeta | undefined {
  if (!importMeta) {
    return undefined;
  }

  return {
    originalUrl: importMeta.originalUrl,
    importedConditions: importMeta.importedConditions ?? 0,
    unsupportedConditions: (importMeta.unsupportedConditions ?? []).map((warning) => ({
      key: warning.key,
      reason: warning.reason,
      raw: warning.raw,
    })),
  };
}

export function normalizeImportedScreenerDefinition(definition: ScreenerDefinition): ScreenerDefinition {
  return {
    ...definition,
    sorts: definition.sorts ?? [],
    columns: definition.columns ?? [],
    importMeta: normalizeImportMeta(definition.importMeta),
  };
}

export function buildScreenerRunPayload(definition: ScreenerDefinition, limit?: number): ScreenerRunPayload {
  return {
    definition: normalizeImportedScreenerDefinition(definition),
    ...(limit === undefined ? {} : { limit }),
  };
}

export function buildScreenerTemplatePayload(input: ScreenerTemplateInput): ScreenerTemplatePayload {
  return {
    name: input.name,
    description: input.description,
    definition: normalizeImportedScreenerDefinition(input.definition),
    source_type: input.sourceType ?? "manual",
  };
}

async function unwrapResponse<T>(promise: Promise<{ json(): Promise<ApiResponse<T>> }>): Promise<T> {
  const response = await promise;
  const payload = await response.json();
  if (!payload.success) {
    throw new Error(payload.message);
  }
  return payload.data;
}

export async function fetchScreenerCatalog(fetcher: FetchLike = fetch as FetchLike): Promise<ScreenerCatalogField[]> {
  return unwrapResponse(fetcher(`${API_BASE}/api/screener/catalog`));
}

export async function runScreenerDefinition(
  definition: ScreenerDefinition,
  limit?: number,
  fetcher: FetchLike = fetch as FetchLike,
): Promise<ScreenerExecutionResult> {
  return unwrapResponse(
    fetcher(`${API_BASE}/api/screener/run`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(buildScreenerRunPayload(definition, limit)),
    }),
  );
}

export async function importEastmoneyScreener(
  url: string,
  fetcher: FetchLike = fetch as FetchLike,
): Promise<ScreenerDefinition> {
  const definition = await unwrapResponse<ScreenerDefinition>(
    fetcher(`${API_BASE}/api/screener/import-eastmoney`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url }),
    }),
  );

  return normalizeImportedScreenerDefinition(definition);
}

export async function listScreenerTemplates(fetcher: FetchLike = fetch as FetchLike): Promise<ScreenerTemplateRecord[]> {
  return unwrapResponse(fetcher(`${API_BASE}/api/screener/templates`));
}

export async function createScreenerTemplate(
  input: ScreenerTemplateInput,
  fetcher: FetchLike = fetch as FetchLike,
): Promise<ScreenerTemplateRecord> {
  return unwrapResponse(
    fetcher(`${API_BASE}/api/screener/templates`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(buildScreenerTemplatePayload(input)),
    }),
  );
}

export async function updateScreenerTemplate(
  id: string,
  input: ScreenerTemplateInput,
  fetcher: FetchLike = fetch as FetchLike,
): Promise<ScreenerTemplateRecord> {
  return unwrapResponse(
    fetcher(`${API_BASE}/api/screener/templates/${encodeURIComponent(id)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(buildScreenerTemplatePayload(input)),
    }),
  );
}

export async function deleteScreenerTemplate(id: string, fetcher: FetchLike = fetch as FetchLike): Promise<string> {
  return unwrapResponse(
    fetcher(`${API_BASE}/api/screener/templates/${encodeURIComponent(id)}`, {
      method: "DELETE",
    }),
  );
}