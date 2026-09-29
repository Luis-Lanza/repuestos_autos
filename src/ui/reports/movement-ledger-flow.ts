import type { ExportResponse, MovementLedgerFilters, MovementLedgerProductOption, MovementLedgerRow, MovementLedgerResponse, ProductOptionsResponse } from "../../commands/movement-ledger.ts";

export type LedgerState = {
  status: "initial" | "loading" | "empty" | "error" | "ready";
  rows: MovementLedgerRow[];
  has_more: boolean;
  page: number;
  draft: MovementLedgerFilters;
  applied: MovementLedgerFilters;
  products: MovementLedgerProductOption[];
  products_status: "loading" | "ready" | "error";
  products_request: number;
  list_request: number;
  export_request: number;
  export_status: "idle" | "pending" | "success" | "cancelled" | "error";
};
const isoDate = (date: Date) => `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
export function currentMonthFilters(now = new Date()): MovementLedgerFilters { return { from: isoDate(new Date(now.getFullYear(), now.getMonth(), 1)), to: isoDate(new Date(now.getFullYear(), now.getMonth() + 1, 0)), product_id: null, movement_type: null }; }
export const initialLedgerState = (now = new Date()): LedgerState => ({ status: "initial", rows: [], has_more: false, page: 1, draft: currentMonthFilters(now), applied: currentMonthFilters(now), products: [], products_status: "loading", products_request: 0, list_request: 0, export_request: 0, export_status: "idle" });
export type LedgerAction =
 | { type: "draft_changed"; filters: MovementLedgerFilters }
 | { type: "filters_applied"; request_id: number }
 | { type: "page_requested"; request_id: number; page: number }
 | { type: "list_succeeded"; request_id: number; response: Extract<MovementLedgerResponse, { kind: "success" }> }
 | { type: "list_failed"; request_id: number }
 | { type: "products_started"; request_id: number }
 | { type: "products_finished"; request_id: number; response: ProductOptionsResponse }
 | { type: "export_started"; request_id: number }
 | { type: "export_finished"; request_id: number; response: ExportResponse };
export function createMovementLedgerFlow(state: LedgerState, action: LedgerAction): LedgerState {
 switch (action.type) {
  case "draft_changed": return { ...state, draft: action.filters };
  case "filters_applied": return { ...state, draft: actionFilters(state.draft), applied: actionFilters(state.draft), page: 1, rows: [], has_more: false, status: "loading", list_request: action.request_id, export_status: "idle" };
  case "page_requested": return { ...state, page: action.page, status: "loading", list_request: action.request_id, export_status: "idle" };
  case "list_succeeded": if (action.request_id !== state.list_request) return state; return { ...state, rows: action.response.rows, page: action.response.page, has_more: action.response.has_more, status: action.response.rows.length ? "ready" : "empty" };
  case "list_failed": return action.request_id === state.list_request ? { ...state, rows: [], has_more: false, status: "error" } : state;
  case "products_started": return { ...state, products_request: action.request_id, products_status: "loading" };
  case "products_finished": if (action.request_id !== state.products_request) return state; return action.response.kind === "success" ? { ...state, products: action.response.products, products_status: "ready" } : { ...state, products_status: "error" };
  case "export_started": return { ...state, export_request: action.request_id, export_status: "pending" };
  case "export_finished": if (action.request_id !== state.export_request) return state; return { ...state, export_status: action.response.kind === "success" ? "success" : action.response.kind === "cancelled" ? "cancelled" : "error" };
 }
}
function actionFilters(filters: MovementLedgerFilters): MovementLedgerFilters { return { ...filters }; }
