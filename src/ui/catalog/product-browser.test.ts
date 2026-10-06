import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { ProductBrowseResult, SalesProductOriginalResponse } from "../../commands/catalog.ts";
import { SalesProductDetail, ProductBrowser, createProductBrowserFlow, initialProductBrowserState, readCatalogViewMode, readSalesViewMode, writeCatalogViewMode, writeSalesViewMode } from "./product-browser.ts";

const page = { products: [{ product_id: 1, category_id: 1, sku: "FLT", name: "Filter", category_name: "Filters", available_quantity: 4, catalog_unit_price_centavos: 2500, sale_price_centavos: 2500, list_price_centavos: 2500, minimum_sale_price_centavos: 2500, purchase_price_centavos: null, primary_location_code: null, revision: 1, attribute_values: [] } satisfies ProductBrowseResult], categories: [{ category_id: 1, name: "Filters" }], page: 1, page_size: 20, total: 1, total_pages: 1 };

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

test("Catalog shows the same assigned generated location in Table and Gallery without adding it to Sales rows", async () => {
  const product = { ...page.products[0], primary_location_code: "A1-SHELF2" };
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [product] } };
  const props = { state, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {} };
  const view = render(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "table" }));
  assert.equal(screen.getByTestId("catalog-table-identity").querySelector("[data-ui-catalog-primary-location]")?.textContent, "Ubicación principal: A1-SHELF2");
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "catalog", catalogViewMode: "gallery" }));
  assert.equal(screen.getByRole("list", { name: "Resultados del catálogo" }).querySelector("[data-ui-catalog-primary-location]")?.textContent, "Ubicación principal: A1-SHELF2");
  view.rerender(createElement(ProductBrowser, { ...props, presentation: "sales" }));
  const salesRow = within(screen.getByRole("list", { name: "Resultados del catálogo" })).getByRole("listitem");
  assert.equal(salesRow.querySelector("[data-ui-sales-primary-location]"), null);
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

test("Sales browse modes retain distinct geometry and protect table price and Add content", async () => {
  const state = { ...initialProductBrowserState, status: "results" as const, result: page };
  const props = { state, presentation: "sales" as const, salesViewMode: "table" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {}, actionLabel: "Agregar" };
  const view = render(createElement(ProductBrowser, props));
  const list = screen.getByRole("list", { name: "Resultados del catálogo" });
  const row = within(list).getByRole("listitem");
  assert.equal(list.getAttribute("data-ui-sales-table"), "true");
  const actionArea = row.querySelector("[data-ui-product-action]")!;
  assert.equal(actionArea.querySelector("[data-ui-unit-price] [data-ui-money]")?.textContent, "Bs 25,00");
  assert.equal(within(actionArea).getByRole("button", { name: "Agregar" }).textContent, "Agregar");

  view.rerender(createElement(ProductBrowser, { ...props, salesViewMode: "gallery" }));
  assert.equal(list.getAttribute("data-ui-sales-table"), null);
  assert.equal(list.getAttribute("data-ui-sales-gallery"), "true");
  const card = within(list).getByRole("listitem");
  assert.equal(card.getAttribute("data-ui-sales-product-card"), "true");
  assert.deepEqual(Array.from(card.children, (child) => child.getAttribute("data-ui-sales-image-area") ? "image" : child.getAttribute("data-ui-sales-product-identity") ? "identity" : child.getAttribute("data-ui-sales-commercial-footer") ? "commercial-footer" : "unexpected"), ["image", "identity", "commercial-footer"]);
  const viewport = card.querySelector("[data-ui-sales-image-area]")!;
  assert.equal(within(viewport).getByRole("img", { name: "Sin imagen" }).getAttribute("data-ui-catalog-image-placeholder"), "true");
  const footer = card.querySelector("[data-ui-sales-commercial-footer]")!;
  assert.equal(footer.querySelector("[data-ui-money]")?.textContent, "Bs 25,00");
  assert.equal(within(footer).getByText("Disponible: 4").getAttribute("data-ui-badge"), "available");
  assert.equal(within(footer).getByRole("button", { name: "Agregar" }).textContent, "Agregar");

  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-sales-table="true"[^}]*grid-template-columns:\s*3rem minmax\(0, 1fr\) max-content/s);
  assert.match(css, /data-ui-sales-table="true"] \[data-ui-product-action\][^}]*white-space:\s*nowrap/s);
  assert.match(css, /data-ui-sales-table="true"] \[data-ui-money\][^}]*word-break:\s*keep-all/s);
  assert.match(css, /data-ui-product-browser-list\]\[data-ui-sales-gallery="true"\]\s*\{[^}]*repeat\(auto-fit, minmax\(min\(100%, 15rem\), 1fr\)\)/s);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*data-ui-product-browser="sales"\] \[data-ui-product-browser-list\]\[data-ui-sales-gallery="true"\]\s*\{[^}]*grid-template-columns:\s*repeat\(2, minmax\(0, 1fr\)\)/s);
  assert.match(css, /data-ui-product-browser-list\]\[data-ui-sales-gallery="true"\] > \[data-ui-sales-product-card\][^}]*grid-template-columns:\s*minmax\(0, 1fr\)/s);
  assert.match(css, /data-ui-sales-image-area\]\s*\{[^}]*inline-size:\s*min\(100%, clamp\(8rem, 14vw, 10\.6667rem\)\)[^}]*aspect-ratio:\s*4 \/ 3[^}]*justify-self:\s*center/s);
  assert.match(css, /data-ui-sales-image-area\] > \[data-ui-product-thumbnail\], \[data-ui-sales-image-area\] > \[data-ui-catalog-image-placeholder\][^}]*inline-size:\s*100%[^}]*block-size:\s*100%/s);
  assert.match(css, /data-ui-sales-commercial-footer\]\s*\{[^}]*grid-template-columns:\s*minmax\(0, 1fr\) auto/s);
  assert.match(css, /data-ui-sales-commercial-footer\] > \[data-ui-action\][^}]*grid-column:\s*1 \/ -1[^}]*inline-size:\s*100%[^}]*white-space:\s*nowrap/s);
  assert.match(css, /@media \(max-width: 400px\)[\s\S]*data-ui-sales-commercial-footer\]\s*\{[^}]*grid-template-columns:\s*minmax\(0, 1fr\)/s);
});

