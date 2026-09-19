import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SaleScreen } from "./sale-screen.ts";

const style = document.createElement("style");
style.textContent = await readFile(new URL("../styles.css", import.meta.url), "utf8");
document.head.append(style);

const UUID = "550e8400-e29b-41d4-a716-446655440060";
const RETRY_UUID = "550e8400-e29b-41d4-a716-446655440061";
const products = [
  { product_id: 1, category_id: 1, sku: "FIL-1", name: "Filtro aceite", category_name: "Filtros", available_quantity: 8, catalog_unit_price_centavos: 8550, list_price_centavos: 8550, minimum_sale_price_centavos: 8550, revision: 2 },
  { product_id: 2, category_id: 1, sku: "FIL-2", name: "Filtro premium", category_name: "Filtros", available_quantity: 1, catalog_unit_price_centavos: 12550, list_price_centavos: 12550, minimum_sale_price_centavos: 12550, revision: 3 },
  { product_id: 3, category_id: 1, sku: "FIL-0", name: "Filtro agotado", category_name: "Filtros", available_quantity: 0, catalog_unit_price_centavos: 5000, list_price_centavos: 5000, minimum_sale_price_centavos: 5000, revision: 1 },
];
const browse = (items: typeof products) => ({ kind: "success", products: items, categories: [{ category_id: 1, name: "Filtros" }], page: 1, page_size: 20, total: items.length, total_pages: items.length ? 1 : 0 });
const success = { kind: "success", sale_id: 9, request_id: UUID, status: "confirmed", confirmed_at: "2026-01-02T10:00:00Z", outcome: "confirmed", lines: [], payments: [], total_centavos: 8550 };
const deferred = <T,>() => { let resolve!: (value: T) => void; let reject!: (reason?: unknown) => void; const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; };
const user = () => userEvent.setup({ document });
async function searchFor(value = "filtro") { const u = user(); await u.type(screen.getByRole("searchbox", { name: "Buscar en el catálogo" }), `${value}{Enter}`); return u; }
async function addFirst() { const u = await searchFor(); await u.click(await screen.findByRole("button", { name: "Agregar", exact: true })); await u.click(screen.getByRole("button", { name: "Revisar y cobrar" })); return u; }
function installUuid(...ids: string[]) { let index = 0; Object.defineProperty(globalThis.crypto, "randomUUID", { configurable: true, value: () => ids[Math.min(index++, ids.length - 1)] ?? UUID }); }

test("automatically loads the active first page once on mount", async () => {
  const calls: unknown[] = [];
  mockIPC((command, payload) => {
    if (command === "browse_products_command") { calls.push(payload); return browse([products[0]]); }
    throw new Error(`Unexpected command: ${command}`);
  });
  const view = render(createElement(SaleScreen));
  await screen.findByText("Filtro aceite");
  assert.deepEqual(calls, [{ request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } }]);
  view.unmount();
});

test("contains the product result viewport between search and pagination without moving sale controls", async () => {
  const manyProducts = Array.from({ length: 100 }, (_, index) => ({ ...products[0], product_id: index + 1, sku: `FIL-${index + 1}` }));
  mockIPC((command) => command === "browse_products_command" ? { ...browse(manyProducts), total_pages: 5 } : Promise.reject(new Error("unexpected command")));
  render(createElement(SaleScreen));
  const catalog = await screen.findByRole("region", { name: "Catálogo" });
  const list = within(catalog).getByRole("list", { name: "Resultados del catálogo" });
  assert.equal(list.getAttribute("data-ui-product-browser-list"), "true");
  assert.equal(list.previousElementSibling?.tagName, "FORM");
  assert.equal(list.nextElementSibling?.getAttribute("data-ui-product-browser-pages"), "true");
  assert.equal(within(catalog).getAllByRole("listitem").length, 100);
  assert.ok(screen.getByRole("region", { name: "Resumen de venta" }));
  assert.equal(screen.queryByRole("region", { name: "Carrito" }), null);
  assert.equal(screen.queryByRole("region", { name: "Pago" }), null);
  assert.match(style.textContent ?? "", /data-ui-product-browser-list[^}]*--product-browser-row-block-size:\s*calc\([^}]*\)[^}]*min-block-size:\s*calc\(\s*var\(--product-browser-row-block-size\)\s*\+\s*var\(--product-browser-row-block-size\)\s*\+\s*var\(--product-browser-row-block-size\)/s);
  assert.match(style.textContent ?? "", /data-ui-product-browser-list[^}]*flex:\s*1 1 auto[^}]*overflow-y:\s*auto/s);
  assert.match(style.textContent ?? "", /data-ui-sale-layout[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\) minmax\(260px,\s*320px\)/s);
  assert.match(style.textContent ?? "", /data-ui-sale-layout[^}]*grid-template-rows:\s*minmax\(0,\s*1fr\)/s);
});

