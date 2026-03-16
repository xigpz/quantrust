import { useEffect, useMemo, useState } from "react";
import { Download, RefreshCw, Save, Search } from "lucide-react";
import { toast } from "sonner";
import { addToWatchlist } from "@/hooks/useMarketData";
import {
  createScreenerTemplate,
  fetchScreenerCatalog,
  importEastmoneyScreener,
  listScreenerTemplates,
  runScreenerDefinition,
  updateScreenerTemplate,
  type ScreenerCatalogField,
  type ScreenerDefinition,
  type ScreenerExecutionResult,
  type ScreenerTemplateRecord,
} from "@/lib/screener";
import { useStockClick } from "@/pages/Dashboard";
import ConditionBuilder from "./screener/ConditionBuilder";
import ScreenerResultsTable from "./screener/ScreenerResultsTable";
import ScreenerTemplateDrawer from "./screener/ScreenerTemplateDrawer";

function createEmptyDefinition(): ScreenerDefinition {
  return {
    name: "Visual Screener",
    description: "",
    logic: {
      id: "root",
      operator: "AND",
      children: [],
    },
    sorts: [{ field: "change_pct", direction: "desc" }],
    columns: ["symbol", "name", "latest_price", "change_pct"],
    source: "manual",
  };
}

function parseValidationErrors(message: string): Record<string, string> {
  try {
    const parsed = JSON.parse(message) as Array<{ condition_id?: string; message?: string }>;
    return parsed.reduce<Record<string, string>>((accumulator, error) => {
      if (error.condition_id && error.message) {
        accumulator[error.condition_id] = error.message;
      }
      return accumulator;
    }, {});
  } catch {
    return {};
  }
}

