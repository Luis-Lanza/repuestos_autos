import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { createSalesHistoryCommands, type SalesHistoryDetail, type SalesHistorySummary } from "../../commands/sales-history.ts";
import { createHistoryFlow, initialHistoryState } from "./history-flow.ts";
import { createSalesHistoryInteraction, HistoryScreen } from "./history-screen.ts";
import { NAVIGATION_ACTION, SCREEN, screenAfter } from "../app.ts";

const summary: SalesHistorySummary = {
  sale_id: 71,
  confirmed_at: "2024-03-10 05:00:00",
  status: "confirmed",
  total_centavos: 2_500,
  line_count: 1,
  payment_count: 1,
  payment_methods: ["cash"],
};

const detail: SalesHistoryDetail = {
  ...summary,
  lines: [{ product_id: 4, sku: null, product_name: null, quantity: 1, unit_price_centavos: 2_500, line_total_centavos: 2_500 }],
  payments: [{ method: "cash", amount_applied_centavos: 2_000, amount_tendered_centavos: 3_000, change_given_centavos: 500 }, { method: "qr", amount_applied_centavos: 500 }],
};

test("moves through loading, bounded list, detail, and predictable back navigation", () => {
  const loading = createHistoryFlow(initialHistoryState, { type: "list_started" });
  const listed = createHistoryFlow(loading, { type: "list_loaded", sales: [summary], has_more: true });
  const detailLoading = createHistoryFlow(listed, { type: "detail_started", sale_id: summary.sale_id });
  const shown = createHistoryFlow(detailLoading, { type: "detail_loaded", detail });
  const back = createHistoryFlow(shown, { type: "back_to_list" });

  assert.equal(loading.status, "loading");
  assert.equal(listed.status, "ready");
  assert.equal(listed.has_more, true);
  assert.equal(detailLoading.status, "loading");
  assert.equal(shown.view, "detail");
  assert.equal(back.view, "list");
  assert.equal(back.sales[0].sale_id, summary.sale_id);
});

test("renders empty and error states without manufacturing sale data", () => {
  const empty = createHistoryFlow(initialHistoryState, { type: "list_loaded", sales: [], has_more: false });
  const failed = createHistoryFlow(empty, { type: "list_failed", message: "Sales history could not be loaded." });

  assert.equal(empty.status, "empty");
  assert.equal(failed.status, "error");
  assert.deepEqual(failed.sales, []);
  assert.equal(failed.detail, null);
});

test("opens history from Sales and returns to Sales", () => {
  const history = screenAfter(SCREEN.SALES, NAVIGATION_ACTION.OPEN_SALES_HISTORY);
  assert.equal(history, SCREEN.SALES_HISTORY);
  assert.equal(screenAfter(history, NAVIGATION_ACTION.RETURN_TO_SALES), SCREEN.SALES);
});

test("runs asynchronous history list, detail, and back through the production UI interaction", async () => {
  const persistedSales = [summary];
  const persistedDetail = detail;
  const calls: Array<{ command: string; payload: Record<string, unknown> }> = [];
  const interaction = createSalesHistoryInteraction(createSalesHistoryCommands(async (command, payload) => {
    calls.push({ command, payload });
    await Promise.resolve();
    return command === "list_sales_history_command"
      ? { kind: "success", sales: persistedSales, has_more: true }
      : { kind: "success", detail: persistedDetail };
  }));
  let state = initialHistoryState;
  const dispatch = (action: Parameters<typeof createHistoryFlow>[1]) => { state = createHistoryFlow(state, action); };

  const listed = interaction.reload("2024-03-10", "2024-03-10", dispatch);
  assert.equal(state.status, "loading");
  await listed;
  assert.equal(state.status, "ready");
  const selected = interaction.select(state.sales[0], dispatch);
  assert.equal(state.view, "detail");
  assert.equal(state.status, "loading");
  await selected;
  assert.equal(state.detail?.lines[0].sku, null);
  assert.match(renderToStaticMarkup(createElement(HistoryScreen, { state, onReload: () => undefined, onSelect: () => undefined, onBack: () => undefined })), /Unavailable/);
  dispatch({ type: "back_to_list" });
  assert.deepEqual(calls.map(({ command }) => command), ["list_sales_history_command", "sale_history_detail_command"]);
  assert.equal((calls[1].payload as { saleId: number }).saleId, 71);
  assert.deepEqual({ state: state.sales, persistedSales, persistedDetail }, { state: [summary], persistedSales: [summary], persistedDetail: detail });
});

test("projects asynchronous command errors without retaining or fabricating history", async () => {
  const interaction = createSalesHistoryInteraction({
    list: async () => ({ kind: "error" as const, code: "persistence_failure" as const, message: "History is unavailable." }),
    detail: async () => ({ kind: "error" as const, code: "sale_not_found" as const, message: "Sale is unavailable." }),
  });
  let state = createHistoryFlow(initialHistoryState, { type: "list_loaded", sales: [summary], has_more: false });
  const dispatch = (action: Parameters<typeof createHistoryFlow>[1]) => { state = createHistoryFlow(state, action); };

  await interaction.reload("2024-03-10", "2024-03-10", dispatch);
  assert.deepEqual({ status: state.status, sales: state.sales, detail: state.detail }, { status: "error", sales: [], detail: null });
  dispatch({ type: "list_loaded", sales: [summary], has_more: false });
  await interaction.select(summary, dispatch);
  assert.deepEqual({ view: state.view, status: state.status, detail: state.detail }, { view: "detail", status: "error", detail: null });
});

test("renders centavo money, unavailable snapshots, payment facts, and the narrowing notice", () => {
  const state = createHistoryFlow(initialHistoryState, { type: "list_loaded", sales: [summary], has_more: true });
  const shown = createHistoryFlow(createHistoryFlow(state, { type: "detail_started", sale_id: summary.sale_id }), { type: "detail_loaded", detail });
  const markup = renderToStaticMarkup(createElement(HistoryScreen, { state: shown, onReload: () => undefined, onSelect: () => undefined, onBack: () => undefined }));

  assert.match(markup, /Bs 25\.00/);
  assert.match(markup, /Unavailable/);
  assert.match(markup, /Cash applied: Bs 20\.00/);
  assert.match(markup, /Tendered: Bs 30\.00/);
  assert.match(markup, /Change: Bs 5\.00/);
  assert.match(markup, /QR applied: Bs 5\.00/);
  assert.match(renderToStaticMarkup(createElement(HistoryScreen, { state, onReload: () => undefined, onSelect: () => undefined, onBack: () => undefined })), /Narrow the date range/);
});