test("Sales table and gallery details show assigned and unassigned locations without changing rows", () => {
  const props = { presentation: "sales" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {} };
  const view = render(createElement(ProductBrowser, { ...props, state: initialProductBrowserState }));
  for (const salesViewMode of ["table", "gallery"] as const) {
    for (const primary_location_code of ["A1-SHELF2", null]) {
      const purchase_price_centavos = primary_location_code === null ? null : 1800;
      const product = { ...page.products[0], primary_location_code, purchase_price_centavos, attribute_values: [] };
      const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [product] } };
      view.rerender(createElement(ProductBrowser, { ...props, state, salesViewMode }));
      const row = within(screen.getByRole("list", { name: "Resultados del catálogo" })).getByRole("listitem");
      assert.equal(within(row).queryByText(/Ubicación|A1-SHELF2|Sin ubicación asignada/), null);
      fireEvent.click(within(row).getByRole("button", { name: "Ver detalles" }));
      const detail = screen.getByRole("dialog", { name: "Filter" });
      assert.ok(within(detail).getByText("Ubicación principal"));
      assert.ok(within(detail).getByText(primary_location_code ?? "Sin ubicación asignada"));
      assert.ok(within(detail).getByText("Precio de compra"));
      assert.ok(within(detail).getByText(purchase_price_centavos === null ? "No registrado" : "Bs 18,00"));
      assert.equal(within(detail).queryByRole("textbox"), null);
      fireEvent.click(within(detail).getByRole("button", { name: "Cerrar detalle del producto" }));
    }
  }
});

