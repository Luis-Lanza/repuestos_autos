import assert from "node:assert/strict";
import test from "node:test";

import {
  createSalesHistoryCommands,
  localDateRangeToUtc,
} from "./sales-history.ts";
import type {
  HistoricalCancellation,
  HistoricalReturn,
  SalesHistoryDetail,
  SalesHistoryListResponse,
} from "./sales-history.ts";

const currentContract = {
  list: {
    kind: "success",
    sales: [
      {
        sale_id: 71,
        confirmed_at: "2024-03-10 05:00:00",
        status: "cancelled",
        total_centavos: 2_500,
        line_count: 2,
        payment_count: 2,
        payment_methods: ["cash", "qr"],
        has_corrections: true,
      },
    ],
    has_more: false,
  },
  detail: {
    kind: "success",
    detail: {
      sale_id: 71,
      confirmed_at: "2024-03-10 05:00:00",
      status: "cancelled",
      total_centavos: 2_500,
      lines: [
        {
          sale_line_id: 101,
          product_id: 1,
          sku: "SKU-1",
          product_name: "Brake pad",
          quantity: 2,
          unit_price_centavos: 1_250,
          line_total_centavos: 2_500,
          returned_quantity: 1,
          cancellation_restored_quantity: 1,
          remaining_returnable_quantity: 0,
        },
      ],
      payments: [
        {
          method: "cash",
          amount_applied_centavos: 2_000,
          amount_tendered_centavos: 3_000,
          change_given_centavos: 1_000,
        },
        { method: "qr", amount_applied_centavos: 500 },
      ],
      returns: [
        {
          return_id: 8,
          request_id: "return-1",
          occurred_at: "2024-03-11 05:00:00",
          lines: [{ sale_line_id: 101, product_id: 1, quantity: 1 }],
        },
      ],
      cancellation: {
        cancellation_id: 9,
        request_id: "cancel-1",
        occurred_at: "2024-03-12 05:00:00",
        reason: "Customer cancelled",
        lines: [
          { sale_line_id: 101, product_id: 1, restored_quantity: 1 },
          { sale_line_id: 102, product_id: 2, restored_quantity: 0 },
        ],
      },
    },
  },
  returned: {
    return_id: 8,
    request_id: "return-1",
    occurred_at: "2024-03-11 05:00:00",
    lines: [{ sale_line_id: 101, product_id: 1, quantity: 1 }],
  },
  cancellation: {
    cancellation_id: 9,
    request_id: "cancel-1",
    occurred_at: "2024-03-12 05:00:00",
    reason: "Customer cancelled",
    lines: [
      { sale_line_id: 101, product_id: 1, restored_quantity: 1 },
      { sale_line_id: 102, product_id: 2, restored_quantity: 0 },
    ],
  },
} satisfies {
  list: SalesHistoryListResponse;
  detail: { kind: "success"; detail: SalesHistoryDetail };
  returned: HistoricalReturn;
  cancellation: HistoricalCancellation;
};
void currentContract;

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

test("projects current corrected history through narrow IPC requests", async () => {
  const calls: unknown[] = [];
  const commands = createSalesHistoryCommands(async (command, payload) => {
    calls.push({ command, payload });
    if (command === "list_sales_history_command") {
      return {
        kind: "success",
        sales: [{ ...currentContract.list.sales[0], sql: "forbidden" }],
        has_more: false,
        schema: "forbidden",
      };
    }
    const detail = currentContract.detail.detail;
    return {
      kind: "success",
      detail: {
        ...detail,
        refund: "forbidden",
        lines: [{ ...detail.lines[0], driver: "forbidden" }],
        payments: [
          { ...detail.payments[0], credit: "forbidden" },
          detail.payments[1],
        ],
        returns: [
          {
            ...detail.returns[0],
            reversal: "forbidden",
            lines: [
              { ...detail.returns[0].lines[0], reimbursement: "forbidden" },
            ],
          },
        ],
        cancellation: {
          ...detail.cancellation!,
          settlement: "forbidden",
          lines: [
            detail.cancellation!.lines[0],
            { ...detail.cancellation!.lines[1], sql: "forbidden" },
          ],
        },
      },
      schema: "forbidden",
    };
  });

  assert.deepEqual(
    await commands.list("2024-03-10", "2024-03-10"),
    currentContract.list,
  );
  assert.deepEqual(await commands.detail(71), currentContract.detail);
  assert.deepEqual(calls, [
    {
      command: "list_sales_history_command",
      payload: { request: localDateRangeToUtc("2024-03-10", "2024-03-10") },
    },
    { command: "sale_history_detail_command", payload: { saleId: 71 } },
  ]);
});

