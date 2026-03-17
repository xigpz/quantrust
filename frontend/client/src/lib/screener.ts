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

const FIELD_LABELS: Record<string, string> = {
  symbol: "代码",
  name: "名称",
  latest_price: "最新价",
  change_pct: "涨跌幅",
  volume: "成交量",
  turnover_rate: "换手率",
  pe_ratio: "市盈率",
  total_market_cap: "总市值",
  roe: "净资产收益率",
};

const OPERATOR_LABELS: Record<ScreenerOperator, string> = {
  ">": "大于",
  ">=": "大于等于",
  "<": "小于",
  "<=": "小于等于",
  "=": "等于",
  between: "区间",
  in: "属于",
  contains: "包含",
};

const LOGIC_LABELS: Record<ScreenerLogic, string> = {
  AND: "且",
  OR: "或",
};

const EXACT_MESSAGE_LABELS: Record<string, string> = {
  "Failed to load templates": "加载模板失败",
  "Failed to initialize screener workbench": "初始化选股工作台失败",
  "Failed to run screener": "运行选股失败",
  "Fix invalid conditions before running again": "请先修正无效条件后再运行",
  "Paste an EastMoney screener URL first": "请先粘贴东方财富选股链接",
  "Import failed": "导入失败",
  "Template saved": "模板已保存",
  "Failed to save template": "保存模板失败",
  "URL is not valid": "链接格式不正确",
  "Only EastMoney screener URLs are supported": "仅支持导入东方财富选股链接",
  "No importable EastMoney filters were found": "没有识别到可导入的东方财富筛选条件",
  "Unsupported operator for selected field": "所选字段不支持当前运算符",
  "unsupported filter format": "暂不支持的筛选格式",
  "field unavailable": "字段暂不可用",
  "unsupported field": "暂不支持该字段",
  "operator unsupported": "运算符暂不支持",
  "could not map condition": "暂时无法映射该条件",
  "remote strategy id import is not supported yet": "暂不支持通过远程策略 ID 导入",
};

type FetchLike = (input: RequestInfo | URL, init?: RequestInit) => Promise<{ json(): Promise<ApiResponse<any>> }>;

export function getScreenerFieldLabel(field: string, fallbackLabel?: string): string {
  return FIELD_LABELS[field] ?? fallbackLabel ?? field;
}

export function getScreenerOperatorLabel(operator: ScreenerOperator): string {
  return OPERATOR_LABELS[operator] ?? operator;
}

export function getScreenerLogicLabel(logic: ScreenerLogic): string {
  return LOGIC_LABELS[logic] ?? logic;
}

export function translateScreenerMessage(message: string): string {
  const direct = EXACT_MESSAGE_LABELS[message];
  if (direct) {
    return direct;
  }

  const unknownFieldMatch = message.match(/^Unknown screener field: (.+)$/);
  if (unknownFieldMatch) {
    return `未知选股字段：${getScreenerFieldLabel(unknownFieldMatch[1], unknownFieldMatch[1])}`;
  }

  const unavailableFieldMatch = message.match(/^Field (.+) is not available yet$/);
  if (unavailableFieldMatch) {
    return `${getScreenerFieldLabel(unavailableFieldMatch[1], unavailableFieldMatch[1])} 暂未可用`;
  }

  const unsupportedOperatorMatch = message.match(/^Operator is not supported for (.+)$/);
  if (unsupportedOperatorMatch) {
    return `${getScreenerFieldLabel(unsupportedOperatorMatch[1], unsupportedOperatorMatch[1])} 不支持当前运算符`;
  }

  return message;
}

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
