import {
  getScreenerFieldLabel,
  getScreenerOperatorLabel,
  translateScreenerMessage,
  type ImportedConditionWarning,
  type ScreenerCatalogField,
  type ScreenerCondition,
  type ScreenerOperator,
  type ScreenerValue,
} from "@/lib/screener";

interface ConditionCardProps {
  condition: ScreenerCondition;
  catalog: ScreenerCatalogField[];
  error?: string;
  warning?: ImportedConditionWarning;
  onChange: (next: ScreenerCondition) => void;
  onRemove: () => void;
}

const FALLBACK_OPERATORS: ScreenerOperator[] = [">=", "<=", "=", "between", "contains"];

function stringifyValue(value: ScreenerValue): string {
  if (Array.isArray(value)) {
    return value.join("..");
  }

  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }

  return value === undefined || value === null ? "" : String(value);
}

function parseValue(operator: ScreenerOperator, raw: string): ScreenerValue {
  if (operator === "between") {
    const [min = "", max = ""] = raw.split("..");
    return [Number(min), Number(max)];
  }

  const numeric = Number(raw);
  if (raw.trim() !== "" && Number.isFinite(numeric) && operator !== "contains") {
    return numeric;
  }

  return raw;
}

export default function ConditionCard({ condition, catalog, error, warning, onChange, onRemove }: ConditionCardProps) {
  const fieldMeta = catalog.find((entry) => entry.field === condition.field);
  const operators = fieldMeta?.operators.length ? fieldMeta.operators : FALLBACK_OPERATORS;

  return (
    <div
      data-testid="condition-card"
      className={`rounded-xl border p-3 space-y-2 ${error ? "border-red-500/60 bg-red-500/5" : "border-border bg-background/70"}`}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs font-medium text-foreground">条件</div>
        <button
          type="button"
          data-testid={`remove-node-${condition.id}`}
          onClick={onRemove}
          className="rounded-md border border-border px-2 py-1 text-[11px] text-muted-foreground hover:text-foreground"
        >
          删除
        </button>
      </div>

      <div className="grid gap-2 md:grid-cols-[1.3fr,0.9fr,1fr]">
        <label className="grid gap-1 text-[11px] text-muted-foreground">
          字段
          <select
            value={condition.field}
            onChange={(event) => {
              const nextField = event.target.value;
              const nextMeta = catalog.find((entry) => entry.field === nextField);
              const nextOperator = nextMeta?.operators[0] ?? operators[0];
              onChange({ ...condition, field: nextField, operator: nextOperator });
            }}
            className="rounded-md border border-border bg-card px-2 py-2 text-xs text-foreground"
          >
            <option value="">选择字段</option>
            {catalog.map((entry) => (
              <option key={entry.field} value={entry.field}>
                {getScreenerFieldLabel(entry.field, entry.label)}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-1 text-[11px] text-muted-foreground">
          运算符
          <select
            value={condition.operator}
            onChange={(event) => onChange({ ...condition, operator: event.target.value as ScreenerOperator })}
            className="rounded-md border border-border bg-card px-2 py-2 text-xs text-foreground"
          >
            {operators.map((operator) => (
              <option key={operator} value={operator}>
                {getScreenerOperatorLabel(operator)}
              </option>
            ))}
          </select>
        </label>

        <label className="grid gap-1 text-[11px] text-muted-foreground">
          值
          <input
            value={stringifyValue(condition.value)}
            onChange={(event) => onChange({ ...condition, value: parseValue(condition.operator, event.target.value) })}
            placeholder={condition.operator === "between" ? "最小值..最大值" : "输入值"}
            className="rounded-md border border-border bg-card px-2 py-2 text-xs text-foreground"
          />
        </label>
      </div>

      {warning ? <div className="text-[11px] text-amber-300">导入提示：{translateScreenerMessage(warning.reason)}</div> : null}
      {error ? <div className="text-[11px] text-red-300">{translateScreenerMessage(error)}</div> : null}
    </div>
  );
}
