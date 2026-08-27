const RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ALERT_CLASSIFICATION = { OUT_OF_STOCK: "out_of_stock", LOW_STOCK: "low_stock" } as const;
const ERROR_CODE = { INVALID_REQUEST: "invalid_request", INVALID_QUANTITY: "invalid_quantity", INVALID_COUNT: "invalid_count", REASON_REQUIRED: "reason_required", MISSING_PRODUCT: "missing_product", INACTIVE_PRODUCT: "inactive_product", UNCHANGED_COUNT: "unchanged_count", QUANTITY_OVERFLOW: "quantity_overflow", PERSISTED_DATA_INVALID: "persisted_data_invalid", PERSISTENCE_FAILURE: "persistence_failure" } as const;
export interface StockEntryRequest { request_id: string; product_id: number; quantity: number; note: string | null; }
export interface PhysicalCountRequest { request_id: string; product_id: number; count: number; reason: string; }
export interface PersistedInventoryOperation { request_id: string; product_id: number; previous_quantity: number; quantity_delta: number; resulting_quantity: number; occurred_at: string; note: string | null; }
export interface InventoryAlert { product_id: number; product_name: string; quantity: number; classification: (typeof ALERT_CLASSIFICATION)[keyof typeof ALERT_CLASSIFICATION]; }
export interface InventoryError { kind: typeof RESPONSE_KIND.ERROR; code: string; message: string; }
export type InventoryResponse = ({ kind: typeof RESPONSE_KIND.SUCCESS } & PersistedInventoryOperation) | InventoryError;
export type InventoryAlertsResponse = { kind: typeof RESPONSE_KIND.SUCCESS; alerts: InventoryAlert[] } | InventoryError;
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const failure = (): InventoryError => ({ kind: RESPONSE_KIND.ERROR, code: "persistence_failure", message: "The inventory operation could not be completed." });
const invalid = (code: "invalid_quantity" | "invalid_count"): InventoryError => ({ kind: RESPONSE_KIND.ERROR, code, message: "The inventory operation could not be completed." });
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null;
const error = (value: RecordValue): InventoryError => typeof value.code === "string" && Object.values(ERROR_CODE).includes(value.code as typeof ERROR_CODE[keyof typeof ERROR_CODE]) ? { kind: RESPONSE_KIND.ERROR, code: value.code, message: "The inventory operation could not be completed." } : failure();
const operation = (value: unknown): InventoryResponse => record(value) && value.kind === RESPONSE_KIND.SUCCESS && ["request_id", "occurred_at"].every((key) => typeof value[key] === "string") && ["product_id", "previous_quantity", "quantity_delta", "resulting_quantity"].every((key) => typeof value[key] === "number") && (typeof value.note === "string" || value.note === null) ? { kind: RESPONSE_KIND.SUCCESS, request_id: value.request_id as string, product_id: value.product_id as number, previous_quantity: value.previous_quantity as number, quantity_delta: value.quantity_delta as number, resulting_quantity: value.resulting_quantity as number, occurred_at: value.occurred_at as string, note: value.note as string | null } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value) : failure();

export function createInventoryCommands(command: Invoke) {
  const confirm = (name: string, request: StockEntryRequest | PhysicalCountRequest) => ("quantity" in request && (!Number.isSafeInteger(request.quantity) || request.quantity <= 0)) || ("count" in request && (!Number.isSafeInteger(request.count) || request.count < 0)) ? Promise.resolve(invalid("quantity" in request ? "invalid_quantity" : "invalid_count")) : command(name, { request: { ...request } }).then(operation).catch(failure);
  return {
    confirmStockEntry: (request: StockEntryRequest) => confirm("confirm_stock_entry_command", { request_id: request.request_id, product_id: request.product_id, quantity: request.quantity, note: request.note }),
    confirmPhysicalCount: (request: PhysicalCountRequest) => confirm("confirm_physical_count_command", { request_id: request.request_id, product_id: request.product_id, count: request.count, reason: request.reason }),
    listAlerts: (): Promise<InventoryAlertsResponse> => command("list_inventory_alerts_command", {}).then((value) => record(value) && value.kind === "alerts" && Array.isArray(value.alerts) ? { kind: RESPONSE_KIND.SUCCESS, alerts: value.alerts.filter(record).filter((alert): alert is RecordValue & { product_id: number; product_name: string; quantity: number; classification: InventoryAlert["classification"] } => typeof alert.product_id === "number" && typeof alert.product_name === "string" && typeof alert.quantity === "number" && (alert.classification === ALERT_CLASSIFICATION.OUT_OF_STOCK || alert.classification === ALERT_CLASSIFICATION.LOW_STOCK)).map((alert) => ({ product_id: alert.product_id, product_name: alert.product_name, quantity: alert.quantity, classification: alert.classification })) } : failure()).catch(failure),
  };
}
const tauriInvoke: Invoke = async (command, payload) => (await import("@tauri-apps/api/core")).invoke(command, payload);
export const inventoryCommands = createInventoryCommands(tauriInvoke);
