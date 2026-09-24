import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ProductBrowser, createProductBrowserFlow, initialProductBrowserState, readCatalogViewMode, readSalesViewMode, writeCatalogViewMode, writeSalesViewMode } from "./product-browser.ts";

const page = { products: [{ product_id: 1, category_id: 1, sku: "FLT", name: "Filter", category_name: "Filters", available_quantity: 4, catalog_unit_price_centavos: 2500, sale_price_centavos: 2500, list_price_centavos: 2500, minimum_sale_price_centavos: 2500, revision: 1 }], categories: [{ category_id: 1, name: "Filters" }], page: 1, page_size: 20, total: 1, total_pages: 1 };

test("names Seleccionar actions by product and SKU without changing visible copy or selection", async () => {
  const products = [page.products[0], { ...page.products[0], product_id: 2, sku: "FLT-2" }];
  const selected: number[] = [];
  render(createElement(ProductBrowser, {
    state: { ...initialProductBrowserState, status: "results", result: { ...page, products, total: 2 } },
    onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event) => event.preventDefault(), onPageChange: () => {},
    onSelect: (product) => selected.push(product.product_id),
  }));
  const rows = within(screen.getByRole("list", { name: "Resultados del catálogo" })).getAllByRole("listitem");
  assert.equal(rows.length, 2);
  for (const [index, sku] of ["FLT", "FLT-2"].entries()) {
    const button = within(rows[index]).getByRole("button", { name: `Seleccionar Filter (SKU: ${sku})` });
    assert.equal(button.textContent, "Seleccionar");
  }
  await userEvent.click(within(rows[1]).getByRole("button", { name: "Seleccionar Filter (SKU: FLT-2)" }));
  assert.deepEqual(selected, [2]);
});

test("renders bounded thumbnails or a visible accessible no-image placeholder in Gallery", () => {
  const product = page.products[0];
  const state = { ...initialProductBrowserState, status: "results" as const, result: page };
  const props = { state, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {} };
  const view = render(createElement(ProductBrowser, props));
  const row = within(screen.getByRole("list", { name: "Resultados del catálogo" })).getByRole("listitem");
  assert.equal(within(row).queryByRole("img"), null);
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "gallery" }));
  const placeholder = within(row).getByRole("img", { name: "Sin imagen" });
  assert.equal(placeholder.getAttribute("data-ui-catalog-image-placeholder"), "true");
  assert.equal(placeholder.textContent, "Sin imagen");
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "gallery", thumbnails: { [product.product_id]: "data:image/jpeg;base64,/9j/2Q==" } }));
  const image = within(row).getByRole("img", { name: "Filter" });
  assert.equal(image.getAttribute("src"), "data:image/jpeg;base64,/9j/2Q==");
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "gallery", thumbnails: { [product.product_id]: "/private/image.jpg" } }));
  assert.equal(within(row).queryByRole("img", { name: "Filter" }), null);
  assert.ok(within(row).getByRole("img", { name: "Sin imagen" }));
});

test("keeps unavailable selection disabled and leaves Agregar and Editar names unchanged", () => {
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [{ ...page.products[0], available_quantity: 0 }] } };
  for (const [actionLabel, presentation] of [["Seleccionar", undefined], ["Agregar", "sales"], ["Editar", undefined]] as const) {
    const view = render(createElement(ProductBrowser, {
      state, actionLabel, presentation,
      onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {},
    }));
    const name = actionLabel === "Seleccionar" ? "Seleccionar Filter (SKU: FLT)" : actionLabel;
    const button = screen.getByRole("button", { name });
    assert.equal(button.textContent, actionLabel);
    if (presentation === "sales") {
      assert.ok(screen.getByRole("button", { name: "Vista de tabla" }));
    } else {
      assert.equal(screen.queryByRole("button", { name: "Vista de tabla" }), null);
    }
    assert.equal((button as HTMLButtonElement).disabled, true);
    view.unmount();
  }
});

test("Catalog Table exposes stable result columns, a thumbnail slot, and an accessible no-image placeholder", () => {
  const state = { ...initialProductBrowserState, status: "results" as const, result: page };
  render(createElement(ProductBrowser, {
    state, presentation: "catalog", catalogViewMode: "table",
    onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {}, actionLabel: "Editar",
  }));
  const scroll = document.querySelector('[data-ui-catalog-table-scroll="true"]');
  assert.ok(scroll);
  assert.equal(scroll?.getAttribute("tabindex"), "0");
  const table = screen.getByRole("list", { name: "Resultados del catálogo" });
  assert.equal(table.getAttribute("data-ui-catalog-table"), "true");
  const row = within(table).getByRole("listitem");
  for (const column of ["thumbnail", "identity", "price", "category", "stock", "edit"]) {
    assert.ok(within(row).getByTestId(`catalog-table-${column}`), `missing ${column} column`);
  }
  assert.ok(within(row).getByRole("img", { name: "Sin imagen" }));
  assert.equal(within(row).getByText("Filter").textContent, "Filter");
  assert.equal(within(row).getByText("Filters").textContent, "Filters");
});

