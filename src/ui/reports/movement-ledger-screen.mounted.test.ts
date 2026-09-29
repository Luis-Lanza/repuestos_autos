import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MovementLedgerScreen } from "./movement-ledger-screen.ts";

const movement = { movement_id: 1, occurred_at: "2025-03-02 10:00:00", product_id: 7, product_name: "Filtro actual", product_sku: "FLT-7", movement_type: "stock_entry", quantity_delta: 3, resulting_quantity: null, reason: null, note: null, sale_id: null, sale_line_id: null };
const products = { kind: "success", products: [{ product_id: 7, product_name: "Filtro actual", product_sku: "FLT-7", active: false }] };
const page = (rows = [movement], has_more = false) => ({ kind: "success", rows, page: 1, page_size: 50, has_more });
function mount(list: (payload: unknown) => unknown = () => page(), exportValue: unknown = { kind: "success" }) {
 const requests: unknown[] = []; const exports: unknown[] = [];
 mockIPC((command, payload) => {
  if (command === "list_movement_ledger_product_options_command") return products;
  if (command === "list_movement_ledger_command") {
   requests.push(payload);
   const requestPage = (payload as { request: { page: number } }).request.page;
   return Promise.resolve(list(payload)).then(value => typeof value === "object" && value !== null && (value as { kind?: unknown }).kind === "success" ? { ...value, page: requestPage } : value);
  }
  if (command === "export_movement_ledger_command") { exports.push(payload); return exportValue; }
  throw new Error(`Unexpected command: ${command}`);
 });
 return { requests, exports };
}
test("observes loading, then waits for initial product and ledger requests to settle", async () => {
 let resolveList!: (value: unknown) => void;
 mount(() => new Promise(resolve => { resolveList = resolve; }));
 render(createElement(MovementLedgerScreen));
 assert.ok(await screen.findByText("Cargando movimientos…"));
 assert.ok(await screen.findByRole("option", { name: /Archivado/ }));
 await act(async () => { resolveList(page()); });
 assert.ok(await screen.findByText("Filtro actual"));
});

test("shows archived historical product, persisted facts truthfully, and filtered PDF feedback", async () => {
 const { exports } = mount(); const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 assert.ok(screen.getByRole("heading", { level: 1, name: "Registro de movimientos" }));
 const product = await screen.findByRole("combobox", { name: "Producto" });
 await screen.findByRole("option", { name: /Archivado/ });
 assert.ok(within(product).getByRole("option", { name: /Archivado/ }));
 await user.selectOptions(product, "7");
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
 await screen.findByRole("option", { name: /Archivado/ });
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-02-03" } });
 fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-02-15" } });
 await user.selectOptions(screen.getByRole("combobox", { name: "Producto" }), "7");
 await user.selectOptions(screen.getByRole("combobox", { name: "Tipo de movimiento" }), "sale");
 await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
 await screen.findByRole("table");
 const expectedFilters = {
  from_utc: new Date(2024, 1, 3).toISOString(),
  to_exclusive_utc: new Date(2024, 1, 16).toISOString(),
  product_id: 7, movement_type: "sale",
 };
 assert.deepEqual((requests[1] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 1, page_size: 50 });
 await user.click(screen.getByRole("button", { name: "Siguiente" }));
 assert.ok(await screen.findByText(/No se pudieron cargar los movimientos/));
 assert.deepEqual((requests[2] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 2, page_size: 50 });
 await user.click(screen.getByRole("button", { name: "Reintentar" }));
 await screen.findByRole("table");
 assert.deepEqual((requests[3] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 2, page_size: 50 });
 assert.ok(screen.getByRole("button", { name: "Siguiente" }));
 await user.click(screen.getByRole("button", { name: "Siguiente" }));
 await waitFor(() => assert.equal((screen.getByRole("button", { name: "Siguiente" }) as HTMLButtonElement).disabled, true));
 assert.deepEqual((requests[4] as { request: Record<string, unknown> }).request, { ...expectedFilters, page: 3, page_size: 50 });
});
test("stale list and export responses cannot replace a newer filter intent", async () => {
 let finishOld!: (value: unknown) => void; let finishExport!: (value: unknown) => void; let lists = 0;
 mount(() => { lists++; return lists === 1 ? new Promise(resolve => { finishOld = resolve; }) : page([{ ...movement, product_name: "Movimiento vigente" }]); }, new Promise(resolve => { finishExport = resolve; }));
 const user = userEvent.setup({ document }); render(createElement(MovementLedgerScreen));
 await screen.findByRole("combobox", { name: "Producto" });
 await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
 assert.ok(await screen.findByText("Movimiento vigente"));
 await user.click(screen.getByRole("button", { name: "Exportar PDF" }));
 await user.click(screen.getByRole("button", { name: "Aplicar filtros" }));
 finishOld(page([{ ...movement, product_name: "Movimiento obsoleto" }]));
 finishExport({ kind: "success" });
 await waitFor(() => assert.ok(screen.getByText("Movimiento vigente")));
 assert.equal(screen.queryByText("Movimiento obsoleto"), null);
 assert.equal(screen.queryByText("El PDF se generó correctamente."), null);
});

test("validates report styling and produces one accessible main/section hierarchy", async () => {
 mount(); render(createElement(MovementLedgerScreen));
 assert.ok(await screen.findByText("Filtro actual"));
 assert.equal(screen.getAllByRole("main").length, 1);
 assert.ok(screen.getByRole("region", { name: "Filtros" }));
 assert.ok(screen.getByRole("region", { name: "Movimientos" }));
 const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
 assert.match(css, /data-ui-movement-ledger[^\n]*display:\s*grid/);
});
