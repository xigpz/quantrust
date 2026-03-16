import type { ImportedConditionWarning, ScreenerCatalogField, ScreenerCondition, ScreenerGroup, ScreenerLogic, ScreenerNode } from "@/lib/screener";
import ConditionCard from "./ConditionCard";

interface ConditionGroupProps {
  group: ScreenerGroup;
  catalog: ScreenerCatalogField[];
  isRoot?: boolean;
  validationErrors?: Record<string, string>;
  importedWarnings?: Record<string, ImportedConditionWarning>;
  onOperatorChange: (groupId: string, operator: ScreenerLogic) => void;
  onAddCondition: (groupId: string) => void;
  onAddGroup: (groupId: string) => void;
  onConditionChange: (conditionId: string, next: ScreenerCondition) => void;
  onRemoveNode: (nodeId: string) => void;
}

function isGroupNode(node: ScreenerNode): node is ScreenerGroup {
  return "children" in node;
}

export default function ConditionGroup({
  group,
  catalog,
  isRoot = false,
  validationErrors,
  importedWarnings,
  onOperatorChange,
  onAddCondition,
  onAddGroup,
  onConditionChange,
  onRemoveNode,
}: ConditionGroupProps) {
  return (
    <div className="space-y-3 rounded-2xl border border-border bg-card/70 p-3">
      <div className="flex flex-wrap items-center gap-2">
        <div className="text-sm font-semibold text-foreground">{isRoot ? "Rule Group" : "Nested Group"}</div>
        <select
          data-testid={`group-operator-${group.id}`}
          value={group.operator}
          onChange={(event) => onOperatorChange(group.id, event.target.value as ScreenerLogic)}
          className="rounded-md border border-border bg-background px-2 py-1 text-xs text-foreground"
        >
          <option value="AND">AND</option>
          <option value="OR">OR</option>
        </select>
        <button
          type="button"
          data-testid={`add-condition-${group.id}`}
          onClick={() => onAddCondition(group.id)}
          className="rounded-md border border-border px-2 py-1 text-xs text-foreground hover:bg-background"
        >
          Add Condition
        </button>
        <button
          type="button"
          onClick={() => onAddGroup(group.id)}
          className="rounded-md border border-border px-2 py-1 text-xs text-foreground hover:bg-background"
        >
          Add Group
        </button>
        {!isRoot ? (
          <button
            type="button"
            data-testid={`remove-node-${group.id}`}
            onClick={() => onRemoveNode(group.id)}
            className="rounded-md border border-border px-2 py-1 text-xs text-muted-foreground hover:text-foreground"
          >
            Remove Group
          </button>
        ) : null}
      </div>

      <div className="space-y-3">
        {group.children.length === 0 ? (
          <div className="rounded-xl border border-dashed border-border px-3 py-4 text-xs text-muted-foreground">
            Add a condition or nested group to start building this rule set.
          </div>
        ) : null}

        {group.children.map((node) =>
          isGroupNode(node) ? (
            <ConditionGroup
              key={node.id}
              group={node}
              catalog={catalog}
              validationErrors={validationErrors}
              importedWarnings={importedWarnings}
              onOperatorChange={onOperatorChange}
              onAddCondition={onAddCondition}
              onAddGroup={onAddGroup}
              onConditionChange={onConditionChange}
              onRemoveNode={onRemoveNode}
            />
          ) : (
            <ConditionCard
              key={node.id}
              condition={node}
              catalog={catalog}
              error={validationErrors?.[node.id]}
              warning={importedWarnings?.[node.id]}
              onChange={(next) => onConditionChange(node.id, next)}
              onRemove={() => onRemoveNode(node.id)}
            />
          ),
        )}
      </div>
    </div>
  );
}