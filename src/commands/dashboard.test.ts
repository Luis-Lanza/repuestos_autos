import assert from "node:assert/strict";
import test from "node:test";

import { createDashboardCommands, currentDashboardRequest } from "./dashboard.ts";

test("decodes a complete dashboard snapshot and keeps one IPC request", async () => {
  const calls: unknown[] = [];
  const commands = createDashboardCommands(async (command, payload) => { calls.push({ command, payload }); return { kind: "success", report: { today: { metrics: { effective_sale_count: 1, effective_total_centavos: 2500, net_units_out: 1, cancelled_sale_count: 0 } }, month: { metrics: { effective_sale_count: 2, effective_total_centavos: 5000, net_units_out: 2, cancelled_sale_count: 1 } }, top_products: [], payment_distribution: [{ method: "cash", amount_applied_centavos: 2500 }], recent_sales: [], stock_alerts: [] } }; });
  assert.equal((await commands.load()).kind, "success");
  assert.equal(calls.length, 1);
  assert.equal((calls[0] as { command: string }).command, "dashboard_command");
});

test("rejects malformed and unsafe aggregate values atomically", async () => {
  for (const report of [
    { today: { metrics: { effective_sale_count: -1, effective_total_centavos: 0, net_units_out: 0, cancelled_sale_count: 0 } }, month: { metrics: { effective_sale_count: 0, effective_total_centavos: 0, net_units_out: 0, cancelled_sale_count: 0 } }, top_products: [], payment_distribution: [], recent_sales: [], stock_alerts: [] },
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
