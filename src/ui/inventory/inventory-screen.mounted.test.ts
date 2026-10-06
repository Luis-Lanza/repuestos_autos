import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { mockIPC as installIPC } from "@tauri-apps/api/mocks";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { App } from "../app.ts";
import { InventoryScreen } from "./inventory-screen.ts";
import { ActionNotificationProvider } from "../visual-system/action-notifications.ts";

const product = { product_id: 1, category_id: 1, sku: "FLT", name: "Filter", category_name: "Filters", available_quantity: 8, catalog_unit_price_centavos: 2500, list_price_centavos: 2500, minimum_sale_price_centavos: 2500, primary_location_code: null, attribute_values: [], revision: 0 };
const browse = (products: typeof product[] = [product]) => ({ products: products.map((item) => ({ product_id: item.product_id, category_id: item.category_id, sku: item.sku, name: item.name, category_name: item.category_name, available_quantity: item.available_quantity, sale_price_centavos: item.sale_price_centavos ?? item.list_price_centavos ?? item.catalog_unit_price_centavos, minimum_sale_price_centavos: item.minimum_sale_price_centavos, primary_location_code: item.primary_location_code })), categories: [{ category_id: 1, name: "Filters" }], page: 1, page_size: 20, total: products.length, total_pages: products.length ? 1 : 0 });
const success = (request_id: string) => ({ kind: "success", request_id, product_id: 1, previous_quantity: 10, quantity_delta: 3, resulting_quantity: 11, occurred_at: "2025-01-01T00:00:00Z", note: null });
function mockIPC(handler: (command: string, payload?: { request?: unknown }) => unknown) {
  installIPC((command, payload) => handler(command === "browse_inventory_products_command" ? "browse_products_command" : command, payload));
}
function installUuid(...ids: string[]) { let index = 0; Object.defineProperty(globalThis.crypto, "randomUUID", { configurable: true, value: () => ids[Math.min(index++, ids.length - 1)] ?? ids.at(-1) }); }

async function searchAndSelect() {
  const user = userEvent.setup({ document });
  await user.type(screen.getByRole("searchbox", { name: "Buscar producto" }), "filter{Enter}");
  const select = await screen.findByRole("button", { name: "Seleccionar Filter (SKU: FLT)" });
  select.focus();
  await user.keyboard("{Enter}");
  return user;
}

test("loads operational Inventory browse while the Catalog session is locked", async () => {
  const calls: string[] = [];
  installIPC((command) => {
    calls.push(command);
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_inventory_products_command") return browse();
    throw new Error(`Unexpected command while Catalog is locked: ${command}`);
  });
  render(createElement(InventoryScreen));
  assert.ok(await screen.findByText("Filter"));
  assert.deepEqual(calls.sort(), ["browse_inventory_products_command", "list_inventory_alerts_command"]);
});

test("shows the selection intro only alongside results, not initial, loading, empty or error feedback", async () => {
  const initialMarkup = renderToStaticMarkup(createElement(InventoryScreen));
  assert.equal(initialMarkup.split("Seleccioná un producto para comenzar.").length - 1, 1);

  let resolveFirst!: (value: ReturnType<typeof browse>) => void;
  let attempts = 0;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") {
      attempts += 1;
      if (attempts === 1) return new Promise((resolve) => { resolveFirst = resolve; });
      if (attempts === 2) return browse([]);
      if (attempts === 3) throw new Error("Local browse unavailable");
      return browse();
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const intro = "Seleccioná un producto para comenzar.";
  assert.ok(screen.getByText("Buscando productos…"));
  assert.equal(screen.queryByText(intro), null);
  resolveFirst(browse([]));
  assert.ok(await screen.findByText("No encontramos productos para “”."));
  assert.equal(screen.queryByText(intro), null);

  const user = userEvent.setup({ document });
  await user.click(screen.getByRole("button", { name: "Buscar" }));
  assert.ok(await screen.findByText("No encontramos productos para “”."));
  assert.equal(screen.queryByText(intro), null);
  await user.click(screen.getByRole("button", { name: "Buscar" }));
  assert.ok(await screen.findByText("No se pudo buscar en el catálogo local. Reintentá."));
  assert.equal(screen.queryByText(intro), null);
  await user.click(screen.getByRole("button", { name: "Buscar" }));
  assert.ok(await screen.findByText("Filter"));
  assert.ok(screen.getByText(intro));
});

