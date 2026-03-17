import { describe, expect, it, vi } from "vitest";
import {
  buildScreenerRunPayload,
  buildScreenerTemplatePayload,
  getScreenerFieldLabel,
  getScreenerLogicLabel,
  getScreenerOperatorLabel,
  importEastmoneyScreener,
  normalizeImportedScreenerDefinition,
  translateScreenerMessage,
  type ScreenerDefinition,
} from "../screener";

function sampleDefinition(): ScreenerDefinition {
  return {
    name: "Momentum",
    description: "Nested logic",
    logic: {
      id: "root",
      operator: "AND",
      children: [
        {
          id: "price-band",
          field: "latest_price",
          operator: "between",
          value: [10, 20],
        },
        {
          id: "or-group",
          operator: "OR",
          children: [
            {
              id: "pct-up",
              field: "change_pct",
              operator: ">=",
              value: 3,
            },
          ],
        },
      ],
    },
    sorts: [{ field: "change_pct", direction: "desc" }],
    columns: ["symbol", "latest_price", "change_pct"],
    source: "manual",
  };
}

describe("screener client helpers", () => {
  it("serializes run payloads with definition and limit", () => {
    expect(buildScreenerRunPayload(sampleDefinition(), 25)).toEqual({
      definition: sampleDefinition(),
      limit: 25,
    });
  });

  it("normalizes import diagnostics from API responses", async () => {
    const fetcher = vi.fn().mockResolvedValue({
      json: async () => ({
        success: true,
        data: {
          ...sampleDefinition(),
          source: "eastmoney_import",
          importMeta: {
            importedConditions: 2,
            unsupportedConditions: [{ key: "roe", reason: "field unavailable" }],
          },
        },
        message: "ok",
      }),
    });

    const definition = await importEastmoneyScreener(
      "https://xuangu.eastmoney.com/result?filters=change_pct:>=:3",
      fetcher,
    );

    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(definition.importMeta).toEqual({
      originalUrl: undefined,
      importedConditions: 2,
      unsupportedConditions: [
        {
          key: "roe",
          reason: "field unavailable",
          raw: undefined,
        },
      ],
    });
  });

  it("preserves nested group logic and columns in template payloads", () => {
    expect(
      buildScreenerTemplatePayload({
        name: "Template",
        description: "Saved",
        definition: sampleDefinition(),
        sourceType: "manual",
      }),
    ).toEqual({
      name: "Template",
      description: "Saved",
      definition: sampleDefinition(),
      source_type: "manual",
    });
  });

  it("keeps partially supported import diagnostics when normalizing imported definitions", () => {
    const normalized = normalizeImportedScreenerDefinition({
      ...sampleDefinition(),
      source: "eastmoney_import",
      importMeta: {
        importedConditions: 1,
        unsupportedConditions: [
          { key: "fancy_metric", reason: "unsupported field" },
          { key: "roe", reason: "field unavailable", raw: "roe:>=:15" },
        ],
      },
    });

    expect(normalized.importMeta?.unsupportedConditions).toEqual([
      { key: "fancy_metric", reason: "unsupported field", raw: undefined },
      { key: "roe", reason: "field unavailable", raw: "roe:>=:15" },
    ]);
  });

  it("preserves nested rules when building template payloads from imported definitions", () => {
    const imported = normalizeImportedScreenerDefinition({
      ...sampleDefinition(),
      source: "eastmoney_import",
      importMeta: {
        importedConditions: 1,
        unsupportedConditions: [{ key: "fancy_metric", reason: "unsupported field" }],
      },
    });

    const payload = buildScreenerTemplatePayload({
      name: "Imported Template",
      description: "From EastMoney",
      definition: imported,
      sourceType: "eastmoney_import",
    });

    expect(payload.definition.logic.children[1]).toEqual(sampleDefinition().logic.children[1]);
    expect(payload.definition.importMeta?.unsupportedConditions).toHaveLength(1);
  });

  it("exposes Chinese display labels for screener fields, operators, and logic", () => {
    expect(getScreenerFieldLabel("latest_price")).toBe("最新价");
    expect(getScreenerOperatorLabel("between")).toBe("区间");
    expect(getScreenerLogicLabel("AND")).toBe("且");
  });

  it("translates common screener error messages into Chinese", () => {
    expect(translateScreenerMessage("Only EastMoney screener URLs are supported")).toBe("仅支持导入东方财富选股链接");
    expect(translateScreenerMessage("Operator is not supported for change_pct")).toBe("涨跌幅 不支持当前运算符");
    expect(translateScreenerMessage("field unavailable")).toBe("字段暂不可用");
  });
});