test("shows every discovery state and ignores reverse-order search completion", async () => {
  const first = deferred<unknown>(), second = deferred<unknown>(), third = deferred<unknown>(), fourth = deferred<unknown>(); let call = 0;
  mockIPC((command) => command === "browse_products_command" ? [first, second, third, fourth][call++].promise : Promise.reject());
  const view = render(createElement(SaleScreen));
  screen.getByText("Buscando productos…");
  const heading = screen.getByRole("heading", { level: 1, name: "Ventas" });
  assert.equal(heading.textContent, "Ventas"); assert.equal(heading.getAttribute("aria-label"), null); assert.equal(screen.getAllByRole("heading", { level: 1 }).length, 1);
  assert.deepEqual(screen.getAllByRole("heading", { level: 2 }).map((node) => node.textContent), ["Catálogo", "Resumen de venta"]);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-sale-layout[\s\S]*grid-template-columns: minmax\(0, 1fr\)/);
  const u = await searchFor("viejo"); screen.getByText("Buscando productos…");
  await u.clear(screen.getByRole("searchbox")); await u.type(screen.getByRole("searchbox"), "nuevo{Enter}");
  await act(() => { third.resolve(browse(products)); return third.promise; });
  screen.getByText("Filtro aceite"); screen.getByText("Disponible: 8"); screen.getByText("Stock bajo: 1"); screen.getByText("Sin stock: 0");
  await act(() => { first.resolve(browse([])); return first.promise; }); screen.getByText("Filtro aceite");
  await u.clear(screen.getByRole("searchbox")); await u.type(screen.getByRole("searchbox"), "tarde{Enter}"); view.unmount();
  await act(() => { fourth.resolve(browse(products)); return fourth.promise; }); assert.equal(document.body.textContent, "")
});

test("refreshes the inventory summary after a confirmed sale", async () => {
  installUuid();
  let refreshed = 0;
  mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : success);
  render(createElement(SaleScreen, { onInventoryAlertsRefresh: () => { refreshed += 1; } }));
  const u = await addFirst();
  await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  await screen.findByRole("heading", { name: "Venta confirmada" });
  assert.equal(refreshed, 1);
});

test("routes nonblank global sales search through the canonical paged browse command", async () => {
  const calls: Array<{ command: string; payload: unknown }> = [];
  mockIPC((command, payload) => { calls.push({ command, payload }); return command === "browse_products_command" ? browse([products[0]]) : Promise.reject(new Error("unexpected command")); });
  render(createElement(SaleScreen));
  await searchFor("filtro");
  assert.deepEqual(calls, [
    { command: "browse_products_command", payload: { request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } } },
    { command: "browse_products_command", payload: { request: { query: "filtro", category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } } },
  ]);
  assert.ok(screen.getByRole("button", { name: "Agregar" }));
});

test("distinguishes empty and failed search while preserving the draft query", async () => {
  let mode: "empty" | "error" = "empty";
  mockIPC((command) => command === "browse_products_command" ? mode === "empty" ? browse([]) : Promise.reject(new Error("native path")) : Promise.reject(new Error("unexpected command")));
  render(createElement(SaleScreen)); const u = await searchFor("correa");
  await screen.findByText("No encontramos productos para “correa”.");
  mode = "error"; await u.click(screen.getByRole("button", { name: "Buscar" }));
  assert.equal((await screen.findByRole("alert")).textContent, "No se pudo buscar en el catálogo local.");
  assert.equal((screen.getByRole("searchbox") as HTMLInputElement).value, "correa");
});

