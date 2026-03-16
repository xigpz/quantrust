import { useEffect, useMemo, useRef, useState } from "react";
import type {
  ImportedConditionWarning,
  ScreenerCatalogField,
  ScreenerCondition,
  ScreenerDefinition,
  ScreenerGroup,
  ScreenerLogic,
  ScreenerNode,
} from "@/lib/screener";
import ConditionGroup from "./ConditionGroup";

interface ConditionBuilderProps {
  catalog: ScreenerCatalogField[];
  definition: ScreenerDefinition;
  importWarningCount: number;
  validationErrors?: Record<string, string>;
  onChange: (definition: ScreenerDefinition) => void;
  onRun: () => void;
}

function isGroupNode(node: ScreenerNode): node is ScreenerGroup {
  return "children" in node;
}

function walkNodes(group: ScreenerGroup, visit: (node: ScreenerNode) => void) {
  for (const child of group.children) {
    visit(child);
    if (isGroupNode(child)) {
      walkNodes(child, visit);
    }
  }
}

function computeNextId(definition: ScreenerDefinition) {
  let maxId = 0;
  walkNodes(definition.logic, (node) => {
    const numeric = Number(node.id.split("-").at(-1));
    if (Number.isFinite(numeric)) {
      maxId = Math.max(maxId, numeric);
    }
  });
  return maxId + 1;
}

function isValueFilled(value: ScreenerCondition["value"]): boolean {
  if (Array.isArray(value)) {
    return value.length === 2 && value.every((entry) => entry !== "" && !Number.isNaN(Number(entry)));
  }

  return value !== "" && value !== undefined && value !== null;
}

function isRunnable(definition: ScreenerDefinition): boolean {
  const visit = (group: ScreenerGroup): boolean => {
    if (group.children.length === 0) {
      return false;
    }

    return group.children.every((child) => {
      if (isGroupNode(child)) {
        return visit(child);
      }

      return Boolean(child.field) && Boolean(child.operator) && isValueFilled(child.value);
    });
  };

  return visit(definition.logic);
}

function replaceGroup(group: ScreenerGroup, targetId: string, update: (group: ScreenerGroup) => ScreenerGroup): ScreenerGroup {
  if (group.id === targetId) {
    return update(group);
  }

  return {
    ...group,
    children: group.children.map((child) => {
      if (!isGroupNode(child)) {
        return child;
      }
      return replaceGroup(child, targetId, update);
    }),
  };
}

function replaceCondition(group: ScreenerGroup, conditionId: string, next: ScreenerCondition): ScreenerGroup {
  return {
    ...group,
    children: group.children.map((child) => {
      if (isGroupNode(child)) {
        return replaceCondition(child, conditionId, next);
      }
      return child.id === conditionId ? next : child;
    }),
  };
}

function removeNode(group: ScreenerGroup, nodeId: string): ScreenerGroup {
  return {
    ...group,
    children: group.children
      .filter((child) => child.id !== nodeId)
      .map((child) => (isGroupNode(child) ? removeNode(child, nodeId) : child)),
  };
}

function collectImportedWarnings(definition: ScreenerDefinition): Record<string, ImportedConditionWarning> {
  const warnings = definition.importMeta?.unsupportedConditions ?? [];
  return warnings.reduce<Record<string, ImportedConditionWarning>>((accumulator, warning, index) => {
    accumulator[`imported-${index + 1}`] = warning;
    return accumulator;
  }, {});
}

