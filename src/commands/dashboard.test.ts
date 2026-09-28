import assert from "node:assert/strict";
import test from "node:test";

import { createDashboardCommands, currentDashboardRequest } from "./dashboard.ts";

const metrics = (profit: unknown = { amount_centavos: 1250, missing_cost_line_count: 0 }) => ({
  effective_sale_count: 1,
  effective_total_centavos: 2500,
  net_units_out: 1,
  cancelled_sale_count: 0,
  realized_gross_profit: profit,
});
const dashboardReport = (profit: unknown = { amount_centavos: 1250, missing_cost_line_count: 0 }) => ({
  today: { metrics: metrics(profit) },
  month: { metrics: metrics({ amount_centavos: 0, missing_cost_line_count: 1 }) },
  top_products: [],
  payment_distribution: [{ method: "cash", amount_applied_centavos: 2500 }],
  recent_sales: [],
  stock_alerts: [],
});

test("decodes a complete dashboard snapshot and keeps one IPC request", async () => {
  const calls: unknown[] = [];
  const commands = createDashboardCommands(async (command, payload) => { calls.push({ command, payload }); return { kind: "success", report: dashboardReport() }; });
  const response = await commands.load();
  assert.equal(response.kind, "success");
  if (response.kind === "success") {
    assert.deepEqual(response.report.today.metrics.realized_gross_profit, { amount_centavos: 1250, missing_cost_line_count: 0 });
    assert.deepEqual(response.report.month.metrics.realized_gross_profit, { amount_centavos: 0, missing_cost_line_count: 1 });
  }
  assert.equal(calls.length, 1);
  assert.equal((calls[0] as { command: string }).command, "dashboard_command");
});

test("decodes signed losses without accepting unsafe profit integers", async () => {
  const loss = await createDashboardCommands(async () => ({ kind: "success", report: dashboardReport({ amount_centavos: -500, missing_cost_line_count: 2 }) })).load();
  assert.equal(loss.kind, "success");
  if (loss.kind === "success") assert.deepEqual(loss.report.today.metrics.realized_gross_profit, { amount_centavos: -500, missing_cost_line_count: 2 });

  const overflow = await createDashboardCommands(async () => ({ kind: "success", report: dashboardReport({ amount_centavos: Number.MAX_SAFE_INTEGER + 1, missing_cost_line_count: 0 }) })).load();
  assert.deepEqual(overflow, { kind: "error", code: "persistence_failure", message: "The dashboard could not be loaded." });
});

test("rejects malformed dashboard snapshots atomically", async () => {
  for (const report of [
    { ...dashboardReport(), today: { metrics: { ...metrics(), effective_sale_count: -1 } } },
    { ...dashboardReport(), month: { metrics: { ...metrics(), realized_gross_profit: { amount_centavos: 1.5, missing_cost_line_count: 0 } } } },
    { ...dashboardReport(), month: { metrics: { ...metrics(), realized_gross_profit: { amount_centavos: 1, missing_cost_line_count: -1 } } } },
    { today: {}, month: {}, top_products: [], payment_distribution: [], recent_sales: [], stock_alerts: [] },
  ]) {
    const commands = createDashboardCommands(async () => ({ kind: "success", report }));
    assert.deepEqual(await commands.load(), { kind: "error", code: "persistence_failure", message: "The dashboard could not be loaded." });
  }
});

test("maps native failures to bounded public errors and creates local half-open bounds", async () => {
  const commands = createDashboardCommands(async () => ({ kind: "error", code: "invalid_range", message: "native details" }));
  assert.deepEqual(await commands.load(), { kind: "error", code: "invalid_range", message: "The dashboard date range is invalid." });
  const bounds = currentDashboardRequest(new Date(2024, 2, 10, 12));
  assert.equal(bounds.today_from_utc, new Date(2024, 2, 10).toISOString());
  assert.equal(bounds.today_to_exclusive_utc, new Date(2024, 2, 11).toISOString());
});
