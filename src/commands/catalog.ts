import { invoke } from "@tauri-apps/api/core";

export const CATALOG_TARGET = { CATEGORY: "category", PRODUCT: "product" } as const;
export const CATALOG_INTENT = { ARCHIVE: "archive", REACTIVATE: "reactivate" } as const;
export const ATTRIBUTE_FIELD_TYPE = { TEXT: "text", NUMBER: "number", OPTION: "option" } as const;
const RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ERROR_CODE = { VALIDATION: "validation_error", LIFECYCLE: "lifecycle_blocked", STALE: "stale_catalog_record", PERSISTENCE: "persistence_failure", UNAVAILABLE: "catalog_unavailable", IMAGE_UNAVAILABLE: "image_unavailable", INVALID_PURCHASE: "invalid_purchase_price", INVALID_SALE: "invalid_sale_price", INVALID_MINIMUM: "invalid_minimum_sale_price", MINIMUM_ABOVE_SALE: "minimum_sale_price_exceeds_sale_price" } as const;
export interface ProductSearchResult { product_id: number; sku: string; name: string; category_name: string; available_quantity: number; purchase_price_centavos: number | null; sale_price_centavos: number; list_price_centavos: number; catalog_unit_price_centavos: number; minimum_sale_price_centavos: number; revision: number; }
export interface ProductBrowseAttribute { definition_id: number; label: string; value: string; }
export interface ProductBrowseResult extends ProductSearchResult { category_id: number; attribute_values: ProductBrowseAttribute[]; }
export interface ProductBrowseCategory { category_id: number; name: string; }
export type ProductStockState = "all" | "low_stock" | "out_of_stock" | "available" | "alerts";
export type ProductActivityState = "active" | "archived" | "all";
export interface ProductBrowseInput { query?: string; category_id?: number | null; stock_state?: ProductStockState; activity?: ProductActivityState; page?: number; page_size?: number; }
export interface ProductBrowsePage { products: ProductBrowseResult[]; categories: ProductBrowseCategory[]; page: number; page_size: number; total: number; total_pages: number; }
export interface CatalogMaintenanceRecord { entity_id: number; target: (typeof CATALOG_TARGET)[keyof typeof CATALOG_TARGET]; label: string; activity: "active" | "archived"; revision: number; active_product_count?: number; }
export interface CatalogCategoryMaintenanceRecord extends CatalogMaintenanceRecord { target: typeof CATALOG_TARGET.CATEGORY; active_product_count: number; }
export interface MaintainCatalogInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; intent: (typeof CATALOG_INTENT)[keyof typeof CATALOG_INTENT]; expected_revision: number; }
export interface CatalogAttributeDefinition { definition_id: number; label: string; field_type: (typeof ATTRIBUTE_FIELD_TYPE)[keyof typeof ATTRIBUTE_FIELD_TYPE]; required: boolean; options: string[]; }
export interface CatalogAttributeValue { definition_id: number; value: string; }
export interface CategoryMetadataDetail { target: typeof CATALOG_TARGET.CATEGORY; entity_id: number; name: string; activity: CatalogMaintenanceRecord["activity"]; revision: number; attribute_definitions: CatalogAttributeDefinition[]; }
export interface ProductMetadataDetail { target: typeof CATALOG_TARGET.PRODUCT; entity_id: number; category_id: number; sku: string; name: string; purchase_price_centavos: number | null; sale_price_centavos: number; minimum_sale_price_centavos: number; activity: CatalogMaintenanceRecord["activity"]; revision: number; attribute_definitions: CatalogAttributeDefinition[]; attribute_values: CatalogAttributeValue[]; }
export type CatalogMetadataDetail = CategoryMetadataDetail | ProductMetadataDetail;
export interface CatalogDetailInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; }
export interface ProductImageInput { product_id: number; expected_revision: number; }
export type ProductImageMutationResponse = { kind: "success"; product_id: number; revision: number } | { kind: "cancelled" } | CatalogMaintenanceError;
export type ProductImageThumbnailResponse = { kind: "success"; product_id: number; revision: number; src: string } | CatalogMaintenanceError;
export interface CategoryEditInput { target: typeof CATALOG_TARGET.CATEGORY; entity_id: number; expected_revision: number; name: string; }
export interface ProductEditInput { target: typeof CATALOG_TARGET.PRODUCT; entity_id: number; expected_revision: number; sku: string; name: string; purchase_price_centavos: number; sale_price_centavos: number; minimum_sale_price_centavos: number; attribute_values: CatalogAttributeValue[]; }
export type CatalogEditInput = CategoryEditInput | ProductEditInput;
export interface CatalogMaintenanceError { kind: typeof RESPONSE_KIND.ERROR; code: (typeof ERROR_CODE)[keyof typeof ERROR_CODE]; message: string; }
export type CatalogMaintenanceResponse = ({ kind: typeof RESPONSE_KIND.SUCCESS } & CatalogMaintenanceRecord) | CatalogMaintenanceError;
export type CatalogMaintenanceListResponse = { kind: typeof RESPONSE_KIND.SUCCESS; records: CatalogMaintenanceRecord[] } | CatalogMaintenanceError;
export type CatalogCategoryMaintenanceListResponse = { kind: typeof RESPONSE_KIND.SUCCESS; records: CatalogCategoryMaintenanceRecord[] } | CatalogMaintenanceError;
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
  const hasSale = Object.hasOwn(value, "sale_price_centavos");
  const hasLegacyList = Object.hasOwn(value, "list_price_centavos");
  const sale = hasSale ? value.sale_price_centavos : hasLegacyList ? value.list_price_centavos : value.catalog_unit_price_centavos;
  const minimum = Object.hasOwn(value, "minimum_sale_price_centavos") ? value.minimum_sale_price_centavos : sale;
  const purchase = value.purchase_price_centavos;
  const validPrices = (!hasSale || positiveSafeInteger(value.sale_price_centavos)) && (!hasLegacyList || positiveSafeInteger(value.list_price_centavos)) && (!Object.hasOwn(value, "catalog_unit_price_centavos") || positiveSafeInteger(value.catalog_unit_price_centavos)) && (purchase === undefined || purchase === null || positiveSafeInteger(purchase));
  return positiveSafeInteger(value.product_id) && typeof value.sku === "string" && typeof value.name === "string" && typeof value.category_name === "string" && nonNegativeSafeInteger(value.available_quantity) && positiveSafeInteger(sale) && positiveSafeInteger(minimum) && nonNegativeSafeInteger(value.revision) && validPrices && minimum <= sale
    ? { product_id: value.product_id, sku: value.sku, name: value.name, category_name: value.category_name, available_quantity: value.available_quantity, purchase_price_centavos: purchase === undefined ? null : purchase as number | null, sale_price_centavos: sale, list_price_centavos: sale, catalog_unit_price_centavos: sale, minimum_sale_price_centavos: minimum, revision: value.revision }
    : null;
};
const searchResults = (value: unknown): ProductSearchResult[] | null => {
  if (!Array.isArray(value)) return null;
  const decoded = value.map(searchProduct);
  return decoded.every((item): item is ProductSearchResult => item !== null) ? decoded : null;
};
const browseAttribute = (value: unknown): ProductBrowseAttribute | null => responseRecord(value) && hasOnlyKeys(value, ["definition_id", "label", "value"]) && positiveSafeInteger(value.definition_id) && typeof value.label === "string" && typeof value.value === "string"
  ? { definition_id: value.definition_id, label: value.label, value: value.value }
  : null;
