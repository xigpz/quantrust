import type { ScreenerTemplateRecord } from "@/lib/screener";

interface ScreenerTemplateDrawerProps {
  templates: ScreenerTemplateRecord[];
  selectedTemplateId: string | null;
  draftName: string;
  draftDescription: string;
  saving: boolean;
  onDraftNameChange: (value: string) => void;
  onDraftDescriptionChange: (value: string) => void;
  onSelectTemplate: (template: ScreenerTemplateRecord) => void;
  onSave: () => void;
}

export default function ScreenerTemplateDrawer({
  templates,
  selectedTemplateId,
  draftName,
  draftDescription,
  saving,
  onDraftNameChange,
  onDraftDescriptionChange,
  onSelectTemplate,
  onSave,
}: ScreenerTemplateDrawerProps) {
  return (
    <div className="flex h-full flex-col rounded-2xl border border-border bg-card/80">
      <div className="border-b border-border px-4 py-3">
        <div className="text-sm font-semibold text-foreground">模板</div>
        <div className="text-xs text-muted-foreground">保存常用策略，后续可以一键加载继续使用。</div>
      </div>

      <div className="space-y-3 border-b border-border px-4 py-4">
        <label className="grid gap-1 text-xs text-muted-foreground">
          名称
          <input
            value={draftName}
            onChange={(event) => onDraftNameChange(event.target.value)}
            placeholder="动量突破"
            className="rounded-md border border-border bg-background px-2 py-2 text-sm text-foreground"
          />
        </label>

        <label className="grid gap-1 text-xs text-muted-foreground">
          说明
          <textarea
            value={draftDescription}
            onChange={(event) => onDraftDescriptionChange(event.target.value)}
            rows={3}
            placeholder="简单记录这套选股策略的思路"
            className="rounded-md border border-border bg-background px-2 py-2 text-sm text-foreground"
          />
        </label>

        <button
          type="button"
          disabled={saving || draftName.trim().length === 0}
          onClick={onSave}
          className="w-full rounded-xl bg-sky-400 px-3 py-2 text-sm font-semibold text-sky-950 disabled:cursor-not-allowed disabled:bg-muted disabled:text-muted-foreground"
        >
          {saving ? "保存中..." : selectedTemplateId ? "更新模板" : "保存模板"}
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto p-4">
        <div className="space-y-2">
          {templates.length === 0 ? (
            <div className="rounded-xl border border-dashed border-border px-3 py-4 text-xs text-muted-foreground">
              还没有已保存模板。
            </div>
          ) : (
            templates.map((template) => (
              <button
                key={template.id}
                type="button"
                onClick={() => onSelectTemplate(template)}
                className={`w-full rounded-xl border px-3 py-3 text-left ${selectedTemplateId === template.id ? "border-sky-400/70 bg-sky-400/10" : "border-border bg-background/50"}`}
              >
                <div className="text-sm font-medium text-foreground">{template.name}</div>
                <div className="mt-1 text-xs text-muted-foreground">{template.description || "暂无说明"}</div>
                <div className="mt-2 text-[11px] text-muted-foreground">{template.definition.columns.length} 列</div>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
