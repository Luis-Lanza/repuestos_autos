import type { DashboardReport } from "../../commands/dashboard.ts";

export type DashboardState =
  | { status: "loading"; request_id: number; report: null }
  | { status: "ready"; request_id: number; report: DashboardReport }
  | { status: "error"; request_id: number; report: null };
export type DashboardAction =
  | { type: "load_started"; request_id: number }
  | { type: "load_succeeded"; request_id: number; report: DashboardReport }
  | { type: "load_failed"; request_id: number };
export const initialDashboardState: DashboardState = { status: "loading", request_id: 0, report: null };
export function createDashboardFlow(state: DashboardState, action: DashboardAction): DashboardState {
  if (action.type === "load_started") return { status: "loading", request_id: action.request_id, report: null };
  if (action.request_id !== state.request_id) return state;
  return action.type === "load_succeeded" ? { status: "ready", request_id: action.request_id, report: action.report } : { status: "error", request_id: action.request_id, report: null };
}
