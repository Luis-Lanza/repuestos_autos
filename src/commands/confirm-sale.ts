import { invoke } from "@tauri-apps/api/core";

export type ConfirmSaleRequest = {
  request_id: string;
  lines: Array<{
    product_id: number;
    quantity: number;
    negotiated_unit_price_centavos: number;
  }>;
  payments: Array<
    | {
        method: "cash";
        amount_applied_centavos: number;
        amount_tendered_centavos: number;
        change_given_centavos: number;
      }
    | { method: "qr"; amount_applied_centavos: number }
  >;
};

export type PersistedSaleSummary = {
  sale_id: number;
  request_id: string;
  status: "confirmed";
  confirmed_at: string;
  outcome: "confirmed";
  lines: Array<{
    product_id: number;
    sku: string;
    product_name: string;
    quantity: number;
    negotiated_unit_price_centavos: number;
    minimum_unit_price_snapshot_centavos: number;
    line_total_centavos: number;
  }>;
  payments: ConfirmSaleRequest["payments"];
  total_centavos: number;
};

export type ConfirmSaleResponse =
  | { kind: "success"; [key: string]: unknown; sale_id: number; request_id: string; status: "confirmed"; confirmed_at: string; outcome: "confirmed"; lines: PersistedSaleSummary["lines"]; payments: PersistedSaleSummary["payments"]; total_centavos: number }
  | { kind: "error"; code: string; message: string };

type Invoke = (command: string, payload: unknown) => Promise<unknown>;

function assertSafeInteger(value: number): void {
  if (!Number.isSafeInteger(value)) throw new Error("Confirmation payload values must be safe integers.");
}

function assertIntegerRequest(request: ConfirmSaleRequest): void {
  for (const line of request.lines) {
    assertSafeInteger(line.product_id);
    assertSafeInteger(line.quantity);
    assertSafeInteger(line.negotiated_unit_price_centavos);
  }
  for (const payment of request.payments) {
    assertSafeInteger(payment.amount_applied_centavos);
    if (payment.method === "cash") {
      assertSafeInteger(payment.amount_tendered_centavos);
      assertSafeInteger(payment.change_given_centavos);
    }
  }
}

export function createConfirmSaleCommand(command: Invoke) {
  return async (request: ConfirmSaleRequest) => {
    assertIntegerRequest(request);
    return command("confirm_sale_command", { request }) as Promise<ConfirmSaleResponse>;
  };
}

export const confirmSale = createConfirmSaleCommand(invoke as Invoke);
