import { invoke } from "@tauri-apps/api/core";

const CONFIRM_SALE_RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const PAYMENT_METHOD = { CASH: "cash", QR: "qr" } as const;

export interface ConfirmSaleLineRequest {
  product_id: number;
  quantity: number;
  captured_unit_price_centavos: number;
  captured_revision: number;
  /** Explicit negotiated-price route. Omitted only for legacy callers. */
  final_unit_price_centavos?: number;
  /** Legacy stale-price acknowledgement, retained only for compatibility. */
  acknowledged_price_centavos?: number;
  acknowledged_revision?: number;
}

export interface ConfirmSalePaymentInput { amount_tendered_centavos: number | null; qr_applied_centavos: number | null; }
export interface ConfirmSaleRequest { request_id: string; lines: ConfirmSaleLineRequest[]; payment: ConfirmSalePaymentInput; }

export interface PersistedSaleLine {
  product_id: number; sku: string; product_name: string; quantity: number;
  /** The charged final price, never the list-price reference. */
  unit_price_centavos: number;
  minimum_unit_price_snapshot_centavos?: number;
  list_price_snapshot_centavos?: number | null;
  line_total_centavos: number;
}
export interface CashPayment { method: typeof PAYMENT_METHOD.CASH; amount_applied_centavos: number; amount_tendered_centavos: number; change_given_centavos: number; }
export interface QrPayment { method: typeof PAYMENT_METHOD.QR; amount_applied_centavos: number; }
export interface PersistedSaleSummary { sale_id: number; request_id: string; status: "confirmed"; confirmed_at: string; outcome: "confirmed"; lines: PersistedSaleLine[]; payments: Array<CashPayment | QrPayment>; total_centavos: number; }
export type ConfirmSaleResponse =
  | ({ kind: typeof CONFIRM_SALE_RESPONSE_KIND.SUCCESS } & PersistedSaleSummary)
  | { kind: "stale_catalog_record"; product_id: number; current_unit_price_centavos: number; current_revision: number }
  | { kind: "minimum_price_violation"; product_id: number; current_minimum_unit_price_centavos: number }
  | { kind: typeof CONFIRM_SALE_RESPONSE_KIND.ERROR; code: string; message: string };