test("Sales product identity opens an accessible read-only detail with every ordered attribute and restores focus", async () => {
  const product = { ...page.products[0], purchase_price_centavos: null, minimum_sale_price_centavos: 1_500, attribute_values: [
    { definition_id: 2, label: "Diámetro", value: "50 mm" },
    { definition_id: 5, label: "Material", value: "  " },
    { definition_id: 8, label: "Marca", value: "Bosch" },
  ] };
  const state = { ...initialProductBrowserState, status: "results" as const, result: { ...page, products: [product] } };
  const props = { state, presentation: "sales" as const, salesViewMode: "table" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {}, onSelect: () => {}, thumbnails: { 1: "data:image/jpeg;base64,/9j/2Q==" }, loadSalesOriginal: async (product_id: number) => ({ kind: "success" as const, product_id, src: "data:image/png;base64,iVBORw0KGgo=" }) };
  const view = render(createElement(ProductBrowser, props));
  for (const mode of ["table", "gallery"] as const) {
    if (mode === "gallery") view.rerender(createElement(ProductBrowser, { ...props, salesViewMode: mode }));
    const row = within(screen.getByRole("list", { name: "Resultados del catálogo" })).getByRole("listitem");
    assert.equal(within(row).getByText("Filter").tagName, "SPAN");
    assert.equal(within(row).queryByRole("button", { name: "Filter" }), null);
    const triggers = within(row).getAllByRole("button", { name: "Ver detalles", exact: true });
    assert.equal(triggers.length, 1);
    const trigger = triggers[0];
    assert.equal(trigger.textContent, "Ver detalles");
    trigger.focus();
    fireEvent.click(trigger);
    const detail = screen.getByRole("dialog", { name: "Filter" });
    const closeButton = within(detail).getByRole("button", { name: "Cerrar detalle del producto" });
    assert.equal(detail.getAttribute("data-ui-density"), "compact");
    const detailContent = detail.querySelector('[data-ui-sales-detail-content="true"]')!;
    assert.deepEqual(Array.from(detailContent.children, (child) => child.hasAttribute("data-ui-sales-detail-image-trigger") ? "image" : child.hasAttribute("data-ui-sales-product-detail-facts") ? "facts" : child.hasAttribute("data-ui-sales-product-detail-attributes") ? "attributes" : "unexpected"), ["image", "facts", "attributes"]);
    assert.equal(document.activeElement, closeButton);
    const zoomTrigger = within(detail).getByRole("button", { name: "Ampliar imagen del producto Filter" });
    await userEvent.keyboard("{Tab}");
    assert.equal(document.activeElement, zoomTrigger);
    assert.ok(detail.contains(document.activeElement));
    await userEvent.keyboard("{Shift>}{Tab}{/Shift}");
    assert.equal(document.activeElement, closeButton);
    assert.ok(detail.contains(document.activeElement));
    assert.equal(zoomTrigger.querySelector("img")?.getAttribute("alt"), "Filter");
    assert.ok(zoomTrigger.querySelector('[data-ui-sales-image-zoom-icon="true"]'));
    for (const fact of ["SKU", "FLT", "Categoría", "Filters", "Stock", "Disponible: 4", "Precio de venta", "Bs 25,00", "Precio mínimo de venta", "Bs 15,00"]) within(detail).getByText(fact);
    assert.ok(within(detail).getByText("Precio de compra"));
    assert.ok(within(detail).getByText("No registrado"));
    const values = Array.from(detail.querySelectorAll("[data-ui-sales-product-detail-attributes] dt, [data-ui-sales-product-detail-attributes] dd"), (node) => node.textContent);
    assert.deepEqual(values, ["Diámetro", "50 mm", "Material", "Sin dato", "Marca", "Bosch"]);
    zoomTrigger.focus();
    await userEvent.keyboard("{Enter}");
    const viewer = screen.getByRole("dialog", { name: "Imagen de Filter" });
    const viewerClose = within(viewer).getByRole("button", { name: "Cerrar imagen ampliada" });
    assert.equal(document.activeElement, viewerClose);
    assert.equal((await within(viewer).findByRole("img", { name: "Filter" })).getAttribute("src"), "data:image/png;base64,iVBORw0KGgo=");
    assert.equal(detail.getAttribute("aria-hidden"), "true");
    fireEvent.keyDown(document, { key: "Escape" });
    assert.equal(screen.queryByRole("dialog", { name: "Imagen de Filter" }), null);
    assert.equal(screen.getByRole("dialog", { name: "Filter" }), detail);
    assert.equal(document.activeElement, zoomTrigger);

    fireEvent.click(zoomTrigger);
    fireEvent.click(screen.getByRole("button", { name: "Cerrar imagen ampliada" }));
    assert.equal(screen.getByRole("dialog", { name: "Filter" }), detail);
    assert.equal(document.activeElement, zoomTrigger);
    fireEvent.click(zoomTrigger);
    const viewerBackdrop = screen.getByRole("dialog", { name: "Imagen de Filter" }).parentElement!;
    fireEvent.mouseDown(viewerBackdrop);
    assert.equal(screen.queryByRole("dialog", { name: "Imagen de Filter" }), null);
    assert.equal(screen.getByRole("dialog", { name: "Filter" }), detail);
    assert.equal(document.activeElement, zoomTrigger);

    fireEvent.keyDown(document, { key: "Escape" });
    assert.equal(screen.queryByRole("dialog", { name: "Filter" }), null);
    assert.equal(document.activeElement, trigger);
  }
});

