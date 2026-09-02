import type { ProductSearchResult } from "../../commands/catalog.ts";
import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";

const INVALID_BS_CORRECTION =
  "Ingresá un monto válido en Bs, con hasta dos decimales.";

function checkedNonNegativeInteger(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0) throw new RangeError(INVALID_BS_CORRECTION);
  return value;
}

export function formatBs(integerCentavos: number): string {
  const centavos = checkedNonNegativeInteger(integerCentavos);
  const whole = Math.floor(centavos / 100);
  const fraction = String(centavos % 100).padStart(2, "0");
  return `Bs ${whole},${fraction}`;
}

export function parseOptionalBs(value: string): number | null {
  if (value === "") return null;
  const match = /^(\d+)(?:,(\d{1,2}))?$/.exec(value);
  if (!match) throw new RangeError(INVALID_BS_CORRECTION);

  const centavos = BigInt(match[1]) * 100n + BigInt((match[2] ?? "").padEnd(2, "0"));
  if (centavos > BigInt(Number.MAX_SAFE_INTEGER))
    throw new RangeError(INVALID_BS_CORRECTION);
  return Number(centavos);
}

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

const UNSAFE_DRAFT_TOTAL = "Draft money must remain within the safe integer range.";

function checkedDraftCentavos(value: number): number {
  if (!Number.isSafeInteger(value) || value < 0) throw new RangeError(UNSAFE_DRAFT_TOTAL);
  return value;
}

export function effectiveDraftUnitPriceCentavos(line: DraftLine): number {
  return checkedDraftCentavos(
    line.acknowledged_price_centavos ?? line.captured_unit_price_centavos,
  );
}

export function draftLineSubtotalCentavos(line: DraftLine): number {
  if (!Number.isSafeInteger(line.quantity) || line.quantity < 1)
    throw new RangeError(UNSAFE_DRAFT_TOTAL);
  const subtotal = BigInt(effectiveDraftUnitPriceCentavos(line)) * BigInt(line.quantity);
  if (subtotal > BigInt(Number.MAX_SAFE_INTEGER)) throw new RangeError(UNSAFE_DRAFT_TOTAL);
  return Number(subtotal);
}

export function draftTotalCentavos(lines: readonly DraftLine[]): number {
  let total = 0n;
  for (const line of lines) {
    total += BigInt(draftLineSubtotalCentavos(line));
    if (total > BigInt(Number.MAX_SAFE_INTEGER)) throw new RangeError(UNSAFE_DRAFT_TOTAL);
  }
  return Number(total);
}

export type DraftPayment = {
  amount_tendered_centavos: string;
  qr_applied_centavos: string;
};

export type CatalogDiscoveryState = {
  status: "initial" | "loading" | "results" | "empty" | "error";
  query: string;
  request_id: number;
  results: ProductSearchResult[];
  error: string | null;
};

export type SaleState = {
  search_results: ProductSearchResult[];
  catalog_discovery: CatalogDiscoveryState;
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
  catalog_discovery: {
    status: "initial",
    query: "",
    request_id: 0,
    results: [],
    error: null,
  },
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
  | { type: "catalog_search_started"; query: string; request_id: number }
  | { type: "catalog_search_succeeded"; request_id: number; results: ProductSearchResult[] }
  | { type: "catalog_search_failed"; request_id: number; message: string }
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
      return {
        ...state,
        search_results: action.results,
        catalog_discovery: {
          ...state.catalog_discovery,
          status: action.results.length === 0 ? "empty" : "results",
          results: action.results,
          error: null,
        },
        feedback: null,
      };
    case "catalog_search_started":
      if (
        !Number.isSafeInteger(action.request_id) ||
        action.request_id <= state.catalog_discovery.request_id
      )
        return state;
      return {
        ...state,
        catalog_discovery: {
          status: "loading",
          query: action.query,
          request_id: action.request_id,
          results: [],
          error: null,
        },
      };
    case "catalog_search_succeeded":
      if (action.request_id !== state.catalog_discovery.request_id) return state;
      return {
        ...state,
        search_results: action.results,
        catalog_discovery: {
          ...state.catalog_discovery,
          status: action.results.length === 0 ? "empty" : "results",
          results: action.results,
          error: null,
        },
      };
    case "catalog_search_failed":
      if (action.request_id !== state.catalog_discovery.request_id) return state;
      return {
        ...state,
        catalog_discovery: {
          ...state.catalog_discovery,
          status: "error",
          results: [],
          error: action.message,
        },
      };
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
    case "remove_product": {
      const removesStaleLine = state.stale_price?.product_id === action.product_id;
      return {
        ...state,
        lines: state.lines.filter(
          (line) => line.product_id !== action.product_id,
        ),
        confirmation: removesStaleLine ? "idle" : state.confirmation,
        stale_price: removesStaleLine ? null : state.stale_price,
        feedback: null,
      };
    }
    case "line_quantity_changed": {
      const quantity = positiveWhole(action.value);
      if (quantity === null)
        return {
          ...state,
          feedback: "Ingresá una cantidad entera mayor que cero.",
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
      return { ...state, confirmation: "error", stale_price: action, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, acknowledged_price_centavos: undefined, acknowledged_revision: undefined } : line), feedback: null };
    case "acknowledge_stale_price":
      if (state.stale_price?.product_id !== action.product_id || state.stale_price.current_unit_price_centavos !== action.current_unit_price_centavos || state.stale_price.current_revision !== action.current_revision) return state;
      return { ...state, confirmation: "idle", stale_price: null, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, acknowledged_price_centavos: action.current_unit_price_centavos, acknowledged_revision: action.current_revision } : line), feedback: "Precio actual aceptado. Confirmá nuevamente para continuar." };
    case "discard":
      return initialSaleState;
  }
}
