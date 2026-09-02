import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SaleScreen } from "./sale-screen.ts";

const style = document.createElement("style");
style.textContent = await readFile(new URL("../styles.css", import.meta.url), "utf8");
document.head.append(style);

const UUID = "550e8400-e29b-41d4-a716-446655440060";
const products = [
  { product_id: 1, sku: "FIL-1", name: "Filtro aceite", category_name: "Filtros", available_quantity: 8, catalog_unit_price_centavos: 8550, revision: 2 },
  { product_id: 2, sku: "FIL-2", name: "Filtro premium", category_name: "Filtros", available_quantity: 1, catalog_unit_price_centavos: 12550, revision: 3 },
  { product_id: 3, sku: "FIL-0", name: "Filtro agotado", category_name: "Filtros", available_quantity: 0, catalog_unit_price_centavos: 5000, revision: 1 },
];
const success = { kind: "success", sale_id: 9, request_id: UUID, status: "confirmed", confirmed_at: "2026-01-02T10:00:00Z", outcome: "confirmed", lines: [], payments: [], total_centavos: 8550 };
const deferred = <T,>() => { let resolve!: (value: T) => void; let reject!: (reason?: unknown) => void; const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; };
const user = () => userEvent.setup({ document });
async function searchFor(value = "filtro") { const u = user(); await u.type(screen.getByRole("searchbox", { name: "Buscar en el catálogo" }), `${value}{Enter}`); return u; }
async function addFirst() { const u = await searchFor(); await u.click(await screen.findByRole("button", { name: "Agregar", exact: true })); return u; }
function installUuid() { Object.defineProperty(globalThis.crypto, "randomUUID", { configurable: true, value: () => UUID }); }

test("shows every discovery state and ignores reverse-order search completion", async () => {
  const first = deferred<typeof products>(), second = deferred<typeof products>(), third = deferred<typeof products>(); let call = 0;
  mockIPC((command) => command === "search_products_command" ? [first, second, third][call++].promise : Promise.reject());
  const view = render(createElement(SaleScreen));
  screen.getByText("Buscá un producto para comenzar.");
  const heading = screen.getByRole("heading", { level: 1, name: "Ventas" });
  assert.equal(heading.textContent, "Ventas"); assert.equal(heading.getAttribute("aria-label"), null); assert.equal(screen.getAllByRole("heading", { level: 1 }).length, 1);
  assert.deepEqual(screen.getAllByRole("heading", { level: 2 }).map((node) => node.textContent), ["Catálogo", "Carrito", "Pago", "Resumen"]);
  assert.match(style.textContent ?? "", /@media \(max-width: 960px\)[\s\S]*data-ui-sale-layout[\s\S]*grid-template-columns: minmax\(0, 1fr\)/);
  const u = await searchFor("viejo"); screen.getByText("Buscando productos…");
  await u.clear(screen.getByRole("searchbox")); await u.type(screen.getByRole("searchbox"), "nuevo{Enter}");
  await act(() => { second.resolve(products); return second.promise; });
  screen.getByText("Filtro aceite"); screen.getByText("Disponible: 8"); screen.getByText("Stock bajo: 1"); screen.getByText("Sin stock: 0");
  await act(() => { first.resolve([]); return first.promise; }); screen.getByText("Filtro aceite");
  await u.clear(screen.getByRole("searchbox")); await u.type(screen.getByRole("searchbox"), "tarde{Enter}"); view.unmount();
  await act(() => { third.resolve(products); return third.promise; }); assert.equal(document.body.textContent, "")
});

test("distinguishes empty and failed search while preserving the draft query", async () => {
  let mode: "empty" | "error" = "empty";
  mockIPC(() => mode === "empty" ? [] : Promise.reject(new Error("native path")));
  render(createElement(SaleScreen)); const u = await searchFor("correa");
  await screen.findByText("No encontramos productos para “correa”.");
  mode = "error"; await u.click(screen.getByRole("button", { name: "Buscar" }));
  assert.equal((await screen.findByRole("alert")).textContent, "No se pudo buscar en el catálogo local.");
  assert.equal((screen.getByRole("searchbox") as HTMLInputElement).value, "correa");
});

