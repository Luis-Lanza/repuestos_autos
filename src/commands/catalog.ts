import { invoke } from "@tauri-apps/api/core";

export const CATALOG_TARGET = { CATEGORY: "category", PRODUCT: "product" } as const;
export const CATALOG_INTENT = { ARCHIVE: "archive", REACTIVATE: "reactivate" } as const;
export const ATTRIBUTE_FIELD_TYPE = { TEXT: "text", NUMBER: "number", OPTION: "option" } as const;
const RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ERROR_CODE = { VALIDATION: "validation_error", LIFECYCLE: "lifecycle_blocked", STALE: "stale_catalog_record", PERSISTENCE: "persistence_failure", UNAVAILABLE: "catalog_unavailable", INVALID_LIST: "invalid_list_price", INVALID_MINIMUM: "invalid_minimum_sale_price", MINIMUM_ABOVE_LIST: "minimum_sale_price_exceeds_list_price" } as const;
export interface ProductSearchResult { product_id: number; sku: string; name: string; category_name: string; available_quantity: number; catalog_unit_price_centavos: number; list_price_centavos: number; minimum_sale_price_centavos: number; revision: number; }
export interface CatalogMaintenanceRecord { entity_id: number; target: (typeof CATALOG_TARGET)[keyof typeof CATALOG_TARGET]; label: string; activity: "active" | "archived"; revision: number; }
export interface MaintainCatalogInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; intent: (typeof CATALOG_INTENT)[keyof typeof CATALOG_INTENT]; expected_revision: number; }
export interface CatalogAttributeDefinition { definition_id: number; label: string; field_type: (typeof ATTRIBUTE_FIELD_TYPE)[keyof typeof ATTRIBUTE_FIELD_TYPE]; required: boolean; options: string[]; }
export interface CatalogAttributeValue { definition_id: number; value: string; }
export interface CategoryMetadataDetail { target: typeof CATALOG_TARGET.CATEGORY; entity_id: number; name: string; activity: CatalogMaintenanceRecord["activity"]; revision: number; attribute_definitions: CatalogAttributeDefinition[]; }
export interface ProductMetadataDetail { target: typeof CATALOG_TARGET.PRODUCT; entity_id: number; category_id: number; sku: string; name: string; list_price_centavos: number; minimum_sale_price_centavos: number; activity: CatalogMaintenanceRecord["activity"]; revision: number; attribute_definitions: CatalogAttributeDefinition[]; attribute_values: CatalogAttributeValue[]; }
export type CatalogMetadataDetail = CategoryMetadataDetail | ProductMetadataDetail;
export interface CatalogDetailInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; }
export interface CategoryEditInput { target: typeof CATALOG_TARGET.CATEGORY; entity_id: number; expected_revision: number; name: string; }
export interface ProductEditInput { target: typeof CATALOG_TARGET.PRODUCT; entity_id: number; expected_revision: number; sku: string; name: string; list_price_centavos: number; minimum_sale_price_centavos: number; attribute_values: CatalogAttributeValue[]; }
export type CatalogEditInput = CategoryEditInput | ProductEditInput;
export interface CatalogMaintenanceError { kind: typeof RESPONSE_KIND.ERROR; code: (typeof ERROR_CODE)[keyof typeof ERROR_CODE]; message: string; }
export type CatalogMaintenanceResponse = ({ kind: typeof RESPONSE_KIND.SUCCESS } & CatalogMaintenanceRecord) | CatalogMaintenanceError;
export type CatalogMaintenanceListResponse = { kind: typeof RESPONSE_KIND.SUCCESS; records: CatalogMaintenanceRecord[] } | CatalogMaintenanceError;
export type CatalogDetailResponse = { kind: typeof RESPONSE_KIND.SUCCESS; detail: CatalogMetadataDetail } | CatalogMaintenanceError;
type Invoke = (command: string, payload?: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null;
const responseRecord = (value: unknown): value is RecordValue => record(value) && !Array.isArray(value);
const safeInteger = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const positiveSafeInteger = (value: unknown): value is number => safeInteger(value) && value > 0;
const nonNegativeSafeInteger = (value: unknown): value is number => safeInteger(value) && value >= 0;
const searchProduct = (value: unknown): ProductSearchResult | null => {
  if (!responseRecord(value)) return null;
  const hasList = Object.hasOwn(value, "list_price_centavos");
  const hasMinimum = Object.hasOwn(value, "minimum_sale_price_centavos");
  const hasCompatibilityAlias = Object.hasOwn(value, "catalog_unit_price_centavos");
  const list = hasList ? value.list_price_centavos : value.catalog_unit_price_centavos;
  const minimum = hasMinimum ? value.minimum_sale_price_centavos : list;
  const catalog = hasCompatibilityAlias ? value.catalog_unit_price_centavos : list;
  const validPrices = (!hasList || positiveSafeInteger(value.list_price_centavos)) && (!hasMinimum || positiveSafeInteger(value.minimum_sale_price_centavos)) && (!hasCompatibilityAlias || positiveSafeInteger(value.catalog_unit_price_centavos));
  return positiveSafeInteger(value.product_id) && typeof value.sku === "string" && typeof value.name === "string" && typeof value.category_name === "string" && nonNegativeSafeInteger(value.available_quantity) && positiveSafeInteger(list) && positiveSafeInteger(minimum) && positiveSafeInteger(catalog) && nonNegativeSafeInteger(value.revision) && validPrices && minimum <= list
    ? { product_id: value.product_id, sku: value.sku, name: value.name, category_name: value.category_name, available_quantity: value.available_quantity, catalog_unit_price_centavos: list, list_price_centavos: list, minimum_sale_price_centavos: minimum, revision: value.revision }
    : null;
};
const searchResults = (value: unknown): ProductSearchResult[] | null => {
  if (!Array.isArray(value)) return null;
  const decoded = value.map(searchProduct);
  return decoded.every((item): item is ProductSearchResult => item !== null) ? decoded : null;
};
const failure = (message: string): CatalogMaintenanceError => ({ kind: RESPONSE_KIND.ERROR, code: ERROR_CODE.PERSISTENCE, message });
const error = (value: RecordValue, fallback: string): CatalogMaintenanceError => Object.values(ERROR_CODE).includes(value.code as CatalogMaintenanceError["code"]) ? { kind: RESPONSE_KIND.ERROR, code: value.code as CatalogMaintenanceError["code"], message: value.code === ERROR_CODE.STALE ? "This catalog record changed. Reload and try again." : value.code === ERROR_CODE.VALIDATION ? "Review the catalog values and try again." : value.code === ERROR_CODE.LIFECYCLE ? "This lifecycle change is not allowed." : value.code === ERROR_CODE.UNAVAILABLE ? "This catalog record is unavailable." : fallback } : failure(fallback);
const maintenanceRecord = (value: unknown): CatalogMaintenanceRecord | null => record(value) && positiveSafeInteger(value.entity_id) && (value.target === CATALOG_TARGET.CATEGORY || value.target === CATALOG_TARGET.PRODUCT) && typeof value.label === "string" && (value.activity === "active" || value.activity === "archived") && nonNegativeSafeInteger(value.revision) ? { entity_id: value.entity_id, target: value.target, label: value.label, activity: value.activity, revision: value.revision } : null;
const attribute = (value: unknown): CatalogAttributeDefinition | null => record(value) && positiveSafeInteger(value.definition_id) && typeof value.label === "string" && Object.values(ATTRIBUTE_FIELD_TYPE).includes(value.field_type as CatalogAttributeDefinition["field_type"]) && typeof value.required === "boolean" && Array.isArray(value.options) && value.options.every((option) => typeof option === "string") ? { definition_id: value.definition_id, label: value.label, field_type: value.field_type as CatalogAttributeDefinition["field_type"], required: value.required, options: [...value.options] } : null;
const attributeValue = (value: unknown): CatalogAttributeValue | null => record(value) && positiveSafeInteger(value.definition_id) && typeof value.value === "string" ? { definition_id: value.definition_id, value: value.value } : null;
const detail = (value: unknown): CatalogMetadataDetail | null => record(value) && positiveSafeInteger(value.entity_id) && typeof value.name === "string" && (value.activity === "active" || value.activity === "archived") && nonNegativeSafeInteger(value.revision) && Array.isArray(value.attribute_definitions) && value.attribute_definitions.every(attribute) ? value.target === CATALOG_TARGET.CATEGORY ? { target: value.target, entity_id: value.entity_id, name: value.name, activity: value.activity, revision: value.revision, attribute_definitions: value.attribute_definitions.map((item) => attribute(item) as CatalogAttributeDefinition) } : value.target === CATALOG_TARGET.PRODUCT && positiveSafeInteger(value.category_id) && typeof value.sku === "string" && positiveSafeInteger(value.list_price_centavos) && positiveSafeInteger(value.minimum_sale_price_centavos) && value.minimum_sale_price_centavos <= value.list_price_centavos && Array.isArray(value.attribute_values) && value.attribute_values.every(attributeValue) ? { target: value.target, entity_id: value.entity_id, category_id: value.category_id, sku: value.sku, name: value.name, list_price_centavos: value.list_price_centavos, minimum_sale_price_centavos: value.minimum_sale_price_centavos, activity: value.activity, revision: value.revision, attribute_definitions: value.attribute_definitions.map((item) => attribute(item) as CatalogAttributeDefinition), attribute_values: value.attribute_values.map((item) => attributeValue(item) as CatalogAttributeValue) } : null : null;

export function createSearchProductsCommand(command: Invoke) { return async (query: string): Promise<ProductSearchResult[]> => { try { const value = searchResults(await command("search_products_command", { request: { query } })); if (!value) throw new Error("invalid search response"); return value; } catch { throw new Error("The product search could not be completed."); } }; }
export function createCatalogMaintenanceCommands(command: Invoke) {
  const loadFailure = "The catalog could not be loaded.";
  const maintainFailure = "The catalog change could not be completed.";
  return {
    list: () => command("list_catalog_maintenance_command").then((value): CatalogMaintenanceListResponse => record(value) && value.kind === RESPONSE_KIND.SUCCESS && Array.isArray(value.records) && value.records.every(maintenanceRecord) ? { kind: RESPONSE_KIND.SUCCESS, records: value.records.map((item) => maintenanceRecord(item) as CatalogMaintenanceRecord) } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, loadFailure) : failure(loadFailure)).catch(() => failure(loadFailure)),
    maintain: (input: MaintainCatalogInput) => command("maintain_catalog_command", { request: { target: input.target, entity_id: input.entity_id, intent: input.intent, expected_revision: input.expected_revision } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
    detail: (input: CatalogDetailInput) => command("catalog_metadata_detail_command", { request: { target: input.target, entity_id: input.entity_id } }).then((value): CatalogDetailResponse => { const item = detail(value); return item ? { kind: RESPONSE_KIND.SUCCESS, detail: item } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, loadFailure) : failure(loadFailure); }).catch(() => failure(loadFailure)),
    edit: (input: CatalogEditInput) => command("edit_catalog_command", { request: input.target === CATALOG_TARGET.CATEGORY ? { target: input.target, entity_id: input.entity_id, expected_revision: input.expected_revision, name: input.name } : { target: input.target, entity_id: input.entity_id, expected_revision: input.expected_revision, sku: input.sku, name: input.name, list_price_centavos: input.list_price_centavos, minimum_sale_price_centavos: input.minimum_sale_price_centavos, attribute_values: input.attribute_values.map((value) => ({ definition_id: value.definition_id, value: value.value })) } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
  };
}
export const searchProducts = createSearchProductsCommand(invoke as Invoke);
export const catalogMaintenanceCommands = createCatalogMaintenanceCommands(invoke as Invoke);