const browseProduct = (value: unknown): ProductBrowseResult | null => {
  const product = searchProduct(value);
  if (!product || !responseRecord(value) || !positiveSafeInteger(value.category_id) || !Array.isArray(value.attribute_values)) return null;
  const attributeValues = value.attribute_values.map(browseAttribute);
  return attributeValues.every((item): item is ProductBrowseAttribute => item !== null)
    ? { ...product, category_id: value.category_id, attribute_values: attributeValues }
    : null;
};
const browsePage = (value: unknown): ProductBrowsePage | null => {
  if (!responseRecord(value) || value.kind !== RESPONSE_KIND.SUCCESS || !Array.isArray(value.products) || !Array.isArray(value.categories)) return null;
  const products = value.products.map(browseProduct);
  const categories = value.categories.map((item): ProductBrowseCategory | null => responseRecord(item) && positiveSafeInteger(item.category_id) && typeof item.name === "string" ? { category_id: item.category_id, name: item.name } : null);
  return safeInteger(value.page) && value.page > 0 && safeInteger(value.page_size) && value.page_size > 0 && value.page_size <= 50 && nonNegativeSafeInteger(value.total) && nonNegativeSafeInteger(value.total_pages) && products.every((item): item is ProductBrowseResult => item !== null) && categories.every((item): item is ProductBrowseCategory => item !== null) ? { products: products as ProductBrowseResult[], categories: categories as ProductBrowseCategory[], page: value.page, page_size: value.page_size, total: value.total, total_pages: value.total_pages } : null;
};
const failure = (message: string): CatalogMaintenanceError => ({ kind: RESPONSE_KIND.ERROR, code: ERROR_CODE.PERSISTENCE, message });
const error = (value: RecordValue, fallback: string): CatalogMaintenanceError => Object.values(ERROR_CODE).includes(value.code as CatalogMaintenanceError["code"]) ? { kind: RESPONSE_KIND.ERROR, code: value.code as CatalogMaintenanceError["code"], message: value.code === ERROR_CODE.STALE ? "This catalog record changed. Reload and try again." : value.code === ERROR_CODE.VALIDATION ? "Review the catalog values and try again." : value.code === ERROR_CODE.LIFECYCLE ? "This lifecycle change is not allowed." : value.code === ERROR_CODE.UNAVAILABLE ? "This catalog record is unavailable." : fallback } : failure(fallback);
const maintenanceRecord = (value: unknown): CatalogMaintenanceRecord | null => record(value) && positiveSafeInteger(value.entity_id) && (value.target === CATALOG_TARGET.CATEGORY || value.target === CATALOG_TARGET.PRODUCT) && typeof value.label === "string" && (value.activity === "active" || value.activity === "archived") && nonNegativeSafeInteger(value.revision) ? { entity_id: value.entity_id, target: value.target, label: value.label, activity: value.activity, revision: value.revision } : null;
const categoryMaintenanceRecord = (value: unknown): CatalogCategoryMaintenanceRecord | null => {
  const item = maintenanceRecord(value);
  return item?.target === CATALOG_TARGET.CATEGORY && responseRecord(value) && nonNegativeSafeInteger(value.active_product_count)
    ? { ...item, target: CATALOG_TARGET.CATEGORY, active_product_count: value.active_product_count }
    : null;
};
const attribute = (value: unknown): CatalogAttributeDefinition | null => record(value) && positiveSafeInteger(value.definition_id) && typeof value.label === "string" && Object.values(ATTRIBUTE_FIELD_TYPE).includes(value.field_type as CatalogAttributeDefinition["field_type"]) && typeof value.required === "boolean" && Array.isArray(value.options) && value.options.every((option) => typeof option === "string") ? { definition_id: value.definition_id, label: value.label, field_type: value.field_type as CatalogAttributeDefinition["field_type"], required: value.required, options: [...value.options] } : null;
const attributeValue = (value: unknown): CatalogAttributeValue | null => record(value) && positiveSafeInteger(value.definition_id) && typeof value.value === "string" ? { definition_id: value.definition_id, value: value.value } : null;
const detail = (value: unknown): CatalogMetadataDetail | null => {
  if (!responseRecord(value) || !positiveSafeInteger(value.entity_id) || typeof value.name !== "string" || (value.activity !== "active" && value.activity !== "archived") || !nonNegativeSafeInteger(value.revision) || !Array.isArray(value.attribute_definitions) || !value.attribute_definitions.every(attribute)) return null;
  const definitions = value.attribute_definitions.map((item) => attribute(item) as CatalogAttributeDefinition);
  if (value.target === CATALOG_TARGET.CATEGORY) return { target: value.target, entity_id: value.entity_id, name: value.name, activity: value.activity, revision: value.revision, attribute_definitions: definitions };
  const sale = positiveSafeInteger(value.sale_price_centavos) ? value.sale_price_centavos : value.list_price_centavos;
  const purchase = value.purchase_price_centavos;
  return value.target === CATALOG_TARGET.PRODUCT && positiveSafeInteger(value.category_id) && typeof value.sku === "string" && positiveSafeInteger(sale) && (purchase === undefined || purchase === null || positiveSafeInteger(purchase)) && positiveSafeInteger(value.minimum_sale_price_centavos) && value.minimum_sale_price_centavos <= sale && Array.isArray(value.attribute_values) && value.attribute_values.every(attributeValue)
    ? { target: value.target, entity_id: value.entity_id, category_id: value.category_id, sku: value.sku, name: value.name, purchase_price_centavos: purchase === undefined ? null : purchase as number | null, sale_price_centavos: sale, minimum_sale_price_centavos: value.minimum_sale_price_centavos, activity: value.activity, revision: value.revision, attribute_definitions: definitions, attribute_values: value.attribute_values.map((item) => attributeValue(item) as CatalogAttributeValue) }
    : null;
};

