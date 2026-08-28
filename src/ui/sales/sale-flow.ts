import type { ProductSearchResult } from "../../commands/catalog.ts";
import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";

export type DraftLine = {
  product_id: number;
  sku: string;
  product_name: string;
  quantity: number;
  captured_unit_price_centavos: number;
  captured_revision: number;
  acknowledged_price_centavos?: number;
  acknowledged_revision?: number;
};

export type DraftPayment = {
  amount_tendered_centavos: string;
  qr_applied_centavos: string;
};

export type SaleState = {
  search_results: ProductSearchResult[];
  lines: DraftLine[];
  payment: DraftPayment;
  feedback: string | null;
  request_id: string | null;
  confirmation: "idle" | "pending" | "error" | "confirmed";
  persisted_summary: PersistedSaleSummary | null;
  stale_price: { product_id: number; current_unit_price_centavos: number; current_revision: number } | null;
};

export const initialSaleState: SaleState = {
  search_results: [],
  lines: [],
  payment: { amount_tendered_centavos: "", qr_applied_centavos: "" },
  feedback: null,
  request_id: null,
  confirmation: "idle",
  persisted_summary: null,
  stale_price: null,
};

export type SaleAction =
  | { type: "search_succeeded"; results: ProductSearchResult[] }
  | { type: "add_product"; product: ProductSearchResult }
  | { type: "remove_product"; product_id: number }
  | { type: "line_quantity_changed"; product_id: number; value: string }
  | {
      type: "payment_changed";
      field: keyof DraftPayment;
      value: string;
    }
  | { type: "confirmation_started"; request_id: string }
  | { type: "confirmation_succeeded"; summary: PersistedSaleSummary }
  | { type: "confirmation_failed"; message: string }
  | { type: "stale_price_detected"; product_id: number; current_unit_price_centavos: number; current_revision: number }
  | { type: "acknowledge_stale_price"; product_id: number; current_unit_price_centavos: number; current_revision: number }
  | { type: "discard" };

function positiveWhole(value: string): number | null {
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : null;
}

export function createSaleFlow(
  state: SaleState,
  action: SaleAction,
): SaleState {
  switch (action.type) {
    case "search_succeeded":
      return { ...state, search_results: action.results, feedback: null };
    case "add_product":
      if (
        action.product.available_quantity < 1 ||
        state.lines.some(
          (line) => line.product_id === action.product.product_id,
        )
      )
        return state;
      return {
        ...state,
        lines: [
          ...state.lines,
          {
            product_id: action.product.product_id,
            sku: action.product.sku,
            product_name: action.product.name,
            quantity: 1,
            captured_unit_price_centavos: action.product.catalog_unit_price_centavos,
            captured_revision: action.product.revision,
          },
        ],
        feedback: null,
      };
    case "remove_product":
      return {
        ...state,
        lines: state.lines.filter(
          (line) => line.product_id !== action.product_id,
        ),
        feedback: null,
      };
    case "line_quantity_changed": {
      const quantity = positiveWhole(action.value);
      if (quantity === null)
        return {
          ...state,
          feedback: "Quantity must be a positive whole number.",
        };
      return {
        ...state,
        lines: state.lines.map((line) =>
          line.product_id === action.product_id ? { ...line, quantity } : line,
        ),
        feedback: null,
      };
    }
    case "payment_changed":
      return {
        ...state,
        payment: { ...state.payment, [action.field]: action.value },
        feedback: null,
      };
    case "confirmation_started":
      return {
        ...state,
        request_id: state.request_id ?? action.request_id,
        confirmation: "pending",
        feedback: null,
      };
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
    case "stale_price_detected":
      return { ...state, confirmation: "error", stale_price: action, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, acknowledged_price_centavos: undefined, acknowledged_revision: undefined } : line), feedback: "Catalog price changed. Review and acknowledge the current price." };
    case "acknowledge_stale_price":
      if (state.stale_price?.product_id !== action.product_id || state.stale_price.current_unit_price_centavos !== action.current_unit_price_centavos || state.stale_price.current_revision !== action.current_revision) return state;
      return { ...state, confirmation: "idle", stale_price: null, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, acknowledged_price_centavos: action.current_unit_price_centavos, acknowledged_revision: action.current_revision } : line), feedback: "Current catalog price acknowledged. Confirm again to continue." };
    case "discard":
      return initialSaleState;
  }
}
