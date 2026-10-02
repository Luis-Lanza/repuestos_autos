import { invoke } from "@tauri-apps/api/core";
import { localDateRangeToUtc } from "./sales-history.ts";

export interface GrossProfitOperation {
  occurred_at: string;
  operation_kind: "venta" | "devolución";
  sale_id: number;
  return_id: number | null;
  product_name: string;
  sku: string;
  signed_quantity: number;
  negotiated_unit_price_centavos: number;
  unit_cost_snapshot_centavos: number | null;
  cost_state: "known" | "unknown";
  signed_gross_profit_centavos: number | null;
}
export interface GrossProfitOperationsPage {
  rows: GrossProfitOperation[];
  page: number;
  page_size: number;
  total: number;
  total_pages: number;
}
export type GrossProfitOperationsResponse =
  | { kind: "success"; report: GrossProfitOperationsPage }
  | { kind: "error"; code: "invalid_request" | "persistence_failure"; message: string };
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type Obj = Record<string, unknown>;
const obj = (value: unknown): Obj | undefined => typeof value === "object" && value !== null && !Array.isArray(value) ? value as Obj : undefined;
const keysAre = (value: Obj, keys: string[]) => Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
const safe = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const positive = (value: unknown): value is number => safe(value) && value > 0;
const validTimestamp = (value: unknown): value is string => typeof value === "string" && /^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?(?:Z|[+-]\d{2}:\d{2})?$/.test(value) && Number.isFinite(Date.parse(value));
const failure = (): GrossProfitOperationsResponse => ({ kind: "error", code: "persistence_failure", message: "No se pudieron cargar las operaciones de ganancia bruta." });
function decodeRow(value: unknown): GrossProfitOperation | undefined {
  const row = obj(value);
  if (!row || !keysAre(row, ["occurred_at", "operation_kind", "sale_id", "return_id", "product_name", "sku", "signed_quantity", "negotiated_unit_price_centavos", "unit_cost_snapshot_centavos", "cost_state", "signed_gross_profit_centavos"])) return;
  if (!validTimestamp(row.occurred_at) || (row.operation_kind !== "venta" && row.operation_kind !== "devolución") || !positive(row.sale_id) || (row.return_id !== null && !positive(row.return_id)) || typeof row.product_name !== "string" || typeof row.sku !== "string" || !safe(row.signed_quantity) || row.signed_quantity === 0 || !safe(row.negotiated_unit_price_centavos) || row.negotiated_unit_price_centavos < 0 || (row.unit_cost_snapshot_centavos !== null && (!safe(row.unit_cost_snapshot_centavos) || row.unit_cost_snapshot_centavos < 0)) || (row.cost_state !== "known" && row.cost_state !== "unknown") || (row.signed_gross_profit_centavos !== null && !safe(row.signed_gross_profit_centavos))) return;
  if (row.operation_kind === "venta" && (row.return_id !== null || row.signed_quantity < 0) || row.operation_kind === "devolución" && (row.return_id === null || row.signed_quantity > 0)) return;
  if (row.cost_state === "unknown" && (row.unit_cost_snapshot_centavos !== null || row.signed_gross_profit_centavos !== null)) return;
  if (row.cost_state === "known") {
    if (row.unit_cost_snapshot_centavos === null || row.unit_cost_snapshot_centavos <= 0 || row.signed_gross_profit_centavos === null) return;
    const expectedProfit = BigInt(row.signed_quantity) * (BigInt(row.negotiated_unit_price_centavos) - BigInt(row.unit_cost_snapshot_centavos));
    if (expectedProfit < BigInt(Number.MIN_SAFE_INTEGER) || expectedProfit > BigInt(Number.MAX_SAFE_INTEGER) || row.signed_gross_profit_centavos !== Number(expectedProfit)) return;
  }
  return { occurred_at: row.occurred_at, operation_kind: row.operation_kind, sale_id: row.sale_id, return_id: row.return_id, product_name: row.product_name, sku: row.sku, signed_quantity: row.signed_quantity, negotiated_unit_price_centavos: row.negotiated_unit_price_centavos, unit_cost_snapshot_centavos: row.unit_cost_snapshot_centavos, cost_state: row.cost_state, signed_gross_profit_centavos: row.signed_gross_profit_centavos };
}
export function decodeGrossProfitOperations(value: unknown): GrossProfitOperationsResponse {
  const response = obj(value);
  if (!response || typeof response.kind !== "string") return failure();
  if (response.kind === "error" && keysAre(response, ["kind", "code", "message"]) && (response.code === "invalid_request" || response.code === "persistence_failure") && typeof response.message === "string") return { kind: "error", code: response.code, message: response.message };
  const report = obj(response.report);
  const rows = report && Array.isArray(report.rows) ? report.rows.map(decodeRow) : undefined;
  if (response.kind !== "success" || !keysAre(response, ["kind", "report"]) || !report || !keysAre(report, ["rows", "page", "page_size", "total", "total_pages"]) || !rows || rows.some(row => row === undefined) || !positive(report.page) || !positive(report.page_size) || report.page_size > 100 || !safe(report.total) || report.total < 0 || !safe(report.total_pages) || report.total_pages < 0 || (report.total === 0 ? report.total_pages !== 0 || report.page !== 1 || rows.length !== 0 : report.total_pages !== Math.ceil(report.total / report.page_size) || report.page > report.total_pages || rows.length !== (report.page < report.total_pages ? report.page_size : report.total - report.page_size * (report.total_pages - 1)))) return failure();
  return { kind: "success", report: { rows: rows as GrossProfitOperation[], page: report.page, page_size: report.page_size, total: report.total, total_pages: report.total_pages } };
}
export function createGrossProfitOperationsCommands(command: Invoke) {
  return { load: (from: string, to: string, page: number) => {
    try { return command("gross_profit_operations_command", { request: { ...localDateRangeToUtc(from, to), page, page_size: 20 } }).then(decodeGrossProfitOperations).catch(failure); }
    catch { return Promise.resolve(failure()); }
  } };
}
export type GrossProfitOperationsExportResponse =
  | { kind: "success" }
  | { kind: "cancelled" }
  | { kind: "error"; code: "invalid_request" | "export_failed" | "resource_limit"; message: string };