test("automatically loads active products and keeps a short result in the explicit inventory viewport structure", async () => {
  const calls: unknown[] = [];
  const threeProducts = Array.from({ length: 3 }, (_, index) => ({ ...product, product_id: index + 1, sku: `FLT-${index + 1}` }));
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") { calls.push(payload); return browse(threeProducts); }
    throw new Error(`Unexpected command: ${command}`);
  });
  const view = render(createElement(InventoryScreen));
  await screen.findByRole("list", { name: "Resultados del catálogo" });
  assert.equal(screen.getByRole("main").hasAttribute("data-ui-density"), false);
  const operation = screen.getByRole("region", { name: "Operación de inventario" });
  const browser = within(operation).getByLabelText("Buscar producto").closest("form")!.parentElement!;
  assert.equal(browser.getAttribute("data-ui-product-browser"), "true");
  const results = within(operation).getByRole("list", { name: "Resultados del catálogo" }).parentElement!;
  assert.equal(results.getAttribute("data-ui-catalog-results"), "true");
  assert.equal(within(results).getAllByRole("listitem").length, 3);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-inventory-layout\] > \[data-ui-panel\]:first-child:has\(> \[data-ui-product-browser\]\) \{[^}]*display:\s*flex;[^}]*min-block-size:\s*0;[^}]*flex-direction:\s*column/);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser\] \{[^}]*min-block-size:\s*0;[^}]*flex:\s*1 1 auto/);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-catalog-results\] \{[^}]*min-block-size:\s*0;[^}]*flex:\s*1 1 auto/);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] \{[^}]*min-block-size:\s*0;[^}]*flex:\s*1 1 auto;[^}]*overflow-y:\s*auto/);
  assert.match(css, /\[data-ui-product-browser-pages\] \{[^}]*flex:\s*0 0 auto/);
  assert.deepEqual(calls, [{ request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } }]);
  view.unmount();
});

test("keeps long Inventory results in the same viewport structure without a row-count mode", async () => {
  const fourProducts = Array.from({ length: 4 }, (_, index) => ({ ...product, product_id: index + 1, sku: `FLT-${index + 1}` }));
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse(fourProducts);
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const list = await screen.findByRole("list", { name: "Resultados del catálogo" });
  assert.equal(within(list).getAllByRole("listitem").length, 4);
  assert.equal(screen.getByRole("main").hasAttribute("data-ui-density"), false);
  const results = list.parentElement!;
  assert.equal(results.getAttribute("data-ui-catalog-results"), "true");
  assert.equal(within(results).getAllByRole("listitem").length, 4);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] \{[^}]*min-block-size:\s*0;[^}]*overflow-y:\s*auto/);
  assert.doesNotMatch(css, /main\[data-ui-inventory\]\[data-ui-density="sparse"\]/);
});

test("contains the inventory product viewport while alerts remain a sibling panel", async () => {
  const manyProducts = Array.from({ length: 100 }, (_, index) => ({ ...product, product_id: index + 1, sku: `FLT-${index + 1}` }));
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return { ...browse(manyProducts), total_pages: 5 };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const operation = await screen.findByRole("region", { name: "Operación de inventario" });
  assert.equal(screen.getByRole("main").getAttribute("data-ui-density"), null);
  const list = within(operation).getByRole("list", { name: "Resultados del catálogo" });
  const results = list.parentElement!;
  assert.equal(results.previousElementSibling?.tagName, "FORM");
  const pages = within(operation).getByRole("navigation", { name: "Páginas de productos" });
  assert.equal(results.nextElementSibling, pages);
  assert.ok(within(pages).getByText("Página 1 de 5"));
  assert.equal((within(pages).getByRole("button", { name: "Anterior" }) as HTMLButtonElement).disabled, true);
  assert.equal((within(pages).getByRole("button", { name: "Siguiente" }) as HTMLButtonElement).disabled, false);
  assert.equal(within(operation).getAllByRole("listitem").length, 100);
  const firstRow = within(operation).getAllByRole("listitem")[0];
  for (const text of ["Filter", "FLT-1", "Filters", "Bs 25,00", "Disponible: 8"]) {
    assert.ok(within(firstRow).getByText(text));
  }
  const firstSelect = within(firstRow).getByRole("button", { name: "Seleccionar Filter (SKU: FLT-1)" });
  assert.equal(firstSelect.textContent, "Seleccionar");
  const secondRow = within(operation).getAllByRole("listitem")[1];
  assert.ok(within(secondRow).getByRole("button", { name: "Seleccionar Filter (SKU: FLT-2)" }));
  assert.ok(screen.getByRole("region", { name: "Alertas de stock" }));
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-inventory-layout[^}]*grid-template-columns:\s*minmax\(0,\s*1\.85fr\) minmax\(260px,\s*1fr\)/s);
  assert.match(css, /\[data-ui-inventory-layout\] \{[^}]*grid-template-rows:\s*minmax\(0,\s*1fr\)/s);
  assert.match(css, /\[data-ui-product-browser-pages\] \{[^}]*flex:\s*0 0 auto/s);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form \{[^}]*inline-size:\s*min\(100%,\s*48rem\)[^}]*max-inline-size:\s*100%/s);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-product-browser\] > form, \[data-ui-product-browser-list\] > li \{\s*grid-template-columns:\s*minmax\(0,\s*1fr\)/s);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] \{[^}]*overflow-y:\s*auto;[^}]*overscroll-behavior:\s*contain/);
  assert.match(css, /\[data-ui-product-browser-list\] \{[^}]*--product-browser-row-block-size:/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] \{[^}]*min-block-size:\s*0;[^}]*overflow-y:\s*auto/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] > li \{[^}]*grid-template-columns:\s*minmax\(0, 2fr\)/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] \[data-ui-badge\] \{[^}]*white-space:\s*normal/);
});

