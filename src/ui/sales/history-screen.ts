import {
  createElement,
  type ChangeEvent,
  type DependencyList,
  type EffectCallback,
  type FormEvent,
  useEffect,
  useMemo,
  useReducer,
  useState,
} from "react";

import { postSaleCommands } from "../../commands/post-sale.ts";
import {
  salesHistoryCommands,
  type SalesHistorySummary,
} from "../../commands/sales-history.ts";
import {
  canOpenCancellation,
  canOpenReturn,
  correctionFocusTarget,
  createHistoryFlow,
  initialHistoryState,
  type CancellationIntent,
  type HistoryAction,
  type HistoryState,
  type ReturnIntent,
} from "./history-flow.ts";

const formatBs = (centavos: number) => `Bs ${(centavos / 100).toFixed(2)}`;
const localToday = () => {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
};

export const correctionControlStyle = { minWidth: 44, minHeight: 44 } as const;

type Focusable = Pick<HTMLElement, "focus">;
export type FocusFinder = (id: string) => Focusable | null;
type EffectRunner = (
  effect: EffectCallback,
  dependencies?: DependencyList,
) => void;

const findFocusableById: FocusFinder = (id) =>
  globalThis.document?.getElementById(id) ?? null;

const focusCorrectionTarget = (target: string | null, find: FocusFinder) => {
  if (target) find(target)?.focus();
};

export type HistoryScreenProps = {
  state: HistoryState;
  onReload: (from: string, to: string) => void;
  onSelect: (sale: SalesHistorySummary) => void;
  onBack: () => void;
  onAction?: (action: HistoryAction) => void;
  onReturnSubmit?: () => void;
  onCancellationSubmit?: () => void;
  onReloadDetail?: (saleId: number) => void;
  runEffect?: EffectRunner;
  findFocusable?: FocusFinder;
};

type HistoryCommands = Pick<typeof salesHistoryCommands, "list" | "detail">;
type InteractionCommands = HistoryCommands & Partial<typeof postSaleCommands>;
type Dispatch = (action: HistoryAction) => void;
type CorrectionIntent = ReturnIntent | CancellationIntent;
type CorrectionResponse =
  { kind: "success" } | { kind: "error"; message: string };

export function createSalesHistoryInteraction(commands: InteractionCommands) {
  const submitting = new Set<string>();
  const reloadDetail = async (saleId: number, dispatch: Dispatch) => {
    dispatch({ type: "detail_started", sale_id: saleId });
    const response = await commands.detail(saleId);
    dispatch(
      response.kind === "success"
        ? { type: "detail_loaded", detail: response.detail }
        : { type: "detail_failed", message: response.message },
    );
  };
  const submit = async (
    state: HistoryState,
    dispatch: Dispatch,
    isReturn: boolean,
    intent: CorrectionIntent | null,
    command: (
      intent: CorrectionIntent,
    ) => Promise<CorrectionResponse> | undefined,
  ) => {
    const started: HistoryAction = {
      type: isReturn ? "return_submit_started" : "cancellation_submit_started",
    };
    const pending = createHistoryFlow(state, started);
    dispatch(started);
    const pendingIntent = isReturn
      ? pending.return_intent
      : pending.cancellation_intent;
    if (
      !intent ||
      pendingIntent?.status !== "pending" ||
      submitting.has(intent.request_id)
    )
      return;
    submitting.add(intent.request_id);
    try {
      const response = await command(intent);
      if (!response || response.kind === "error") {
        dispatch({
          type: isReturn
            ? "return_submit_failed"
            : "cancellation_submit_failed",
          message:
            response?.kind === "error"
              ? response.message
              : "The inventory correction could not be completed.",
        });
        return;
      }
      dispatch({
        type: isReturn
          ? "return_submit_succeeded"
          : "cancellation_submit_succeeded",
      });
      await reloadDetail(intent.sale_id, dispatch);
    } finally {
      submitting.delete(intent.request_id);
    }
  };
  const submitReturn = (state: HistoryState, dispatch: Dispatch) =>
    submit(state, dispatch, true, state.return_intent, (intent) =>
      commands.createReturn?.({
        request_id: intent.request_id,
        sale_id: intent.sale_id,
        lines: Object.entries((intent as ReturnIntent).lines).map(
          ([sale_line_id, quantity]) => ({
            sale_line_id: Number(sale_line_id),
            quantity: Number(quantity),
          }),
        ),
      }),
    );
  const submitCancellation = (state: HistoryState, dispatch: Dispatch) =>
    submit(state, dispatch, false, state.cancellation_intent, (intent) =>
      commands.cancelSale?.({
        request_id: intent.request_id,
        sale_id: intent.sale_id,
        reason: (intent as CancellationIntent).reason,
      }),
    );
  return {
    reload: async (from: string, to: string, dispatch: Dispatch) => {
      dispatch({ type: "list_started" });
      const response = await commands.list(from, to);
      dispatch(
        response.kind === "success"
          ? {
              type: "list_loaded",
              sales: response.sales,
              has_more: response.has_more,
            }
          : { type: "list_failed", message: response.message },
      );
    },
    select: async (sale: SalesHistorySummary, dispatch: Dispatch) => {
      await reloadDetail(sale.sale_id, dispatch);
    },
    reloadDetail,
    submitReturn,
    submitCancellation,
  };
}

