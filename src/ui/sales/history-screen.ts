import { createElement, type FormEvent, useEffect, useReducer, useState } from "react";

import { salesHistoryCommands, type SalesHistorySummary } from "../../commands/sales-history.ts";
import { createHistoryFlow, initialHistoryState, type HistoryAction, type HistoryState } from "./history-flow.ts";

const formatBs = (centavos: number) => `Bs ${(centavos / 100).toFixed(2)}`;
const localToday = () => {
  const date = new Date();
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
};

export type HistoryScreenProps = {
  state: HistoryState;
  onReload: (from: string, to: string) => void;
  onSelect: (sale: SalesHistorySummary) => void;
  onBack: () => void;
};

type HistoryCommands = Pick<typeof salesHistoryCommands, "list" | "detail">;
type Dispatch = (action: HistoryAction) => void;

export function createSalesHistoryInteraction(commands: HistoryCommands) {
  return {
    reload: async (from: string, to: string, dispatch: Dispatch) => {
      dispatch({ type: "list_started" });
      const response = await commands.list(from, to);
      dispatch(response.kind === "success" ? { type: "list_loaded", sales: response.sales, has_more: response.has_more } : { type: "list_failed", message: response.message });
    },
    select: async (sale: SalesHistorySummary, dispatch: Dispatch) => {
      dispatch({ type: "detail_started", sale_id: sale.sale_id });
      const response = await commands.detail(sale.sale_id);
      dispatch(response.kind === "success" ? { type: "detail_loaded", detail: response.detail } : { type: "detail_failed", message: response.message });
    },
  };
}

export function HistoryScreen({ state, onReload, onSelect, onBack }: HistoryScreenProps) {
  const [from, setFrom] = useState(localToday);
  const [to, setTo] = useState(localToday);
  const submit = (event: FormEvent<HTMLFormElement>) => { event.preventDefault(); onReload(from, to); };
  if (state.view === "detail") {
    return createElement("main", { "aria-labelledby": "sales-history-detail-heading", "aria-busy": state.status === "loading" },
      createElement("button", { type: "button", onClick: onBack }, "Back to history"),
      createElement("h1", { id: "sales-history-detail-heading" }, "Sale history detail"),
      state.status === "loading" ? createElement("p", { role: "status" }, "Loading sale details…") : null,
      state.status === "error" ? createElement("p", { role: "alert" }, state.message) : null,
      state.detail ? createElement("section", null,
        createElement("p", null, `Sale ${state.detail.sale_id} · ${state.detail.confirmed_at}`),
        createElement("p", null, `Total: ${formatBs(state.detail.total_centavos)}`),
        createElement("h2", null, "Items"),
        createElement("ul", { "aria-label": "Historical sale items" }, state.detail.lines.map((line) => createElement("li", { key: `${line.product_id}-${line.sku ?? "unavailable"}` }, `${line.sku ?? "Unavailable"} — ${line.product_name ?? "Unavailable"}; ${line.quantity} × ${formatBs(line.unit_price_centavos)} = ${formatBs(line.line_total_centavos)}`))),
        createElement("h2", null, "Payments"),
        createElement("ul", { "aria-label": "Historical sale payments" }, state.detail.payments.map((payment, index) => createElement("li", { key: `${payment.method}-${index}` }, payment.method === "cash" ? `Cash applied: ${formatBs(payment.amount_applied_centavos)}; Tendered: ${formatBs(payment.amount_tendered_centavos)}; Change: ${formatBs(payment.change_given_centavos)}` : `QR applied: ${formatBs(payment.amount_applied_centavos)}`))),
      ) : null,
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
  const interaction = createSalesHistoryInteraction(salesHistoryCommands);
  const reload = (from: string, to: string) => interaction.reload(from, to, dispatch);
  const select = (sale: SalesHistorySummary) => interaction.select(sale, dispatch);
  useEffect(() => { const today = localToday(); void reload(today, today); }, []);
  return createElement(HistoryScreen, { state, onReload: (from, to) => void reload(from, to), onSelect: (sale) => void select(sale), onBack: () => dispatch({ type: "back_to_list" }) });
}