const PRODUCT_IMAGE_MAX_BYTES = 2 * 1024 * 1024;
const BASE64_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const canonicalBase64 = (value: string) => /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(value)
  && (value.endsWith("==") ? BASE64_ALPHABET.indexOf(value.at(-3) ?? "") % 16 === 0 : value.endsWith("=") ? BASE64_ALPHABET.indexOf(value.at(-2) ?? "") % 4 === 0 : true);
const hasOnlyKeys = (value: RecordValue, keys: readonly string[]) => Object.keys(value).every((key) => keys.includes(key));
const safeImageError = (value: unknown, fallback: string): CatalogMaintenanceError => {
  if (!responseRecord(value) || !hasOnlyKeys(value, ["kind", "code", "message"]) || value.kind !== RESPONSE_KIND.ERROR || typeof value.code !== "string" || typeof value.message !== "string") return failure(fallback);
  const code = Object.values(ERROR_CODE).find((candidate) => candidate === value.code);
  if (!code) return failure(fallback);
  return { kind: RESPONSE_KIND.ERROR, code, message: code === ERROR_CODE.STALE ? "This catalog record changed. Reload and try again." : code === ERROR_CODE.UNAVAILABLE || code === ERROR_CODE.IMAGE_UNAVAILABLE ? "This product image is unavailable." : fallback };
};
const imageMutation = (value: unknown, fallback: string): ProductImageMutationResponse => {
  if (responseRecord(value) && value.kind === "cancelled" && hasOnlyKeys(value, ["kind"])) return { kind: "cancelled" };
  if (responseRecord(value) && value.kind === RESPONSE_KIND.SUCCESS && hasOnlyKeys(value, ["kind", "product_id", "revision"]) && positiveSafeInteger(value.product_id) && nonNegativeSafeInteger(value.revision)) return { kind: "success", product_id: value.product_id, revision: value.revision };
  return safeImageError(value, fallback);
};
const imageThumbnail = (value: unknown): ProductImageThumbnailResponse => {
  const fallback = "The product image could not be loaded.";
  if (responseRecord(value) && value.kind === RESPONSE_KIND.SUCCESS && hasOnlyKeys(value, ["kind", "product_id", "revision", "mime_type", "encoding", "bytes"]) && positiveSafeInteger(value.product_id) && nonNegativeSafeInteger(value.revision) && value.mime_type === "image/jpeg" && value.encoding === "base64" && typeof value.bytes === "string" && value.bytes.length <= 4 * Math.ceil(PRODUCT_IMAGE_MAX_BYTES / 3) && canonicalBase64(value.bytes) && value.bytes.startsWith("/9j/")) {
    const decodedLength = value.bytes.length === 0 ? 0 : value.bytes.length / 4 * 3 - (value.bytes.endsWith("==") ? 2 : value.bytes.endsWith("=") ? 1 : 0);
    if (decodedLength > 0 && decodedLength <= PRODUCT_IMAGE_MAX_BYTES) return { kind: "success", product_id: value.product_id, revision: value.revision, src: `data:image/jpeg;base64,${value.bytes}` };
  }
  return safeImageError(value, fallback);
};

