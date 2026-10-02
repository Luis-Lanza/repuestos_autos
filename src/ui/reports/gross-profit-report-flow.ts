import type { GrossProfitReport } from "../../commands/gross-profit.ts";
import type { GrossProfitOperationsPage } from "../../commands/gross-profit-operations.ts";

export interface Period { from: string; to: string; }
export interface GrossProfitReportState {
  draft: Period;
  applied: Period;
  range_id: number;
  operations_request_id: number;
  summary_status: "loading" | "ready" | "empty" | "error";
  operations_status: "loading" | "ready" | "empty" | "error";
  report: GrossProfitReport | null;
  operations: GrossProfitOperationsPage | null;
  requested_page: number;
}
export type GrossProfitReportAction =
  | { type: "draft_changed"; period: Period }
  | { type: "range_applied"; period: Period; range_id: number; operations_request_id: number }
  | { type: "retry"; operations_request_id: number }
  | { type: "page_requested"; page: number; operations_request_id: number }
  | { type: "summary_succeeded"; range_id: number; report: GrossProfitReport }
  | { type: "summary_failed"; range_id: number }
  | { type: "operations_succeeded"; range_id: number; operations_request_id: number; report: GrossProfitOperationsPage }
  | { type: "operations_failed"; range_id: number; operations_request_id: number };

export function initialGrossProfitReportState(period: Period): GrossProfitReportState {
  return { draft: period, applied: period, range_id: 0, operations_request_id: 0, summary_status: "loading", operations_status: "loading", report: null, operations: null, requested_page: 1 };
}
export function createGrossProfitReportFlow(state: GrossProfitReportState, action: GrossProfitReportAction): GrossProfitReportState {
  switch (action.type) {
    case "draft_changed": return { ...state, draft: action.period };
    case "range_applied": return { ...state, applied: action.period, range_id: action.range_id, operations_request_id: action.operations_request_id, summary_status: "loading", operations_status: "loading", report: null, operations: null, requested_page: 1 };
    case "retry": if (action.operations_request_id <= state.operations_request_id) return state; return { ...state, operations_request_id: action.operations_request_id, summary_status: "loading", operations_status: "loading", report: null, operations: null, requested_page: 1 };
    case "page_requested": if (state.operations_status !== "ready" || !state.operations || action.page < 1 || action.page > state.operations.total_pages || action.operations_request_id <= state.operations_request_id) return state; return { ...state, operations_request_id: action.operations_request_id, operations_status: "loading", requested_page: action.page };
    case "summary_succeeded": if (action.range_id !== state.range_id) return state; return { ...state, report: action.report, summary_status: action.report.activity_count === 0 ? "empty" : "ready" };
    case "summary_failed": return action.range_id === state.range_id ? { ...state, report: null, summary_status: "error" } : state;
    case "operations_succeeded": if (action.range_id !== state.range_id || action.operations_request_id !== state.operations_request_id || action.report.page !== state.requested_page) return state; return { ...state, operations: action.report, operations_status: action.report.total === 0 ? "empty" : "ready" };
    case "operations_failed": return action.range_id === state.range_id && action.operations_request_id === state.operations_request_id ? { ...state, operations: null, operations_status: "error" } : state;
  }
}
