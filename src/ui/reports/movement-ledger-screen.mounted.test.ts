import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MovementLedgerScreen } from "./movement-ledger-screen.ts";

const movement = { movement_id: 1, occurred_at: "2025-03-02 10:00:00", product_id: 7, product_name: "Filtro actual", product_sku: "FLT-7", movement_type: "stock_entry", quantity_delta: 3, resulting_quantity: null, reason: null, note: null, sale_id: null, sale_line_id: null };
const foundProduct = { product_id: 7, product_name: "Filtro actual", product_sku: "FLT-7", active: false };
const page = (rows = [movement], has_more = false) => ({ kind: "success", rows, page: 1, page_size: 50, has_more });
const productPage = (products = [foundProduct], page = 1, has_more = false) => ({ kind: "success", products, page, page_size: 20, has_more });
function mount(list: (payload: unknown) => unknown = () => page(), exportValue: unknown = { kind: "success" }, search: (payload: unknown) => unknown = () => productPage()) {
 const requests: unknown[] = []; const exports: unknown[] = []; const searches: unknown[] = [];
 mockIPC((command, payload) => {
  if (command === "list_movement_ledger_product_options_command") { searches.push(payload); return search(payload); }
  if (command === "list_movement_ledger_command") {
   requests.push(payload);
   const requestPage = (payload as { request: { page: number } }).request.page;
   return Promise.resolve(list(payload)).then(value => typeof value === "object" && value !== null && (value as { kind?: unknown }).kind === "success" ? { ...value, page: requestPage } : value);
  }
  if (command === "export_movement_ledger_command") { exports.push(payload); return exportValue; }
  throw new Error(`Unexpected command: ${command}`);
 });
 return { requests, exports, searches };
}
async function submitSearch(query = "filtro") {
 const user = userEvent.setup({ document });
 const input = screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" });
 await user.type(input, `${query}{Enter}`);
 return user;
}
test("applies shared dates to the gross-profit mode and exposes only its historical PDF action", async () => {
 const movementExports: unknown[] = []; const profitExports: unknown[] = []; const movementRequests: unknown[] = [];
 mockIPC((command, payload) => {
  if (command === "list_movement_ledger_product_options_command") return productPage([]);
  if (command === "list_movement_ledger_command") { movementRequests.push(payload); return page(); }
  if (command === "export_movement_ledger_command") { movementExports.push(payload); return { kind: "success" }; }
  if (command === "gross_profit_report_command") return report;
  if (command === "gross_profit_operations_command") return historicalOperations;
  if (command === "export_gross_profit_operations_command") { profitExports.push(payload); return { kind: "success" }; }
  throw new Error(`Unexpected command: ${command}`);
 });
 const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 await screen.findByRole("table");
 assert.equal((screen.getByRole("combobox", { name: "Tipo de movimiento" }) as HTMLSelectElement).value, "");
 await user.selectOptions(screen.getByRole("combobox", { name: "Tipo de movimiento" }), "gross_profit");
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-03-10" } });
 fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-03-20" } });
 await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
 assert.ok(await screen.findByRole("heading", { name: "Bs 12.50" }));
 assert.ok(await screen.findByText("Venta #7"));
 assert.equal(screen.getAllByLabelText("Desde").length, 1);
 assert.equal(screen.getAllByLabelText("Hasta").length, 1);
 assert.equal(screen.queryByRole("button", { name: "Aplicar período" }), null);
 assert.equal(screen.queryByRole("searchbox", { name: "Buscar producto por nombre o SKU" }), null);
 assert.equal(screen.getAllByRole("button", { name: "Exportar PDF" }).length, 1);
 assert.equal(screen.queryByRole("region", { name: "Movimientos" }), null);
 await user.click(screen.getByRole("button", { name: "Exportar PDF" }));
 assert.ok(await screen.findByText("PDF guardado correctamente."));
 assert.equal(movementExports.length, 0); assert.equal(profitExports.length, 1);
 assert.equal((profitExports[0] as { request: { from: { local_date: string }; to_exclusive: { local_date: string } } }).request.from.local_date, "2024-03-10");
 assert.equal((profitExports[0] as { request: { to_exclusive: { local_date: string } } }).request.to_exclusive.local_date, "2024-03-21");
 assert.equal((movementRequests[0] as { request: { movement_type: unknown } }).request.movement_type, null);
});
const report = { kind: "success", report: { amount_centavos: 1250, missing_cost_line_count: 0, activity_count: 1 } };
const historicalOperations = { kind: "success", report: { rows: [{ occurred_at: "2024-03-10T11:30:00Z", operation_kind: "venta", sale_id: 7, return_id: null, product_name: "Filtro histórico", sku: "SKU-7", signed_quantity: 2, negotiated_unit_price_centavos: 1250, unit_cost_snapshot_centavos: 500, cost_state: "known", signed_gross_profit_centavos: 1500 }], page: 1, page_size: 20, total: 1, total_pages: 1 } };
test("starts with loading ledger and does not eagerly request an unbounded product list", async () => {
 const { searches } = mount(); render(createElement(MovementLedgerScreen));
 assert.ok(await screen.findByText("Cargando movimientos…"));
 assert.equal(searches.length, 0);
 assert.ok(await screen.findByText("Filtro actual"));
});
test("submitted search supports Enter, bounded selectable archived products, and clear returns to all products", async () => {
 const { searches } = mount(); const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 const input = screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" });
 await user.type(input, "filtro");
 assert.equal(searches.length, 0);
 await user.keyboard("{Enter}");
 const select = await screen.findByRole("button", { name: "Seleccionar Filtro actual (SKU: FLT-7)" });
 assert.deepEqual(searches[0], { request: { query: "filtro", page: 1, page_size: 20 } });
 const results = screen.getByRole("list", { name: "Resultados de productos" });
 assert.equal(results.querySelectorAll("li").length, 1);
 const filters = screen.getByRole("region", { name: "Filtros" }).querySelector("[data-ui-ledger-filters]")!;
 assert.equal(results.parentElement, filters.querySelector("[data-ui-ledger-product-feedback-row]"));
 assert.equal(filters.querySelector("[data-ui-ledger-filter-row]")?.contains(results), false);
 await user.click(select);
 assert.ok(screen.getByText("Filtro actual · FLT-7 · Archivado"));
 await user.click(screen.getByRole("button", { name: "Quitar producto" }));
 assert.ok(screen.getByText(/Todos los productos/));
 assert.equal(screen.queryByRole("list", { name: "Resultados de productos" }), null);
});
test("renders explicit product search loading, empty, error, and retry states", async () => {
 let searches = 0;
 const { searches: requests } = mount(() => page(), { kind: "success" }, () => {
  searches++;
  if (searches === 1) return new Promise(() => {});
  if (searches === 2) return { kind: "success", products: [], page: 1, page_size: 20, has_more: false };
  if (searches === 3) return { kind: "error", code: "persistence_failure", message: "private" };
  return productPage();
 });
 const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 await user.type(screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" }), "xyz{Enter}");
 assert.ok(await screen.findByText("Buscando productos…"));
 await act(async () => { /* first request intentionally remains pending */ });
 await user.clear(screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" }));
 await user.type(screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" }), "nada{Enter}");
 assert.ok(await screen.findByText("No encontramos productos para “nada”."));
 await user.clear(screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" }));
 await user.type(screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" }), "error{Enter}");
 assert.ok(await screen.findByText(/No se pudo buscar en el catálogo local/));
 await user.click(screen.getByRole("button", { name: "Reintentar" }));
 assert.ok(await screen.findByRole("button", { name: "Seleccionar Filtro actual (SKU: FLT-7)" }));
 assert.equal(requests.length, 4);
});
test("stale product search responses are rejected independently of ledger list requests", async () => {
 let resolveOld!: (value: unknown) => void; let searchCount = 0;
 const { searches } = mount(() => page(), { kind: "success" }, () => ++searchCount === 1 ? new Promise(resolve => { resolveOld = resolve; }) : productPage([{ ...foundProduct, product_name: "Resultado vigente" }]));
 const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 const input = screen.getByRole("searchbox", { name: "Buscar producto por nombre o SKU" });
 await user.type(input, "viejo{Enter}");
 await user.clear(input); await user.type(input, "nuevo{Enter}");
 assert.ok(await screen.findByRole("button", { name: "Seleccionar Resultado vigente (SKU: FLT-7)" }));
 await act(async () => { resolveOld(productPage([{ ...foundProduct, product_name: "Resultado obsoleto" }])); });
 assert.equal(screen.queryByText(/Resultado obsoleto/), null);
 assert.ok(screen.getByText("Filtro actual"));
 assert.equal(searches.length, 2);
});
test("shows archived historical product, persisted facts truthfully, and exports active filters", async () => {
 const { exports } = mount(); const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 assert.ok(screen.getByRole("heading", { level: 1, name: "Reportes" }));
 await submitSearch();
 await user.click(await screen.findByRole("button", { name: "Seleccionar Filtro actual (SKU: FLT-7)" }));
 await user.selectOptions(screen.getByRole("combobox", { name: "Tipo de movimiento" }), "sale");
 await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
 await screen.findByText("Filtro actual");
 assert.ok(screen.getByText("datos actuales")); assert.ok(screen.getByText("No registrado")); assert.ok(screen.getByText("No disponible"));
 assert.ok(screen.getByText("+3"));
 await user.click(screen.getByRole("button", { name: "Exportar PDF" }));
 assert.equal(await screen.findByText("El PDF se generó correctamente.").then(node => node.textContent), "El PDF se generó correctamente.");
 const [from, to] = ["Desde", "Hasta"].map(label => (screen.getByLabelText(label) as HTMLInputElement).value);
 const localStart = (value: string) => new Date(Number(value.slice(0, 4)), Number(value.slice(5, 7)) - 1, Number(value.slice(8, 10))).toISOString();
 const dayAfter = (value: string) => new Date(Number(value.slice(0, 4)), Number(value.slice(5, 7)) - 1, Number(value.slice(8, 10)) + 1).toISOString();
 assert.deepEqual(exports, [{ request: { from_utc: localStart(from), to_exclusive_utc: dayAfter(to), product_id: 7, movement_type: "sale" } }]);
 assert.doesNotMatch(screen.getByText("El PDF se generó correctamente.").textContent ?? "", /(?:[A-Za-z]:\\|\\\\|\/)/);
});
test("preserves custom applied date, product, and movement filters across pagination and retry", async () => {
 let calls = 0;
 const { requests } = mount(() => {
  calls++;
  if (calls === 1) return page([], false);
  if (calls === 3) return { kind: "error", code: "persistence_failure", message: "private" };
  return page([movement], calls !== 5);
 });
 const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 assert.ok(await screen.findByText("No hay movimientos para los filtros aplicados."));
 await submitSearch(); await user.click(await screen.findByRole("button", { name: "Seleccionar Filtro actual (SKU: FLT-7)" }));
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-02-03" } });
 fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-02-15" } });
 await user.selectOptions(screen.getByRole("combobox", { name: "Tipo de movimiento" }), "sale");
 await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
 await screen.findByRole("table");
 const expectedFilters = { from_utc: new Date(2024, 1, 3).toISOString(), to_exclusive_utc: new Date(2024, 1, 16).toISOString(), product_id: 7, movement_type: "sale" };
 assert.deepEqual((requests[1] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 1, page_size: 50 });
 await user.click(screen.getByRole("button", { name: "Siguiente" }));
 assert.ok(await screen.findByText(/No se pudieron cargar los movimientos/));
 assert.deepEqual((requests[2] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 2, page_size: 50 });
 await user.click(screen.getByRole("button", { name: "Reintentar" }));
 await screen.findByRole("table");
 assert.deepEqual((requests[3] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 2, page_size: 50 });
 await user.click(screen.getByRole("button", { name: "Siguiente" }));
 await waitFor(() => assert.equal((screen.getByRole("button", { name: "Siguiente" }) as HTMLButtonElement).disabled, true));
 assert.deepEqual((requests[4] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 3, page_size: 50 });
});
test("keeps date, control, and product-feedback rows structurally separate and responsive", async () => {
 mount(); render(createElement(MovementLedgerScreen));
 assert.ok(await screen.findByText("Filtro actual"));
 assert.equal(screen.getAllByRole("main").length, 1);
 assert.ok(screen.getByRole("region", { name: "Filtros" }));
 assert.ok(screen.getByRole("region", { name: "Resultados" }));
 const filters = screen.getByRole("region", { name: "Filtros" }).querySelector("[data-ui-ledger-filters]")!;
 assert.equal(filters.children[0].getAttribute("data-ui-ledger-date-row"), "true");
 assert.deepEqual([...filters.children[0].querySelectorAll("label")].map(label => label.textContent), ["Desde", "Hasta"]);
 const controls = filters.querySelector("[data-ui-ledger-filter-row]")!;
 assert.deepEqual([...controls.children].map(child => child.getAttribute("data-ui-ledger-product-filter") ? "Producto" : child.getAttribute("data-ui-ledger-apply-form") ? "Aplicar filtros" : child.querySelector("label")?.textContent ?? child.textContent?.trim()), ["Producto", "Tipo de movimiento", "Aplicar filtros"]);
 const applyForm = controls.querySelector("[data-ui-ledger-apply-form]") as HTMLFormElement;
 assert.ok(applyForm, "the apply action remains a semantic form submission");
 assert.equal(applyForm.querySelector("[data-ui-action]")?.getAttribute("type"), "submit");
 assert.equal(applyForm.querySelector("[data-ui-ledger-apply-label-space]")?.textContent, "Tipo de movimiento");
 assert.equal(applyForm.querySelector("[data-ui-ledger-apply-label-space]")?.getAttribute("aria-hidden"), "true");
 assert.ok(screen.getByRole("button", { name: "Aplicar filtros" }));
 const productFeedback = filters.querySelector("[data-ui-ledger-product-feedback-row]")!;
 assert.equal(productFeedback.parentElement, filters);
 assert.equal(productFeedback.getAttribute("data-ui-ledger-product-feedback-row"), "true");
 assert.ok(productFeedback.querySelector("[data-ui-feedback]"), "initial search guidance belongs to the dedicated third row");
 assert.equal(productFeedback.querySelector("[data-ui-ledger-product-search]"), null);
 assert.equal(productFeedback.querySelector("[data-ui-ledger-product-results]"), null);
 assert.equal(controls.querySelector("[data-ui-ledger-product-feedback-row]"), null);
 const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
 assert.match(css, /data-ui-movement-ledger[^\n]*display:\s*grid/);
 assert.match(css, /\[data-ui-ledger-filters\] \{ container: movement-ledger-filters \/ inline-size; display: grid;[^}]*\}/);
 assert.match(css, /\[data-ui-ledger-filter-row\][^}]*align-items: start/);
 assert.match(css, /\[data-ui-ledger-apply-form\][^}]*display: grid[^}]*gap: var\(--space-2\)/);
 assert.match(css, /\[data-ui-ledger-apply-label-space\][^}]*visibility: hidden/);
 assert.match(css, /\[data-ui-ledger-product-feedback-row\][^}]*grid-column: 1 \/ -1/);
 assert.match(css, /@container movement-ledger-filters \(min-width: 48rem\)/);
});