test("keeps the browse controls in submit order with a panel-width four-control layout and narrow fallback", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const operation = screen.getByRole("region", { name: "Operación de inventario" });
  const form = within(operation).getByRole("searchbox", { name: "Buscar producto" }).closest("form")!;
  assert.deepEqual([...form.children].map((child) => child.tagName === "BUTTON" ? child.textContent?.trim() : child.querySelector("label")?.textContent), ["Buscar producto", "Categoría", "Estado del stock", "Buscar"]);
  assert.equal((within(form).getByRole("button", { name: "Buscar" }) as HTMLButtonElement).type, "submit");
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser\] \{ container: inventory-browse \/ inline-size; \}/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form \{ inline-size: min\(100%, 48rem\); max-inline-size: 100%; \}/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form > \[data-ui-field\]:nth-child\(3\) \{ grid-column: 1 \/ -1; \}/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form > \*,[\s\S]*form :is\(input, select, button\) \{ min-inline-size: 0; \}/);
  assert.match(css, /\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form :is\(input, select\) \{ inline-size: 100%; \}/);
  // Structural guard only: jsdom cannot measure a WebView's panel width or prove one visual row.
  assert.match(css, /@container inventory-browse \(min-width: 36rem\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns: minmax\(0, 1fr\) minmax\(0, 11\.5rem\) minmax\(0, 7\.5rem\) minmax\(0, auto\)/);
  assert.match(css, /@container inventory-browse \(min-width: 36rem\)[\s\S]*form > \[data-ui-field\]:nth-child\(3\) \{ grid-column: 3; \}[\s\S]*form > \[data-ui-action\] \{ grid-column: 4; grid-row: 1; \}/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser\] > form > \[data-ui-action\] \{ grid-column: auto; grid-row: auto; \}[\s\S]*\[data-ui-product-browser\] > form, \[data-ui-product-browser-list\] > li \{ grid-template-columns: minmax\(0, 1fr\); \}/);
});

test("keeps browse first and read-only alerts second across desktop and compact layout", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [{ product_id: 2, product_name: "Correa", quantity: 0, classification: "out_of_stock" }] };
    if (command === "browse_products_command") return browse();
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const main = screen.getByRole("main", { name: "Inventario" });
  const operation = within(main).getByRole("region", { name: "Operación de inventario" });
  const alerts = within(main).getByRole("region", { name: "Alertas de stock" });
  assert.ok(operation.compareDocumentPosition(alerts) & Node.DOCUMENT_POSITION_FOLLOWING);
  assert.ok(within(operation).getByRole("searchbox", { name: "Buscar producto" }));
  assert.ok(await within(operation).findByText("Filter"));
  assert.match(alerts.textContent ?? "", /Solo lectura/);
  assert.ok(await within(alerts).findByText("Correa"));
  assert.equal(within(alerts).queryAllByRole("button").length, 0);
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*\[data-ui-inventory-layout\] \{ grid-template-columns: minmax\(0, 1fr\); grid-template-rows: minmax\(min-content, 1fr\) max-content; \}/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-catalog-results\] \{[^}]*min-block-size:\s*auto;[^}]*flex:\s*0 0 auto/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*\[data-ui-inventory-layout\] \[data-ui-product-browser-list\] \{[^}]*min-block-size:\s*auto;[^}]*flex:\s*0 0 auto;[^}]*overflow-y:\s*visible/);
  assert.match(css, /\[data-ui-shell-content\] \{[^}]*overflow: auto/);
  assert.match(css, /--size-shell-sidebar: 208px/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*--size-shell-sidebar: 176px/);
});

test("uses the alert stock filter for sidebar entry without a duplicate browse", async () => {
  const calls: unknown[] = [];
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") { calls.push(payload); return browse(); }
    throw new Error(`Unexpected command: ${command}`);
  });
  const view = render(createElement(InventoryScreen, { initialStockState: "alerts" }));
  await screen.findByText("Filter");
  assert.deepEqual(calls, [{ request: { query: null, category_id: null, stock_state: "alerts", activity: "active", page: 1, page_size: 20 } }]);
  view.unmount();
});