test("Sales original zoom ignores product changes and late rejected generations", async () => {
  const requests: Array<{ productId: number; resolve: (value: SalesProductOriginalResponse) => void; reject: (reason: Error) => void }> = [];
  const loadOriginal = (productId: number) => new Promise<SalesProductOriginalResponse>((resolve, reject) => requests.push({ productId, resolve, reject }));
  const props = { product: { ...page.products[0], attribute_values: [] }, thumbnails: { 1: "data:image/jpeg;base64,/9j/2Q==", 2: "data:image/jpeg;base64,/9j/2Q==" }, loadOriginal, triggerRef: { current: null }, onClose: () => {} };
  const view = render(createElement(SalesProductDetail, props));
  fireEvent.click(screen.getByRole("button", { name: "Ampliar imagen del producto Filter" }));
  view.rerender(createElement(SalesProductDetail, { ...props, product: { ...props.product, product_id: 2, name: "Other filter" } }));
  assert.deepEqual(requests.map((request) => request.productId), [1, 2]);
  await act(async () => requests[0].resolve({ kind: "success", product_id: 1, src: "data:image/png;base64,iVBORw0KGgo=" }));
  const viewer = screen.getByRole("dialog", { name: "Imagen de Other filter" });
  assert.equal(within(viewer).queryByRole("img"), null);
  assert.equal(within(viewer).getByRole("status").textContent, "Cargando imagen…");
  fireEvent.keyDown(document, { key: "Escape" });
  fireEvent.click(screen.getByRole("button", { name: "Ampliar imagen del producto Other filter" }));
  await act(async () => requests[1].reject(new Error("late private rejection")));
  const reopened = screen.getByRole("dialog", { name: "Imagen de Other filter" });
  assert.equal(within(reopened).queryByRole("alert"), null);
  assert.equal(within(reopened).getByRole("status").textContent, "Cargando imagen…");
  await act(async () => requests[2].resolve({ kind: "success", product_id: 2, src: "data:image/webp;base64,UklGRgAAAABXRUJQ" }));
  assert.equal(within(reopened).getByRole("img", { name: "Other filter" }).getAttribute("src"), "data:image/webp;base64,UklGRgAAAABXRUJQ");
  view.unmount();
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

test("Sales search, category, view, and submit controls share an accessible form toolbar", async () => {
  const props = { state: { ...initialProductBrowserState, status: "results" as const, result: page }, presentation: "sales" as const, salesViewMode: "table" as const, onQueryChange: () => {}, onCategoryChange: () => {}, onSubmit: (event: { preventDefault(): void }) => event.preventDefault(), onPageChange: () => {} };
  const view = render(createElement(ProductBrowser, props));
  const form = view.container.querySelector('[data-ui-product-browser="sales"] > form')!;
  const search = within(form).getByRole("searchbox", { name: "Buscar en el catálogo" });
  const category = within(form).getByRole("combobox", { name: "Categoría" });
  const views = within(form).getByRole("group", { name: "Presentación de ventas" });
  const submit = within(form).getByRole("button", { name: "Buscar" });
  assert.deepEqual([search.closest("[data-ui-field]"), category.closest("[data-ui-field]"), views, submit], Array.from(form.children));
  assert.equal((within(views).getByRole("button", { name: "Vista de tabla" })).getAttribute("aria-pressed"), "true");
  assert.equal((within(views).getByRole("button", { name: "Vista de galería" })).getAttribute("aria-pressed"), "false");

  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-product-browser="sales"\]\s*\{[^}]*container:\s*sales-browse\s*\/\s*inline-size/s);
  assert.match(css, /\[data-ui-product-browser="sales"\]\s*\[data-ui-sale-search\]\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s*;[^}]*\}/s);
  assert.match(css, /@container sales-browse \(min-width:\s*40rem\)[\s\S]*\[data-ui-product-browser="sales"\] \[data-ui-sale-search\]\s*\{[^}]*grid-template-columns:\s*minmax\(12rem,\s*1\.6fr\) minmax\(10rem,\s*1fr\) auto auto/s);
  assert.equal(css.includes('[data-ui-catalog-workspace] [data-ui-product-browser] > form'), true);
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
