import assert from "node:assert/strict";
import test from "node:test";

import {
  createSalesHistoryCommands,
  localDateRangeToUtc,
} from "./sales-history.ts";

test("converts each local date boundary independently across DST", () => {
  const localMidnight = (year: number, month: number, day: number) =>
    new Date(
      `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}T00:00:00${day === 10 ? "-05:00" : "-04:00"}`,
    );

  assert.deepEqual(
    localDateRangeToUtc("2024-03-10", "2024-03-10", localMidnight),
    {
      from_utc: "2024-03-10T05:00:00.000Z",
      to_exclusive_utc: "2024-03-11T04:00:00.000Z",
    },
  );
});

test("sends narrow tagged IPC requests and preserves centavos and nullable snapshots", async () => {
  const calls: unknown[] = [];
  const commands = createSalesHistoryCommands(async (command, payload) => {
    calls.push({ command, payload });
    return command === "list_sales_history_command"
      ? {
          kind: "success",
          sales: [
            {
              sale_id: 71,
              confirmed_at: "2024-03-10 05:00:00",
              status: "confirmed",
              total_centavos: 2_500,
              line_count: 1,
              payment_count: 1,
              payment_methods: ["cash"],
            },
          ],
          has_more: false,
        }
      : {
          kind: "success",
          detail: {
            sale_id: 71,
            confirmed_at: "2024-03-10 05:00:00",
            status: "confirmed",
            total_centavos: 2_500,
            lines: [
              {
                product_id: 1,
                sku: null,
                product_name: null,
                quantity: 1,
                unit_price_centavos: 2_500,
                line_total_centavos: 2_500,
              },
            ],
            payments: [
              {
                method: "cash",
                amount_applied_centavos: 2_500,
                amount_tendered_centavos: 3_000,
                change_given_centavos: 500,
              },
            ],
          },
        };
  });

  const listed = await commands.list("2024-03-10", "2024-03-10");
  const detail = await commands.detail(71);

  assert.equal(listed.kind, "success");
  assert.equal(listed.sales[0].total_centavos, 2_500);
  assert.equal(detail.kind, "success");
  assert.equal(detail.detail.lines[0].sku, null);
  assert.equal(detail.detail.lines[0].product_name, null);
  assert.deepEqual(calls, [
    {
      command: "list_sales_history_command",
      payload: { request: localDateRangeToUtc("2024-03-10", "2024-03-10") },
    },
    { command: "sale_history_detail_command", payload: { saleId: 71 } },
  ]);
});

test("projects malformed and rejected history IPC values as persistence failures", async () => {
  const commands = createSalesHistoryCommands(async () => ({ kind: "unexpected" }));
  assert.deepEqual(await commands.list("2024-01-01", "2024-01-01"), {
    kind: "error",
    code: "persistence_failure",
    message: "Sales history could not be loaded.",
  });
});