test("renders Spanish selection, whole-unit projection, pending lock, and success", async () => {
  let resolve!: (value: ReturnType<typeof success>) => void;
  let requestId = "";
  let confirmations = 0;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") {
      confirmations += 1;
      requestId = String((payload?.request as { request_id?: unknown }).request_id);
      return new Promise((done) => { resolve = done; });
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  screen.getByRole("heading", { name: "Inventario", level: 1 });
  const user = await searchAndSelect();
  assert.ok(screen.getByText("Filter"));
  assert.ok(screen.getByText("SKU: FLT"));
  assert.ok(screen.getByText("Stock actual: 8"));
  const operation = screen.getByRole("region", { name: "Operación de inventario" });
  const alerts = screen.getByRole("region", { name: "Alertas de stock" });
  assert.equal(within(operation).queryByRole("searchbox"), null);
  assert.ok(within(operation).getByRole("group", { name: "Operación" }));
  assert.equal((within(operation).getByRole("radio", { name: /Entrada de stock/ }) as HTMLInputElement).checked, true);
  assert.equal((within(operation).getByRole("radio", { name: /Conteo físico/ }) as HTMLInputElement).checked, false);
  assert.ok(within(alerts).getByText("No hay alertas de stock."));
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "18,00");
  assert.ok(screen.getByText("Saldo proyectado: 11"));
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  const saving = screen.getByRole("button", { name: "Guardando…" });
  assert.equal((saving as HTMLButtonElement).disabled, true);
  assert.equal((screen.getByRole("radio", { name: /Entrada de stock/ }) as HTMLInputElement).disabled, true);
  assert.equal((screen.getByRole("radio", { name: /Conteo físico/ }) as HTMLInputElement).disabled, true);
  assert.equal((screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }) as HTMLInputElement).disabled, true);
  assert.equal((screen.getByRole("button", { name: "Nueva operación" }) as HTMLButtonElement).disabled, true);
  assert.ok(screen.getByRole("region", { name: "Alertas de stock" }));
  await user.click(saving);
  assert.equal(confirmations, 1);
  resolve(success(requestId));
  assert.ok(await screen.findByText("Operación guardada. Stock actual: 11."));
  assert.ok(screen.getByText("Saldo proyectado desactualizado. Revisá el stock actual."));
});

test("keeps entry and count control rows before one-sided descriptions", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  const fields = () => [...document.querySelectorAll("[data-ui-inventory-fields] > [data-ui-field]")];
  const assertControlRow = (field: Element, name: string) => {
    assert.equal(field.children[0].tagName, "LABEL");
    assert.equal(field.children[0].textContent, name);
    assert.equal(field.children[1].getAttribute("data-ui-field-control") !== null, true);
    const input = field.querySelector("input")!;
    assert.equal((field.children[0] as HTMLLabelElement).htmlFor, input.id);
    return input;
  };
  let [quantity, purchase, sale, minimum, note] = fields();
  let quantityInput = assertControlRow(quantity, "Cantidad (unidades enteras)");
  assertControlRow(purchase, "Precio de compra (Bs)");
  assertControlRow(sale, "Precio de venta (Bs)");
  assertControlRow(minimum, "Precio mínimo de venta (Bs)");
  assertControlRow(note, "Nota (opcional)");
  assert.equal(quantity.children[2].tagName, "SMALL");
  assert.equal(quantityInput.getAttribute("aria-describedby"), quantity.children[2].id);
  assert.equal(note.children.length, 2);
  await user.type(quantityInput, "0");
  [quantity, purchase, sale, minimum, note] = fields();
  quantityInput = assertControlRow(quantity, "Cantidad (unidades enteras)");
  assertControlRow(purchase, "Precio de compra (Bs)");
  assertControlRow(sale, "Precio de venta (Bs)");
  assertControlRow(minimum, "Precio mínimo de venta (Bs)");
  assertControlRow(note, "Nota (opcional)");
  assert.equal(quantity.children[3].getAttribute("data-ui-field-error"), "true");
  assert.equal(quantityInput.getAttribute("aria-invalid"), "true");
  assert.equal(quantityInput.getAttribute("aria-describedby"), `${quantity.children[2].id} ${quantity.children[3].id}`);
  assert.equal(note.children.length, 2);

  await user.click(screen.getByRole("radio", { name: /Conteo físico/ }));
  let [count, reason] = fields();
  assertControlRow(count, "Conteo físico (unidades enteras)");
  assertControlRow(reason, "Motivo");
  assert.equal(screen.queryByRole("textbox", { name: /Precio/ }), null);
  let reasonInput = assertControlRow(reason, "Motivo");
  assert.equal(count.children.length, 2);
  assert.equal(reason.children[2].getAttribute("data-ui-field-error"), "true");
  assert.equal(reasonInput.getAttribute("aria-invalid"), "true");
  assert.equal(reasonInput.getAttribute("aria-describedby"), reason.children[2].id);
  await user.type(reasonInput, "Recuento");
  [count, reason] = fields();
  assertControlRow(count, "Conteo físico (unidades enteras)");
  reasonInput = assertControlRow(reason, "Motivo");
  assert.equal(reason.children.length, 2);
  assert.equal(reasonInput.hasAttribute("aria-invalid"), false);

  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /\[data-ui-inventory-fields\] \{[^}]*grid-template-columns:\s*repeat\(2, minmax\(0, 1fr\)\)[^}]*align-items:\s*start/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*\[data-ui-inventory-fields\] \{ grid-template-columns: repeat\(2, minmax\(0, 1fr\)\); \}/);
});

