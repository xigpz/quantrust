// @vitest-environment jsdom

import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import ConditionBuilder from "../ConditionBuilder";
import type { ScreenerCatalogField, ScreenerDefinition } from "@/lib/screener";

(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

function render(element: ReactNode) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  let root: Root;

  act(() => {
    root = createRoot(container);
    root.render(element);
  });

  return {
    container,
    unmount() {
      act(() => {
        root.unmount();
      });
      container.remove();
    },
  };
}

function click(element: Element) {
  act(() => {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

function change(element: HTMLInputElement | HTMLSelectElement, value: string) {
  act(() => {
    element.value = value;
    element.dispatchEvent(new Event("input", { bubbles: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

function sampleDefinition(): ScreenerDefinition {
  return {
    name: "Builder",
    logic: {
      id: "root",
      operator: "AND",
      children: [],
    },
    sorts: [],
    columns: ["symbol", "latest_price"],
    source: "manual",
  };
}

const catalog: ScreenerCatalogField[] = [
  {
    field: "latest_price",
    label: "Latest Price",
    category: "quote",
    valueType: "number",
    operators: [">=", "between"],
    dataSource: "quote_cache",
    eastmoneyCompatible: true,
    status: "ready",
  },
  {
    field: "change_pct",
    label: "Change %",
    category: "quote",
    valueType: "number",
    operators: [">=", "<="],
    dataSource: "quote_cache",
    eastmoneyCompatible: true,
    status: "ready",
  },
];

afterEach(() => {
  document.body.innerHTML = "";
});

describe("ConditionBuilder", () => {
  it("adds and removes conditions", () => {
    const onChange = vi.fn();
    const view = render(
      <ConditionBuilder
        catalog={catalog}
        definition={sampleDefinition()}
        importWarningCount={0}
        onChange={onChange}
        onRun={vi.fn()}
      />,
    );

    click(view.container.querySelector('[data-testid="add-condition-root"]')!);
    expect(view.container.querySelectorAll('[data-testid="condition-card"]').length).toBe(1);

    click(view.container.querySelector('[data-testid="remove-node-condition-1"]')!);
    expect(view.container.querySelectorAll('[data-testid="condition-card"]').length).toBe(0);
    expect(onChange).toHaveBeenCalled();

    view.unmount();
  });

  it("toggles group logic between AND and OR", () => {
    const changes: ScreenerDefinition[] = [];
    const view = render(
      <ConditionBuilder
        catalog={catalog}
        definition={sampleDefinition()}
        importWarningCount={0}
        onChange={(definition) => {
          changes.push(definition);
        }}
        onRun={vi.fn()}
      />,
    );

    change(
      view.container.querySelector('[data-testid="group-operator-root"]') as HTMLSelectElement,
      "OR",
    );

    expect(changes.at(-1)?.logic.operator).toBe("OR");
    view.unmount();
  });

  it("blocks run when invalid conditions exist and renders import warnings", () => {
    const onRun = vi.fn();
    const view = render(
      <ConditionBuilder
        catalog={catalog}
        definition={sampleDefinition()}
        importWarningCount={2}
        onChange={vi.fn()}
        onRun={onRun}
      />,
    );

    click(view.container.querySelector('[data-testid="add-condition-root"]')!);

    const runButton = view.container.querySelector('[data-testid="run-screener"]') as HTMLButtonElement;
    expect(runButton.disabled).toBe(true);
    expect(view.container.textContent).toContain("2 unsupported");

    click(runButton);
    expect(onRun).not.toHaveBeenCalled();

    view.unmount();
  });

  it("renders validation errors on the matching condition card", () => {
    const view = render(
      <ConditionBuilder
        catalog={catalog}
        definition={{
          ...sampleDefinition(),
          logic: {
            id: "root",
            operator: "AND",
            children: [
              {
                id: "condition-7",
                field: "change_pct",
                operator: ">=",
                value: 3,
              },
            ],
          },
        }}
        importWarningCount={0}
        validationErrors={{ "condition-7": "Unsupported operator for selected field" }}
        onChange={vi.fn()}
        onRun={vi.fn()}
      />,
    );

    expect(view.container.textContent).toContain("Unsupported operator for selected field");
    view.unmount();
  });
});