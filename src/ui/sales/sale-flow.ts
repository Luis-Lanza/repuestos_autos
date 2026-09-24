import type { ProductSearchResult } from "../../commands/catalog.ts";
import type { PersistedSaleSummary } from "../../commands/confirm-sale.ts";

const INVALID_BS_CORRECTION = "Ingresá un monto válido en Bs, con hasta dos decimales.";
export const INVALID_FINAL_PRICE = "Ingresá un precio de venta válido en Bs, con hasta dos decimales.";
export const POSITIVE_FINAL_PRICE = "El precio de venta debe ser mayor que cero.";
function checkedNonNegativeInteger(value: number): number { if (!Number.isSafeInteger(value) || value < 0) throw new RangeError(INVALID_BS_CORRECTION); return value; }
export function formatBs(integerCentavos: number): string { const centavos = checkedNonNegativeInteger(integerCentavos); return `Bs ${Math.floor(centavos / 100)},${String(centavos % 100).padStart(2, "0")}`; }
export function formatBsInput(integerCentavos: number): string { return formatBs(integerCentavos).slice(3); }
export function parseOptionalBs(value: string): number | null {
  if (value === "") return null;
  const match = /^(\d+)(?:,(\d{1,2}))?$/.exec(value);
  if (!match) throw new RangeError(INVALID_BS_CORRECTION);
  const centavos = BigInt(match[1]) * 100n + BigInt((match[2] ?? "").padEnd(2, "0"));
  if (centavos > BigInt(Number.MAX_SAFE_INTEGER)) throw new RangeError(INVALID_BS_CORRECTION);
  return Number(centavos);
}

export type DraftLine = {
  product_id: number; sku: string; product_name: string; quantity: number;
  captured_unit_price_centavos: number; captured_revision: number;
  sale_price_centavos: number; minimum_price_centavos: number;
  /** The controlled Spanish input, kept as text so malformed edits remain visible. */
  final_price_input: string;
  /** Legacy acknowledgement fields remain readable for old flow callers only. */
  acknowledged_price_centavos?: number; acknowledged_revision?: number;
};
const UNSAFE_DRAFT_TOTAL = "Draft money must remain within the safe integer range.";
function checkedDraftCentavos(value: number): number { if (!Number.isSafeInteger(value) || value < 0) throw new RangeError(UNSAFE_DRAFT_TOTAL); return value; }
export function finalPriceCentavos(line: DraftLine): number | null { try { return parseOptionalBs(line.final_price_input); } catch { return null; } }
export function effectiveDraftUnitPriceCentavos(line: DraftLine): number {
  const explicit = finalPriceCentavos(line);
  return checkedDraftCentavos(explicit ?? line.acknowledged_price_centavos ?? line.captured_unit_price_centavos);
}
export function draftLineSubtotalCentavos(line: DraftLine): number {
  if (!Number.isSafeInteger(line.quantity) || line.quantity < 1) throw new RangeError(UNSAFE_DRAFT_TOTAL);
  const subtotal = BigInt(effectiveDraftUnitPriceCentavos(line)) * BigInt(line.quantity);
  if (subtotal > BigInt(Number.MAX_SAFE_INTEGER)) throw new RangeError(UNSAFE_DRAFT_TOTAL);
  return Number(subtotal);
}
export function draftTotalCentavos(lines: readonly DraftLine[]): number { let total = 0n; for (const line of lines) { total += BigInt(draftLineSubtotalCentavos(line)); if (total > BigInt(Number.MAX_SAFE_INTEGER)) throw new RangeError(UNSAFE_DRAFT_TOTAL); } return Number(total); }
export function draftTotalUnits(lines: readonly DraftLine[]): number {
  let total = 0;
  for (const line of lines) {
    if (!Number.isSafeInteger(line.quantity) || line.quantity < 0) throw new RangeError(UNSAFE_DRAFT_TOTAL);
    total += line.quantity;
    if (!Number.isSafeInteger(total)) throw new RangeError(UNSAFE_DRAFT_TOTAL);
  }
  return total;
}