test("requests the owning App to refresh sidebar alerts after a successful mutation", async () => {
  let ownerRefreshes = 0;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") return success("inventory-request");
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, { onInventoryAlertsRefresh: () => { ownerRefreshes += 1; } }));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "18,00");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await screen.findByText("Operación guardada. Stock actual: 11.");
  assert.equal(ownerRefreshes, 1);
});

test("shows loading/no-results, physical-count validation, and exact prioritized stock cues", async () => {
  let searches = 0;
  let resolveSearch!: (value: unknown) => void;
  let physicalCount: unknown;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [
      { product_id: 2, product_name: "Bujía", quantity: 1, classification: "low_stock" },
      { product_id: 3, product_name: "Correa", quantity: 0, classification: "out_of_stock" },
    ] };
    if (command === "browse_products_command") return ++searches === 1 ? browse([]) : searches === 2 ? new Promise((resolve) => { resolveSearch = (value) => resolve(browse(value as typeof product[])); }) : browse();
    if (command === "confirm_physical_count_command") { physicalCount = (payload?.request as { count?: unknown }).count; return success("count-request"); }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  assert.ok(screen.getByText("Buscando productos…"));
  await waitFor(() => assert.equal(searches, 1));
  const user = userEvent.setup({ document });
  const search = screen.getByRole("searchbox", { name: "Buscar producto" });
  await user.type(search, "nada{Enter}");
  assert.ok(screen.getByText("Buscando productos…"));
  resolveSearch([]);
  assert.ok(await screen.findByText("No encontramos productos para “nada”."));
  await user.clear(search);
  await user.type(search, "filter{Enter}");
  await user.click(await screen.findByRole("button", { name: "Seleccionar Filter (SKU: FLT)" }));
  const stockEntry = screen.getByRole("radio", { name: /Entrada de stock/ }) as HTMLInputElement;
  stockEntry.focus();
  await user.keyboard("{ArrowRight}");
  const physicalCountChoice = screen.getByRole("radio", { name: /Conteo físico/ }) as HTMLInputElement;
  assert.equal(physicalCountChoice.checked, true);
  assert.equal(document.activeElement, physicalCountChoice);
  assert.equal(screen.queryByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), null);
  await user.type(screen.getByRole("textbox", { name: "Motivo" }), "Producto dañado");
  assert.equal(screen.queryByText("Saldo proyectado: 0"), null);
  assert.equal((screen.getByRole("button", { name: "Confirmar operación" }) as HTMLButtonElement).disabled, true);
  await user.type(screen.getByRole("spinbutton", { name: "Conteo físico (unidades enteras)" }), "0");
  await user.type(screen.getByLabelText("Contraseña del catálogo"), "current-password");
  assert.ok(screen.getByText("Saldo proyectado: 0"));
  assert.ok(within(screen.getByRole("region", { name: "Operación de inventario" })).getByRole("textbox", { name: "Motivo" }));
  assert.ok(within(screen.getByRole("region", { name: "Alertas de stock" })).getByText("Correa"));
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await waitFor(() => assert.equal(physicalCount, 0));
  assert.equal((physicalCountChoice as HTMLInputElement).checked, true);
  const items = within(screen.getByRole("region", { name: "Alertas de stock" })).getAllByRole("listitem");
  assert.match(items[0].textContent ?? "", /Sin stock: 0.*Correa/);
  assert.match(items[1].textContent ?? "", /Stock bajo: 1.*Bujía/);
});

test("requires an accessible Catalog password for physical counts and never exposes the secret", async () => {
  const requests: Array<Record<string, unknown>> = [];
  let attempts = 0;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_physical_count_command") {
      requests.push(payload?.request as Record<string, unknown>);
      attempts += 1;
      return attempts === 1 ? { kind: "error", code: "catalog_password_invalid", message: "native secret" } : success("authorized-count");
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  await user.click(screen.getByRole("radio", { name: /Conteo físico/ }));
  await user.type(screen.getByRole("spinbutton", { name: "Conteo físico (unidades enteras)" }), "0");
  await user.type(screen.getByRole("textbox", { name: "Motivo" }), "Recuento");
  const submit = screen.getByRole("button", { name: "Confirmar operación" });
  assert.equal((submit as HTMLButtonElement).disabled, true);
  const password = screen.getByLabelText("Contraseña del catálogo") as HTMLInputElement;
  assert.equal(password.type, "password");
  await user.type(password, "catalog-secret");
  assert.equal((submit as HTMLButtonElement).disabled, false);
  await user.click(screen.getByRole("radio", { name: /Entrada de stock/ }));
  await user.click(screen.getByRole("radio", { name: /Conteo físico/ }));
  assert.equal((screen.getByLabelText("Contraseña del catálogo") as HTMLInputElement).value, "");
  await user.type(screen.getByLabelText("Contraseña del catálogo"), "catalog-secret");
  await user.click(submit);
  const error = await screen.findByRole("alert");
  assert.match(error.textContent ?? "", /La contraseña del catálogo no es correcta/);
  assert.doesNotMatch(document.body.textContent ?? "", /catalog-secret|native secret/);
  assert.equal(password.value, "");
  await user.type(password, "current-secret");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await screen.findByText("Operación guardada. Stock actual: 11.");
  assert.equal(password.value, "");
  assert.deepEqual(requests.map(({ catalog_password, ...rest }) => rest), [
    { request_id: requests[0].request_id, product_id: 1, count: 0, reason: "Recuento" },
    { request_id: requests[1].request_id, product_id: 1, count: 0, reason: "Recuento" },
  ]);
  assert.deepEqual(requests.map((request) => request.catalog_password), ["catalog-secret", "current-secret"]);
  await user.click(screen.getByRole("button", { name: "Nueva operación" }));
  await user.click(await screen.findByRole("button", { name: "Seleccionar Filter (SKU: FLT)" }));
  await user.click(screen.getByRole("radio", { name: /Conteo físico/ }));
  assert.equal((screen.getByLabelText("Contraseña del catálogo") as HTMLInputElement).value, "");
});