test("Catalog Table and Gallery expose the same product facts and explicit Edit action", () => {
  const products = [{ ...page.products[0], available_quantity: 0 }, { ...page.products[0], product_id: 2, sku: "FLT-2", name: "Second filter", available_quantity: 1 }];
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products, total: 2 } };
  const props = { state, presentation: "catalog" as const, catalogViewMode: "table" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {}, actionLabel: "Editar" };
  const table = render(createElement(ProductBrowser, props));
  assert.equal(within(screen.getByRole("list", { name: "Resultados del catálogo" })).getAllByRole("button", { name: "Editar" }).length, 2);
  table.rerender(createElement(ProductBrowser, { ...props, catalogViewMode: "gallery" }));
  const gallery = screen.getByRole("list", { name: "Resultados del catálogo" });
  assert.equal(gallery.getAttribute("data-ui-catalog-gallery"), "true");
  assert.equal(within(gallery).getAllByText("Sin imagen").length, 2);
  assert.equal(within(gallery).getByText("Filter").textContent, "Filter");
  assert.equal(within(gallery).getByText("FLT").textContent, "FLT");
  assert.equal(within(gallery).getByText("Second filter").textContent, "Second filter");
  assert.equal(within(gallery).getByText("FLT-2").textContent, "FLT-2");
  assert.equal(within(gallery).getAllByRole("button", { name: "Editar" }).length, 2);
  assert.equal(within(gallery).getAllByText("Sin stock: 0").length, 1);
  assert.equal(within(gallery).getAllByText("Stock bajo: 1").length, 1);
});

test("Catalog, Gallery, and Sales display canonical sale price ahead of legacy aliases", () => {
  const canonicalProduct = { ...page.products[0], sale_price_centavos: 3_000, list_price_centavos: 2_500, catalog_unit_price_centavos: 2_000 };
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [canonicalProduct] } };
  const props = { state, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {} };
  const view = render(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "table" }));
  assert.equal(screen.getByTestId("catalog-table-price").textContent, "Bs 30,00");
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "gallery" }));
  assert.equal(within(screen.getByRole("list", { name: "Resultados del catálogo" })).getByText("Bs 30,00").textContent, "Bs 30,00");
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "sales" }));
  assert.equal(within(screen.getByRole("list", { name: "Resultados del catálogo" })).getByText("Bs 30,00").textContent, "Bs 30,00");
});

test("Catalog browse falls back to the legacy list price when canonical sale price is absent", () => {
  const legacyProduct = (({ sale_price_centavos: _, ...product }) => ({
    ...product,
    list_price_centavos: 3_000,
    catalog_unit_price_centavos: 2_000,
  }))(page.products[0]);
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [legacyProduct] } };
  render(createElement(ProductBrowser, {
    state, presentation: "catalog", catalogViewMode: "table",
    onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event) => event.preventDefault(), onPageChange: () => {},
  }));

  assert.equal(screen.getByTestId("catalog-table-price").textContent, "Bs 30,00");
});

test("Catalog toolbar view controls are icon-only, named, and depict table and gallery", () => {
  const props = { state: { ...initialProductBrowserState, status: "results" as const, result: page }, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {} };
  render(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "table" }));
  const group = screen.getByRole("group", { name: "Presentación del catálogo" });
  const table = within(group).getByRole("button", { name: "Vista de tabla" });
  const gallery = within(group).getByRole("button", { name: "Vista de galería" });
  assert.equal(table.textContent, "");
  assert.equal(gallery.textContent, "");
  assert.equal(table.getAttribute("title"), "Vista de tabla");
  assert.equal(gallery.getAttribute("title"), "Vista de galería");
  assert.equal(within(table).getByRole("img", { hidden: true }).getAttribute("data-ui-icon"), "table");
  assert.equal(within(gallery).getByRole("img", { hidden: true }).getAttribute("data-ui-icon"), "gallery");
});

