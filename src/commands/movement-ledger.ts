import { invoke } from "@tauri-apps/api/core";
import { localDateRangeToUtc } from "./sales-history.ts";

export const MOVEMENT_TYPES = ["opening_stock", "stock_entry", "sale", "return", "adjustment", "cancellation"] as const;
export type MovementType = typeof MOVEMENT_TYPES[number];
export interface MovementLedgerRow { movement_id: number; occurred_at: string; product_id: number; product_name: string; product_sku: string; movement_type: MovementType; quantity_delta: number; resulting_quantity: number | null; reason: string | null; note: string | null; sale_id: number | null; sale_line_id: number | null; }
export interface MovementLedgerProductOption { product_id: number; product_name: string; product_sku: string; active: boolean; }
export interface MovementLedgerFilters { from: string; to: string; product_id: number | null; movement_type: MovementType | null; }
export type LedgerFailure = { kind: "error"; code: string; message: string };
export type MovementLedgerResponse = { kind: "success"; rows: MovementLedgerRow[]; page: number; page_size: number; has_more: boolean } | LedgerFailure;
export type ProductOptionsResponse = { kind: "success"; products: MovementLedgerProductOption[]; page: number; page_size: number; has_more: boolean } | LedgerFailure;
export const PRODUCT_OPTIONS_PAGE_SIZE = 20;
export type ExportResponse = { kind: "success" } | { kind: "cancelled" } | LedgerFailure;
type Invoke = (command: string, payload?: Record<string, unknown>) => Promise<unknown>;
type Obj = Record<string, unknown>;
const obj = (value: unknown): Obj | undefined => typeof value === "object" && value !== null && !Array.isArray(value) ? value as Obj : undefined;
const str = (value: unknown): value is string => typeof value === "string";
const int = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const positive = (value: unknown): value is number => int(value) && value > 0;
const nullableInt = (value: unknown): value is number | null => value === null || int(value);
const nullableString = (value: unknown): value is string | null => value === null || str(value);
const failure = (): LedgerFailure => ({ kind: "error", code: "invalid_response", message: "No se pudo leer la respuesta del registro de movimientos." });
const validMovementType = (value: unknown): value is MovementType => MOVEMENT_TYPES.includes(value as MovementType);
function decodeRow(value: unknown): MovementLedgerRow | undefined { const row = obj(value); return row && positive(row.movement_id) && str(row.occurred_at) && positive(row.product_id) && str(row.product_name) && str(row.product_sku) && validMovementType(row.movement_type) && int(row.quantity_delta) && nullableInt(row.resulting_quantity) && nullableString(row.reason) && nullableString(row.note) && nullableInt(row.sale_id) && nullableInt(row.sale_line_id) ? { movement_id: row.movement_id, occurred_at: row.occurred_at, product_id: row.product_id, product_name: row.product_name, product_sku: row.product_sku, movement_type: row.movement_type, quantity_delta: row.quantity_delta, resulting_quantity: row.resulting_quantity, reason: row.reason, note: row.note, sale_id: row.sale_id, sale_line_id: row.sale_line_id } : undefined; }
function decodeError(value: Obj): LedgerFailure { return value.kind === "error" && str(value.code) && str(value.message) ? { kind: "error", code: value.code, message: value.message } : failure(); }
function decodeList(value: unknown): MovementLedgerResponse { const response = obj(value); if (!response) return failure(); if (response.kind === "error") return decodeError(response); const rows = Array.isArray(response.rows) ? response.rows.map(decodeRow) : undefined; return response.kind === "success" && rows && rows.every((row): row is MovementLedgerRow => row !== undefined) && positive(response.page) && positive(response.page_size) && typeof response.has_more === "boolean" ? { kind: "success", rows, page: response.page, page_size: response.page_size, has_more: response.has_more } : failure(); }
function decodeProducts(value: unknown): ProductOptionsResponse { const response = obj(value); if (!response) return failure(); if (response.kind === "error") return decodeError(response); const products = Array.isArray(response.products) ? response.products.map((value) => { const item = obj(value); return item && positive(item.product_id) && str(item.product_name) && str(item.product_sku) && typeof item.active === "boolean" ? { product_id: item.product_id, product_name: item.product_name, product_sku: item.product_sku, active: item.active } : undefined; }) : undefined; return response.kind === "success" && products && products.length <= PRODUCT_OPTIONS_PAGE_SIZE && products.every((item): item is MovementLedgerProductOption => item !== undefined) && positive(response.page) && positive(response.page_size) && response.page_size <= PRODUCT_OPTIONS_PAGE_SIZE && typeof response.has_more === "boolean" ? { kind: "success", products, page: response.page, page_size: response.page_size, has_more: response.has_more } : failure(); }
function decodeExport(value: unknown): ExportResponse { const response = obj(value); if (!response) return failure(); if (response.kind === "success") return { kind: "success" }; if (response.kind === "cancelled") return { kind: "cancelled" }; if (response.kind === "error") return decodeError(response); return failure(); }
function filterPayload(filters: MovementLedgerFilters) { return { ...localDateRangeToUtc(filters.from, filters.to), product_id: filters.product_id, movement_type: filters.movement_type }; }
export function createMovementLedgerCommands(command: Invoke) { return {
  list: (filters: MovementLedgerFilters, page: number, pageSize = 50) => command("list_movement_ledger_command", { request: { ...filterPayload(filters), page, page_size: pageSize } }).then(decodeList).catch(failure),
  productOptions: (query: string, page = 1, pageSize = PRODUCT_OPTIONS_PAGE_SIZE) => command("list_movement_ledger_product_options_command", { request: { query, page, page_size: pageSize } }).then(decodeProducts).catch(failure),
  export: (filters: MovementLedgerFilters) => command("export_movement_ledger_command", { request: filterPayload(filters) }).then(decodeExport).catch(failure),
}; }
export const movementLedgerCommands = createMovementLedgerCommands(invoke as Invoke);