test("locks both operation cards while a physical count is pending", async () => {
  let resolve!: (value: ReturnType<typeof success>) => void;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_physical_count_command") return new Promise((done) => { resolve = done; });
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  await user.click(screen.getByRole("radio", { name: /Conteo físico/ }));
  await user.type(screen.getByRole("spinbutton", { name: "Conteo físico (unidades enteras)" }), "0");
  await user.type(screen.getByRole("textbox", { name: "Motivo" }), "Recuento de depósito");
  await user.type(screen.getByLabelText("Contraseña del catálogo"), "current-password");
  assert.ok(screen.getByText("Saldo proyectado: 0"));
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  assert.equal((screen.getByRole("radio", { name: /Entrada de stock/ }) as HTMLInputElement).disabled, true);
  assert.equal((screen.getByRole("radio", { name: /Conteo físico/ }) as HTMLInputElement).disabled, true);
  assert.equal((screen.getByRole("spinbutton", { name: "Conteo físico (unidades enteras)" }) as HTMLInputElement).disabled, true);
  resolve(success("count-request"));
  assert.ok(await screen.findByText("Operación guardada. Stock actual: 11."));
});

test("renders category and stock filters through the paged inventory browse contract", async () => {
  const calls: unknown[] = [];
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") { calls.push(payload); return browse(); }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  const user = userEvent.setup({ document });
  await user.selectOptions(screen.getByRole("combobox", { name: "Estado del stock" }), "out_of_stock");
  await user.click(screen.getByRole("button", { name: "Buscar" }));
  await user.selectOptions(await screen.findByRole("combobox", { name: "Categoría" }), "1");
  await user.click(screen.getByRole("button", { name: "Buscar" }));
  assert.deepEqual(calls, [
    { request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } },
    { request: { query: null, category_id: null, stock_state: "out_of_stock", activity: "active", page: 1, page_size: 20 } },
    { request: { query: null, category_id: 1, stock_state: "out_of_stock", activity: "active", page: 1, page_size: 20 } },
  ]);
});

test("localizes failure and preserves retry", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") return { kind: "error", code: "persistence_failure", message: "Native" };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "18,00");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  assert.match((await screen.findByRole("alert")).textContent ?? "", /^No se pudo guardar la operación de inventario\. Reintentá\./);
  assert.ok(screen.getByRole("button", { name: "Reintentar" }));
});

test("retries an exact inventory envelope and replaces its identity after edits and a new operation", async () => {
  installUuid("inventory-request-1", "inventory-request-2", "inventory-request-3");
  const requests: Array<{ request_id: string; product_id: number; quantity: number; unit_purchase_price_centavos: number; sale_price_centavos?: number; note: string | null }> = [];
  let confirmations = 0;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") {
      const request = payload?.request as typeof requests[number];
      requests.push(request);
      confirmations += 1;
      return confirmations < 3 ? { kind: "error", code: "persistence_failure", message: "Native" } : success(request.request_id);
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "18,00");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await screen.findByRole("alert");
  await user.click(screen.getByRole("button", { name: "Reintentar" }));
  await waitFor(() => assert.equal(requests.length, 2));
  assert.deepEqual(requests[1], requests[0]);

  await user.type(screen.getByRole("textbox", { name: "Precio de venta (Bs)" }), "30,00");
  await user.type(screen.getByRole("textbox", { name: "Nota (opcional)" }), "Conteo de depósito");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await screen.findByText("Operación guardada. Stock actual: 11.");
  assert.notEqual(requests[2].request_id, requests[1].request_id);
  assert.equal(requests[2].note, "Conteo de depósito");
  assert.equal(requests[2].sale_price_centavos, 3000);

  await user.click(screen.getByRole("button", { name: "Nueva operación" }));
  await user.click(await screen.findByRole("button", { name: "Seleccionar Filter (SKU: FLT)" }));
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "18,00");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await waitFor(() => assert.equal(requests.length, 4));
  assert.notEqual(requests[3].request_id, requests[2].request_id);
});

