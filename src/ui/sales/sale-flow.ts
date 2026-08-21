import type { ProductSearchResult } from "../../commands/catalog.ts";
import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";

export type DraftLine = {
  product_id: number;
  sku: string;
  product_name: string;
  quantity: number;
  negotiated_unit_price_centavos: number;
  minimum_unit_price_centavos: number;
};

export type DraftPayment =
  | {
      method: "cash";
      amount_applied_centavos: string;
      amount_tendered_centavos: string;
      change_given_centavos: string;
    }
  | { method: "qr"; amount_applied_centavos: string };

export type SaleState = {
  search_results: ProductSearchResult[];
  lines: DraftLine[];
  payments: DraftPayment[];
  feedback: string | null;
  request_id: string | null;
  confirmation: "idle" | "pending" | "error" | "confirmed";
  persisted_summary: PersistedSaleSummary | null;
};

export const initialSaleState: SaleState = {
  search_results: [],
  lines: [],
  payments: [],
  feedback: null,
  request_id: null,
  confirmation: "idle",
  persisted_summary: null,
};

export type SaleAction =
  | { type: "search_succeeded"; results: ProductSearchResult[] }
  | { type: "add_product"; product: ProductSearchResult }
  | { type: "remove_product"; product_id: number }
  | { type: "line_quantity_changed"; product_id: number; value: string }
  | { type: "line_price_changed"; product_id: number; value: string }
  | { type: "cash_payment_changed"; amount_applied_centavos: string; amount_tendered_centavos: string; change_given_centavos: string }
  | { type: "qr_payment_changed"; amount_applied_centavos: string }
  | { type: "confirmation_started"; request_id: string }
  | { type: "confirmation_succeeded"; summary: PersistedSaleSummary }
  | { type: "confirmation_failed"; message: string }
  | { type: "discard" };

function positiveWhole(value: string): number | null {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : null;
}

function centavos(value: string): number | null {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed >= 0 ? parsed : null;
}

export function createSaleFlow(state: SaleState, action: SaleAction): SaleState {
  switch (action.type) {
    case "search_succeeded":
      return { ...state, search_results: action.results, feedback: null };
    case "add_product":
      if (action.product.available_quantity < 1 || state.lines.some((line) => line.product_id === action.product.product_id)) return state;
      return {
        ...state,
        lines: [...state.lines, {
          product_id: action.product.product_id,
          sku: action.product.sku,
          product_name: action.product.name,
          quantity: 1,
          negotiated_unit_price_centavos: action.product.minimum_unit_price_centavos,
          minimum_unit_price_centavos: action.product.minimum_unit_price_centavos,
        }],
        feedback: null,
      };
    case "remove_product":
      return { ...state, lines: state.lines.filter((line) => line.product_id !== action.product_id), feedback: null };
    case "line_quantity_changed": {
      const quantity = positiveWhole(action.value);
      if (quantity === null) return { ...state, feedback: "Quantity must be a positive whole number." };
      return { ...state, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, quantity } : line), feedback: null };
    }
    case "line_price_changed": {
      const price = centavos(action.value);
      if (price === null) return { ...state, feedback: "Price must be a non-negative whole number of centavos." };
      return { ...state, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, negotiated_unit_price_centavos: price } : line), feedback: null };
    }
    case "cash_payment_changed":
      return {
        ...state,
        payments: [{
          method: "cash",
          amount_applied_centavos: action.amount_applied_centavos,
          amount_tendered_centavos: action.amount_tendered_centavos,
          change_given_centavos: action.change_given_centavos,
        }],
        feedback: null,
      };
    case "qr_payment_changed": {
      const cash = state.payments.find((payment) => payment.method === "cash");
      return { ...state, payments: [...(cash ? [cash] : []), { method: "qr", amount_applied_centavos: action.amount_applied_centavos }], feedback: null };
    }
    case "confirmation_started":
      return { ...state, request_id: state.request_id ?? action.request_id, confirmation: "pending", feedback: null };
    case "confirmation_succeeded":
      return {
        ...state,
        request_id: action.summary.request_id,
        confirmation: "confirmed",
        persisted_summary: action.summary,
        feedback: null,
      };
    case "confirmation_failed":
      return { ...state, confirmation: "error", feedback: action.message };
    case "discard":
      return initialSaleState;
  }
}
