import { invoke } from "@tauri-apps/api/core";

export const CATALOG_TARGET = { CATEGORY: "category", PRODUCT: "product" } as const;
export const CATALOG_INTENT = { ARCHIVE: "archive", REACTIVATE: "reactivate" } as const;
const RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ERROR_CODE = { VALIDATION: "validation_error", LIFECYCLE: "lifecycle_blocked", STALE: "stale_catalog_record", PERSISTENCE: "persistence_failure" } as const;
export interface ProductSearchResult { product_id: number; sku: string; name: string; category_name: string; available_quantity: number; catalog_unit_price_centavos: number; }
export interface CatalogMaintenanceRecord { entity_id: number; target: (typeof CATALOG_TARGET)[keyof typeof CATALOG_TARGET]; label: string; activity: "active" | "archived"; revision: number; }
export interface MaintainCatalogInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; intent: (typeof CATALOG_INTENT)[keyof typeof CATALOG_INTENT]; expected_revision: number; }
export interface CatalogMaintenanceError { kind: typeof RESPONSE_KIND.ERROR; code: (typeof ERROR_CODE)[keyof typeof ERROR_CODE]; message: string; }
export type CatalogMaintenanceResponse = ({ kind: typeof RESPONSE_KIND.SUCCESS } & CatalogMaintenanceRecord) | CatalogMaintenanceError;
export type CatalogMaintenanceListResponse = { kind: typeof RESPONSE_KIND.SUCCESS; records: CatalogMaintenanceRecord[] } | CatalogMaintenanceError;
type Invoke = (command: string, payload?: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null;
const failure = (message: string): CatalogMaintenanceError => ({ kind: RESPONSE_KIND.ERROR, code: ERROR_CODE.PERSISTENCE, message });
const error = (value: RecordValue, fallback: string): CatalogMaintenanceError => Object.values(ERROR_CODE).includes(value.code as CatalogMaintenanceError["code"]) ? { kind: RESPONSE_KIND.ERROR, code: value.code as CatalogMaintenanceError["code"], message: value.code === ERROR_CODE.STALE ? "This catalog record changed. Reload and try again." : value.code === ERROR_CODE.VALIDATION ? "Review the catalog values and try again." : value.code === ERROR_CODE.LIFECYCLE ? "This lifecycle change is not allowed." : fallback } : failure(fallback);
const maintenanceRecord = (value: unknown): CatalogMaintenanceRecord | null => record(value) && typeof value.entity_id === "number" && (value.target === CATALOG_TARGET.CATEGORY || value.target === CATALOG_TARGET.PRODUCT) && typeof value.label === "string" && (value.activity === "active" || value.activity === "archived") && typeof value.revision === "number" ? { entity_id: value.entity_id, target: value.target, label: value.label, activity: value.activity, revision: value.revision } : null;

export function createSearchProductsCommand(command: Invoke) { return (query: string) => command("search_products_command", { request: { query } }) as Promise<ProductSearchResult[]>; }
export function createCatalogMaintenanceCommands(command: Invoke) {
  const loadFailure = "The catalog could not be loaded.";
  const maintainFailure = "The catalog change could not be completed.";
  return {
    list: () => command("list_catalog_maintenance_command").then((value): CatalogMaintenanceListResponse => record(value) && value.kind === RESPONSE_KIND.SUCCESS && Array.isArray(value.records) && value.records.every(maintenanceRecord) ? { kind: RESPONSE_KIND.SUCCESS, records: value.records.map((item) => maintenanceRecord(item) as CatalogMaintenanceRecord) } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, loadFailure) : failure(loadFailure)).catch(() => failure(loadFailure)),
    maintain: (input: MaintainCatalogInput) => command("maintain_catalog_command", { request: { target: input.target, entity_id: input.entity_id, intent: input.intent, expected_revision: input.expected_revision } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
  };
}
export const searchProducts = createSearchProductsCommand(invoke as Invoke);
export const catalogMaintenanceCommands = createCatalogMaintenanceCommands(invoke as Invoke);