export type DraftPayment = { amount_tendered_centavos: string; qr_applied_centavos: string; };
export type CatalogDiscoveryState = { status: "initial" | "loading" | "results" | "empty" | "error"; query: string; request_id: number; results: ProductSearchResult[]; error: string | null; };
export type SaleState = {
  search_results: ProductSearchResult[]; catalog_discovery: CatalogDiscoveryState; lines: DraftLine[]; payment: DraftPayment;
  feedback: string | null; request_id: string | null; confirmation: "idle" | "pending" | "error" | "confirmed";
  persisted_summary: PersistedSaleSummary | null; price_errors: Record<number, string>; focus_price_product_id: number | null;
  /** Legacy response state retained for compatibility; the current screen never offers acknowledgement. */
  stale_price: { product_id: number; current_unit_price_centavos: number; current_revision: number } | null;
};
export const initialSaleState: SaleState = { search_results: [], catalog_discovery: { status: "initial", query: "", request_id: 0, results: [], error: null }, lines: [], payment: { amount_tendered_centavos: "", qr_applied_centavos: "" }, feedback: null, request_id: null, confirmation: "idle", persisted_summary: null, price_errors: {}, focus_price_product_id: null, stale_price: null };

export type SaleAction =
  | { type: "search_succeeded"; results: ProductSearchResult[] }
  | { type: "catalog_search_started"; query: string; request_id: number }
  | { type: "catalog_search_succeeded"; request_id: number; results: ProductSearchResult[] }
  | { type: "catalog_search_failed"; request_id: number; message: string }
  | { type: "add_product"; product: ProductSearchResult }
  | { type: "remove_product"; product_id: number }
  | { type: "line_quantity_changed"; product_id: number; value: string }
  | { type: "line_final_price_changed"; product_id: number; value: string }
  | { type: "payment_changed"; field: keyof DraftPayment; value: string }
  | { type: "confirmation_started"; request_id: string }
  | { type: "confirmation_succeeded"; summary: PersistedSaleSummary }
  | { type: "confirmation_failed"; message: string }
  | { type: "final_price_validation_failed"; message: string }
  | { type: "minimum_price_violation"; product_id: number; current_minimum_unit_price_centavos: number }
  | { type: "stale_price_detected"; product_id: number; current_unit_price_centavos: number; current_revision: number }
  | { type: "acknowledge_stale_price"; product_id: number; current_unit_price_centavos: number; current_revision: number }
  | { type: "discard" };
function positiveWhole(value: string): number | null { const parsed = Number(value); return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : null; }
function priceError(value: string, minimum: number): string | undefined {
  let parsed: number | null;
  try { parsed = parseOptionalBs(value); } catch { return INVALID_FINAL_PRICE; }
  if (parsed === null) return INVALID_FINAL_PRICE;
  if (parsed <= 0) return POSITIVE_FINAL_PRICE;
  return parsed < minimum ? `El precio de venta no puede ser menor que el precio mínimo de ${formatBs(minimum)}.` : undefined;
}
function resetIntent(state: SaleState): SaleState { return { ...state, request_id: null, confirmation: "idle", feedback: null, focus_price_product_id: null }; }
function withPriceError(state: SaleState, product_id: number, error: string | undefined): Record<number, string> { const errors = { ...state.price_errors }; if (error) errors[product_id] = error; else delete errors[product_id]; return errors; }

