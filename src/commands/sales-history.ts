import { invoke } from "@tauri-apps/api/core";

const KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ERROR_CODE = { INVALID_RANGE: "invalid_range", SALE_NOT_FOUND: "sale_not_found", PERSISTENCE_FAILURE: "persistence_failure" } as const;
const PAYMENT_METHOD = { CASH: "cash", QR: "qr" } as const;
export interface SalesHistorySummary { sale_id: number; confirmed_at: string; status: string; total_centavos: number; line_count: number; payment_count: number; payment_methods: Array<(typeof PAYMENT_METHOD)[keyof typeof PAYMENT_METHOD]>; }
export interface HistoricalLine { product_id: number; sku: string | null; product_name: string | null; quantity: number; unit_price_centavos: number; line_total_centavos: number; }
export interface CashPayment { method: typeof PAYMENT_METHOD.CASH; amount_applied_centavos: number; amount_tendered_centavos: number; change_given_centavos: number; }
export interface QrPayment { method: typeof PAYMENT_METHOD.QR; amount_applied_centavos: number; }
export interface SalesHistoryDetail { sale_id: number; confirmed_at: string; status: string; total_centavos: number; lines: HistoricalLine[]; payments: Array<CashPayment | QrPayment>; }
export interface SalesHistoryError { kind: typeof KIND.ERROR; code: (typeof ERROR_CODE)[keyof typeof ERROR_CODE]; message: string; }
export type SalesHistoryListResponse = { kind: typeof KIND.SUCCESS; sales: SalesHistorySummary[]; has_more: boolean } | SalesHistoryError;
export type SalesHistoryDetailResponse = { kind: typeof KIND.SUCCESS; detail: SalesHistoryDetail } | SalesHistoryError;
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
type LocalMidnight = (year: number, month: number, day: number) => Date;
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null;
const failure = (): SalesHistoryError => ({ kind: KIND.ERROR, code: ERROR_CODE.PERSISTENCE_FAILURE, message: "Sales history could not be loaded." });
const localMidnight: LocalMidnight = (year, month, day) => new Date(year, month, day);
const validDate = (value: string): [number, number, number] => /^\d{4}-\d{2}-\d{2}$/.test(value) && Number.isFinite(new Date(`${value}T12:00:00`).getTime()) ? [Number(value.slice(0, 4)), Number(value.slice(5, 7)) - 1, Number(value.slice(8, 10))] : (() => { throw new Error("Dates must use YYYY-MM-DD."); })();
export function localDateRangeToUtc(from: string, to: string, makeLocalMidnight: LocalMidnight = localMidnight) { const [fromYear, fromMonth, fromDay] = validDate(from); const [toYear, toMonth, toDay] = validDate(to); const fromDate = makeLocalMidnight(fromYear, fromMonth, fromDay); const toDate = makeLocalMidnight(toYear, toMonth, toDay + 1); return { from_utc: fromDate.toISOString(), to_exclusive_utc: toDate.toISOString() }; }
const list = (value: unknown): SalesHistoryListResponse => record(value) && value.kind === KIND.SUCCESS && Array.isArray(value.sales) && typeof value.has_more === "boolean" ? value as SalesHistoryListResponse : record(value) && value.kind === KIND.ERROR && Object.values(ERROR_CODE).includes(value.code as SalesHistoryError["code"]) ? { kind: KIND.ERROR, code: value.code as SalesHistoryError["code"], message: typeof value.message === "string" ? value.message : failure().message } : failure();
const detail = (value: unknown): SalesHistoryDetailResponse => record(value) && value.kind === KIND.SUCCESS && record(value.detail) ? value as SalesHistoryDetailResponse : list(value) as SalesHistoryDetailResponse;
export function createSalesHistoryCommands(command: Invoke) { return { list: (from: string, to: string) => command("list_sales_history_command", { request: localDateRangeToUtc(from, to) }).then(list).catch(failure), detail: (saleId: number) => command("sale_history_detail_command", { saleId }).then(detail).catch(failure) }; }
export const salesHistoryCommands = createSalesHistoryCommands(invoke as Invoke);