test("renders stock actions and manages whole quantities, subtotals, total, removal and discard", async () => {
  mockIPC((command) => command === "browse_products_command" ? browse(products) : Promise.reject(new Error("unexpected command"))); render(createElement(SaleScreen)); const u = await searchFor();
  const add = await screen.findAllByRole("button", { name: "Agregar" });
  assert.equal((add[2] as HTMLButtonElement).disabled, true); await u.click(add[0]); assert.equal((add[0] as HTMLButtonElement).disabled, true);
  await u.click(screen.getByRole("button", { name: "Revisar y cobrar" }));
  const dialog = screen.getByRole("dialog", { name: "Revisar y cobrar" });
  assert.equal(dialog.getAttribute("data-ui-dialog-layout"), "checkout");
  const checkoutContent = dialog.querySelector("[data-ui-checkout-content]");
  assert.equal(checkoutContent?.getAttribute("data-ui-density"), "sparse");
  assert.match(style.textContent ?? "", /data-ui-checkout-dialog[^}]*:has\(> \[data-ui-checkout-content\]\[data-ui-density="sparse"\]\)/);
  assert.match(style.textContent ?? "", /data-ui-checkout-content\]\[data-ui-density="sparse"\][\s\S]*data-ui-sale-cart[\s\S]*overflow-y:\s*visible/);
  const cart = within(dialog).getByRole("region", { name: "Carrito" });
  const rail = dialog.querySelector("[data-ui-checkout-rail]");
  assert.ok(rail);
  assert.equal(cart.parentElement?.getAttribute("data-ui-checkout-content"), "true");
  assert.equal(rail?.querySelector("[data-ui-checkout-total]")?.textContent, "Total actual: Bs 85,50");
  assert.equal(dialog.querySelector("[data-ui-dialog-actions]")?.getAttribute("data-ui-dialog-actions"), "true");
  screen.getByRole("heading", { name: "Carrito" }); screen.getAllByText("FIL-1"); screen.getAllByText("Bs 85,50"); screen.getByText("Total: Bs 85,50");
  const quantity = screen.getByRole("spinbutton", { name: "Cantidad de Filtro aceite" }); fireEvent.change(quantity, { target: { value: "2" } });
  screen.getByText("Subtotal: Bs 171,00"); screen.getByText("Total: Bs 171,00");
  assert.equal(rail?.querySelector("[data-ui-checkout-total]")?.textContent, "Total actual: Bs 171,00");
  within(screen.getByRole("region", { name: "Resumen de venta" })).getByText("Unidades: 2");
  quantity.focus(); fireEvent.change(quantity, { target: { value: "0" } }); screen.getByText("Ingresá una cantidad entera mayor que cero."); assert.equal(document.activeElement, quantity);
  await u.click(screen.getByRole("button", { name: "Quitar Filtro aceite" })); screen.getByText("El carrito está vacío.");
  await u.click(screen.getByRole("button", { name: "Volver" }));
  assert.equal(screen.getByRole("button", { name: "Revisar y cobrar" }).getAttribute("aria-expanded"), "false");
});

test("exposes each checkout row as a compact two-level item with unique actions", async () => {
  mockIPC((command) => command === "browse_products_command" ? browse(products) : Promise.reject(new Error("unexpected command")));
  render(createElement(SaleScreen));
  const u = await searchFor();
  const add = await screen.findAllByRole("button", { name: "Agregar" });
  await u.click(add[0]); await u.click(add[1]); await u.click(screen.getByRole("button", { name: "Revisar y cobrar" }));

  const denseCheckoutContent = screen.getByRole("dialog", { name: "Revisar y cobrar" }).querySelector("[data-ui-checkout-content]");
  assert.equal(denseCheckoutContent?.getAttribute("data-ui-density"), null);
  const cart = screen.getByRole("list", { name: "Carrito" });
  assert.equal(cart.querySelector("table"), null);
  assert.equal(cart.querySelector("[data-ui-aligned-data]"), null);
  const rows = within(cart).getAllByRole("listitem");
  assert.equal(rows.length, 2);
  for (const [row, product] of rows.map((value, index) => [value, products[index]] as const)) {
    assert.equal(row.children.length, 2);
    assert.equal(row.firstElementChild?.getAttribute("data-ui-sale-cart-primary"), "true");
    assert.equal(row.lastElementChild?.getAttribute("data-ui-sale-cart-controls"), "true");
    assert.ok(within(row).getByText(product.name));
    assert.ok(within(row).getByText(product.sku));
    assert.ok(within(row).getByText(`Lista ${product.list_price_centavos === 8550 ? "Bs 85,50" : "Bs 125,50"} · Mín. ${product.minimum_sale_price_centavos === 8550 ? "Bs 85,50" : "Bs 125,50"}`));
    assert.equal(within(row).getByRole("button", { name: `Quitar ${product.name}` }).getAttribute("aria-label"), `Quitar ${product.name}`);
    assert.ok(within(row).getByRole("spinbutton", { name: `Cantidad de ${product.name}` }));
  }
  assert.equal(within(rows[0]).getByRole("textbox", { name: "Precio de venta (Bs)" }).getAttribute("id"), "sale-final-price-1");
});

