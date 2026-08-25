import { invoke } from "@tauri-apps/api/core";

const CONFIRM_SALE_RESPONSE_KIND = {
  SUCCESS: "success",
  ERROR: "error",
} as const;

const PAYMENT_METHOD = {
  CASH: "cash",
  QR: "qr",
} as const;

export interface ConfirmSaleLineRequest {
  product_id: number;
  quantity: number;
}

export interface ConfirmSalePaymentInput {
  amount_tendered_centavos: number | null;
  qr_applied_centavos: number | null;
}

export interface ConfirmSaleRequest {
  request_id: string;
  lines: ConfirmSaleLineRequest[];
  payment: ConfirmSalePaymentInput;
}

export interface PersistedSaleLine {
  product_id: number;
  sku: string;
  product_name: string;
  quantity: number;
  unit_price_centavos: number;
  line_total_centavos: number;
}

export interface CashPayment {
  method: typeof PAYMENT_METHOD.CASH;
  amount_applied_centavos: number;
  amount_tendered_centavos: number;
  change_given_centavos: number;
}

export interface QrPayment {
  method: typeof PAYMENT_METHOD.QR;
  amount_applied_centavos: number;
}

export interface PersistedSaleSummary {
  sale_id: number;
  request_id: string;
  status: "confirmed";
  confirmed_at: string;
  outcome: "confirmed";
  lines: PersistedSaleLine[];
  payments: Array<CashPayment | QrPayment>;
  total_centavos: number;
}

export type ConfirmSaleResponse =
  | ({ kind: typeof CONFIRM_SALE_RESPONSE_KIND.SUCCESS } & PersistedSaleSummary)
  | {
      kind: typeof CONFIRM_SALE_RESPONSE_KIND.ERROR;
      code: string;
      message: string;
    };

type Invoke = (command: string, payload: unknown) => Promise<unknown>;

function assertPositiveSafeInteger(value: number, field: string): void {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${field} must be a positive safe integer.`);
  }
}

function assertNonNegativeSafeInteger(value: number, field: string): void {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${field} must be a non-negative safe integer.`);
  }
}

function assertIntegerRequest(request: ConfirmSaleRequest): void {
  for (const line of request.lines) {
    assertPositiveSafeInteger(line.product_id, "Product ID");
    assertPositiveSafeInteger(line.quantity, "Quantity");
  }
  if (request.payment.amount_tendered_centavos !== null) {
    assertNonNegativeSafeInteger(
      request.payment.amount_tendered_centavos,
      "Tendered cash",
    );
  }
  if (request.payment.qr_applied_centavos !== null) {
    assertNonNegativeSafeInteger(
      request.payment.qr_applied_centavos,
      "QR amount",
    );
  }
}

export function createConfirmSaleCommand(command: Invoke) {
  return async (request: ConfirmSaleRequest): Promise<ConfirmSaleResponse> => {
    assertIntegerRequest(request);
    return command("confirm_sale_command", {
      request,
    }) as Promise<ConfirmSaleResponse>;
  };
}

export const confirmSale = createConfirmSaleCommand(invoke as Invoke);