test("projects malformed and rejected history IPC values as persistence failures", async () => {
  const commands = createSalesHistoryCommands(async () => ({
    kind: "unexpected",
  }));
  assert.deepEqual(await commands.list("2024-01-01", "2024-01-01"), {
    kind: "error",
    code: "persistence_failure",
    message: "Sales history could not be loaded.",
  });
});

test("maps decoder exceptions to persistence failures", async () => {
  const response = { kind: "success" } as Record<string, unknown>;
  Object.defineProperty(response, "sales", {
    get() {
      throw new Error("hostile accessor");
    },
  });
  const commands = createSalesHistoryCommands(async () => response);

  assert.deepEqual(await commands.list("2024-01-01", "2024-01-01"), {
    kind: "error",
    code: "persistence_failure",
    message: "Sales history could not be loaded.",
  });
});

type Wire = { list: Record<string, unknown>; detail: Record<string, unknown> };
type Method = "list" | "detail";
type Mutation = (value: Wire) => void;
const failure = {
  kind: "error",
  code: "persistence_failure",
  message: "Sales history could not be loaded.",
};
const wire = (): Wire => JSON.parse(JSON.stringify(currentContract)) as Wire;
const detailValue = (value: Wire) =>
  value.detail.detail as Record<string, unknown>;
const rows = (value: Record<string, unknown>, name: string) =>
  value[name] as Record<string, unknown>[];
const summaryValue = (value: Wire) => rows(value.list, "sales")[0];
const originalLine = (value: Wire) => rows(detailValue(value), "lines")[0];
const returned = (value: Wire) => rows(detailValue(value), "returns")[0];
const returnedLine = (value: Wire) => rows(returned(value), "lines")[0];
const cancelled = (value: Wire) =>
  detailValue(value).cancellation as Record<string, unknown>;
const cancelledLine = (value: Wire) => rows(cancelled(value), "lines")[1];
const payment = (value: Wire, index = 0) =>
  rows(detailValue(value), "payments")[index];
const call = (
  commands: ReturnType<typeof createSalesHistoryCommands>,
  method: Method,
) =>
  method === "list"
    ? commands.list("2024-01-01", "2024-01-01")
    : commands.detail(71);