export function createSearchProductsCommand(command: Invoke) { return async (query: string): Promise<ProductSearchResult[]> => { try { const value = searchResults(await command("search_products_command", { request: { query } })); if (!value) throw new Error("invalid search response"); return value; } catch { throw new Error("The product search could not be completed."); } }; }
export function createBrowseProductsCommand(command: Invoke) {
  return async (input: ProductBrowseInput = {}): Promise<ProductBrowsePage> => {
    const request = { query: input.query?.trim() || null, category_id: input.category_id ?? null, stock_state: input.stock_state ?? "all", activity: input.activity ?? "active", page: input.page ?? 1, page_size: input.page_size ?? 20 };
    try {
      const value = browsePage(await command("browse_products_command", { request }));
      if (!value) throw new Error("invalid browse response");
      return value;
    } catch {
      throw new Error("The product catalog could not be loaded.");
    }
  };
}
export function createCatalogProductImageCommands(command: Invoke) {
  const updateFailure = "The product image could not be updated.";
  return {
    choose: (input: ProductImageInput): Promise<ProductImageMutationResponse> => command("choose_product_image_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision } }).then((value) => imageMutation(value, updateFailure)).catch(() => failure(updateFailure)),
    remove: (input: ProductImageInput): Promise<ProductImageMutationResponse> => command("remove_product_image_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision } }).then((value) => imageMutation(value, updateFailure)).catch(() => failure(updateFailure)),
    thumbnail: (input: ProductImageInput): Promise<ProductImageThumbnailResponse> => command("catalog_product_image_thumbnail_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision } }).then(imageThumbnail).catch(() => failure("The product image could not be loaded.")),
  };
}

export function createCatalogMaintenanceCommands(command: Invoke) {
  const loadFailure = "The catalog could not be loaded.";
  const maintainFailure = "The catalog change could not be completed.";
  return {
    list: () => command("list_catalog_maintenance_command").then((value): CatalogMaintenanceListResponse => record(value) && value.kind === RESPONSE_KIND.SUCCESS && Array.isArray(value.records) && value.records.every(maintenanceRecord) ? { kind: RESPONSE_KIND.SUCCESS, records: value.records.map((item) => maintenanceRecord(item) as CatalogMaintenanceRecord) } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, loadFailure) : failure(loadFailure)).catch(() => failure(loadFailure)),
    listCategories: () => command("list_catalog_categories_command").then((value): CatalogCategoryMaintenanceListResponse => record(value) && value.kind === RESPONSE_KIND.SUCCESS && Array.isArray(value.records) && value.records.every(categoryMaintenanceRecord) ? { kind: RESPONSE_KIND.SUCCESS, records: value.records.map((item) => categoryMaintenanceRecord(item) as CatalogCategoryMaintenanceRecord) } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, loadFailure) : failure(loadFailure)).catch(() => failure(loadFailure)),
    maintain: (input: MaintainCatalogInput) => command("maintain_catalog_command", { request: { target: input.target, entity_id: input.entity_id, intent: input.intent, expected_revision: input.expected_revision } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
    detail: (input: CatalogDetailInput) => command("catalog_metadata_detail_command", { request: { target: input.target, entity_id: input.entity_id } }).then((value): CatalogDetailResponse => { const item = detail(value); return item ? { kind: RESPONSE_KIND.SUCCESS, detail: item } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, loadFailure) : failure(loadFailure); }).catch(() => failure(loadFailure)),
    edit: (input: CatalogEditInput) => command("edit_catalog_command", { request: input.target === CATALOG_TARGET.CATEGORY ? { target: input.target, entity_id: input.entity_id, expected_revision: input.expected_revision, name: input.name } : { target: input.target, entity_id: input.entity_id, expected_revision: input.expected_revision, sku: input.sku, name: input.name, purchase_price_centavos: input.purchase_price_centavos, sale_price_centavos: input.sale_price_centavos, minimum_sale_price_centavos: input.minimum_sale_price_centavos, attribute_values: input.attribute_values.map((value) => ({ definition_id: value.definition_id, value: value.value })) } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
  };
}
export const searchProducts = createSearchProductsCommand(invoke as Invoke);
export const browseProducts = createBrowseProductsCommand(invoke as Invoke);
export const catalogMaintenanceCommands = createCatalogMaintenanceCommands(invoke as Invoke);
export const catalogProductImageCommands = createCatalogProductImageCommands(invoke as Invoke);