test("disables empty-cart confirmation without invoking the native contract", async () => {
  let confirms = 0;
  mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : (confirms++, success));
  render(createElement(SaleScreen));
  const u = await addFirst();
  await u.click(screen.getByRole("button", { name: "Quitar Filtro aceite" }));

  const confirm = screen.getByRole("button", { name: "Confirmar venta" }) as HTMLButtonElement;
  assert.equal(confirm.disabled, true);
  assert.equal(document.activeElement, screen.getByRole("button", { name: "Volver" }));
  fireEvent.click(confirm);
  assert.equal(confirms, 0);
  assert.equal(screen.getByRole("dialog", { name: "Revisar y cobrar" }).isConnected, true);
});

test("parses cash-only, QR-only and mixed Bs values into the exact command envelope", async () => {
  installUuid();
  for (const [cash, qr, expected] of [["85,50", "", [8550, null]], ["", "85,50", [null, 8550]], ["50", "35,5", [5000, 3550]]] as const) {
    let envelope: unknown; mockIPC((command, payload) => command === "browse_products_command" ? browse([products[0]]) : (envelope = payload, success));
    const view = render(createElement(SaleScreen)); const u = await addFirst();
    if (cash) await u.type(screen.getByRole("textbox", { name: "Efectivo recibido" }), cash);
    if (qr) await u.type(screen.getByRole("textbox", { name: "Pago QR" }), qr);
    await u.click(screen.getByRole("button", { name: "Confirmar venta" })); await screen.findByRole("heading", { name: "Venta confirmada" });
    assert.deepEqual(envelope, { request: { request_id: UUID, lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 8550, captured_revision: 2, final_unit_price_centavos: 8550 }], payment: { amount_tendered_centavos: expected[0], qr_applied_centavos: expected[1] } } });
    view.unmount();
  }
});

test("rejects malformed payment before invoke, associates correction and focuses the first field", async () => {
  let confirms = 0; mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : (confirms++, success));
  render(createElement(SaleScreen)); const u = await addFirst(); const cash = screen.getByRole("textbox", { name: "Efectivo recibido" });
  await u.type(cash, "1.25"); await u.type(screen.getByRole("textbox", { name: "Pago QR" }), "x"); await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  assert.equal(confirms, 0); assert.equal(document.activeElement, cash); assert.equal(cash.getAttribute("aria-invalid"), "true"); screen.getAllByText("Ingresá un monto válido en Bs, con hasta dos decimales.");
});

test("shows a neutral specific message for a request conflict", async () => {
  mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : ({
    kind: "error",
    code: "request_conflict",
    message: "SQLite digest details",
  }));
  render(createElement(SaleScreen));
  const u = await addFirst();
  await u.click(screen.getByRole("button", { name: "Confirmar venta" }));

  const message = await screen.findByRole("alert");
  assert.equal(
    message.textContent,
    "El ID de solicitud ya fue usado con datos de venta diferentes. Revisá la venta antes de intentar nuevamente.",
  );
  assert.doesNotMatch(message.textContent ?? "", /sqlite|digest/i);
  assert.equal(screen.queryByRole("heading", { name: "Venta confirmada" }), null);
});

