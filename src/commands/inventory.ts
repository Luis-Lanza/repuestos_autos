const RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ALERT_CLASSIFICATION = { OUT_OF_STOCK: "out_of_stock", LOW_STOCK: "low_stock" } as const;
const ERROR_CODE = { INVALID_REQUEST: "invalid_request", INVALID_QUANTITY: "invalid_quantity", INVALID_COUNT: "invalid_count", INVALID_PRICE: "invalid_price", REASON_REQUIRED: "reason_required", MISSING_PRODUCT: "missing_product", INACTIVE_PRODUCT: "inactive_product", UNCHANGED_COUNT: "unchanged_count", QUANTITY_OVERFLOW: "quantity_overflow", PERSISTED_DATA_INVALID: "persisted_data_invalid", REQUEST_CONFLICT: "request_conflict", PERSISTENCE_FAILURE: "persistence_failure" } as const;
export interface StockEntryRequest { request_id: string; product_id: number; quantity: number; unit_purchase_price_centavos: number; sale_price_centavos?: number; minimum_sale_price_centavos?: number; note: string | null; }
export interface PhysicalCountRequest { request_id: string; product_id: number; count: number; reason: string; }
export interface PersistedInventoryOperation { request_id: string; product_id: number; previous_quantity: number; quantity_delta: number; resulting_quantity: number; occurred_at: string; note: string | null; }
export interface InventoryAlert { product_id: number; product_name: string; quantity: number; classification: (typeof ALERT_CLASSIFICATION)[keyof typeof ALERT_CLASSIFICATION]; }
export interface InventoryError { kind: typeof RESPONSE_KIND.ERROR; code: string; message: string; }
export type InventoryResponse = ({ kind: typeof RESPONSE_KIND.SUCCESS } & PersistedInventoryOperation) | InventoryError;
export type InventoryAlertsResponse = { kind: typeof RESPONSE_KIND.SUCCESS; alerts: InventoryAlert[] } | InventoryError;
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const failure = (): InventoryError => ({ kind: RESPONSE_KIND.ERROR, code: "persistence_failure", message: "The inventory operation could not be completed." });
const invalid = (code: "invalid_quantity" | "invalid_count" | "invalid_price"): InventoryError => ({ kind: RESPONSE_KIND.ERROR, code, message: "The inventory operation could not be completed." });
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null;
const responseRecord = (value: unknown): value is RecordValue => record(value) && !Array.isArray(value);
const safeInteger = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const alert = (value: unknown): InventoryAlert | null => responseRecord(value) && safeInteger(value.product_id) && typeof value.product_name === "string" && safeInteger(value.quantity) && (value.classification === ALERT_CLASSIFICATION.OUT_OF_STOCK || value.classification === ALERT_CLASSIFICATION.LOW_STOCK) ? { product_id: value.product_id, product_name: value.product_name, quantity: value.quantity, classification: value.classification } : null;
const alertError = (value: unknown): InventoryError => responseRecord(value) && typeof value.code === "string" && typeof value.message === "string" && Object.values(ERROR_CODE).includes(value.code as typeof ERROR_CODE[keyof typeof ERROR_CODE]) ? { kind: RESPONSE_KIND.ERROR, code: value.code, message: "The inventory operation could not be completed." } : failure();
const error = (value: RecordValue): InventoryError => typeof value.code === "string" && Object.values(ERROR_CODE).includes(value.code as typeof ERROR_CODE[keyof typeof ERROR_CODE]) ? { kind: RESPONSE_KIND.ERROR, code: value.code, message: "The inventory operation could not be completed." } : failure();
const operation = (value: unknown): InventoryResponse => record(value) && value.kind === RESPONSE_KIND.SUCCESS && ["request_id", "occurred_at"].every((key) => typeof value[key] === "string") && ["product_id", "previous_quantity", "quantity_delta", "resulting_quantity"].every((key) => typeof value[key] === "number") && (typeof value.note === "string" || value.note === null) ? { kind: RESPONSE_KIND.SUCCESS, request_id: value.request_id as string, product_id: value.product_id as number, previous_quantity: value.previous_quantity as number, quantity_delta: value.quantity_delta as number, resulting_quantity: value.resulting_quantity as number, occurred_at: value.occurred_at as string, note: value.note as string | null } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value) : failure();

export function createInventoryCommands(command: Invoke) {
  const confirm = (name: string, request: StockEntryRequest | PhysicalCountRequest) => ("quantity" in request && (!Number.isSafeInteger(request.quantity) || request.quantity <= 0)) || ("count" in request && (!Number.isSafeInteger(request.count) || request.count < 0)) ? Promise.resolve(invalid("quantity" in request ? "invalid_quantity" : "invalid_count")) : command(name, { request: { ...request } }).then(operation).catch(failure);
  const validPrice = (value: number | undefined) => value === undefined || (Number.isSafeInteger(value) && value > 0);
  return {
    confirmStockEntry: (request: StockEntryRequest) => {
      if (!Number.isSafeInteger(request.unit_purchase_price_centavos) || request.unit_purchase_price_centavos <= 0 || !validPrice(request.sale_price_centavos) || !validPrice(request.minimum_sale_price_centavos) || (request.sale_price_centavos !== undefined && request.minimum_sale_price_centavos !== undefined && request.minimum_sale_price_centavos > request.sale_price_centavos)) return Promise.resolve(invalid("invalid_price"));
      return confirm("confirm_stock_entry_command", { request_id: request.request_id, product_id: request.product_id, quantity: request.quantity, unit_purchase_price_centavos: request.unit_purchase_price_centavos, ...(request.sale_price_centavos === undefined ? {} : { sale_price_centavos: request.sale_price_centavos }), ...(request.minimum_sale_price_centavos === undefined ? {} : { minimum_sale_price_centavos: request.minimum_sale_price_centavos }), note: request.note });
    },
    confirmPhysicalCount: (request: PhysicalCountRequest) => confirm("confirm_physical_count_command", { request_id: request.request_id, product_id: request.product_id, count: request.count, reason: request.reason }),
    listAlerts: (): Promise<InventoryAlertsResponse> => command("list_inventory_alerts_command", {}).then((value) => responseRecord(value) && value.kind === "alerts" && Array.isArray(value.alerts) && value.alerts.every((item) => alert(item)) ? { kind: RESPONSE_KIND.SUCCESS, alerts: value.alerts.map((item) => alert(item) as InventoryAlert) } : responseRecord(value) && value.kind === RESPONSE_KIND.ERROR ? alertError(value) : failure()).catch(failure),
  };
}
const tauriInvoke: Invoke = async (command, payload) => (await import("@tauri-apps/api/core")).invoke(command, payload);
export const inventoryCommands = createInventoryCommands(tauriInvoke);
