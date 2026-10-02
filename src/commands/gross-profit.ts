import { invoke } from "@tauri-apps/api/core";
import { localDateRangeToUtc } from "./sales-history.ts";

export interface GrossProfitReport { amount_centavos: number; missing_cost_line_count: number; activity_count: number; }
export type GrossProfitResponse = { kind: "success"; report: GrossProfitReport } | { kind: "error"; code: "invalid_range" | "persistence_failure"; message: string };
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type Obj = Record<string, unknown>;
const obj = (value: unknown): Obj | undefined => typeof value === "object" && value !== null && !Array.isArray(value) ? value as Obj : undefined;
const keysAre = (value: Obj, keys: string[]) => Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
const failure = (): GrossProfitResponse => ({ kind: "error", code: "persistence_failure", message: "No se pudo cargar el informe de ganancia bruta." });
export function decodeGrossProfit(value: unknown): GrossProfitResponse {
  const response = obj(value);
  if (!response || typeof response.kind !== "string") return failure();
  if (response.kind === "success") {
    const report = obj(response.report);
    if (!keysAre(response, ["kind", "report"]) || !report || !keysAre(report, ["amount_centavos", "missing_cost_line_count", "activity_count"]) || typeof report.amount_centavos !== "number" || !Number.isSafeInteger(report.amount_centavos) || typeof report.missing_cost_line_count !== "number" || !Number.isSafeInteger(report.missing_cost_line_count) || report.missing_cost_line_count < 0 || typeof report.activity_count !== "number" || !Number.isSafeInteger(report.activity_count) || report.activity_count < 0) return failure();
    return { kind: "success", report: { amount_centavos: report.amount_centavos, missing_cost_line_count: report.missing_cost_line_count, activity_count: report.activity_count } };
  }
  if (response.kind === "error" && keysAre(response, ["kind", "code", "message"]) && (response.code === "invalid_range" || response.code === "persistence_failure") && typeof response.message === "string") return { kind: "error", code: response.code, message: response.message };
  return failure();
}
export function createGrossProfitCommands(command: Invoke) {
  return { load: (from: string, to: string) => {
    try { return command("gross_profit_report_command", { request: localDateRangeToUtc(from, to) }).then(decodeGrossProfit).catch(failure); }
    catch { return Promise.resolve(failure()); }
  } };
}
export const grossProfitCommands = createGrossProfitCommands(invoke as Invoke);
