import type {
  SalesHistoryDetail,
  SalesHistorySummary,
} from "../../commands/sales-history.ts";

export type HistoryState = {
  view: "list" | "detail";
  status: "idle" | "loading" | "ready" | "empty" | "error";
  sales: SalesHistorySummary[];
  has_more: boolean;
  selected_id: number | null;
  detail: SalesHistoryDetail | null;
  message: string | null;
};

export const initialHistoryState: HistoryState = {
  view: "list",
  status: "idle",
  sales: [],
  has_more: false,
  selected_id: null,
  detail: null,
  message: null,
};

export type HistoryAction =
  | { type: "list_started" }
  | { type: "list_loaded"; sales: SalesHistorySummary[]; has_more: boolean }
  | { type: "list_failed"; message: string }
  | { type: "detail_started"; sale_id: number }
  | { type: "detail_loaded"; detail: SalesHistoryDetail }
  | { type: "detail_failed"; message: string }
  | { type: "back_to_list" };

export function createHistoryFlow(
  state: HistoryState,
  action: HistoryAction,
): HistoryState {
  switch (action.type) {
    case "list_started":
      return { ...state, view: "list", status: "loading", detail: null, message: null };
    case "list_loaded":
      return { ...state, view: "list", status: action.sales.length ? "ready" : "empty", sales: action.sales, has_more: action.has_more, selected_id: null, detail: null, message: null };
    case "list_failed":
      return { ...state, view: "list", status: "error", sales: [], has_more: false, detail: null, message: action.message };
    case "detail_started":
      return { ...state, view: "detail", status: "loading", selected_id: action.sale_id, detail: null, message: null };
    case "detail_loaded":
      return { ...state, view: "detail", status: "ready", selected_id: action.detail.sale_id, detail: action.detail, message: null };
    case "detail_failed":
      return { ...state, view: "detail", status: "error", detail: null, message: action.message };
    case "back_to_list":
      return { ...state, view: "list", status: state.sales.length ? "ready" : "empty", selected_id: null, detail: null, message: null };
  }
}