export default function ConditionBuilder({
  catalog,
  definition,
  importWarningCount,
  validationErrors,
  onChange,
  onRun,
}: ConditionBuilderProps) {
  const [draft, setDraft] = useState(definition);
  const nextId = useRef(1);

  useEffect(() => {
    setDraft(definition);
    nextId.current = computeNextId(definition);
  }, [definition]);

  const importedWarnings = useMemo(() => collectImportedWarnings(draft), [draft]);
  const canRun = isRunnable(draft);

  const updateDefinition = (next: ScreenerDefinition) => {
    setDraft(next);
    onChange(next);
  };

  const makeCondition = (): ScreenerCondition => ({
    id: `condition-${nextId.current++}`,
    field: "",
    operator: ">=",
    value: "",
  });

  const makeGroup = (): ScreenerGroup => ({
    id: `group-${nextId.current++}`,
    operator: "AND",
    children: [],
  });

  const toggleColumn = (field: string) => {
    const selected = draft.columns.includes(field)
      ? draft.columns.filter((entry) => entry !== field)
      : [...draft.columns, field];
    updateDefinition({ ...draft, columns: selected });
  };

  return (
    <div className="flex h-full flex-col gap-4 p-4">
      <div className="space-y-3 rounded-2xl border border-border bg-card/80 p-4">
        <div className="flex items-start justify-between gap-3">
          <div>
            <div className="text-sm font-semibold text-foreground">Screener Workbench</div>
            <p className="mt-1 text-xs text-muted-foreground">
              Build nested conditions, choose result columns, and run against cached market quotes.
            </p>
          </div>
          <button
            type="button"
            data-testid="run-screener"
            disabled={!canRun}
            onClick={() => {
              if (canRun) {
                onRun();
              }
            }}
            className="rounded-xl bg-emerald-500 px-3 py-2 text-xs font-semibold text-emerald-950 disabled:cursor-not-allowed disabled:bg-muted disabled:text-muted-foreground"
          >
            Run Screener
          </button>
        </div>

        <div className="flex flex-wrap gap-2 text-[11px] text-muted-foreground">
          <span className="rounded-full border border-border px-2 py-1">{draft.columns.length} columns</span>
          <span className="rounded-full border border-border px-2 py-1">{catalog.length} fields</span>
          {importWarningCount > 0 ? (
            <span className="rounded-full border border-amber-400/30 bg-amber-400/10 px-2 py-1 text-amber-200">
              {importWarningCount} unsupported imported conditions
            </span>
          ) : null}
        </div>
      </div>

      <ConditionGroup
        group={draft.logic}
        catalog={catalog}
        isRoot
        validationErrors={validationErrors}
        importedWarnings={importedWarnings}
        onOperatorChange={(groupId, operator) => {
          updateDefinition({
            ...draft,
            logic: replaceGroup(draft.logic, groupId, (group) => ({ ...group, operator })),
          });
        }}
        onAddCondition={(groupId) => {
          updateDefinition({
            ...draft,
            logic: replaceGroup(draft.logic, groupId, (group) => ({
              ...group,
              children: [...group.children, makeCondition()],
            })),
          });
        }}
        onAddGroup={(groupId) => {
          updateDefinition({
            ...draft,
            logic: replaceGroup(draft.logic, groupId, (group) => ({
              ...group,
              children: [...group.children, makeGroup()],
            })),
          });
        }}
        onConditionChange={(conditionId, next) => {
          updateDefinition({
            ...draft,
            logic: replaceCondition(draft.logic, conditionId, next),
          });
        }}
        onRemoveNode={(nodeId) => {
          updateDefinition({
            ...draft,
            logic: removeNode(draft.logic, nodeId),
          });
        }}
      />

      <div className="rounded-2xl border border-border bg-card/70 p-4">
        <div className="text-sm font-semibold text-foreground">Result Columns</div>
        <div className="mt-3 grid grid-cols-2 gap-2">
          {catalog.map((field) => (
            <label key={field.field} className="flex items-center gap-2 rounded-lg border border-border px-2 py-2 text-xs text-foreground">
              <input
                type="checkbox"
                checked={draft.columns.includes(field.field)}
                onChange={() => toggleColumn(field.field)}
              />
              <span>{field.label}</span>
            </label>
          ))}
        </div>
      </div>
    </div>
  );
}