export function createSaleFlow(state: SaleState, action: SaleAction): SaleState {
  switch (action.type) {
    case "search_succeeded": return { ...state, search_results: action.results, catalog_discovery: { ...state.catalog_discovery, status: action.results.length === 0 ? "empty" : "results", results: action.results, error: null }, feedback: null };
    case "catalog_search_started":
      if (!Number.isSafeInteger(action.request_id) || action.request_id <= state.catalog_discovery.request_id) return state;
      return { ...state, catalog_discovery: { status: "loading", query: action.query, request_id: action.request_id, results: [], error: null } };
    case "catalog_search_succeeded":
      if (action.request_id !== state.catalog_discovery.request_id) return state;
      return { ...state, search_results: action.results, catalog_discovery: { ...state.catalog_discovery, status: action.results.length === 0 ? "empty" : "results", results: action.results, error: null } };
    case "catalog_search_failed":
      if (action.request_id !== state.catalog_discovery.request_id) return state;
      return { ...state, catalog_discovery: { ...state.catalog_discovery, status: "error", results: [], error: action.message } };
    case "add_product":
      if (action.product.available_quantity < 1 || state.lines.some((line) => line.product_id === action.product.product_id)) return state;
      { const salePrice = action.product.sale_price_centavos ?? action.product.list_price_centavos ?? action.product.catalog_unit_price_centavos;
        const minimumPrice = Number.isSafeInteger(action.product.minimum_sale_price_centavos) ? action.product.minimum_sale_price_centavos : salePrice;
        return { ...resetIntent(state), lines: [...state.lines, { product_id: action.product.product_id, sku: action.product.sku, product_name: action.product.name, quantity: 1, captured_unit_price_centavos: salePrice, captured_revision: action.product.revision, sale_price_centavos: salePrice, minimum_price_centavos: minimumPrice, final_price_input: formatBsInput(salePrice) }], feedback: null }; }
    case "remove_product": {
      if (!state.lines.some((line) => line.product_id === action.product_id)) return state;
      const price_errors = { ...state.price_errors }; delete price_errors[action.product_id];
      return { ...resetIntent(state), lines: state.lines.filter((line) => line.product_id !== action.product_id), price_errors, stale_price: state.stale_price?.product_id === action.product_id ? null : state.stale_price };
    }
    case "line_quantity_changed": {
      const quantity = positiveWhole(action.value);
      if (quantity === null) return { ...state, feedback: "Ingresá una cantidad entera mayor que cero." };
      const line = state.lines.find((candidate) => candidate.product_id === action.product_id);
      if (!line || line.quantity === quantity) return { ...state, feedback: null };
      return { ...resetIntent(state), lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, quantity } : line) };
    }
    case "line_final_price_changed": {
      const line = state.lines.find((candidate) => candidate.product_id === action.product_id);
      if (!line) return state;
      const error = priceError(action.value, line.minimum_price_centavos);
      if (line.final_price_input === action.value) return { ...state, price_errors: withPriceError(state, action.product_id, error), focus_price_product_id: error ? action.product_id : null, feedback: null };
      return { ...resetIntent(state), stale_price: null, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, final_price_input: action.value, acknowledged_price_centavos: undefined, acknowledged_revision: undefined } : line), price_errors: withPriceError(state, action.product_id, error) };
    }
    case "payment_changed": if (state.payment[action.field] === action.value) return { ...state, feedback: null }; return { ...resetIntent(state), payment: { ...state.payment, [action.field]: action.value } };
    case "confirmation_started": return { ...state, request_id: state.request_id ?? action.request_id, confirmation: "pending", feedback: null };
    case "confirmation_succeeded": return { ...state, request_id: action.summary.request_id, confirmation: "confirmed", persisted_summary: action.summary, feedback: null };
    case "confirmation_failed": return { ...state, confirmation: "error", feedback: action.message };
    case "final_price_validation_failed": {
      const product_id = state.lines.find((line) => finalPriceCentavos(line) === 0)?.product_id;
      if (product_id === undefined) return { ...state, confirmation: "error", feedback: POSITIVE_FINAL_PRICE };
      return { ...resetIntent(state), confirmation: "error", price_errors: { ...state.price_errors, [product_id]: POSITIVE_FINAL_PRICE }, focus_price_product_id: product_id, feedback: POSITIVE_FINAL_PRICE };
    }
    case "minimum_price_violation": {
      const line = state.lines.find((candidate) => candidate.product_id === action.product_id);
      if (!line) return { ...state, confirmation: "error", feedback: "El precio mínimo actual no está disponible." };
      const message = `El precio mínimo actual es ${formatBs(action.current_minimum_unit_price_centavos)}. Ajustá el precio de venta para continuar.`;
      return { ...resetIntent(state), confirmation: "error", lines: state.lines.map((candidate) => candidate.product_id === action.product_id ? { ...candidate, minimum_price_centavos: action.current_minimum_unit_price_centavos } : candidate), price_errors: { ...withPriceError(state, action.product_id, priceError(line.final_price_input, action.current_minimum_unit_price_centavos)), [action.product_id]: message }, focus_price_product_id: action.product_id, feedback: message };
    }
    case "stale_price_detected": return { ...resetIntent(state), confirmation: "error", stale_price: action, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, final_price_input: "", acknowledged_price_centavos: undefined, acknowledged_revision: undefined } : line) };
    case "acknowledge_stale_price":
      if (state.stale_price?.product_id !== action.product_id || state.stale_price.current_unit_price_centavos !== action.current_unit_price_centavos || state.stale_price.current_revision !== action.current_revision) return state;
      return { ...resetIntent(state), stale_price: null, lines: state.lines.map((line) => line.product_id === action.product_id ? { ...line, final_price_input: "", acknowledged_price_centavos: action.current_unit_price_centavos, acknowledged_revision: action.current_revision } : line), feedback: "Precio actual aceptado. Confirmá nuevamente para continuar." };
    case "discard": return initialSaleState;
  }
}