export function decodeGrossProfitOperationsExport(value: unknown): GrossProfitOperationsExportResponse {
  const response = obj(value);
  if (!response || typeof response.kind !== "string") return { kind: "error", code: "export_failed", message: "No se pudo exportar el informe PDF." };
  if ((response.kind === "success" || response.kind === "cancelled") && keysAre(response, ["kind"])) return { kind: response.kind };
  if (response.kind === "error" && keysAre(response, ["kind", "code", "message"])) {
    const allowedMessages = response.code === "invalid_request"
      ? ["El período del informe no es válido."]
      : response.code === "export_failed"
        ? ["No se pudo generar el informe PDF.", "No se pudo guardar el informe PDF."]
        : response.code === "resource_limit"
          ? ["El informe supera los límites de recursos y no se guardó."]
          : [];
    if (allowedMessages.includes(response.message as string)) return { kind: "error", code: response.code as "invalid_request" | "export_failed" | "resource_limit", message: response.message as string };
  }
  return { kind: "error", code: "export_failed", message: "No se pudo exportar el informe PDF." };
}

function exportDateBound(localDate: string, utc: string, exclusiveDay = false) {
  const [year, month, day] = localDate.split("-").map(Number);
  const midnight = new Date(year!, month! - 1, day! + (exclusiveDay ? 1 : 0));
  return { local_date: exclusiveDay ? `${midnight.getFullYear()}-${String(midnight.getMonth() + 1).padStart(2, "0")}-${String(midnight.getDate()).padStart(2, "0")}` : localDate, utc, utc_offset_minutes: midnight.getTimezoneOffset() };
}
export function createGrossProfitOperationsExportCommand(command: Invoke) {
  return (from: string, to: string): Promise<GrossProfitOperationsExportResponse> => {
    try {
      const bounds = localDateRangeToUtc(from, to);
      return command("export_gross_profit_operations_command", { request: { from: exportDateBound(from, bounds.from_utc), to_exclusive: exportDateBound(to, bounds.to_exclusive_utc, true) } })
        .then(decodeGrossProfitOperationsExport)
        .catch(() => ({ kind: "error", code: "export_failed", message: "No se pudo exportar el informe PDF." }));
    } catch {
      return Promise.resolve({ kind: "error", code: "invalid_request", message: "El período del informe no es válido." });
    }
  };
}

export const grossProfitOperationsCommands = createGrossProfitOperationsCommands(invoke as Invoke);
export const exportGrossProfitOperationsCommand = createGrossProfitOperationsExportCommand(invoke as Invoke);