test("Sales table and gallery render ordered non-empty attribute summaries, thumbnails, and add state", () => {
  const product = { ...page.products[0], attribute_values: [{ definition_id: 1, label: "Material", value: "" }, { definition_id: 2, label: "Diámetro", value: "50 mm" }, { definition_id: 3, label: "Marca", value: "Bosch" }, { definition_id: 4, label: "Modelo", value: "Ignorar" }] };
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [product] } };
  const props = { state, presentation: "sales" as const, salesViewMode: "table" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {}, actionLabel: "Agregar", disabledProductIds: new Set([1]), thumbnails: { 1: "data:image/jpeg;base64,/9j/2Q==" } };
  const view = render(createElement(ProductBrowser, props));
  const list = screen.getByRole("list", { name: "Resultados del catálogo" });
  assert.equal(list.getAttribute("data-ui-sales-table"), "true");
  assert.ok(within(list).getByRole("img", { name: "Filter" }));
  assert.equal(within(list).getByText("Diámetro: 50 mm").textContent, "Diámetro: 50 mm");
  assert.equal(within(list).getByText("Marca: Bosch").textContent, "Marca: Bosch");
  assert.equal(within(list).queryByText("Modelo: Ignorar"), null);
  assert.equal((within(list).getByRole("button", { name: "Agregado" }) as HTMLButtonElement).disabled, true);
  view.rerender(createElement(ProductBrowser, { ...props, salesViewMode: "gallery" }));
  assert.equal(list.getAttribute("data-ui-sales-gallery"), "true");
  assert.ok(within(list).getByRole("img", { name: "Filter" }));
  assert.equal(within(list).getByText("Diámetro: 50 mm").textContent, "Diámetro: 50 mm");
});

test("Sales product identity opens an accessible read-only detail with every ordered attribute and restores focus", async () => {
  const product = { ...page.products[0], purchase_price_centavos: null, minimum_sale_price_centavos: 1_500, attribute_values: [
    { definition_id: 2, label: "Diámetro", value: "50 mm" },
    { definition_id: 5, label: "Material", value: "  " },
    { definition_id: 8, label: "Marca", value: "Bosch" },
  ] };
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [product] } };
  const props = { state, presentation: "sales" as const, salesViewMode: "table" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {} };
  const view = render(createElement(ProductBrowser, props));
  for (const mode of ["table", "gallery"] as const) {
    if (mode === "gallery") view.rerender(createElement(ProductBrowser, { ...props, salesViewMode: mode }));
    const trigger = screen.getByRole("button", { name: "Ver detalles de Filter (SKU: FLT)" });
    trigger.focus();
    await userEvent.click(trigger);
    const detail = screen.getByRole("dialog", { name: "Filter" });
    const closeButton = within(detail).getByRole("button", { name: "Cerrar detalle del producto" });
    assert.equal(document.activeElement, closeButton);
    await userEvent.keyboard("{Tab}");
    assert.equal(document.activeElement, closeButton);
    assert.ok(detail.contains(document.activeElement));
    await userEvent.keyboard("{Shift>}{Tab}{/Shift}");
    assert.equal(document.activeElement, closeButton);
    assert.ok(detail.contains(document.activeElement));
    assert.ok(within(detail).getByRole("img", { name: "Sin imagen" }));
    for (const fact of ["SKU", "FLT", "Categoría", "Filters", "Stock", "Disponible: 4", "Precio de compra", "No registrado", "Precio de venta", "Bs 25,00", "Precio mínimo de venta", "Bs 15,00"]) within(detail).getByText(fact);
    const values = Array.from(detail.querySelectorAll("[data-ui-sales-product-detail-attributes] dt, [data-ui-sales-product-detail-attributes] dd"), (node) => node.textContent);
    assert.deepEqual(values, ["Diámetro", "50 mm", "Material", "Sin dato", "Marca", "Bosch"]);
    await userEvent.keyboard("{Escape}");
    assert.equal(screen.queryByRole("dialog", { name: "Filter" }), null);
    assert.equal(document.activeElement, trigger);
  }
});

test("Sales view preference is separate from Catalog and defaults safely", () => {
  const values = new Map<string, string>();
  const storage = { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => { values.set(key, value); } };
  assert.equal(readSalesViewMode(storage), "table");
  writeCatalogViewMode("gallery", storage);
  assert.equal(readSalesViewMode(storage), "table");
  writeSalesViewMode("gallery", storage);
  assert.equal(readSalesViewMode(storage), "gallery");
  assert.equal(values.get("catalog.product-browser.view-mode"), "gallery");
  assert.equal(values.get("sales.product-browser.view-mode"), "gallery");
});

