import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { createPostSaleCommands } from "../../commands/post-sale.ts";
import { createSalesHistoryCommands, type SalesHistoryDetail, type SalesHistorySummary } from "../../commands/sales-history.ts";
import {
  correctionFocusTarget,
  createHistoryFlow,
  initialHistoryState,
} from "./history-flow.ts";
import {
  createSalesHistoryInteraction,
  HistoryScreen,
} from "./history-screen.ts";
import { NAVIGATION_ACTION, SCREEN, screenAfter } from "../app.ts";

const summary: SalesHistorySummary = {
  sale_id: 71,
  confirmed_at: "2024-03-10 05:00:00",
  status: "confirmed",
  total_centavos: 2_500,
  line_count: 1,
  payment_count: 1,
  payment_methods: ["cash"],
  has_corrections: false,
};

const emptyHistoryList = async () => ({
  kind: "success" as const,
  sales: [],
  has_more: false,
});

const detail: SalesHistoryDetail = {
  ...summary,
  lines: [{ sale_line_id: 41, product_id: 4, sku: null, product_name: null, quantity: 1, unit_price_centavos: 2_500, line_total_centavos: 2_500, returned_quantity: 0, cancellation_restored_quantity: 0, remaining_returnable_quantity: 1 }],
  payments: [{ method: "cash", amount_applied_centavos: 2_000, amount_tendered_centavos: 3_000, change_given_centavos: 500 }, { method: "qr", amount_applied_centavos: 500 }],
  returns: [],
  cancellation: null,
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

test("submits exact return and cancellation requests once, then reloads history", async () => {
  const requests: unknown[] = [];
  let resolveReturn: (() => void) | undefined;
  const returnedDetail = {
    ...detail,
    lines: [
      {
        ...detail.lines[0],
        remaining_returnable_quantity: 0,
        returned_quantity: 1,
      },
    ],
    returns: [
      {
        return_id: 12,
        request_id: "return-uuid",
        occurred_at: "2024-03-11 05:00:00",
        lines: [{ sale_line_id: 41, product_id: 4, quantity: 1 }],
      },
    ],
  };
  const cancelledDetail = {
    ...returnedDetail,
    status: "cancelled" as const,
    cancellation: {
      cancellation_id: 14,
      request_id: "cancellation-uuid",
      occurred_at: "2024-03-11 05:00:00",
      reason: "Duplicate sale",
      lines: [{ sale_line_id: 41, product_id: 4, restored_quantity: 0 }],
    },
  };
  let persistedDetail = returnedDetail;
  const interaction = createSalesHistoryInteraction({
    list: emptyHistoryList,
    detail: async () => ({ kind: "success" as const, detail: persistedDetail }),
    createReturn: async (request) => {
      requests.push(request);
      await new Promise<void>((resolve) => {
        resolveReturn = resolve;
      });
      return {
        kind: "success" as const,
        result: {
          request_id: request.request_id,
          return_id: 12,
          sale_id: request.sale_id,
          status: "confirmed" as const,
          occurred_at: "2024-03-11 05:00:00",
          lines: [{ sale_line_id: 41, product_id: 4, quantity: 1 }],
        },
      };
    },
    cancelSale: async (request) => {
      requests.push(request);
      return {
        kind: "success" as const,
        result: {
          request_id: request.request_id,
          cancellation_id: 14,
          sale_id: request.sale_id,
          status: "cancelled" as const,
          occurred_at: "2024-03-11 05:00:00",
          reason: request.reason,
          lines: [{ sale_line_id: 41, product_id: 4, restored_quantity: 0 }],
        },
      };
    },
  });
  let state = flow(
    initialHistoryState,
    { type: "detail_loaded", detail },
    { type: "return_intent_opened", request_id: "return-uuid" },
    { type: "return_line_selected", sale_line_id: 41, selected: true },
    { type: "return_quantity_changed", sale_line_id: 41, value: "1" },
  );
  const dispatch = (action: Parameters<typeof createHistoryFlow>[1]) => {
    state = createHistoryFlow(state, action);
  };

  const first = interaction.submitReturn(state, dispatch);
  const second = interaction.submitReturn(state, dispatch);
  assert.equal(state.return_intent?.status, "pending");
  assert.deepEqual(requests, [
    {
      request_id: "return-uuid",
      sale_id: 71,
      lines: [{ sale_line_id: 41, quantity: 1 }],
    },
  ]);
  resolveReturn?.();
  await Promise.all([first, second]);
  assert.equal(state.detail, returnedDetail);
  persistedDetail = cancelledDetail;
  state = flow(
    state,
    { type: "cancellation_intent_opened", request_id: "cancellation-uuid" },
    { type: "cancellation_reason_changed", value: " Duplicate sale " },
    { type: "cancellation_confirmation_changed", confirmed: true },
  );
  await interaction.submitCancellation(state, dispatch);
  assert.deepEqual(requests[1], {
    request_id: "cancellation-uuid",
    sale_id: 71,
    reason: "Duplicate sale",
  });
  assert.equal(state.detail, cancelledDetail);
});

test("keeps return values for typed conflicts and malformed command results", async () => {
  let attempt = 0;
  const interaction = createSalesHistoryInteraction({
    list: emptyHistoryList,
    detail: async () => ({ kind: "success" as const, detail }),
    ...createPostSaleCommands(async () =>
      ++attempt === 1
        ? {
            kind: "error",
            code: "request_conflict",
            message: "private backend detail",
          }
        : {
            kind: "success",
            result: {
              request_id: "return-uuid",
              return_id: 12,
              sale_id: 71,
              status: "confirmed",
              occurred_at: "2024-03-11 05:00:00",
              lines: [{ sale_line_id: 41, product_id: 4, quantity: "wrong" }],
            },
          },
    ),
  });
  let state = flow(
    initialHistoryState,
    { type: "detail_loaded", detail },
    { type: "return_intent_opened", request_id: "return-uuid" },
    { type: "return_line_selected", sale_line_id: 41, selected: true },
    { type: "return_quantity_changed", sale_line_id: 41, value: "1" },
  );
  const dispatch = (action: Parameters<typeof createHistoryFlow>[1]) => {
    state = createHistoryFlow(state, action);
  };

  await interaction.submitReturn(state, dispatch);
  assert.equal(state.return_intent?.status, "error");
  assert.deepEqual(state.return_intent?.lines, { 41: "1" });
  assert.equal(
    state.return_intent?.error,
    "The inventory correction could not be completed.",
  );
  await interaction.submitReturn(state, dispatch);
  assert.equal(state.return_intent?.status, "error");
  const markup = renderToStaticMarkup(
    createElement(HistoryScreen, {
      state,
      onReload: () => undefined,
      onSelect: () => undefined,
      onBack: () => undefined,
      onReloadDetail: () => undefined,
    }),
  );
  assert.match(markup, /role="alert"/);
  assert.match(markup, /Reload sale detail/);
  state = createHistoryFlow(state, { type: "return_submit_started" });
  assert.match(
    renderToStaticMarkup(
      createElement(HistoryScreen, {
        state,
        onReload: () => undefined,
        onSelect: () => undefined,
        onBack: () => undefined,
      }),
    ),
    /aria-busy="true"/,
  );
});

const flow = (
  state: Parameters<typeof createHistoryFlow>[0],
  ...actions: Parameters<typeof createHistoryFlow>[1][]
) => actions.reduce(createHistoryFlow, state);
const openedReturn = (currentDetail = detail, request_id = "return-uuid") =>
  flow(
    initialHistoryState,
    { type: "detail_started", sale_id: currentDetail.sale_id },
    { type: "detail_loaded", detail: currentDetail },
    { type: "return_intent_opened", request_id },
  );
const selectedReturn = (currentDetail = detail, request_id?: string) =>
  flow(openedReturn(currentDetail, request_id), {
    type: "return_line_selected",
    sale_line_id: 41,
    selected: true,
  });

test("keeps exact repeated-product lines and requires an explicit retry before reload", () => {
  const repeatedProductDetail = {
    ...detail,
    lines: [
      { ...detail.lines[0], remaining_returnable_quantity: 1 },
      {
        ...detail.lines[0],
        sale_line_id: 42,
        remaining_returnable_quantity: 2,
      },
    ],
  };
  const ready = flow(
    selectedReturn(repeatedProductDetail, "return-uuid-1"),
    { type: "return_quantity_changed", sale_line_id: 41, value: "1" },
    { type: "return_line_selected", sale_line_id: 42, selected: true },
    { type: "return_quantity_changed", sale_line_id: 42, value: "2" },
  );
  const pending = flow(ready, { type: "return_submit_started" });
  const conflicted = flow(pending, {
    type: "return_submit_failed",
    message: "Return conflicts with current history.",
  });
  const staleSuccess = flow(conflicted, { type: "return_submit_succeeded" });
  const retried = flow(conflicted, { type: "return_submit_started" });
  const succeeded = flow(retried, { type: "return_submit_succeeded" });

  assert.deepEqual(pending.return_intent?.lines, { 41: "1", 42: "2" });
  assert.equal(conflicted.return_intent?.request_id, "return-uuid-1");
  assert.equal(staleSuccess, conflicted);
  assert.equal(retried.return_intent?.status, "pending");
  assert.equal(succeeded.return_intent?.status, "reload_requested");
  assert.deepEqual(succeeded.detail?.lines, repeatedProductDetail.lines);
});

test("prevents pending duplicates and validates local return quantities", () => {
  const opened = openedReturn();
  const selected = selectedReturn();
  const pending = flow(
    selected,
    { type: "return_quantity_changed", sale_line_id: 41, value: "1" },
    { type: "return_submit_started" },
  );
  const invalid = (value: string) =>
    flow(
      selected,
      { type: "return_quantity_changed", sale_line_id: 41, value },
      { type: "return_submit_started" },
    );

  assert.equal(flow(pending, { type: "return_submit_started" }), pending);
  assert.equal(
    flow(opened, { type: "return_submit_started" }).return_intent?.error,
    "Select at least one return line.",
  );
  for (const value of ["", "0", "-1", "1.5", "9007199254740992"])
    assert.equal(
      invalid(value).return_intent?.error,
      "Return quantities must be positive whole numbers.",
    );
  assert.equal(
    invalid("2").return_intent?.error,
    "Return quantity exceeds the persisted remaining availability.",
  );
  assert.equal(
    openedReturn({
      ...detail,
      lines: [{ ...detail.lines[0], remaining_returnable_quantity: 0 }],
    }).return_intent,
    null,
  );
  assert.equal(
    openedReturn({ ...detail, status: "cancelled" }).return_intent,
    null,
  );
});

test("keeps a cancellation intent stable through validation, retry, and reload", () => {
  const fullyReturned = {
    ...detail,
    lines: [{ ...detail.lines[0], remaining_returnable_quantity: 0 }],
  };
  const opened = flow(
    initialHistoryState,
    { type: "detail_loaded", detail: fullyReturned },
    { type: "cancellation_intent_opened", request_id: "cancellation-uuid" },
  );
  const blank = flow(opened, { type: "cancellation_submit_started" });
  const whitespace = flow(
    opened,
    { type: "cancellation_reason_changed", value: "   " },
    { type: "cancellation_confirmation_changed", confirmed: true },
    { type: "cancellation_submit_started" },
  );
  const unconfirmed = flow(
    opened,
    { type: "cancellation_reason_changed", value: "  Duplicate sale  " },
    { type: "cancellation_confirmation_changed", confirmed: false },
    { type: "cancellation_submit_started" },
  );
  const pending = flow(
    unconfirmed,
    {
      type: "cancellation_confirmation_changed",
      confirmed: true,
    },
    { type: "cancellation_submit_started" },
  );
  const failed = flow(pending, {
    type: "cancellation_submit_failed",
    message: "Cancellation conflicts with current history.",
  });
  const staleSuccess = flow(failed, { type: "cancellation_submit_succeeded" });
  const retried = flow(failed, { type: "cancellation_submit_started" });
  const succeeded = flow(retried, { type: "cancellation_submit_succeeded" });
  const persistedCancellation = flow(
    { ...failed, detail: { ...fullyReturned, status: "cancelled" } },
    { type: "cancellation_submit_started" },
  );

  assert.equal(opened.cancellation_intent?.request_id, "cancellation-uuid");
  assert.equal(opened.cancellation_intent?.reason, "");
  assert.equal(
    blank.cancellation_intent?.error,
    "A cancellation reason is required.",
  );
  assert.equal(
    whitespace.cancellation_intent?.error,
    "A cancellation reason is required.",
  );
  assert.equal(unconfirmed.cancellation_intent?.reason, "Duplicate sale");
  assert.equal(
    unconfirmed.cancellation_intent?.error,
    "Confirm the cancellation before submitting.",
  );
  assert.equal(flow(pending, { type: "cancellation_submit_started" }), pending);
  assert.equal(pending.detail, fullyReturned);
  assert.equal(failed.cancellation_intent?.request_id, "cancellation-uuid");
  assert.equal(failed.cancellation_intent?.reason, "Duplicate sale");
  assert.equal(failed.cancellation_intent?.confirmed, true);
  assert.equal(failed.detail, fullyReturned);
  assert.equal(staleSuccess, failed);
  assert.equal(retried.cancellation_intent?.status, "pending");
  assert.equal(retried.cancellation_intent?.request_id, "cancellation-uuid");
  assert.equal(succeeded.cancellation_intent?.status, "reload_requested");
  assert.equal(succeeded.cancellation_intent?.request_id, "cancellation-uuid");
  assert.deepEqual(succeeded.detail, fullyReturned);
  assert.equal(persistedCancellation.cancellation_intent?.status, "error");
  assert.equal(
    persistedCancellation.cancellation_intent?.error,
    "This sale is no longer eligible for cancellation. Reload history.",
  );
  assert.equal(
    flow(
      initialHistoryState,
      { type: "detail_loaded", detail: { ...detail, status: "cancelled" } },
      { type: "cancellation_intent_opened", request_id: "other-uuid" },
      { type: "cancellation_submit_started" },
    ).cancellation_intent,
    null,
  );
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

test("renders immutable lifecycle facts and persisted correction presentation", () => {
  const correctedDetail: SalesHistoryDetail = {
    ...detail,
    status: "cancelled",
    lines: [
      {
        ...detail.lines[0],
        sale_line_id: 41,
        remaining_returnable_quantity: 0,
      },
      {
        ...detail.lines[0],
        sale_line_id: 42,
        remaining_returnable_quantity: 0,
      },
    ],
    returns: [
      {
        return_id: 81,
        request_id: "return-request",
        occurred_at: "2024-03-11 05:00:00",
        lines: [{ sale_line_id: 41, product_id: 4, quantity: 1 }],
      },
    ],
    cancellation: {
      cancellation_id: 91,
      request_id: "cancellation-request",
      occurred_at: "2024-03-12 05:00:00",
      reason: "Duplicate sale",
      lines: [
        { sale_line_id: 41, product_id: 4, restored_quantity: 0 },
        { sale_line_id: 42, product_id: 4, restored_quantity: 0 },
      ],
    },
  };
  const shown = flow(initialHistoryState, {
    type: "detail_loaded",
    detail: correctedDetail,
  });
  const markup = renderToStaticMarkup(
    createElement(HistoryScreen, {
      state: shown,
      onReload: () => undefined,
      onSelect: () => undefined,
      onBack: () => undefined,
    }),
  );

  assert.match(markup, /Lifecycle status: Cancelled/);
  assert.match(markup, /Original sale items/);
  assert.match(markup, /Original payment facts/);
  assert.match(markup, /Inventory correction history/);
  assert.match(markup, /Returned quantity: 1/);
  assert.match(markup, /Cancellation reason: Duplicate sale/);
  assert.match(markup, /Restored quantity: 0/);
});

test("renders sale-line keyed correction forms from persisted status and quantities", () => {
  const repeatedLines = {
    ...detail,
    lines: [
      {
        ...detail.lines[0],
        sale_line_id: 41,
        remaining_returnable_quantity: 1,
      },
      {
        ...detail.lines[0],
        sale_line_id: 42,
        remaining_returnable_quantity: 2,
      },
    ],
  };
  const returnState = flow(
    initialHistoryState,
    { type: "detail_loaded", detail: repeatedLines },
    { type: "return_intent_opened", request_id: "return-id" },
    { type: "return_line_selected", sale_line_id: 42, selected: true },
  );
  const cancellationState = flow(
    initialHistoryState,
    { type: "detail_loaded", detail: repeatedLines },
    { type: "cancellation_intent_opened", request_id: "cancellation-id" },
  );
  const fullyReturned = flow(initialHistoryState, {
    type: "detail_loaded",
    detail: {
      ...repeatedLines,
      lines: repeatedLines.lines.map((line) => ({
        ...line,
        remaining_returnable_quantity: 0,
      })),
    },
  });
  const render = (state: typeof returnState) =>
    renderToStaticMarkup(
      createElement(HistoryScreen, {
        state,
        onReload: () => undefined,
        onSelect: () => undefined,
        onBack: () => undefined,
      }),
    );

  assert.match(render(returnState), /aria-label="Return items to inventory"/);
  assert.match(render(returnState), /name="return-line-41"/);
  assert.match(render(returnState), /name="return-line-42"/);
  assert.match(render(returnState), /max="2"/);
  assert.match(render(cancellationState), /Cancellation reason/);
  assert.match(
    render(cancellationState),
    /I confirm this inventory correction/,
  );
  assert.doesNotMatch(render(fullyReturned), /Begin item return/);
  assert.match(render(fullyReturned), /Begin sale cancellation/);
});

test("renders keyboard-operable correction forms with inventory-only language", () => {
  const state = flow(
    initialHistoryState,
    { type: "detail_loaded", detail },
    { type: "return_intent_opened", request_id: "return-keyboard" },
    { type: "return_line_selected", sale_line_id: 41, selected: true },
    { type: "cancellation_intent_opened", request_id: "cancellation-keyboard" },
  );
  const markup = renderToStaticMarkup(
    createElement(HistoryScreen, {
      state,
      onReload: () => undefined,
      onSelect: () => undefined,
      onBack: () => undefined,
    }),
  );

  assert.match(markup, /<form(?=[^>]*aria-label="Return items to inventory")/);
  assert.match(
    markup,
    /<input(?=[^>]*id="return-line-41")(?=[^>]*type="checkbox")/,
  );
  assert.match(
    markup,
    /<input(?=[^>]*id="return-quantity-41")(?=[^>]*type="number")/,
  );
  assert.match(markup, /<form(?=[^>]*aria-label="Cancel sale")/);
  assert.match(markup, /<input(?=[^>]*id="cancellation-reason")/);
  assert.match(
    markup,
    /<input(?=[^>]*id="cancellation-confirmation")(?=[^>]*type="checkbox")/,
  );
  assert.match(
    markup,
    /<button(?=[^>]*type="submit")[^>]*>Record inventory return<\/button>/,
  );
  assert.match(
    markup,
    /<button(?=[^>]*type="submit")[^>]*>Record sale cancellation<\/button>/,
  );
  assert.doesNotMatch(
    markup,
    /\b(refund|reimbursement|payment reversal|credit|settlement)\b/i,
  );
});

test("derives deterministic correction focus in the reducer and applies it through HistoryScreen's effect", () => {
  const repeatedLines = {
    ...detail,
    lines: [
      {
        ...detail.lines[0],
        sale_line_id: 42,
        remaining_returnable_quantity: 2,
      },
      {
        ...detail.lines[0],
        sale_line_id: 41,
        remaining_returnable_quantity: 1,
      },
    ],
  };
  const invalidReturn = flow(
    initialHistoryState,
    { type: "detail_loaded", detail: repeatedLines },
    { type: "return_intent_opened", request_id: "return-focus" },
    { type: "return_line_selected", sale_line_id: 42, selected: true },
    { type: "return_quantity_changed", sale_line_id: 42, value: "1" },
    { type: "return_line_selected", sale_line_id: 41, selected: true },
    { type: "return_quantity_changed", sale_line_id: 41, value: "0" },
    { type: "return_submit_started" },
  );
  assert.deepEqual(invalidReturn.return_intent?.validation, {
    message: "Return quantities must be positive whole numbers.",
    focus_target: "return-quantity-41",
  });
  assert.equal(correctionFocusTarget(invalidReturn), "return-quantity-41");
  let focused: string | null = null;
  let scheduledEffects = 0;
  renderToStaticMarkup(
    createElement(HistoryScreen, {
      state: invalidReturn,
      onReload: () => undefined,
      onSelect: () => undefined,
      onBack: () => undefined,
      runEffect: (effect) => {
        scheduledEffects += 1;
        effect();
      },
      findFocusable: (id) => ({
        focus: () => {
          focused = id;
        },
      }),
    }),
  );
  assert.equal(scheduledEffects, 1);
  assert.equal(focused, "return-quantity-41");

  const missingSelection = flow(
    initialHistoryState,
    { type: "detail_loaded", detail: repeatedLines },
    { type: "return_intent_opened", request_id: "return-focus-empty" },
    { type: "return_submit_started" },
  );
  assert.equal(correctionFocusTarget(missingSelection), "return-line-41");

  const missingReason = flow(
    initialHistoryState,
    { type: "detail_loaded", detail: repeatedLines },
    { type: "cancellation_intent_opened", request_id: "cancellation-focus" },
    { type: "cancellation_submit_started" },
  );
  const missingConfirmation = flow(
    missingReason,
    { type: "cancellation_reason_changed", value: "Duplicate sale" },
    { type: "cancellation_submit_started" },
  );
  assert.equal(correctionFocusTarget(missingReason), "cancellation-reason");
  assert.equal(
    correctionFocusTarget(missingConfirmation),
    "cancellation-confirmation",
  );
});

test("renders correction controls with 44px minimum targets", () => {
  const state = flow(
    initialHistoryState,
    { type: "detail_loaded", detail },
    { type: "return_intent_opened", request_id: "return-target" },
    { type: "return_line_selected", sale_line_id: 41, selected: true },
    { type: "cancellation_intent_opened", request_id: "cancellation-target" },
  );
  const markup = renderToStaticMarkup(
    createElement(HistoryScreen, {
      state,
      onReload: () => undefined,
      onSelect: () => undefined,
      onBack: () => undefined,
    }),
  );

  assert.match(
    markup,
    /id="return-line-41"(?=[^>]*style="min-width:44px;min-height:44px")/,
  );
  assert.match(
    markup,
    /id="return-quantity-41"(?=[^>]*style="min-width:44px;min-height:44px")/,
  );
  assert.match(
    markup,
    /id="cancellation-reason"(?=[^>]*style="min-width:44px;min-height:44px")/,
  );
  assert.match(
    markup,
    /id="cancellation-confirmation"(?=[^>]*style="min-width:44px;min-height:44px")/,
  );
  assert.match(
    markup,
    /<button(?=[^>]*style="min-width:44px;min-height:44px")[^>]*>Record inventory return<\/button>/,
  );
  assert.match(
    markup,
    /<button(?=[^>]*style="min-width:44px;min-height:44px")[^>]*>Record sale cancellation<\/button>/,
  );
});
