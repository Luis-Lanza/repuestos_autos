import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { GrossProfitReportScreen } from "./gross-profit-report-screen.ts";

const report = (amount_centavos = 1250, missing_cost_line_count = 0, activity_count = 1) => ({ kind: "success", report: { amount_centavos, missing_cost_line_count, activity_count } });

test("shows signed total and explicit partial-cost disclosure for applied dates", async () => {
  let request: unknown;
  mockIPC((command, payload) => { assert.equal(command, "gross_profit_report_command"); request = payload; return report(-1250, 2); });
  render(createElement(GrossProfitReportScreen));
  assert.ok(await screen.findByRole("heading", { name: "Bs −12.50" }));
  assert.ok(screen.getByRole("region", { name: "Ganancia bruta" }));
  assert.equal(screen.queryByRole("main"), null);
  assert.match(screen.getByText(/2 línea\(s\) no tienen costo histórico conocido/).textContent ?? "", /se excluyen del total/);
  assert.ok(request);
});

test("keeps the movement-free report empty state and supports retry after bounded error", async () => {
  let calls = 0;
  mockIPC(() => ++calls === 1 ? { kind: "error", code: "persistence_failure", message: "bounded" } : report(0, 0, 0));
  render(createElement(GrossProfitReportScreen));
  await screen.findByRole("button", { name: "Reintentar" });
  fireEvent.click(screen.getByRole("button", { name: "Reintentar" }));
  assert.ok(await screen.findByText(/No hay ventas ni devoluciones en el período aplicado/));
  assert.equal(calls, 2);
});

test("renders a zero total when sales or returns occurred despite zero known profit", async () => {
  mockIPC(() => report(0, 0, 2));
  render(createElement(GrossProfitReportScreen));
  assert.ok(await screen.findByRole("heading", { name: "Bs 0.00" }));
  assert.equal(screen.queryByText(/No hay ventas ni devoluciones en el período aplicado/), null);
});

test("ignores an older in-flight period after applying a newer range", async () => {
  let finishInitial!: (value: unknown) => void;
  let calls = 0;
  mockIPC(() => ++calls === 1 ? new Promise(resolve => { finishInitial = resolve; }) : report(2750));
  render(createElement(GrossProfitReportScreen));
  const from = screen.getByLabelText("Desde") as HTMLInputElement;
  const to = screen.getByLabelText("Hasta") as HTMLInputElement;
  fireEvent.change(from, { target: { value: "2024-03-20" } });
  fireEvent.change(to, { target: { value: "2024-03-25" } });
  fireEvent.click(screen.getByRole("button", { name: "Aplicar período" }));
  assert.ok(await screen.findByRole("heading", { name: "Bs 27.50" }));
  finishInitial(report(100));
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.ok(screen.getByRole("heading", { name: "Bs 27.50" }));
});

test("date fields are drafts until apply and reversed periods cannot be applied", async () => {
  const calls: unknown[] = [];
  mockIPC((command, payload) => { calls.push([command, payload]); return report(); });
  render(createElement(GrossProfitReportScreen));
  await screen.findByRole("heading", { name: "Bs 12.50" });
  const from = screen.getByLabelText("Desde") as HTMLInputElement;
  const to = screen.getByLabelText("Hasta") as HTMLInputElement;
  fireEvent.change(from, { target: { value: "2024-03-20" } });
  fireEvent.change(to, { target: { value: "2024-03-10" } });
  assert.equal((screen.getByRole("button", { name: "Aplicar período" }) as HTMLButtonElement).disabled, true);
  assert.equal(calls.length, 1);
  fireEvent.change(to, { target: { value: "2024-03-25" } });
  fireEvent.click(screen.getByRole("button", { name: "Aplicar período" }));
  await waitFor(() => assert.equal(calls.length, 2));
});
