export type DashboardRequest = {
  today_from_utc: string;
  today_to_exclusive_utc: string;
  month_from_utc: string;
  month_to_exclusive_utc: string;
};
export type DashboardMetrics = {
  effective_sale_count: number;
  effective_total_centavos: number;
  net_units_out: number;
  cancelled_sale_count: number;
};
export type DashboardReport = {
  today: { metrics: DashboardMetrics };
  month: { metrics: DashboardMetrics };
  top_products: Array<{ product_id: number; sku: string; product_name: string; net_units_out: number }>;
  payment_distribution: Array<{ method: "cash" | "qr"; amount_applied_centavos: number }>;
  recent_sales: Array<{ sale_id: number; confirmed_at: string; status: "confirmed" | "cancelled"; total_centavos: number }>;
  stock_alerts: Array<{ product_id: number; sku: string; product_name: string; quantity: number; classification: "out_of_stock" | "low_stock" }>;
};
export type DashboardError = { kind: "error"; code: "invalid_range" | "persistence_failure"; message: string };
export type DashboardResponse = { kind: "success"; report: DashboardReport } | DashboardError;
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const errorMessage = "The dashboard could not be loaded.";
const isRecord = (value: unknown): value is RecordValue => typeof value === "object" && value !== null && !Array.isArray(value);
const safeInteger = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const nonNegativeInteger = (value: unknown): value is number => safeInteger(value) && value >= 0;
const positiveInteger = (value: unknown): value is number => safeInteger(value) && value > 0;
const failure = (): DashboardError => ({ kind: "error", code: "persistence_failure", message: errorMessage });
const decodeError = (value: RecordValue): DashboardError => value.code === "invalid_range" ? { kind: "error", code: "invalid_range", message: "The dashboard date range is invalid." } : value.code === "persistence_failure" ? failure() : failure();
const metrics = (value: unknown): DashboardMetrics | null => isRecord(value) && nonNegativeInteger(value.effective_sale_count) && nonNegativeInteger(value.effective_total_centavos) && nonNegativeInteger(value.net_units_out) && nonNegativeInteger(value.cancelled_sale_count) ? { effective_sale_count: value.effective_sale_count, effective_total_centavos: value.effective_total_centavos, net_units_out: value.net_units_out, cancelled_sale_count: value.cancelled_sale_count } : null;
const report = (value: unknown): DashboardReport | null => {
  if (!isRecord(value) || !isRecord(value.today) || !isRecord(value.month)) return null;
  const today = metrics(value.today.metrics); const month = metrics(value.month.metrics);
  if (!today || !month || !Array.isArray(value.top_products) || !Array.isArray(value.payment_distribution) || !Array.isArray(value.recent_sales) || !Array.isArray(value.stock_alerts)) return null;
  const top_products = value.top_products.map((item) => isRecord(item) && positiveInteger(item.product_id) && typeof item.sku === "string" && typeof item.product_name === "string" && nonNegativeInteger(item.net_units_out) ? { product_id: item.product_id, sku: item.sku, product_name: item.product_name, net_units_out: item.net_units_out } : null);
  const payment_distribution = value.payment_distribution.map((item) => isRecord(item) && (item.method === "cash" || item.method === "qr") && nonNegativeInteger(item.amount_applied_centavos) ? { method: item.method, amount_applied_centavos: item.amount_applied_centavos } : null);
  const recent_sales = value.recent_sales.map((item) => isRecord(item) && positiveInteger(item.sale_id) && typeof item.confirmed_at === "string" && (item.status === "confirmed" || item.status === "cancelled") && nonNegativeInteger(item.total_centavos) ? { sale_id: item.sale_id, confirmed_at: item.confirmed_at, status: item.status, total_centavos: item.total_centavos } : null);
  const stock_alerts = value.stock_alerts.map((item) => isRecord(item) && positiveInteger(item.product_id) && typeof item.sku === "string" && typeof item.product_name === "string" && nonNegativeInteger(item.quantity) && (item.classification === "out_of_stock" || item.classification === "low_stock") ? { product_id: item.product_id, sku: item.sku, product_name: item.product_name, quantity: item.quantity, classification: item.classification } : null);
  if (top_products.some((item) => item === null) || payment_distribution.some((item) => item === null) || recent_sales.some((item) => item === null) || stock_alerts.some((item) => item === null)) return null;
  return { today: { metrics: today }, month: { metrics: month }, top_products: top_products as DashboardReport["top_products"], payment_distribution: payment_distribution as DashboardReport["payment_distribution"], recent_sales: recent_sales as DashboardReport["recent_sales"], stock_alerts: stock_alerts as DashboardReport["stock_alerts"] };
};
const decode = (value: unknown): DashboardResponse => isRecord(value) && value.kind === "success" && report(value.report) ? { kind: "success", report: report(value.report)! } : isRecord(value) && value.kind === "error" ? decodeError(value) : failure();

export function currentDashboardRequest(now = new Date()): DashboardRequest {
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const tomorrow = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
  const month = new Date(now.getFullYear(), now.getMonth(), 1);
  const nextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
  return { today_from_utc: today.toISOString(), today_to_exclusive_utc: tomorrow.toISOString(), month_from_utc: month.toISOString(), month_to_exclusive_utc: nextMonth.toISOString() };
}

export function createDashboardCommands(command: Invoke) {
  return { load: (): Promise<DashboardResponse> => command("dashboard_command", { request: currentDashboardRequest() }).then(decode).catch(failure) };
}
const tauriInvoke: Invoke = async (command, payload) => (await import("@tauri-apps/api/core")).invoke(command, payload);
export const dashboardCommands = createDashboardCommands(tauriInvoke);