test("view controls are accessible, selected, and only rendered for Catalog", async () => {
  const props = { state: { ...initialProductBrowserState, status: "results" as const, result: page }, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {} };
  const changed: string[] = [];
  const view = render(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "table", onCatalogViewModeChange: (mode: "table" | "gallery") => changed.push(mode) }));
  const table = screen.getByRole("button", { name: "Vista de tabla" });
  const gallery = screen.getByRole("button", { name: "Vista de galería" });
  assert.equal(table.getAttribute("aria-pressed"), "true");
  assert.equal(gallery.getAttribute("aria-pressed"), "false");
  assert.ok(Number.parseFloat(getComputedStyle(table).minHeight || "0") >= 44 || table.hasAttribute("data-ui-catalog-view-toggle"));
  gallery.focus();
  await userEvent.keyboard("{Enter}");
  assert.deepEqual(changed, ["gallery"]);
  view.rerender(createElement(ProductBrowser, props));
  assert.equal(screen.queryByRole("button", { name: "Vista de tabla" }), null);
  assert.equal(view.container.querySelector("[data-ui-catalog-toolbar-item]"), null);
});

test("defensively defaults and persists Catalog view preference", () => {
  const values = new Map<string, string>();
  const storage = { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => { values.set(key, value); } };
  assert.equal(readCatalogViewMode(storage), "table");
  values.set("catalog.product-browser.view-mode", "invalid");
  assert.equal(readCatalogViewMode(storage), "table");
  values.set("catalog.product-browser.view-mode", "gallery");
  assert.equal(readCatalogViewMode(storage), "gallery");
  assert.doesNotThrow(() => writeCatalogViewMode("gallery", storage));
  assert.equal(values.get("catalog.product-browser.view-mode"), "gallery");
  assert.equal(readCatalogViewMode({ getItem: () => { throw new Error("blocked"); }, setItem: () => { throw new Error("blocked"); } }), "table");
  assert.doesNotThrow(() => writeCatalogViewMode("gallery", { getItem: () => null, setItem: () => { throw new Error("blocked"); } }));
});

test("keeps shared ProductBrowser viewport defaults outside Inventory", async () => {
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-product-browser-list\] \{[^}]*min-block-size:\s*calc\([^}]*\);[^}]*flex:\s*1 1 auto;[^}]*overflow-y:\s*auto/);
  assert.match(css, /\[data-ui-product-browser-pages\] \{[^}]*flex:\s*0 0 auto/);
});

test("keeps the newest browse response when requests complete out of order", () => {
  const first = createProductBrowserFlow(initialProductBrowserState, { type: "browse_started", query: "old", category_id: null, stock_state: "all", activity: "active", page: 1, request_id: 1 });
  const second = createProductBrowserFlow(first, { type: "browse_started", query: "new", category_id: 1, stock_state: "low_stock", activity: "active", page: 1, request_id: 2 });
  const current = createProductBrowserFlow(second, { type: "browse_succeeded", request_id: 2, result: page });
  const stale = createProductBrowserFlow(current, { type: "browse_failed", request_id: 1, message: "stale" });

  assert.equal(stale.query, "new");
  assert.equal(stale.category_id, 1);
  assert.equal(stale.status, "results");
  assert.equal(stale.error, null);
});

test("changing query or a browse filter resets the page without pretending a request succeeded", () => {
  const queried = createProductBrowserFlow({ ...initialProductBrowserState, page: 4 }, { type: "query_changed", value: "brake" });
  const state = createProductBrowserFlow(queried, { type: "category_changed", value: 3 });
  const stock = createProductBrowserFlow(state, { type: "stock_state_changed", value: "out_of_stock" });

  assert.deepEqual([stock.page, stock.query, stock.category_id, stock.stock_state, stock.status], [1, "brake", 3, "out_of_stock", "initial"]);
});

test("invalidates in-flight responses when any browse filter changes", () => {
  const filters = [
    { type: "query_changed" as const, value: "brake" },
    { type: "category_changed" as const, value: 3 },
    { type: "stock_state_changed" as const, value: "out_of_stock" as const },
    { type: "activity_changed" as const, value: "archived" as const },
  ];

  for (const filter of filters) {
    const loading = createProductBrowserFlow(initialProductBrowserState, {
      type: "browse_started", query: "old", category_id: null, stock_state: "all", activity: "active", page: 1, request_id: 1,
    });
    const changed = createProductBrowserFlow(loading, filter);
    const stale = createProductBrowserFlow(changed, { type: "browse_succeeded", request_id: 1, result: page });

    assert.equal(changed.request_id, 2, filter.type);
    assert.equal(stale, changed, `${filter.type} must reject the old response`);
  }
});
