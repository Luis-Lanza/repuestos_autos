import type { InventoryAlert, PersistedInventoryOperation } from "../../commands/inventory.ts";

const CONFIRMATION = { IDLE: "idle", PENDING: "pending", ERROR: "error", CONFIRMED: "confirmed" } as const;
const OPERATION = { STOCK_ENTRY: "stock_entry", PHYSICAL_COUNT: "physical_count" } as const;
export interface InventoryProduct { product_id: number; name: string; available_quantity: number; }
export interface InventoryState { product: InventoryProduct | null; operation: (typeof OPERATION)[keyof typeof OPERATION]; entry_quantity: string; physical_count: string; note: string; reason: string; request_id: string | null; confirmation: (typeof CONFIRMATION)[keyof typeof CONFIRMATION]; result: PersistedInventoryOperation | null; feedback: string | null; advisory_notice: string | null; alerts: InventoryAlert[]; }
export const initialInventoryState: InventoryState = { product: null, operation: OPERATION.STOCK_ENTRY, entry_quantity: "", physical_count: "", note: "", reason: "", request_id: null, confirmation: CONFIRMATION.IDLE, result: null, feedback: null, advisory_notice: null, alerts: [] };
export type InventoryAction =
  | { type: "product_selected"; product: InventoryProduct }
  | { type: "operation_changed"; operation: InventoryState["operation"] }
  | { type: "entry_quantity_changed"; value: string }
  | { type: "physical_count_changed"; value: string }
  | { type: "note_changed"; value: string }
  | { type: "reason_changed"; value: string }
  | { type: "confirmation_started"; request_id: string }
  | { type: "confirmation_failed"; message: string }
  | { type: "confirmation_succeeded"; result: PersistedInventoryOperation }
  | { type: "alerts_refreshed"; alerts: InventoryAlert[] }
  | { type: "discard" };
const resetIntent = (state: InventoryState) => ({ ...state, request_id: null, confirmation: CONFIRMATION.IDLE, result: null, feedback: null, advisory_notice: null });
export function createInventoryFlow(state: InventoryState, action: InventoryAction): InventoryState {
  switch (action.type) {
    case "product_selected": return { ...resetIntent(state), product: action.product };
    case "operation_changed": return { ...resetIntent(state), operation: action.operation };
    case "entry_quantity_changed": return { ...resetIntent(state), entry_quantity: action.value };
    case "physical_count_changed": return { ...resetIntent(state), physical_count: action.value };
    case "note_changed": return { ...state, note: action.value };
    case "reason_changed": return { ...state, reason: action.value };
    case "confirmation_started": return { ...state, request_id: state.request_id ?? action.request_id, confirmation: CONFIRMATION.PENDING, feedback: null };
    case "confirmation_failed": return { ...state, confirmation: CONFIRMATION.ERROR, feedback: action.message };
    case "confirmation_succeeded": return { ...state, request_id: null, confirmation: CONFIRMATION.CONFIRMED, result: action.result, feedback: null, advisory_notice: state.product?.available_quantity === action.result.previous_quantity ? null : "Stock changed after the preview." };
    case "alerts_refreshed": return { ...state, alerts: action.alerts };
    case "discard": return initialInventoryState;
  }
}

export function projectedBalance(state: InventoryState): number | null {
  if (!state.product) return null;
  const value = Number(state.operation === OPERATION.STOCK_ENTRY ? state.entry_quantity : state.physical_count);
  return Number.isSafeInteger(value) && value >= 0 ? state.operation === OPERATION.STOCK_ENTRY ? state.product.available_quantity + value : value : null;
}