export default function ScreenerPanel() {
  const [catalog, setCatalog] = useState<ScreenerCatalogField[]>([]);
  const [definition, setDefinition] = useState<ScreenerDefinition>(createEmptyDefinition);
  const [results, setResults] = useState<ScreenerExecutionResult["rows"]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [templates, setTemplates] = useState<ScreenerTemplateRecord[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null);
  const [templateName, setTemplateName] = useState("Momentum Setup");
  const [templateDescription, setTemplateDescription] = useState("");
  const [importUrl, setImportUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});
  const { openStock } = useStockClick();

  const importWarningCount = definition.importMeta?.unsupportedConditions.length ?? 0;

  const reloadTemplates = async () => {
    try {
      setTemplates(await listScreenerTemplates());
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to load templates");
    }
  };

  useEffect(() => {
    let mounted = true;
    (async () => {
      try {
        const [nextCatalog, nextTemplates] = await Promise.all([fetchScreenerCatalog(), listScreenerTemplates()]);
        if (!mounted) {
          return;
        }
        setCatalog(nextCatalog);
        setTemplates(nextTemplates);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : "Failed to initialize screener workbench");
      }
    })();

    return () => {
      mounted = false;
    };
  }, []);

  const handleRun = async () => {
    setLoading(true);
    setValidationErrors({});
    try {
      const result = await runScreenerDefinition(definition, 80);
      setResults(result.rows);
      setTotalCount(result.total_count);
      toast.success(`Matched ${result.total_count} symbols`);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to run screener";
      const fieldErrors = parseValidationErrors(message);
      setValidationErrors(fieldErrors);
      toast.error(Object.keys(fieldErrors).length > 0 ? "Fix invalid conditions before running again" : message);
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async () => {
    if (!importUrl.trim()) {
      toast.error("Paste an EastMoney screener URL first");
      return;
    }

    setLoading(true);
    try {
      const imported = await importEastmoneyScreener(importUrl.trim());
      setDefinition(imported);
      setTemplateName(imported.name || "Imported EastMoney Screener");
      setTemplateDescription(imported.description || "Imported from EastMoney");
      setSelectedTemplateId(null);
      setValidationErrors({});
      toast.success(`Imported ${imported.importMeta?.importedConditions ?? 0} conditions`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Import failed");
    } finally {
      setLoading(false);
    }
  };

  const handleSaveTemplate = async () => {
    setSaving(true);
    try {
      if (selectedTemplateId) {
        await updateScreenerTemplate(selectedTemplateId, {
          name: templateName,
          description: templateDescription,
          definition,
          sourceType: definition.source || "manual",
        });
      } else {
        const created = await createScreenerTemplate({
          name: templateName,
          description: templateDescription,
          definition,
          sourceType: definition.source || "manual",
        });
        setSelectedTemplateId(created.id);
      }

      await reloadTemplates();
      toast.success("Template saved");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Failed to save template");
    } finally {
      setSaving(false);
    }
  };

  const resultColumns = useMemo(() => {
    return definition.columns.length > 0 ? definition.columns : Object.keys(results[0] ?? {});
  }, [definition.columns, results]);

  return (
    <div className="flex h-full flex-col gap-4 p-4">
      <div className="rounded-[28px] border border-border bg-card/80 px-5 py-4 shadow-sm">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div>
            <div className="text-lg font-semibold text-foreground">EastMoney Screener Workbench</div>
            <div className="mt-1 text-sm text-muted-foreground">
              Compose nested rules, import compatible EastMoney links, and save reusable templates.
            </div>
          </div>

          <div className="flex flex-col gap-3 xl:w-[620px]">
            <label className="grid gap-2 text-xs text-muted-foreground">
              Import EastMoney URL
              <div className="flex gap-2">
                <input
                  value={importUrl}
                  onChange={(event) => setImportUrl(event.target.value)}
                  placeholder="https://xuangu.eastmoney.com/result?..."
                  className="min-w-0 flex-1 rounded-xl border border-border bg-background px-3 py-2 text-sm text-foreground"
                />
                <button
                  type="button"
                  onClick={handleImport}
                  disabled={loading}
                  className="inline-flex items-center gap-2 rounded-xl border border-border px-3 py-2 text-sm text-foreground disabled:cursor-not-allowed disabled:text-muted-foreground"
                >
                  <Download className="h-4 w-4" />
                  Import
                </button>
                <button
                  type="button"
                  onClick={handleRun}
                  disabled={loading}
                  className="inline-flex items-center gap-2 rounded-xl bg-emerald-500 px-3 py-2 text-sm font-semibold text-emerald-950 disabled:cursor-not-allowed disabled:bg-muted disabled:text-muted-foreground"
                >
                  {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
                  Run
                </button>
                <button
                  type="button"
                  onClick={handleSaveTemplate}
                  disabled={saving}
                  className="inline-flex items-center gap-2 rounded-xl border border-border px-3 py-2 text-sm text-foreground disabled:cursor-not-allowed disabled:text-muted-foreground"
                >
                  <Save className="h-4 w-4" />
                  Save
                </button>
              </div>
            </label>
          </div>
        </div>
      </div>

      <div className="grid min-h-0 flex-1 gap-4 xl:grid-cols-[minmax(360px,430px)_minmax(0,1fr)_300px]">
        <div className="min-h-0 overflow-auto rounded-[28px] border border-border bg-card/40">
          <ConditionBuilder
            catalog={catalog}
            definition={definition}
            importWarningCount={importWarningCount}
            validationErrors={validationErrors}
            onChange={setDefinition}
            onRun={handleRun}
          />
        </div>

        <ScreenerResultsTable
          rows={results}
          columns={resultColumns}
          totalCount={totalCount}
          loading={loading}
          onSelectStock={(symbol, name) => openStock(symbol, name)}
          onAddToWatchlist={async (symbol, name) => {
            const response = await addToWatchlist({ symbol, name: name || symbol });
            if (response.success) {
              toast.success(`${symbol} added to watchlist`);
            } else {
              toast.error(response.message);
            }
          }}
        />

        <ScreenerTemplateDrawer
          templates={templates}
          selectedTemplateId={selectedTemplateId}
          draftName={templateName}
          draftDescription={templateDescription}
          saving={saving}
          onDraftNameChange={setTemplateName}
          onDraftDescriptionChange={setTemplateDescription}
          onSelectTemplate={(template) => {
            setSelectedTemplateId(template.id);
            setTemplateName(template.name);
            setTemplateDescription(template.description || "");
            setDefinition(template.definition);
            setValidationErrors({});
          }}
          onSave={handleSaveTemplate}
        />
      </div>
    </div>
  );
}