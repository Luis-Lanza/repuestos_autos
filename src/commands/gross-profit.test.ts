import assert from "node:assert/strict";
import test from "node:test";
import { createGrossProfitCommands, decodeGrossProfit } from "./gross-profit.ts";

test("gross-profit command sends only the inclusive local-date UTC boundaries and decodes signed totals", async () => {
  const calls: unknown[] = [];
  const commands = createGrossProfitCommands(async (name, payload) => { calls.push([name, payload]); return { kind: "success", report: { amount_centavos: -125, missing_cost_line_count: 2, activity_count: 3 } }; });
  assert.deepEqual(await commands.load("2024-03-10", "2024-03-10"), { kind: "success", report: { amount_centavos: -125, missing_cost_line_count: 2, activity_count: 3 } });
  const [name, payload] = calls[0] as [string, { request: Record<string, unknown> }];
  assert.equal(name, "gross_profit_report_command");
  assert.deepEqual(Object.keys(payload.request).sort(), ["from_utc", "to_exclusive_utc"]);
  assert.equal(payload.request.from_utc, new Date(2024, 2, 10).toISOString());
  assert.equal(payload.request.to_exclusive_utc, new Date(2024, 2, 11).toISOString());
});

test("strict decoder rejects unknown fields, unsafe amounts, missing activity, and unrecognized errors", () => {
  assert.equal(decodeGrossProfit({ kind: "success", report: { amount_centavos: Number.MAX_SAFE_INTEGER + 1, missing_cost_line_count: 0, activity_count: 1 } }).kind, "error");
  assert.equal(decodeGrossProfit({ kind: "success", report: { amount_centavos: 1, missing_cost_line_count: 0, activity_count: 1 }, path: "secret" }).kind, "error");
  assert.equal(decodeGrossProfit({ kind: "success", report: { amount_centavos: 0, missing_cost_line_count: 0 } }).kind, "error");
  assert.equal(decodeGrossProfit({ kind: "success", report: { amount_centavos: 0, missing_cost_line_count: 0, activity_count: -1 } }).kind, "error");
  assert.equal(decodeGrossProfit({ kind: "error", code: "internal", message: "secret" }).code, "persistence_failure");
});
