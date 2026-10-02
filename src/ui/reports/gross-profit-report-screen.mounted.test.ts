import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { GrossProfitReportScreen } from "./gross-profit-report-screen.ts";
const report = (amount_centavos = 1250, missing_cost_line_count = 0, activity_count = 1) => ({ kind: "success", report: { amount_centavos, missing_cost_line_count, activity_count } });
const operation = (id: number, page = 1) => ({ occurred_at: "2024-03-10T11:30:00Z", operation_kind: "venta", sale_id: id, return_id: null, product_name: "Filtro", sku: `SKU-${id}`, signed_quantity: 2, negotiated_unit_price_centavos: 1250, unit_cost_snapshot_centavos: null, cost_state: "unknown", signed_gross_profit_centavos: null });
const operations = (rows = [operation(7)], page = 1, total = 1) => ({ kind: "success", report: { rows, page, page_size: 20, total, total_pages: Math.ceil(total / 20) } });
test("shows signed summary, missing-cost disclosure, and accessible Spanish operation table", async () => {
 mockIPC(command => command === "gross_profit_report_command" ? report(-1250, 2) : operations());
 render(createElement(GrossProfitReportScreen));
 assert.ok(await screen.findByRole("heading", { name: "Bs −12.50" }));
 assert.ok(screen.getByRole("region", { name: "Ganancia bruta" }));
 assert.match(screen.getByText(/2 línea\(s\) no tienen costo histórico conocido/).textContent ?? "", /se excluyen del total/);
 const table = await screen.findByRole("table", { name: /Operaciones del período aplicado:/ });
 for (const name of ["Fecha y hora", "Operación", "Producto", "Cantidad", "Precio final", "Costo histórico", "Ganancia bruta"]) assert.ok(screen.getByRole("columnheader", { name }));
 assert.ok(withinText(table, "Venta #7")); assert.ok(withinText(table, "Filtro (SKU SKU-7)")); assert.ok(withinText(table, "No disponible (no calculable)"));
 assert.equal(screen.getByText("+2").textContent, "+2");
});
function withinText(root: HTMLElement, text: string) { return Array.from(root.querySelectorAll("td")).some(cell => cell.textContent === text); }
test("keeps no activity distinct from zero profit with activity and exposes bounded next-page navigation", async () => {
 let calls = 0;
 mockIPC(command => command === "gross_profit_report_command" ? report(0, 0, 0) : (++calls === 1 ? operations(Array.from({ length: 20 }, (_, index) => operation(index + 1)), 1, 21) : operations([operation(21)], 2, 21)));
 render(createElement(GrossProfitReportScreen));
 assert.ok(await screen.findByRole("heading", { name: "Bs 0.00" }));
 assert.ok(await screen.findByRole("button", { name: "Siguiente" }));
 assert.ok(screen.getByText("Página 1 de 2"));
 fireEvent.click(screen.getByRole("button", { name: "Siguiente" }));
 assert.ok(await screen.findByText("Página 2 de 2")); assert.ok(await screen.findByText("Venta #21"));
 assert.equal((screen.getByRole("button", { name: "Siguiente" }) as HTMLButtonElement).disabled, true);
});
test("pages during summary loading or failure without dropping table or pager focus", async () => {
 for (const summaryState of ["loading", "failure"] as const) {
  let finishSummary!: (value: unknown) => void;
  let operationCalls = 0;
  let finishPage!: (value: unknown) => void;
  mockIPC(command => {
   if (command === "gross_profit_report_command") return summaryState === "loading" ? new Promise(resolve => { finishSummary = resolve; }) : { kind: "error", code: "persistence_failure", message: "bounded" };
   operationCalls += 1;
   if (operationCalls === 1) return operations(Array.from({ length: 20 }, (_, index) => operation(index + 1)), 1, 21);
   return new Promise(resolve => { finishPage = resolve; });
  });
  const mounted = render(createElement(GrossProfitReportScreen));
  const next = await screen.findByRole("button", { name: "Siguiente" });
  next.focus();
  fireEvent.click(next);
  await waitFor(() => assert.equal(screen.getByRole("navigation", { name: "Paginación de operaciones" }).getAttribute("aria-busy"), "true"));
  fireEvent.click(next);
  assert.equal(operationCalls, 2);
  assert.equal(document.activeElement, next);
  assert.ok(screen.getByRole("table", { name: /Operaciones del período aplicado:/ }));
  assert.ok(screen.getByText("Página 1 de 2"));
  assert.equal((next as HTMLButtonElement).disabled, true);
  if (summaryState === "loading") finishSummary(report());
  await act(async () => { finishPage(operations([operation(21)], 2, 21)); });
  assert.ok(await screen.findByText("Página 2 de 2"));
  if (summaryState === "failure") assert.ok(screen.getByText(/No se pudo cargar el informe/));
  mounted.unmount();
 }
});
test("a changed applied range rejects a stale pending page completion", async () => {
 let finishPage!: (value: unknown) => void;
 let operationCalls = 0;
 mockIPC(command => {
  if (command === "gross_profit_report_command") return report();
  operationCalls += 1;
  if (operationCalls === 1) return operations(Array.from({ length: 20 }, (_, index) => operation(index + 1)), 1, 21);
  if (operationCalls === 2) return new Promise(resolve => { finishPage = resolve; });
  return operations([operation(42)], 1, 1);
 });
 render(createElement(GrossProfitReportScreen));
 fireEvent.click(await screen.findByRole("button", { name: "Siguiente" }));
 await waitFor(() => assert.equal(screen.getByRole("navigation", { name: "Paginación de operaciones" }).getAttribute("aria-busy"), "true"));
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-03-15" } });
 fireEvent.click(screen.getByRole("button", { name: "Aplicar período" }));
 assert.ok(await screen.findByText("Venta #42"));
 await act(async () => { finishPage(operations([operation(21)], 2, 21)); });
 assert.ok(screen.getByText("Venta #42"));
 assert.ok(screen.getByText("Página 1 de 1"));
});
test("shows empty range and retries a bounded error", async () => {
 let calls = 0;
 mockIPC(command => command === "gross_profit_report_command" ? report(0, 0, 0) : ++calls === 1 ? { kind: "error", code: "persistence_failure", message: "bounded" } : operations([], 1, 0));
 render(createElement(GrossProfitReportScreen));
 assert.ok(await screen.findByText(/No hay ventas ni devoluciones en el período aplicado/));
 fireEvent.click(screen.getAllByRole("button", { name: "Reintentar" })[0]!);
 assert.ok(await screen.findByText("No hay operaciones en el período aplicado."));
 assert.equal(calls, 2);
});
test("ignores stale summary and operations from an older date range", async () => {
 let finishInitialSummary!: (value: unknown) => void; let finishInitialOperations!: (value: unknown) => void;
 let summaryCalls = 0; let operationCalls = 0;
 mockIPC(command => {
  const initial = command === "gross_profit_report_command" ? ++summaryCalls === 1 : ++operationCalls === 1;
  if (initial && command === "gross_profit_report_command") return new Promise(resolve => { finishInitialSummary = resolve; });
  if (initial && command === "gross_profit_operations_command") return new Promise(resolve => { finishInitialOperations = resolve; });
  return command === "gross_profit_report_command" ? report(2750) : operations([operation(25)]);
 });
 render(createElement(GrossProfitReportScreen));
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-03-20" } });
 fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-03-25" } });
 fireEvent.click(screen.getByRole("button", { name: "Aplicar período" }));
 assert.ok(await screen.findByRole("heading", { name: "Bs 27.50" })); assert.ok(await screen.findByText("Venta #25"));
 await act(async () => { finishInitialSummary(report(100)); finishInitialOperations(operations([operation(1)])); await new Promise(resolve => setTimeout(resolve, 0)); });
 assert.ok(screen.getByRole("heading", { name: "Bs 27.50" })); assert.ok(screen.getByText("Venta #25"));
});
test("shows the applied range in the operations caption even when summary loading fails", async () => {
 mockIPC(command => command === "gross_profit_report_command" ? { kind: "error", code: "persistence_failure", message: "bounded" } : operations());
 render(createElement(GrossProfitReportScreen));
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-03-10" } });
 fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-03-20" } });
 fireEvent.click(screen.getByRole("button", { name: "Aplicar período" }));
 const table = await screen.findByRole("table", { name: "Operaciones del período aplicado: 2024-03-10 al 2024-03-20" });
 assert.ok(table);
 assert.ok(await screen.findByText(/No se pudo cargar el informe/));
});
test("date fields stay drafts until apply and reversed ranges cannot be applied", async () => {
 const calls: string[] = [];
 mockIPC((command, payload) => { calls.push(command); return command === "gross_profit_report_command" ? report() : operations(); });
 render(createElement(GrossProfitReportScreen)); await screen.findByRole("heading", { name: "Bs 12.50" });
 fireEvent.change(screen.getByLabelText("Desde"), { target: { value: "2024-03-20" } }); fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-03-10" } });
 assert.equal((screen.getByRole("button", { name: "Aplicar período" }) as HTMLButtonElement).disabled, true);
 assert.equal(calls.length, 2); fireEvent.change(screen.getByLabelText("Hasta"), { target: { value: "2024-03-25" } });
 fireEvent.click(screen.getByRole("button", { name: "Aplicar período" }));
 await waitFor(() => assert.equal(calls.length, 4));
});