test("renders stock actions and manages whole quantities, subtotals, total, removal and discard", async () => {
  mockIPC(() => products); render(createElement(SaleScreen)); const u = await searchFor();
  const add = await screen.findAllByRole("button", { name: "Agregar" });
  assert.equal((add[2] as HTMLButtonElement).disabled, true); await u.click(add[0]); assert.equal((add[0] as HTMLButtonElement).disabled, true);
  screen.getByRole("heading", { name: "Carrito" }); screen.getAllByText("FIL-1"); screen.getAllByText("Bs 85,50"); screen.getByText("Total: Bs 85,50");
  const quantity = screen.getByRole("spinbutton", { name: "Cantidad de Filtro aceite" }); fireEvent.change(quantity, { target: { value: "2" } });
  screen.getByText("Subtotal: Bs 171,00"); screen.getByText("Total: Bs 171,00");
  quantity.focus(); fireEvent.change(quantity, { target: { value: "0" } }); screen.getByText("Ingresá una cantidad entera mayor que cero."); assert.equal(document.activeElement, quantity);
  await u.click(screen.getByRole("button", { name: "Quitar" })); screen.getByText("El carrito está vacío.");
  await u.click(add[1]); await u.type(screen.getByRole("textbox", { name: "Efectivo recibido" }), "10"); await u.click(screen.getByRole("button", { name: "Descartar borrador" }));
  screen.getByText("El carrito está vacío."); assert.equal((screen.getByRole("textbox", { name: "Efectivo recibido" }) as HTMLInputElement).value, "");
});

test("parses cash-only, QR-only and mixed Bs values into the exact command envelope", async () => {
  installUuid();
  for (const [cash, qr, expected] of [["85,50", "", [8550, null]], ["", "85,50", [null, 8550]], ["50", "35,5", [5000, 3550]]] as const) {
    let envelope: unknown; mockIPC((command, payload) => command === "search_products_command" ? [products[0]] : (envelope = payload, success));
    const view = render(createElement(SaleScreen)); const u = await addFirst();
    if (cash) await u.type(screen.getByRole("textbox", { name: "Efectivo recibido" }), cash);
    if (qr) await u.type(screen.getByRole("textbox", { name: "Pago QR" }), qr);
    await u.click(screen.getByRole("button", { name: "Confirmar venta" })); await screen.findByRole("heading", { name: "Sale confirmed" });
    assert.deepEqual(envelope, { request: { request_id: UUID, lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 8550, captured_revision: 2 }], payment: { amount_tendered_centavos: expected[0], qr_applied_centavos: expected[1] } } });
    view.unmount();
  }
});

test("rejects malformed payment before invoke, associates correction and focuses the first field", async () => {
  let confirms = 0; mockIPC((command) => command === "search_products_command" ? [products[0]] : (confirms++, success));
  render(createElement(SaleScreen)); const u = await addFirst(); const cash = screen.getByRole("textbox", { name: "Efectivo recibido" });
  await u.type(cash, "1.25"); await u.type(screen.getByRole("textbox", { name: "Pago QR" }), "x"); await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  assert.equal(confirms, 0); assert.equal(document.activeElement, cash); assert.equal(cash.getAttribute("aria-invalid"), "true"); screen.getAllByText("Ingresá un monto válido en Bs, con hasta dos decimales.");
});