export function HistoryScreen({
  state,
  onReload,
  onSelect,
  onBack,
  onAction,
  onReturnSubmit,
  onCancellationSubmit,
  onReloadDetail,
  runEffect = useEffect,
  findFocusable = findFocusableById,
}: HistoryScreenProps) {
  const [from, setFrom] = useState(localToday);
  const [to, setTo] = useState(localToday);
  const focusTarget = correctionFocusTarget(state);
  runEffect(() => {
    focusCorrectionTarget(focusTarget, findFocusable);
  }, [focusTarget, findFocusable]);
  const submit = (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); onReload(from, to); };
  if (state.view === "detail") {
    return createElement("main", { "aria-labelledby": "sales-history-detail-heading", "aria-busy": state.status === "loading" },
      createElement("button", { type: "button", onClick: onBack }, "Back to history"),
      createElement("h1", { id: "sales-history-detail-heading" }, "Sale history detail"),
      state.status === "loading" ? createElement("p", { role: "status" }, "Loading sale details…") : null,
      state.status === "error" ? createElement("p", { role: "alert" }, state.message) : null,
      state.detail
        ? createElement(
            "section",
            null,
            createElement(
              "p",
              null,
              `Sale ${state.detail.sale_id} · ${state.detail.confirmed_at}`,
            ),
            createElement(
              "p",
              null,
              `Lifecycle status: ${state.detail.status === "confirmed" ? "Confirmed" : "Cancelled"}`,
            ),
            createElement(
              "p",
              null,
              `Original sale total: ${formatBs(state.detail.total_centavos)}`,
            ),
            createElement(
              "section",
              { "aria-labelledby": "original-sale-items-heading" },
              createElement(
                "h2",
                { id: "original-sale-items-heading" },
                "Original sale items",
              ),
              createElement(
                "ul",
                { "aria-label": "Original sale items" },
                state.detail.lines.map((line) =>
                  createElement(
                    "li",
                    { key: line.sale_line_id },
                    `Sale line ${line.sale_line_id}: ${line.sku ?? "Unavailable"} — ${line.product_name ?? "Unavailable"}; ${line.quantity} × ${formatBs(line.unit_price_centavos)} = ${formatBs(line.line_total_centavos)}`,
                  ),
                ),
              ),
            ),
            createElement(
              "section",
              { "aria-labelledby": "original-payment-facts-heading" },
              createElement(
                "h2",
                { id: "original-payment-facts-heading" },
                "Original payment facts",
              ),
              createElement(
                "ul",
                { "aria-label": "Original payment facts" },
                state.detail.payments.map((payment, index) =>
                  createElement(
                    "li",
                    { key: `${payment.method}-${index}` },
                    payment.method === "cash"
                      ? `Cash applied: ${formatBs(payment.amount_applied_centavos)}; Tendered: ${formatBs(payment.amount_tendered_centavos)}; Change: ${formatBs(payment.change_given_centavos)}`
                      : `QR applied: ${formatBs(payment.amount_applied_centavos)}`,
                  ),
                ),
              ),
            ),
            createElement(
              "section",
              { "aria-labelledby": "inventory-correction-history-heading" },
              createElement(
                "h2",
                { id: "inventory-correction-history-heading" },
                "Inventory correction history",
              ),
              state.detail.returns.length === 0 &&
                state.detail.cancellation === null
                ? createElement("p", null, "No inventory corrections recorded.")
                : null,
              state.detail.returns.map((returned) =>
                createElement(
                  "section",
                  {
                    key: returned.return_id,
                    "aria-label": `Return ${returned.return_id}`,
                  },
                  createElement(
                    "h3",
                    null,
                    `Return ${returned.return_id} · ${returned.occurred_at}`,
                  ),
                  createElement(
                    "p",
                    null,
                    `Request ID: ${returned.request_id}`,
                  ),
                  createElement(
                    "ul",
                    null,
                    returned.lines.map((line) =>
                      createElement(
                        "li",
                        { key: line.sale_line_id },
                        `Sale line ${line.sale_line_id} · Product ${line.product_id} · Returned quantity: ${line.quantity}`,
                      ),
                    ),
                  ),
                ),
              ),
              state.detail.cancellation
                ? createElement(
                    "section",
                    {
                      "aria-label": `Cancellation ${state.detail.cancellation.cancellation_id}`,
                    },
                    createElement(
                      "h3",
                      null,
                      `Cancellation ${state.detail.cancellation.cancellation_id} · ${state.detail.cancellation.occurred_at}`,
                    ),
                    createElement(
                      "p",
                      null,
                      `Cancellation reason: ${state.detail.cancellation.reason}`,
                    ),
                    createElement(
                      "p",
                      null,
                      `Request ID: ${state.detail.cancellation.request_id}`,
                    ),
                    createElement(
                      "ul",
                      null,
                      state.detail.cancellation.lines.map((line) =>
                        createElement(
                          "li",
                          { key: line.sale_line_id },
                          `Sale line ${line.sale_line_id} · Product ${line.product_id} · Restored quantity: ${line.restored_quantity}`,
                        ),
                      ),
                    ),
                  )
                : null,
            ),
            canOpenReturn(state)
              ? createElement(
                  "button",
                  {
                    type: "button",
                    onClick: () =>
                      onAction?.({
                        type: "return_intent_opened",
                        request_id: crypto.randomUUID(),
                      }),
                    style: correctionControlStyle,
                  },
                  "Begin item return",
                )
              : null,
            state.return_intent
              ? createElement(
                  "form",
                  {
                    "aria-label": "Return items to inventory",
                    "aria-busy": state.return_intent.status === "pending",
                    onSubmit: (event: FormEvent<HTMLFormElement>) => {
                      event.preventDefault();
                      if (onReturnSubmit) onReturnSubmit();
                      else onAction?.({ type: "return_submit_started" });
                    },
                  },
                  createElement("h2", null, "Return items to inventory"),
                  createElement(
                    "p",
                    null,
                    "Select original sale lines and quantities from persisted remaining availability.",
                  ),
                  state.detail.lines.map((line) => {
                    const selected =
                      String(line.sale_line_id) in state.return_intent!.lines;
                    const disabled =
                      state.return_intent!.status === "pending" ||
                      line.remaining_returnable_quantity <= 0;
                    return createElement(
                      "fieldset",
                      { key: line.sale_line_id, disabled },
                      createElement(
                        "legend",
                        null,
                        `Sale line ${line.sale_line_id} · Remaining returnable quantity: ${line.remaining_returnable_quantity}`,
                      ),
                      createElement(
                        "label",
                        { htmlFor: `return-line-${line.sale_line_id}` },
                        "Include this original sale line",
                        createElement("input", {
                          id: `return-line-${line.sale_line_id}`,
                          name: `return-line-${line.sale_line_id}`,
                          type: "checkbox",
                          checked: selected,
                          style: correctionControlStyle,
                          onChange: (event: ChangeEvent<HTMLInputElement>) =>
                            onAction?.({
                              type: "return_line_selected",
                              sale_line_id: line.sale_line_id,
                              selected: event.target.checked,
                            }),
                        }),
                      ),
                      selected
                        ? createElement(
                            "label",
                            { htmlFor: `return-quantity-${line.sale_line_id}` },
                            "Return quantity",
                            createElement("input", {
                              id: `return-quantity-${line.sale_line_id}`,
                              name: `return-quantity-${line.sale_line_id}`,
                              type: "number",
                              min: 1,
                              max: line.remaining_returnable_quantity,
                              step: 1,
                              style: correctionControlStyle,
                              value:
                                state.return_intent!.lines[line.sale_line_id],
                              onChange: (
                                event: ChangeEvent<HTMLInputElement>,
                              ) =>
                                onAction?.({
                                  type: "return_quantity_changed",
                                  sale_line_id: line.sale_line_id,
                                  value: event.target.value,
                                }),
                            }),
                          )
                        : null,
                    );
                  }),
                  state.return_intent.error
                    ? createElement(
                        "p",
                        { role: "alert" },
                        state.return_intent.error,
                      )
                    : null,
                  state.return_intent.error && onReloadDetail
                    ? createElement(
                        "button",
                        {
                          type: "button",
                          onClick: () =>
                            onReloadDetail(state.return_intent!.sale_id),
                          style: correctionControlStyle,
                        },
                        "Reload sale detail",
                      )
                    : null,
                  createElement(
                    "button",
                    {
                      type: "submit",
                      disabled: state.return_intent.status === "pending",
                      style: correctionControlStyle,
                    },
                    "Record inventory return",
                  ),
                )
              : null,
            canOpenCancellation(state)
              ? createElement(
                  "button",
                  {
                    type: "button",
                    onClick: () =>
                      onAction?.({
                        type: "cancellation_intent_opened",
                        request_id: crypto.randomUUID(),
                      }),
                    style: correctionControlStyle,
                  },
                  "Begin sale cancellation",
                )
              : null,
            state.cancellation_intent
              ? createElement(
                  "form",
                  {
                    "aria-label": "Cancel sale",
                    "aria-busy": state.cancellation_intent.status === "pending",
                    onSubmit: (event: FormEvent<HTMLFormElement>) => {
                      event.preventDefault();
                      if (onCancellationSubmit) onCancellationSubmit();
                      else onAction?.({ type: "cancellation_submit_started" });
                    },
                  },
                  createElement("h2", null, "Cancel sale"),
                  createElement(
                    "label",
                    { htmlFor: "cancellation-reason" },
                    "Cancellation reason",
                    createElement("input", {
                      id: "cancellation-reason",
                      name: "cancellation-reason",
                      value: state.cancellation_intent.reason,
                      disabled: state.cancellation_intent.status === "pending",
                      style: correctionControlStyle,
                      onChange: (event: ChangeEvent<HTMLInputElement>) =>
                        onAction?.({
                          type: "cancellation_reason_changed",
                          value: event.target.value,
                        }),
                    }),
                  ),
                  createElement(
                    "label",
                    { htmlFor: "cancellation-confirmation" },
                    "I confirm this inventory correction. Original payment facts remain unchanged.",
                    createElement("input", {
                      id: "cancellation-confirmation",
                      name: "cancellation-confirmation",
                      type: "checkbox",
                      checked: state.cancellation_intent.confirmed,
                      disabled: state.cancellation_intent.status === "pending",
                      style: correctionControlStyle,
                      onChange: (event: ChangeEvent<HTMLInputElement>) =>
                        onAction?.({
                          type: "cancellation_confirmation_changed",
                          confirmed: event.target.checked,
                        }),
                    }),
                  ),
                  state.cancellation_intent.error
                    ? createElement(
                        "p",
                        { role: "alert" },
                        state.cancellation_intent.error,
                      )
                    : null,
                  state.cancellation_intent.error && onReloadDetail
                    ? createElement(
                        "button",
                        {
                          type: "button",
                          onClick: () =>
                            onReloadDetail(state.cancellation_intent!.sale_id),
                          style: correctionControlStyle,
                        },
                        "Reload sale detail",
                      )
                    : null,
                  createElement(
                    "button",
                    {
                      type: "submit",
                      disabled: state.cancellation_intent.status === "pending",
                      style: correctionControlStyle,
                    },
                    "Record sale cancellation",
                  ),
                )
              : null,
          )
        : null,
    );
  }
  return createElement("main", { "aria-labelledby": "sales-history-heading", "aria-busy": state.status === "loading" },
    createElement("h1", { id: "sales-history-heading" }, "Sales history"),
    createElement("form", { onSubmit: submit },
      createElement("label", { htmlFor: "history-from" }, "From", createElement("input", { id: "history-from", type: "date", value: from, onChange: (event) => setFrom(event.target.value) })),
      createElement("label", { htmlFor: "history-to" }, "To", createElement("input", { id: "history-to", type: "date", value: to, onChange: (event) => setTo(event.target.value) })),
      createElement("button", { type: "submit", disabled: state.status === "loading" }, "Load history"),
    ),
    state.status === "loading" ? createElement("p", { role: "status" }, "Loading sales history…") : null,
    state.status === "empty" ? createElement("p", { role: "status" }, "No confirmed sales match this date range.") : null,
    state.status === "error" ? createElement("div", null, createElement("p", { role: "alert" }, state.message), createElement("button", { type: "button", onClick: () => onReload(from, to) }, "Retry history")) : null,
    state.status === "ready" ? createElement("ul", { "aria-label": "Sales history results" }, state.sales.map((sale) => createElement("li", { key: sale.sale_id }, createElement("button", { type: "button", onClick: () => onSelect(sale) }, `Sale ${sale.sale_id} · ${sale.confirmed_at} · ${formatBs(sale.total_centavos)}`), ` (${sale.line_count} lines, ${sale.payment_count} payments)`))) : null,
    state.has_more ? createElement("p", { role: "status" }, "More matching sales exist. Narrow the date range.") : null,
  );
}

export function SalesHistoryScreen() {
  const [state, dispatch] = useReducer(createHistoryFlow, initialHistoryState);
  const interaction = useMemo(
    () =>
      createSalesHistoryInteraction({
        ...salesHistoryCommands,
        ...postSaleCommands,
      }),
    [],
  );
  const reload = (from: string, to: string) =>
    interaction.reload(from, to, dispatch);
  const select = (sale: SalesHistorySummary) =>
    interaction.select(sale, dispatch);
  useEffect(() => {
    const today = localToday();
    void reload(today, today);
  }, []);
  return createElement(HistoryScreen, {
    state,
    onReload: (from, to) => void reload(from, to),
    onSelect: (sale) => void select(sale),
    onBack: () => dispatch({ type: "back_to_list" }),
    onAction: dispatch,
    onReturnSubmit: () => void interaction.submitReturn(state, dispatch),
    onCancellationSubmit: () =>
      void interaction.submitCancellation(state, dispatch),
    onReloadDetail: (saleId) => void interaction.reloadDetail(saleId, dispatch),
  });
}