test("omits blank optional prices from the stock-entry IPC request", async () => {
  let request: Record<string, unknown> | undefined;
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") { request = payload?.request as Record<string, unknown>; return success(String(request.request_id)); }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await screen.findByText("Operación guardada. Stock actual: 11.");
  assert.ok(request);
  assert.equal("unit_purchase_price_centavos" in request, false);
  assert.equal("sale_price_centavos" in request, false);
  assert.equal("minimum_sale_price_centavos" in request, false);
});

test("shows a specific neutral message for reused inventory requests", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") return { kind: "error", code: "request_conflict", message: "Native storage and digest details" };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen, {}));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.type(screen.getByRole("textbox", { name: "Precio de compra (Bs)" }), "18,00");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  const feedback = await screen.findByRole("alert");
  assert.match(feedback.textContent ?? "", /^El ID de solicitud ya fue usado con datos de inventario diferentes\. Reintentá con los datos correctos\./);
  assert.equal(screen.queryByText(/Operación guardada/), null);
  assert.doesNotMatch(feedback.textContent ?? "", /storage|digest|sqlite|database/i);
  assert.ok(screen.getByRole("button", { name: "Reintentar" }));
});

test("shows the assigned generated location in the selected product without changing its global stock balance", async () => {
  const located = { ...product, primary_location_code: "A1-SHELF2" };
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse([located]);
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  const selection = screen.getByText("Ubicación principal: A1-SHELF2");
  assert.equal(selection.getAttribute("data-ui-inventory-primary-location"), "true");
  assert.ok(screen.getByText("Stock actual: 8"));
  assert.equal(screen.getByRole("button", { name: "Nueva operación" }).textContent, "Nueva operación");
  await user.click(screen.getByRole("button", { name: "Nueva operación" }));
  assert.equal(screen.queryByText("Ubicación principal: A1-SHELF2"), null);
});

test("uses All Stock for normal Inventory entry and stock-alert cue navigation", async () => {
  const browseRequests: unknown[] = [];
  let cueVisible = false;
  mockIPC((command, payload) => {
    if (command === "license_status_command") return { kind: "status", code: "active" };
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: cueVisible ? [{ product_id: 2, product_name: "Correa", quantity: 0, classification: "out_of_stock" }] : [] };
    if (command === "dashboard_command") return { kind: "error", code: "persistence_failure", message: "unavailable" };
    if (command === "browse_products_command") { browseRequests.push(payload); return browse(); }
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(App));
  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  const inventory = within(navigation).getByRole("button", { name: /^Inventario/ });
  await user.click(inventory);
  const stockFilter = await screen.findByRole("combobox", { name: "Estado del stock" }) as HTMLSelectElement;
  assert.equal(stockFilter.value, "all");
  assert.ok(await screen.findByRole("region", { name: "Alertas de stock" }));
  assert.deepEqual(browseRequests.at(-1), { request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } });

  cueVisible = true;
  await user.click(within(navigation).getByRole("button", { name: "Métricas" }));
  await screen.findByRole("heading", { name: "Métricas" });
  await waitFor(() => assert.match(inventory.textContent ?? "", /⚠ 1 alerta de stock/));
  await user.click(inventory);
  const returnedFilter = await screen.findByRole("combobox", { name: "Estado del stock" }) as HTMLSelectElement;
  assert.equal(returnedFilter.value, "all");
  assert.ok(screen.getByRole("region", { name: "Alertas de stock" }));
  assert.deepEqual(browseRequests.at(-1), { request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } });
});