test("locks every draft mutation and submitted intent during deferred confirmation", async () => {
  installUuid(); const pending = deferred<unknown>(); let confirms = 0, searches = 0, submitted: unknown;
  mockIPC((command, payload) => command === "browse_products_command" ? (searches++, browse(products)) : (confirms++, submitted = payload, pending.promise));
  render(createElement(SaleScreen)); const u = await searchFor(); await u.click((await screen.findAllByRole("button", { name: "Agregar" }))[0]); await u.click(screen.getByRole("button", { name: "Revisar y cobrar" })); await u.type(screen.getByRole("textbox", { name: "Pago QR" }), "85,50");
  fireEvent.click(screen.getByRole("button", { name: "Confirmar venta" })); fireEvent.click(screen.getByRole("button", { name: "Confirmando…" }));
  const controls = [screen.getByRole("searchbox"), screen.getByRole("button", { name: "Buscar" }), screen.getAllByRole("button", { name: "Agregar" })[1], screen.getByRole("spinbutton"), screen.getByRole("button", { name: "Quitar Filtro aceite" }), screen.getByRole("textbox", { name: "Efectivo recibido" }), screen.getByRole("textbox", { name: "Pago QR" }), screen.getByRole("button", { name: "Descartar borrador" }), screen.getByRole("button", { name: "Confirmando…" })];
  assert.ok(controls.every((control) => (control as HTMLInputElement).disabled)); assert.equal(screen.getByRole("main").getAttribute("aria-busy"), "true");
  fireEvent.change(controls[0], { target: { value: "otro" } }); fireEvent.click(controls[1]); fireEvent.click(controls[2]); fireEvent.change(controls[3], { target: { value: "2" } }); fireEvent.click(controls[4]); fireEvent.change(controls[5], { target: { value: "1" } }); fireEvent.change(controls[6], { target: { value: "2" } }); fireEvent.click(controls[7]);
  assert.equal(confirms, 1); assert.equal(searches, 2); assert.equal((screen.getByRole("searchbox") as HTMLInputElement).value, "filtro"); assert.equal((screen.getByRole("spinbutton") as HTMLInputElement).value, "1"); assert.equal((screen.getByRole("textbox", { name: "Pago QR" }) as HTMLInputElement).value, "85,50");
  assert.deepEqual(submitted, { request: { request_id: UUID, lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 8550, captured_revision: 2, final_unit_price_centavos: 8550 }], payment: { amount_tendered_centavos: null, qr_applied_centavos: 8550 } } });
  await act(() => { pending.resolve({ kind: "error", code: "insufficient_stock", message: "Insufficient stock is available." }); return pending.promise; }); screen.getByText("No hay stock suficiente para completar la venta.");
});

test("blocks a backend minimum violation, preserves the draft, and focuses its final price", async () => {
  installUuid(UUID, RETRY_UUID); let attempt = 0;
  mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : (++attempt === 1 ? { kind: "minimum_price_violation", product_id: 1, current_minimum_unit_price_centavos: 9000 } : success));
  render(createElement(SaleScreen)); const u = await addFirst(); await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  const finalPrice = screen.getByRole("textbox", { name: "Precio de venta (Bs)" });
  screen.getAllByText("El precio mínimo actual es Bs 90,00. Ajustá el precio de venta para continuar.");
  assert.equal((finalPrice as HTMLInputElement).value, "85,50"); assert.equal(document.activeElement, finalPrice);
  await u.clear(finalPrice); await u.type(finalPrice, "90,00"); await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  await screen.findByRole("heading", { name: "Venta confirmada" });
});

test("keeps cart facts bounded when a final price error is mounted", async () => {
  mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : Promise.reject(new Error("confirmation must be blocked")));
  render(createElement(SaleScreen));
  const u = await addFirst();
  const finalPrice = screen.getByRole("textbox", { name: "Precio de venta (Bs)" });
  await u.clear(finalPrice);
  await u.type(finalPrice, "80");
  await u.click(screen.getByRole("button", { name: "Confirmar venta" }));

  const cart = screen.getByRole("list", { name: "Carrito" });
  assert.equal(cart.getAttribute("data-ui-sale-cart"), "true");
  assert.equal(cart.querySelector("table"), null);
  const line = within(cart).getByRole("listitem");
  assert.equal(line.children.length, 2);
  assert.equal(line.firstElementChild?.getAttribute("data-ui-sale-cart-primary"), "true");
  assert.equal(line.lastElementChild?.getAttribute("data-ui-sale-cart-controls"), "true");
  assert.equal(within(line).getByRole("button", { name: "Quitar Filtro aceite" }).getAttribute("aria-label"), "Quitar Filtro aceite");
  assert.equal(finalPrice.getAttribute("aria-invalid"), "true");
  const errorId = finalPrice.getAttribute("aria-describedby");
  assert.ok(errorId);
  assert.equal(document.getElementById(errorId)?.textContent, "El precio de venta no puede ser menor que el precio mínimo de Bs 85,50.");
  assert.match(style.textContent ?? "", /\[data-ui-sale-cart\]\s*\{[^}]*display:\s*grid[^}]*list-style:\s*none/s);
  assert.match(style.textContent ?? "", /\[data-ui-sale-cart-row\]\s*\{[^}]*display:\s*grid[^}]*border-block-end:/s);
  assert.match(style.textContent ?? "", /\[data-ui-sale-cart-primary\]\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+auto\s+auto/s);
  assert.match(style.textContent ?? "", /\[data-ui-sale-cart-controls\]\s*\{[^}]*grid-template-columns:\s*minmax\(7rem,\s*1fr\)\s+minmax\(10rem,\s*1fr\)\s+minmax\(max-content,\s*auto\)/s);
  assert.doesNotMatch(style.textContent ?? "", /\[data-ui-sale-cart\][^\n]*table-layout/);
  assert.match(style.textContent ?? "", /@media \(min-width: 961px\)[\s\S]*data-ui-checkout-dialog\]\[data-ui-dialog-layout="checkout"\][\s\S]*inline-size:\s*min\(1040px,\s*100%\)[\s\S]*overflow:\s*hidden/s);
  assert.match(style.textContent ?? "", /@media \(min-width: 961px\)[\s\S]*data-ui-checkout-rail[\s\S]*grid-column:\s*2/s);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-checkout-dialog\]\[data-ui-dialog-layout="checkout"\][\s\S]*overflow:\s*auto/s);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-sale-cart-primary[\s\S]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+auto/s);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-sale-cart-controls[\s\S]*grid-template-columns:\s*minmax\(0,\s*1fr\)/s);
});

