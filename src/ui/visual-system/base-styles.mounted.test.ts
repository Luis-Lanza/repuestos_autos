import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ProductBrowser, initialProductBrowserState, type CatalogViewMode } from "../catalog/product-browser.ts";

async function mountBaseStyles(width?: number) {
  const style = document.createElement("style");
  style.textContent = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  document.head.append(style);
  if (width !== undefined) {
    // jsdom does not evaluate viewport media queries. Activate width-only rules
    // in source order to check declarations/cascade, not layout or wheel routing.
    const activeRules = (rules: CSSRuleList): string[] => Array.from(rules).flatMap((rule) => {
      if (rule.type === 1) return [rule.cssText];
      if (rule.type !== 4) return [];
      const media = rule as CSSMediaRule;
      const bounds = [...media.conditionText.matchAll(/\((min|max)-width:\s*(\d+)px\)/g)];
      if (!bounds.length || !bounds.every(([, bound, pixels]) => bound === "min" ? width >= Number(pixels) : width <= Number(pixels))) return [];
      return activeRules(media.cssRules);
    });
    style.textContent = activeRules(style.sheet!.cssRules).join("\n");
  }
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
        createElement("ul", { "data-ui-product-browser-list": true, "data-ui-catalog-table": "true" },
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

function catalogBrowser(mode: CatalogViewMode) {
  return createElement(ProductBrowser, {
    presentation: "catalog", catalogViewMode: mode,
    state: { ...initialProductBrowserState, status: "results", result: {
      products: [{ product_id: 1, revision: 1, category_id: 1, sku: "FIL-001", name: "Filtro", category_name: "Filtros", available_quantity: 2, purchase_price_centavos: null, sale_price_centavos: 12550, list_price_centavos: 12550, catalog_unit_price_centavos: 12550, minimum_sale_price_centavos: 10000, primary_location_code: null, attribute_values: [] }],
      categories: [], page: 1, page_size: 20, total: 1, total_pages: 1,
    } },
    onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: () => {}, onPageChange: () => {},
  });
}

for (const width of [961, 960, 600]) {
  test(`Catalog table wrapper alone owns both scroll axes at ${width}px; Gallery and Edit retain scrolling`, async () => {
    const style = await mountBaseStyles(width);
    try {
      render(createElement("div", { "data-ui-catalog-workspace": true },
        catalogBrowser("table"), catalogBrowser("gallery"),
        createElement("section", { "data-ui-catalog-edit-dialog": true },
          createElement("div", { "data-ui-catalog-edit-content": true }, "Editar producto")),
      ));
      const wrapper = document.querySelector("[data-ui-catalog-table-scroll]")!;
      const table = wrapper.querySelector('[data-ui-product-browser-list][data-ui-catalog-table="true"]')!;
      assert.ok(table, "real ProductBrowser table carries the generic list attribute");
      assert.equal(table.tagName, "UL");
      assert.equal(styleOf(table, "overflow-y"), "visible");
      assert.equal(styleOf(table, "overflow-x"), "visible");
      assert.equal(styleOf(wrapper, "overflow-x"), "auto");
      assert.equal(styleOf(wrapper, "overflow-y"), "auto");
      if (width >= 961) assert.equal(styleOf(wrapper, "min-block-size"), "0px");
      assert.equal(styleOf(table, "overscroll-behavior"), "auto");
      const gallery = document.querySelector('[data-ui-catalog-gallery="true"]')!;
      assert.equal(gallery.closest("[data-ui-catalog-table-scroll]"), null);
      assert.equal(styleOf(gallery, "overflow-y"), "auto");
      assert.equal(styleOf(gallery, "overscroll-behavior"), "contain");
      assert.equal(styleOf(gallery, "grid-template-columns"), `repeat(${width >= 961 ? 5 : 2}, minmax(0, 1fr))`);
      const edit = document.querySelector("[data-ui-catalog-edit-content]")!;
      assert.equal(styleOf(edit, "min-block-size"), "0px");
      assert.equal(styleOf(edit, "overflow-y"), "auto");
      assert.equal(styleOf(edit, "overscroll-behavior"), "contain");
    } finally {
      style.remove();
    }
  });
}

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
