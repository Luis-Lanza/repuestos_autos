import type { ExportResponse, MovementLedgerFilters, MovementLedgerProductOption, MovementLedgerResponse, ProductOptionsResponse } from "../../commands/movement-ledger.ts";

export type LedgerState = {
  status: "initial" | "loading" | "empty" | "error" | "ready";
  rows: import("../../commands/movement-ledger.ts").MovementLedgerRow[];
  has_more: boolean;
  page: number;
  draft: MovementLedgerFilters;
  applied: MovementLedgerFilters;
  product_query: string;
  product_submitted_query: string;
  selected_product: MovementLedgerProductOption | null;
  products: MovementLedgerProductOption[];
  products_status: "initial" | "loading" | "ready" | "empty" | "error";
  products_page: number;
  products_has_more: boolean;
  products_request: number;
  list_request: number;
  export_request: number;
  export_status: "idle" | "pending" | "success" | "cancelled" | "error";
};
const isoDate = (date: Date) => `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
export function currentMonthFilters(now = new Date()): MovementLedgerFilters { return { from: isoDate(new Date(now.getFullYear(), now.getMonth(), 1)), to: isoDate(new Date(now.getFullYear(), now.getMonth() + 1, 0)), product_id: null, movement_type: null }; }
export const initialLedgerState = (now = new Date()): LedgerState => ({ status: "initial", rows: [], has_more: false, page: 1, draft: currentMonthFilters(now), applied: currentMonthFilters(now), product_query: "", product_submitted_query: "", selected_product: null, products: [], products_status: "initial", products_page: 1, products_has_more: false, products_request: 0, list_request: 0, export_request: 0, export_status: "idle" });
export type LedgerAction =
 | { type: "draft_changed"; filters: MovementLedgerFilters }
 | { type: "product_query_changed"; query: string; request_id: number }
 | { type: "product_search_started"; request_id: number; query: string; page: number }
 | { type: "product_search_succeeded"; request_id: number; response: Extract<ProductOptionsResponse, { kind: "success" }> }
 | { type: "product_search_failed"; request_id: number }
 | { type: "product_selected"; product: MovementLedgerProductOption; request_id: number }
 | { type: "product_cleared"; request_id: number }
 | { type: "filters_applied"; request_id: number }
 | { type: "page_requested"; request_id: number; page: number }
 | { type: "list_succeeded"; request_id: number; response: Extract<MovementLedgerResponse, { kind: "success" }> }
 | { type: "list_failed"; request_id: number }
 | { type: "export_started"; request_id: number }
 | { type: "export_finished"; request_id: number; response: ExportResponse };
export function createMovementLedgerFlow(state: LedgerState, action: LedgerAction): LedgerState {
 switch (action.type) {
  case "draft_changed": return { ...state, draft: action.filters };
  case "product_query_changed": return { ...state, product_query: action.query, products_request: action.request_id, products: [], products_status: "initial", products_has_more: false };
  case "product_search_started": return { ...state, product_submitted_query: action.query, products_status: "loading", products: [], products_page: action.page, products_has_more: false, products_request: action.request_id };
  case "product_search_succeeded": if (action.request_id !== state.products_request) return state; return { ...state, products: action.response.products, products_page: action.response.page, products_has_more: action.response.has_more, products_status: action.response.products.length ? "ready" : "empty" };
  case "product_search_failed": return action.request_id === state.products_request ? { ...state, products: [], products_has_more: false, products_status: "error" } : state;
  case "product_selected": return { ...state, draft: { ...state.draft, product_id: action.product.product_id }, selected_product: action.product, product_query: "", products: [], products_status: "initial", products_request: action.request_id };
  case "product_cleared": return { ...state, draft: { ...state.draft, product_id: null }, selected_product: null, product_query: "", product_submitted_query: "", products: [], products_status: "initial", products_has_more: false, products_request: action.request_id };
  case "filters_applied": return { ...state, draft: actionFilters(state.draft), applied: actionFilters(state.draft), page: 1, rows: [], has_more: false, status: "loading", list_request: action.request_id, export_status: "idle" };
  case "page_requested": return { ...state, page: action.page, status: "loading", list_request: action.request_id, export_status: "idle" };
  case "list_succeeded": if (action.request_id !== state.list_request) return state; return { ...state, rows: action.response.rows, page: action.response.page, has_more: action.response.has_more, status: action.response.rows.length ? "ready" : "empty" };
  case "list_failed": return action.request_id === state.list_request ? { ...state, rows: [], has_more: false, status: "error" } : state;
  case "export_started": return { ...state, export_request: action.request_id, export_status: "pending" };
  case "export_finished": if (action.request_id !== state.export_request) return state; return { ...state, export_status: action.response.kind === "success" ? "success" : action.response.kind === "cancelled" ? "cancelled" : "error" };
 }
}
function actionFilters(filters: MovementLedgerFilters): MovementLedgerFilters { return { ...filters }; }