test("shared inventory outcomes have one owner; dismissal preserves fields, advisory and retry envelope", async () => {
  installUuid("notice-request-1", "notice-request-2", "notice-request-3");
  const requests: Array<Record<string, unknown>> = [];
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") {
      requests.push(payload?.request as Record<string, unknown>);
      return requests.length === 1 ? { kind: "error", code: "persistence_failure", message: "Native" } : success(String(requests.at(-1)!.request_id));
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(ActionNotificationProvider, null, createElement(InventoryScreen)));
  const user = await searchAndSelect();
  const quantity = screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" });
  await user.type(quantity, "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
  assert.ok(await within(host).findByText("No se pudo guardar la operación de inventario. Reintentá."));
  const operation = screen.getByRole("region", { name: "Operación de inventario" });
  assert.equal(within(operation).queryByText("No se pudo guardar la operación de inventario. Reintentá."), null);
  await user.click(within(host).getByRole("button", { name: /Cerrar notificación/ }));
  assert.equal((quantity as HTMLInputElement).value, "3");
  await user.click(within(operation).getByRole("button", { name: "Reintentar" }));
  assert.ok(await within(host).findByText("Operación guardada. Stock actual: 11."));
  assert.equal(within(operation).queryByText("Operación guardada. Stock actual: 11."), null);
  assert.deepEqual(requests[1], requests[0]);
  assert.ok(within(operation).getByText("Saldo proyectado desactualizado. Revisá el stock actual."));
  await user.click(within(host).getByRole("button", { name: /Cerrar notificación/ }));
  assert.ok(within(operation).getByText("Saldo proyectado desactualizado. Revisá el stock actual."));
  assert.equal((quantity as HTMLInputElement).value, "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await within(host).findByText("Operación guardada. Stock actual: 11.");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await waitFor(() => assert.equal(within(host).getAllByText("Operación guardada. Stock actual: 11.").length, 2));
  assert.notEqual(requests[2].request_id, requests[3].request_id);
});

test("a duplicate click and an unmounted inventory completion never publish", async () => {
  let resolve!: (value: ReturnType<typeof success>) => void;
  let confirmations = 0;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") { confirmations++; return new Promise((done) => { resolve = done; }); }
    throw new Error(`Unexpected command: ${command}`);
  });
  const view = render(createElement(ActionNotificationProvider, null, createElement(InventoryScreen)));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  await user.click(screen.getByRole("button", { name: "Guardando…" }));
  assert.equal(confirmations, 1);
  view.rerender(createElement(ActionNotificationProvider));
  await act(async () => { resolve(success("late-request")); });
  assert.equal(screen.queryByText("Operación guardada. Stock actual: 11."), null);
  assert.equal(document.querySelectorAll("[data-ui-action-notice]").length, 0);
});

test("accepted save notifies even when the subsequent read-only alert refresh fails", async () => {
  let saved = false;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return saved ? { kind: "error", code: "persistence_failure", message: "Native" } : { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") { saved = true; return success("saved-request"); }
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(ActionNotificationProvider, null, createElement(InventoryScreen)));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  assert.ok(await within(screen.getByRole("region", { name: "Notificaciones de acciones" })).findByText("Operación guardada. Stock actual: 11."));
  assert.ok(await within(screen.getByRole("region", { name: "Alertas de stock" })).findByText("Las alertas de stock no están disponibles."));
  assert.equal(document.querySelectorAll("[data-ui-action-notice]").length, 1);
});

for (const code of ["invalid_price", "persisted_data_invalid"] as const) {
  test(`shared host leaves ${code} validation/recovery feedback inline`, async () => {
    mockIPC((command) => {
      if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
      if (command === "browse_products_command") return browse();
      if (command === "confirm_stock_entry_command") return { kind: "error", code, message: "Native" };
      throw new Error(`Unexpected command: ${command}`);
    });
    render(createElement(ActionNotificationProvider, null, createElement(InventoryScreen)));
    const user = await searchAndSelect();
    await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
    await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
    assert.ok(await within(screen.getByRole("region", { name: "Operación de inventario" })).findByText("No se pudo guardar la operación de inventario. Reintentá."));
    assert.equal(document.querySelectorAll("[data-ui-action-notice]").length, 0);
  });
}

test("shared host does not migrate request-conflict recovery guidance", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "browse_products_command") return browse();
    if (command === "confirm_stock_entry_command") return { kind: "error", code: "request_conflict", message: "Native" };
    throw new Error(`Unexpected command: ${command}`);
  });
  render(createElement(ActionNotificationProvider, null, createElement(InventoryScreen)));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Cantidad (unidades enteras)" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirmar operación" }));
  assert.ok(await within(screen.getByRole("region", { name: "Operación de inventario" })).findByText(/El ID de solicitud/));
  assert.equal(document.querySelectorAll("[data-ui-action-notice]").length, 0);
});

test("clears the public stock cue while alerts load or are unavailable", async () => {
  let resolveAlerts!: (value: unknown) => void;
  let calls = 0;
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") {
      calls += 1;
      if (calls === 1) return new Promise((resolve) => { resolveAlerts = resolve; });
      return { kind: "alerts", alerts: [] };
    }
    if (command === "browse_products_command") return browse([]);
    throw new Error(`Unexpected command: ${command}`);
  });
  const cues: Array<string | null> = [];
  render(createElement(InventoryScreen, { onAlertCueChange: (cue) => cues.push(cue) }));
  assert.ok(screen.getByText("Cargando alertas de stock…"));
  await waitFor(() => assert.equal(typeof resolveAlerts, "function"));
  resolveAlerts({ kind: "error", code: "persistence_failure", message: "Native" });
  assert.ok(await screen.findByText("Las alertas de stock no están disponibles."));
  assert.equal(cues.at(-1), null);
  await userEvent.click(screen.getByRole("button", { name: "Reintentar" }));
  await waitFor(() => assert.equal(calls, 2));
  assert.ok(await screen.findByText("No hay alertas de stock."));
});
