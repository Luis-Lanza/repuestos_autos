import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

async function mountBaseStyles() {
  const style = document.createElement("style");
  style.textContent = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  document.head.append(style);
  return style;
}

function styleOf(element: Element, property: string) {
  let value = getComputedStyle(element).getPropertyValue(property).trim();
  for (let depth = 0; depth < 3; depth += 1) {
    const variable = value.match(/^var\((--[^)]+)\)$/)?.[1];
    if (!variable) break;
    value = getComputedStyle(element).getPropertyValue(variable).trim()
      || getComputedStyle(document.documentElement).getPropertyValue(variable).trim();
  }
  return value;
}

test("Catalog table columns shrink and wrap while the results region retains optional horizontal scrolling", async () => {
  const style = await mountBaseStyles();
  try {
    render(createElement("div", { "data-ui-catalog-workspace": true },
      createElement("div", { "data-ui-catalog-table-scroll": true },
        createElement("ul", { "data-ui-catalog-table": "true" },
          createElement("li", { "data-ui-catalog-table-row": true },
            createElement("div", { "data-ui-catalog-table-thumbnail": true }),
            createElement("div", { "data-ui-catalog-table-identity": true }, "A long product identity"),
            createElement("span", { "data-testid": "catalog-table-price" }, "Bs 125,50"),
            createElement("span", { "data-ui-catalog-table-category": true }, "A long category name"),
            createElement("span", { "data-testid": "catalog-table-stock" }, createElement("span", { "data-ui-badge": "available" }, "Disponible")),
            createElement("div", null, "Editar"),
          ),
        ),
      ),
    ));
    const scrollRegion = document.querySelector("[data-ui-catalog-table-scroll]")!;
    const table = document.querySelector('[data-ui-catalog-table="true"]')!;
    const row = table.querySelector("li")!;
    const identity = document.querySelector("[data-ui-catalog-table-identity]")!;
    const category = document.querySelector("[data-ui-catalog-table-category]")!;
    const price = document.querySelector('[data-testid="catalog-table-price"]')!;
    const stock = document.querySelector('[data-testid="catalog-table-stock"] [data-ui-badge]')!;
    assert.equal(styleOf(scrollRegion, "overflow-x"), "auto");
    assert.equal(styleOf(table, "min-inline-size"), "0px");
    assert.equal(styleOf(table, "flex-grow"), "1");
    assert.equal(styleOf(row, "min-inline-size"), "0px");
    assert.match(styleOf(row, "grid-template-columns"), /minmax\(0, 2fr\)/);
    assert.equal(styleOf(identity, "min-inline-size"), "0px");
    assert.equal(styleOf(identity, "overflow-wrap"), "anywhere");
    assert.equal(styleOf(category, "min-inline-size"), "0px");
    assert.equal(styleOf(category, "overflow-wrap"), "anywhere");
    assert.equal(styleOf(price, "white-space"), "nowrap");
    assert.equal(styleOf(stock, "white-space"), "nowrap");
    assert.doesNotMatch(styleOf(row, "grid-template-columns"), /67rem/);
  } finally {
    style.remove();
  }
});

test("base styles expose generic controls and the typography hierarchy", async () => {
  const style = await mountBaseStyles();
  const user = userEvent.setup({ document });
  try {
    render(createElement("main", null,
      createElement("p", { "data-ui-type": "overline" }, "Operación"),
      createElement("h1", null, "Inventario"),
      createElement("h2", null, "Conteo físico"),
      createElement("p", null, "Actualizá las unidades disponibles."),
      createElement("p", { "data-ui-type": "display" }, "Bs 125,50"),
      createElement("label", null, "Producto", createElement("input", { "aria-label": "Producto" })),
      createElement("small", null, "Solo unidades enteras"),
      createElement("code", null, "FIL-ACE-001"),
      createElement("button", null, "Guardar"),
      createElement("button", { "data-ui-control-size": "prominent" }, "Confirmar"),
      createElement("p", { role: "status", "data-ui-status": "success" }, "Operación guardada"),
      createElement("button", { "aria-busy": "true" }, "Guardando…"),
    ));

    const input = screen.getByRole("textbox", { name: "Producto" });
    const button = screen.getByRole("button", { name: "Guardar" });
    assert.equal(styleOf(input, "min-height"), "44px");
    assert.equal(styleOf(screen.getByRole("button", { name: "Confirmar" }), "min-height"), "48px");

    const h1 = screen.getByRole("heading", { level: 1 });
    const h2 = screen.getByRole("heading", { level: 2 });
    assert.deepEqual([styleOf(h1, "font-size"), styleOf(h1, "line-height"), styleOf(h1, "font-weight")], ["24px", "30px", "650"]);
    assert.deepEqual([styleOf(h2, "font-size"), styleOf(h2, "line-height"), styleOf(h2, "font-weight")], ["18px", "24px", "650"]);
    const display = screen.getByText("Bs 125,50");
    assert.deepEqual([styleOf(display, "font-size"), styleOf(display, "font-weight")], ["28px", "650"]);
    assert.equal(styleOf(display, "font-variant-numeric"), "tabular-nums");
    assert.deepEqual([styleOf(input, "font-size"), styleOf(input, "font-weight")], ["15px", "400"]);
    assert.deepEqual([styleOf(screen.getByText("Producto"), "font-size"), styleOf(screen.getByText("Producto"), "font-weight")], ["13px", "600"]);
    assert.equal(styleOf(screen.getByText("Solo unidades enteras"), "font-size"), "13px");
    assert.equal(styleOf(screen.getByText("Operación"), "text-transform"), "uppercase");
    assert.match(styleOf(screen.getByText("FIL-ACE-001"), "font-family"), /Cascadia Mono/);
    assert.match(styleOf(document.body, "font-family"), /Segoe UI/);
    assert.equal(styleOf(screen.getByRole("status"), "font-weight"), "600");
    assert.equal(styleOf(screen.getByRole("button", { name: "Guardando…" }), "cursor"), "progress");

    await user.tab();
    assert.equal(document.activeElement, input);
    input.dataset.uiFocusVisible = "true";
    assert.equal(styleOf(input, "outline-style"), "solid");
  } finally {
    style.remove();
  }
});