test("maps malformed history response fields and invoke rejections to persistence failures", async () => {
  const cases: Array<[Method, Mutation]> = [
    ["list", (value) => delete value.list.kind],
    ["detail", (value) => (value.detail.kind = "unexpected")],
    ["list", (value) => delete value.list.has_more],
    ["list", (value) => (value.list.sales = {})],
    ["list", (value) => (summaryValue(value).payment_methods = {})],
    ["list", (value) => (summaryValue(value).sale_id = "71")],
    ["list", (value) => (summaryValue(value).confirmed_at = 1)],
    ["list", (value) => (summaryValue(value).line_count = "2")],
    ["list", (value) => (summaryValue(value).payment_count = "2")],
    ["list", (value) => (summaryValue(value).total_centavos = "2500")],
    ["list", (value) => (summaryValue(value).status = "unknown")],
    ["list", (value) => delete summaryValue(value).has_corrections],
    ["detail", (value) => (detailValue(value).lines = {})],
    ["detail", (value) => (detailValue(value).payments = {})],
    ["detail", (value) => (detailValue(value).returns = {})],
    ["detail", (value) => (detailValue(value).sale_id = "71")],
    ["detail", (value) => (detailValue(value).status = "unknown")],
    ["detail", (value) => (detailValue(value).confirmed_at = 1)],
    ["detail", (value) => (detailValue(value).total_centavos = "2500")],
    ["detail", (value) => (originalLine(value).sale_line_id = "101")],
    ["detail", (value) => (originalLine(value).product_id = "1")],
    ["detail", (value) => (originalLine(value).quantity = "2")],
    ["detail", (value) => (originalLine(value).unit_price_centavos = "1250")],
    ["detail", (value) => (originalLine(value).line_total_centavos = "2500")],
    ["detail", (value) => (originalLine(value).returned_quantity = "1")],
    [
      "detail",
      (value) => (originalLine(value).cancellation_restored_quantity = "1"),
    ],
    [
      "detail",
      (value) => (originalLine(value).remaining_returnable_quantity = "0"),
    ],
    ["detail", (value) => (originalLine(value).sku = 1)],
    ["detail", (value) => (originalLine(value).product_name = 1)],
    ["detail", (value) => (returned(value).return_id = "8")],
    ["detail", (value) => (returned(value).request_id = 1)],
    ["detail", (value) => (returned(value).occurred_at = 1)],
    ["detail", (value) => (returned(value).lines = {})],
    ["detail", (value) => (returnedLine(value).sale_line_id = "101")],
    ["detail", (value) => (returnedLine(value).product_id = "1")],
    ["detail", (value) => (returnedLine(value).quantity = "1")],
    ["detail", (value) => (cancelled(value).cancellation_id = "9")],
    ["detail", (value) => (cancelled(value).request_id = 1)],
    ["detail", (value) => (cancelled(value).occurred_at = 1)],
    ["detail", (value) => (cancelled(value).reason = 1)],
    ["detail", (value) => (cancelled(value).lines = {})],
    ["detail", (value) => (cancelledLine(value).sale_line_id = "102")],
    ["detail", (value) => (cancelledLine(value).product_id = "2")],
    ["detail", (value) => (cancelledLine(value).restored_quantity = "0")],
    ["detail", (value) => (payment(value).method = "card")],
    ["detail", (value) => (payment(value).amount_applied_centavos = "2000")],
    ["detail", (value) => (payment(value).amount_tendered_centavos = "3000")],
    ["detail", (value) => (payment(value).change_given_centavos = "1000")],
    ["detail", (value) => (payment(value, 1).method = "cash")],
    ["detail", (value) => (payment(value, 1).amount_applied_centavos = "500")],
  ];

  for (const [method, mutate] of cases) {
    const value = wire();
    mutate(value);
    const commands = createSalesHistoryCommands(async (command) =>
      command === "list_sales_history_command" ? value.list : value.detail,
    );
    assert.deepEqual(await call(commands, method), failure);
  }

  const rejected = createSalesHistoryCommands(async () =>
    Promise.reject(new Error("offline")),
  );
  assert.deepEqual(await rejected.list("2024-01-01", "2024-01-01"), failure);
  assert.deepEqual(await rejected.detail(71), failure);
});

test("preserves server correction order and accepts the current uncorrected shape", async () => {
  const corrected = wire();
  const detail = detailValue(corrected);
  const returns = rows(detail, "returns");
  returns.push({
    ...returns[0],
    return_id: 7,
    occurred_at: "2024-03-10 05:00:00",
    lines: [
      { sale_line_id: 102, product_id: 2, quantity: 1 },
      rows(returns[0], "lines")[0],
    ],
  });
  detail.returns = [returns[1], returns[0]];
  const cancellation = detail.cancellation as Record<string, unknown>;
  cancellation.lines = [...rows(cancellation, "lines")].reverse();
  const commands = createSalesHistoryCommands(async (command) =>
    command === "list_sales_history_command"
      ? corrected.list
      : corrected.detail,
  );
  const projected = await commands.detail(71);
  assert.equal(projected.kind, "success");
  if (projected.kind === "success") {
    assert.deepEqual(
      projected.detail.returns.map((value) => value.return_id),
      [7, 8],
    );
    assert.deepEqual(
      projected.detail.returns[0].lines.map((value) => value.sale_line_id),
      [102, 101],
    );
    assert.deepEqual(
      projected.detail.cancellation?.lines.map((value) => value.sale_line_id),
      [102, 101],
    );
  }

  const uncorrected = wire();
  const summary = rows(uncorrected.list, "sales")[0];
  const uncorrectedDetail = detailValue(uncorrected);
  summary.status = "confirmed";
  summary.has_corrections = false;
  uncorrectedDetail.status = "confirmed";
  rows(uncorrectedDetail, "lines")[0].returned_quantity = 0;
  rows(uncorrectedDetail, "lines")[0].cancellation_restored_quantity = 0;
  rows(uncorrectedDetail, "lines")[0].remaining_returnable_quantity = 2;
  uncorrectedDetail.returns = [];
  uncorrectedDetail.cancellation = null;
  const legacy = createSalesHistoryCommands(async (command) =>
    command === "list_sales_history_command"
      ? uncorrected.list
      : uncorrected.detail,
  );
  assert.deepEqual(
    await legacy.list("2024-01-01", "2024-01-01"),
    uncorrected.list,
  );
  assert.deepEqual(await legacy.detail(71), uncorrected.detail);
});
