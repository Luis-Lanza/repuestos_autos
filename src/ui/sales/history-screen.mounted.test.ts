import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SalesHistoryScreen } from "./history-screen.ts";

const summary = (sale_id: number) => ({
  sale_id,
  confirmed_at: "2024-03-10 05:00:00",
  status: "confirmed",
  total_centavos: 2_500,
  line_count: 1,
  payment_count: 1,
  payment_methods: ["cash"],
  has_corrections: false,
});
const detail = (sale_id: number) => ({
  ...summary(sale_id),
  lines: [{
    sale_line_id: sale_id,
    product_id: 4,
    sku: "FLT",
    product_name: `Filter ${sale_id}`,
    quantity: 1,
    unit_price_centavos: 2_500,
    line_total_centavos: 2_500,
    returned_quantity: 0,
    cancellation_restored_quantity: 0,
    remaining_returnable_quantity: 1,
  }],
  payments: [{
    method: "cash",
    amount_applied_centavos: 2_500,
    amount_tendered_centavos: 2_500,
    change_given_centavos: 0,
  }],
  returns: [],
  cancellation: null,
});
const deferred = <Value,>() => {
  let resolve!: (value: Value) => void;
  const promise = new Promise<Value>((done) => { resolve = done; });
  return { promise, resolve };
};

test("keeps the newer mounted list when an older load finishes late", async () => {
  const lists = [deferred<unknown>(), deferred<unknown>()];
  let listCall = 0;
  mockIPC((command) => {
    if (command === "list_sales_history_command") return lists[listCall++].promise;
    throw new Error(command);
  });
  render(createElement(SalesHistoryScreen));
  const load = screen.getByRole("button", { name: "Load history" });

  fireEvent.submit(load.closest("form")!);
  lists[1].resolve({ kind: "success", sales: [summary(72)], has_more: false });
  assert.ok(await screen.findByRole("button", { name: /Sale 72/ }));
  lists[0].resolve({ kind: "success", sales: [summary(71)], has_more: false });
  await waitFor(() => assert.ok(screen.getByRole("button", { name: /Sale 72/ })));
});

test("late correction success cannot replace a newer selected sale", async () => {
  const correction = deferred<unknown>();
  const detailCalls: number[] = [];
  mockIPC((command, payload) => {
    if (command === "list_sales_history_command")
      return { kind: "success", sales: [summary(71), summary(72)], has_more: false };
    if (command === "sale_history_detail_command") {
      const saleId = Number(payload?.saleId);
      detailCalls.push(saleId);
      return { kind: "success", detail: detail(saleId) };
    }
    if (command === "create_sale_return_command") return correction.promise;
    throw new Error(command);
  });
  render(createElement(SalesHistoryScreen));
  const user = userEvent.setup({ document });
  await user.click(await screen.findByRole("button", { name: /Sale 71/ }));
  await user.click(await screen.findByRole("button", { name: "Begin item return" }));
  await user.click(screen.getByRole("checkbox", { name: "Include this original sale line" }));
  await user.type(screen.getByRole("spinbutton", { name: "Return quantity" }), "1");
  await user.click(screen.getByRole("button", { name: "Record inventory return" }));
  await user.click(screen.getByRole("button", { name: "Back to history" }));
  await user.click(screen.getByRole("button", { name: /Sale 72/ }));
  assert.ok(await screen.findByText(/Sale 72 ·/));

  correction.resolve({
    kind: "success",
    result: {
      request_id: "late-return",
      return_id: 9,
      sale_id: 71,
      status: "confirmed",
      occurred_at: "2024-03-11 05:00:00",
      lines: [{ sale_line_id: 71, product_id: 4, quantity: 1 }],
    },
  });
  await waitFor(() => assert.ok(screen.getByText(/Sale 72 ·/)));
  assert.deepEqual(detailCalls, [71, 72]);
});

test("ignores mounted list completion after unmount", async () => {
  const list = deferred<unknown>();
  mockIPC((command) => {
    if (command === "list_sales_history_command") return list.promise;
    throw new Error(command);
  });
  const view = render(createElement(SalesHistoryScreen));
  assert.ok(screen.getByText("Loading sales history…"));
  view.unmount();
  list.resolve({ kind: "success", sales: [summary(71)], has_more: false });
  await list.promise;
  assert.equal(document.body.textContent, "");
});