test("locks every draft mutation and submitted intent during deferred confirmation", async () => {
  installUuid(); const pending = deferred<unknown>(); let confirms = 0, searches = 0, submitted: unknown;
  mockIPC((command, payload) => command === "search_products_command" ? (searches++, products) : (confirms++, submitted = payload, pending.promise));
  render(createElement(SaleScreen)); const u = await searchFor(); await u.click((await screen.findAllByRole("button", { name: "Agregar" }))[0]); await u.type(screen.getByRole("textbox", { name: "Pago QR" }), "85,50");
  fireEvent.click(screen.getByRole("button", { name: "Confirmar venta" })); fireEvent.click(screen.getByRole("button", { name: "Confirmando…" }));
  const controls = [screen.getByRole("searchbox"), screen.getByRole("button", { name: "Buscar" }), screen.getAllByRole("button", { name: "Agregar" })[1], screen.getByRole("spinbutton"), screen.getByRole("button", { name: "Quitar" }), screen.getByRole("textbox", { name: "Efectivo recibido" }), screen.getByRole("textbox", { name: "Pago QR" }), screen.getByRole("button", { name: "Descartar borrador" }), screen.getByRole("button", { name: "Confirmando…" })];
  assert.ok(controls.every((control) => (control as HTMLInputElement).disabled)); assert.equal(screen.getByRole("main").getAttribute("aria-busy"), "true");
  fireEvent.change(controls[0], { target: { value: "otro" } }); fireEvent.click(controls[1]); fireEvent.click(controls[2]); fireEvent.change(controls[3], { target: { value: "2" } }); fireEvent.click(controls[4]); fireEvent.change(controls[5], { target: { value: "1" } }); fireEvent.change(controls[6], { target: { value: "2" } }); fireEvent.click(controls[7]);
  assert.equal(confirms, 1); assert.equal(searches, 1); assert.equal((screen.getByRole("searchbox") as HTMLInputElement).value, "filtro"); assert.equal((screen.getByRole("spinbutton") as HTMLInputElement).value, "1"); assert.equal((screen.getByRole("textbox", { name: "Pago QR" }) as HTMLInputElement).value, "85,50");
  assert.deepEqual(submitted, { request: { request_id: UUID, lines: [{ product_id: 1, quantity: 1, captured_unit_price_centavos: 8550, captured_revision: 2 }], payment: { amount_tendered_centavos: null, qr_applied_centavos: 8550 } } });
  await act(() => { pending.resolve({ kind: "error", code: "insufficient_stock" }); return pending.promise; }); screen.getByText("No hay stock suficiente para completar la venta.");
});

test("blocks a stale price until exact acknowledgement and retries with the same UUID", async () => {
  installUuid(); const ids: string[] = []; let attempt = 0;
  mockIPC((command, payload) => { if (command === "search_products_command") return [products[0]]; ids.push(String((payload?.request as { request_id: string }).request_id)); return ++attempt === 1 ? { kind: "stale_catalog_record", product_id: 1, current_unit_price_centavos: 9000, current_revision: 4 } : success; });
  render(createElement(SaleScreen)); const u = await addFirst(); await u.click(screen.getByRole("button", { name: "Confirmar venta" }));
  screen.getByText("El precio de Filtro aceite cambió de Bs 85,50 a Bs 90,00."); const accept = screen.getByRole("button", { name: "Aceptar precio actual" });
  assert.equal(document.activeElement, accept); assert.equal((screen.getByRole("button", { name: "Confirmar venta" }) as HTMLButtonElement).disabled, true);
  await u.click(accept); screen.getByText("Precio actual aceptado. Confirmá nuevamente para continuar.");
  await u.click(screen.getByRole("button", { name: "Confirmar venta" })); await screen.findByRole("heading", { name: "Sale confirmed" }); assert.deepEqual(ids, [UUID, UUID]);
});

test("discards late confirmation after unmount and keeps the existing success handoff", async () => {
  installUuid(); const pending = deferred<unknown>(); mockIPC((command) => command === "search_products_command" ? [products[0]] : pending.promise);
  const view = render(createElement(SaleScreen)); await addFirst(); fireEvent.click(screen.getByRole("button", { name: "Confirmar venta" }));
  assert.equal((screen.getByRole("button", { name: "Descartar borrador" }) as HTMLButtonElement).disabled, true); view.unmount();
  await act(() => { pending.resolve(success); return pending.promise; }); assert.equal(document.body.textContent, "");
});