type Invoke = (command: string, payload: unknown) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null && !Array.isArray(value);
const safeInteger = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const positiveSafeInteger = (value: unknown): value is number => safeInteger(value) && value > 0;
const nonNegativeSafeInteger = (value: unknown): value is number => safeInteger(value) && value >= 0;
const saleErrorMessages: Record<string, string> = {
  invalid_request: "The request shape is invalid.", invalid_quantity: "Quantity must be a positive whole number.", invalid_payment: "Payment values are invalid.", invalid_final_price: "The final price must be positive.",
  inactive_product: "The product is inactive.", missing_product: "The product was not found.", minimum_price_violation: "The final price is below the current minimum price.", insufficient_stock: "Insufficient stock is available.", request_conflict: "The request ID was already used with different sale data.", persistence_failure: "The sale could not be persisted.",
};
const genericSaleError = (): ConfirmSaleResponse => ({ kind: "error", code: "persistence_failure", message: saleErrorMessages.persistence_failure });
const decodedArray = <T>(value: unknown, decode: (item: unknown) => T | null): T[] | null => {
  if (!Array.isArray(value)) return null;
  const decoded = value.map(decode);
  return decoded.every((item): item is T => item !== null) ? decoded : null;
};
const saleLine = (value: unknown): PersistedSaleLine | null => {
  if (!record(value) || !positiveSafeInteger(value.product_id) || typeof value.sku !== "string" || typeof value.product_name !== "string" || !positiveSafeInteger(value.quantity) || !positiveSafeInteger(value.unit_price_centavos) || !positiveSafeInteger(value.line_total_centavos)) return null;
  if (Object.hasOwn(value, "minimum_unit_price_snapshot_centavos") && (!safeInteger(value.minimum_unit_price_snapshot_centavos) || value.minimum_unit_price_snapshot_centavos <= 0)) return null;
  if (Object.hasOwn(value, "list_price_snapshot_centavos") && value.list_price_snapshot_centavos !== null && (!safeInteger(value.list_price_snapshot_centavos) || value.list_price_snapshot_centavos <= 0)) return null;
  return {
    product_id: value.product_id, sku: value.sku, product_name: value.product_name, quantity: value.quantity,
    unit_price_centavos: value.unit_price_centavos,
    ...(Object.hasOwn(value, "minimum_unit_price_snapshot_centavos") ? { minimum_unit_price_snapshot_centavos: value.minimum_unit_price_snapshot_centavos as number } : {}),
    ...(Object.hasOwn(value, "list_price_snapshot_centavos") ? { list_price_snapshot_centavos: value.list_price_snapshot_centavos as number | null } : {}),
    line_total_centavos: value.line_total_centavos,
  };
};
const payment = (value: unknown): CashPayment | QrPayment | null => record(value) && value.method === PAYMENT_METHOD.CASH && positiveSafeInteger(value.amount_applied_centavos) && positiveSafeInteger(value.amount_tendered_centavos) && nonNegativeSafeInteger(value.change_given_centavos) ? { method: PAYMENT_METHOD.CASH, amount_applied_centavos: value.amount_applied_centavos, amount_tendered_centavos: value.amount_tendered_centavos, change_given_centavos: value.change_given_centavos } : record(value) && value.method === PAYMENT_METHOD.QR && positiveSafeInteger(value.amount_applied_centavos) ? { method: PAYMENT_METHOD.QR, amount_applied_centavos: value.amount_applied_centavos } : null;
const saleError = (value: unknown): ConfirmSaleResponse | null => record(value) && typeof value.code === "string" && typeof value.message === "string" && Object.hasOwn(saleErrorMessages, value.code) ? { kind: CONFIRM_SALE_RESPONSE_KIND.ERROR, code: value.code, message: saleErrorMessages[value.code] } : null;
const saleResponse = (value: unknown): ConfirmSaleResponse | null => {
  if (!record(value)) return null;
  if (value.kind === CONFIRM_SALE_RESPONSE_KIND.ERROR) return saleError(value);
  if (value.kind === "stale_catalog_record" && positiveSafeInteger(value.product_id) && positiveSafeInteger(value.current_unit_price_centavos) && nonNegativeSafeInteger(value.current_revision)) return { kind: "stale_catalog_record", product_id: value.product_id, current_unit_price_centavos: value.current_unit_price_centavos, current_revision: value.current_revision };
  if (value.kind === "minimum_price_violation" && positiveSafeInteger(value.product_id) && positiveSafeInteger(value.current_minimum_unit_price_centavos)) return { kind: "minimum_price_violation", product_id: value.product_id, current_minimum_unit_price_centavos: value.current_minimum_unit_price_centavos };
  if (value.kind === CONFIRM_SALE_RESPONSE_KIND.SUCCESS && positiveSafeInteger(value.sale_id) && typeof value.request_id === "string" && value.status === "confirmed" && typeof value.confirmed_at === "string" && value.outcome === "confirmed" && positiveSafeInteger(value.total_centavos)) {
    const lines = decodedArray(value.lines, saleLine); const payments = decodedArray(value.payments, payment);
    return lines && payments ? { kind: CONFIRM_SALE_RESPONSE_KIND.SUCCESS, sale_id: value.sale_id, request_id: value.request_id, status: "confirmed", confirmed_at: value.confirmed_at, outcome: "confirmed", lines, payments, total_centavos: value.total_centavos } : null;
  }
  return null;
};
function assertPositiveSafeInteger(value: number, field: string): void { if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`${field} must be a positive safe integer.`); }
function assertNonNegativeSafeInteger(value: number, field: string): void { if (!Number.isSafeInteger(value) || value < 0) throw new Error(`${field} must be a non-negative safe integer.`); }
const CANONICAL_UUID_V4 = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
function assertRequestId(requestId: string): void { if (!CANONICAL_UUID_V4.test(requestId)) throw new Error("Request ID must be a canonical UUID v4."); }
function assertIntegerRequest(request: ConfirmSaleRequest): void {
  assertRequestId(request.request_id);
  for (const line of request.lines) {
    assertPositiveSafeInteger(line.product_id, "Product ID"); assertPositiveSafeInteger(line.quantity, "Quantity"); assertPositiveSafeInteger(line.captured_unit_price_centavos, "Captured price"); assertNonNegativeSafeInteger(line.captured_revision, "Captured revision");
    if (line.final_unit_price_centavos !== undefined) assertPositiveSafeInteger(line.final_unit_price_centavos, "Final price");
    if ((line.acknowledged_price_centavos === undefined) !== (line.acknowledged_revision === undefined)) throw new Error("Price acknowledgement must include its revision.");
    if (line.acknowledged_price_centavos !== undefined) assertPositiveSafeInteger(line.acknowledged_price_centavos, "Acknowledged price");
    if (line.acknowledged_revision !== undefined) assertNonNegativeSafeInteger(line.acknowledged_revision, "Acknowledged revision");
  }
  if (request.payment.amount_tendered_centavos !== null) assertNonNegativeSafeInteger(request.payment.amount_tendered_centavos, "Tendered cash");
  if (request.payment.qr_applied_centavos !== null) assertNonNegativeSafeInteger(request.payment.qr_applied_centavos, "QR amount");
}
export function createConfirmSaleCommand(command: Invoke) {
  return async (request: ConfirmSaleRequest): Promise<ConfirmSaleResponse> => {
    assertIntegerRequest(request);
    try {
      const value = await command("confirm_sale_command", { request: {
        request_id: request.request_id,
        lines: request.lines.map(({ product_id, quantity, captured_unit_price_centavos, captured_revision, final_unit_price_centavos, acknowledged_price_centavos, acknowledged_revision }) => ({ product_id, quantity, captured_unit_price_centavos, captured_revision, ...(final_unit_price_centavos === undefined ? {} : { final_unit_price_centavos }), ...(acknowledged_price_centavos === undefined ? {} : { acknowledged_price_centavos, acknowledged_revision }) })),
        payment: { amount_tendered_centavos: request.payment.amount_tendered_centavos, qr_applied_centavos: request.payment.qr_applied_centavos },
      } });
      return saleResponse(value) ?? genericSaleError();
    } catch { return genericSaleError(); }
  };
}
export const confirmSale = createConfirmSaleCommand(invoke as Invoke);