test("discards late confirmation after unmount and keeps the existing success handoff", async () => {
  installUuid(); const pending = deferred<unknown>(); mockIPC((command) => command === "browse_products_command" ? browse([products[0]]) : pending.promise);
  const view = render(createElement(SaleScreen)); await addFirst(); fireEvent.click(screen.getByRole("button", { name: "Confirmar venta" }));
  assert.equal((screen.getByRole("button", { name: "Descartar borrador" }) as HTMLButtonElement).disabled, true); assert.equal((screen.getByRole("button", { name: "Volver" }) as HTMLButtonElement).disabled, true); view.unmount();
  await act(() => { pending.resolve(success); return pending.promise; }); assert.equal(document.body.textContent, "");
});

test("replaces the draft with a stable persisted summary and resets through its sole action", async () => {
  installUuid();
  const catalog = [{ ...products[0] }];
  const confirmed = {
    ...success, confirmed_at: "2026-08-14T10:42:00Z", total_centavos: 10_050,
    lines: [
      { product_id: 1, sku: "HIST-1", product_name: "Filtro histórico", quantity: 2, unit_price_centavos: 4_000, line_total_centavos: 8_000 },
      { product_id: 99, sku: "SKU no disponible", product_name: "Producto no disponible", quantity: 1, unit_price_centavos: 2_050, line_total_centavos: 2_050 },
    ],
    payments: [
      { method: "cash" as const, amount_applied_centavos: 6_000, amount_tendered_centavos: 7_000, change_given_centavos: 1_000 },
      { method: "qr" as const, amount_applied_centavos: 4_050 },
    ],
  };
  let response = confirmed;
  mockIPC((command) => command === "browse_products_command" ? browse(catalog) : response);
  const view = render(createElement(SaleScreen)); const u = await addFirst(); await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  await screen.findByRole("heading", { name: "Venta confirmada" });
  const persistedSummary = screen.getByRole("main");
  assert.equal(persistedSummary.getAttribute("data-ui-persisted-summary"), "true");
  const summaryLayout = persistedSummary.querySelector("[data-ui-summary-layout]");
  assert.ok(summaryLayout);
  assert.equal(summaryLayout?.firstElementChild?.getAttribute("data-ui-summary-articles"), "true");
  assert.equal(summaryLayout?.lastElementChild?.getAttribute("data-ui-summary-rail"), "true");
  assert.equal(summaryLayout?.querySelector("[data-ui-summary-articles] table")?.getAttribute("data-ui-aligned-data"), "true");
  assert.equal(summaryLayout?.querySelector("[data-ui-summary-payments] table")?.getAttribute("data-ui-aligned-data"), "true");
  assert.equal(summaryLayout?.querySelector("[data-ui-summary-rail] button")?.textContent, "Nueva venta");
  screen.getByText("Venta #9"); screen.getByText("14/08/2026, 10:42"); screen.getByText("Filtro histórico"); screen.getByText("SKU no disponible");
  for (const fact of ["Bs 40,00", "Bs 80,00", "Bs 60,00", "Bs 70,00", "Bs 10,00", "Bs 40,50", "Bs 100,50"]) screen.getByText(fact);
  assert.equal(screen.getAllByText("Bs 20,50").length, 2);
  const summaryMain = screen.getByRole("main"), beforeMutation = summaryMain.textContent;
  const roles = ["button", "link", "textbox", "spinbutton", "combobox", "checkbox", "radio", "menuitem"] as const;
  const controls = roles.flatMap((role) => within(summaryMain).queryAllByRole(role));
  assert.deepEqual(controls.map((control) => control.textContent), ["Nueva venta"]);
  for (const role of roles) assert.equal(within(summaryMain).queryAllByRole(role, { name: /editar|imprimir|compartir|reembolsar|recibo|edit|print|share|refund|receipt/i }).length, 0);
  catalog[0].name = "Catálogo mutado";
  confirmed.lines.forEach((line) => Object.assign(line, { product_id: 0, sku: "MUT", product_name: "Resultado mutado", quantity: 0, unit_price_centavos: 0, line_total_centavos: 0 }));
  confirmed.payments.forEach((payment) => Object.assign(payment, { amount_applied_centavos: 0, amount_tendered_centavos: 0, change_given_centavos: 0 })); confirmed.total_centavos = 0;
  view.rerender(createElement(SaleScreen)); assert.equal(screen.getByRole("main").textContent, beforeMutation);
  assert.match(style.textContent ?? "", /\[data-ui-persisted-summary\][\s\S]*grid-template-rows:\s*auto auto minmax\(0,\s*1fr\)[\s\S]*overflow:\s*hidden/);
  assert.match(style.textContent ?? "", /\[data-ui-summary-layout\][\s\S]*grid-template-columns:\s*minmax\(0,\s*1fr\) minmax\(260px,\s*320px\)/);
  assert.match(style.textContent ?? "", /\[data-ui-summary-articles\][\s\S]*overflow-y:\s*auto[\s\S]*overscroll-behavior:\s*contain/);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-persisted-summary[\s\S]*block-size:\s*auto[\s\S]*overflow:\s*visible/);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-summary-layout[\s\S]*grid-template-columns:\s*minmax\(0,\s*1fr\)/);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-summary-articles[\s\S]*overflow-y:\s*visible/);
  await u.click(screen.getByRole("button", { name: "Nueva venta" }));
  response = { ...success, sale_id: 10, confirmed_at: "2026-08-14 10:42:00", lines: [{ product_id: 1, sku: "FIL-1", product_name: "Filtro aceite", quantity: 1, unit_price_centavos: 8_550, line_total_centavos: 8_550 }], payments: [{ method: "qr" as const, amount_applied_centavos: 8_550 }], total_centavos: 8_550 };
  await addFirst(); await u.click(screen.getByRole("button", { name: "Confirmar venta" })); await screen.findByText("Venta #10");
  screen.getByText("14/08/2026, 10:42"); assert.ok(screen.getAllByText("Bs 85,50").length >= 3);
  assert.equal(within(screen.getByRole("table", { name: "Pagos confirmados" })).queryAllByRole("row").length, 2);
});

test("starts and renders the active first page browse after Nueva venta", async () => {
  installUuid();
  const resetBrowse = deferred<unknown>();
  const browseCalls: unknown[] = [];
  let browseCount = 0;
  mockIPC((command, payload) => {
    if (command === "browse_products_command") {
      browseCalls.push(payload);
      browseCount += 1;
      return browseCount === 3 ? resetBrowse.promise : browse([products[0]]);
    }
    return success;
  });

  render(createElement(SaleScreen));
  const u = await addFirst();
  await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  await screen.findByRole("heading", { name: "Venta confirmada" });

  await u.click(screen.getByRole("button", { name: "Nueva venta" }));
  assert.equal(screen.queryByRole("dialog", { name: "Revisar y cobrar" }), null);
  assert.equal(browseCalls.length, 3);
  assert.deepEqual(browseCalls[2], { request: { query: null, category_id: null, stock_state: "all", activity: "active", page: 1, page_size: 20 } });
  screen.getByText("Buscando productos…");

  await act(() => { resetBrowse.resolve(browse([products[1]])); return resetBrowse.promise; });
  screen.getByText("Filtro premium");
});
