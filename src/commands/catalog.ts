import { invoke } from "@tauri-apps/api/core";

export const CATALOG_TARGET = { CATEGORY: "category", PRODUCT: "product" } as const;
export const CATALOG_INTENT = { ARCHIVE: "archive", REACTIVATE: "reactivate" } as const;
export const ATTRIBUTE_FIELD_TYPE = { TEXT: "text", NUMBER: "number", OPTION: "option" } as const;
const RESPONSE_KIND = { SUCCESS: "success", ERROR: "error" } as const;
const ERROR_CODE = { DUPLICATE_SKU: "duplicate_sku", CATALOG_ACCESS: "catalog_access_required", VALIDATION: "validation_error", LIFECYCLE: "lifecycle_blocked", STALE: "stale_catalog_record", STALE_SCHEMA: "stale_category_schema", PERSISTENCE: "persistence_failure", UNAVAILABLE: "catalog_unavailable", IMAGE_UNAVAILABLE: "image_unavailable", INVALID_PURCHASE: "invalid_purchase_price", INVALID_SALE: "invalid_sale_price", INVALID_MINIMUM: "invalid_minimum_sale_price", MINIMUM_ABOVE_SALE: "minimum_sale_price_exceeds_sale_price" } as const;
export interface ProductSearchResult { product_id: number; revision?: number; sku: string; name: string; category_name: string; available_quantity: number; purchase_price_centavos: number | null; sale_price_centavos: number; list_price_centavos: number; catalog_unit_price_centavos: number; minimum_sale_price_centavos: number; }
export interface ProductBrowseAttribute { definition_id: number; label: string; value: string; }
export interface ProductBrowseResult extends ProductSearchResult { revision: number; category_id: number; primary_location_code: string | null; attribute_values: ProductBrowseAttribute[]; }
export interface ProductBrowseCategory { category_id: number; name: string; }
export type ProductStockState = "all" | "low_stock" | "out_of_stock" | "available" | "alerts";
export type ProductActivityState = "active" | "archived" | "all";
export interface ProductBrowseInput { query?: string; category_id?: number | null; stock_state?: ProductStockState; activity?: ProductActivityState; page?: number; page_size?: number; }
export interface ProductBrowsePage { products: ProductBrowseResult[]; categories: ProductBrowseCategory[]; page: number; page_size: number; total: number; total_pages: number; }
export interface InventoryBrowseProduct { product_id: number; category_id: number; sku: string; name: string; category_name: string; available_quantity: number; sale_price_centavos: number; minimum_sale_price_centavos: number; primary_location_code: string | null; }
export interface InventoryBrowsePage { products: InventoryBrowseProduct[]; categories: ProductBrowseCategory[]; page: number; page_size: number; total: number; total_pages: number; }
export interface SalesBrowseProduct { product_id: number; category_id: number; sku: string; name: string; category_name: string; available_quantity: number; purchase_price_centavos: number | null; sale_price_centavos: number; list_price_centavos: number; catalog_unit_price_centavos: number; minimum_sale_price_centavos: number; primary_location_code: string | null; attribute_values: ProductBrowseAttribute[]; }
export interface SalesBrowsePage { products: SalesBrowseProduct[]; categories: ProductBrowseCategory[]; page: number; page_size: number; total: number; total_pages: number; }
export type ProductBrowserProduct = ProductBrowseResult | SalesBrowseProduct;
export type ProductBrowserPage = ProductBrowsePage | SalesBrowsePage;
export interface CatalogMaintenanceRecord { entity_id: number; target: (typeof CATALOG_TARGET)[keyof typeof CATALOG_TARGET]; label: string; activity: "active" | "archived"; revision: number; active_product_count?: number; }
export interface CatalogCategoryMaintenanceRecord extends CatalogMaintenanceRecord { target: typeof CATALOG_TARGET.CATEGORY; active_product_count: number; }
export interface MaintainCatalogInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; intent: (typeof CATALOG_INTENT)[keyof typeof CATALOG_INTENT]; expected_revision: number; }
export interface CatalogAttributeDefinition { definition_id: number; label: string; field_type: (typeof ATTRIBUTE_FIELD_TYPE)[keyof typeof ATTRIBUTE_FIELD_TYPE]; required: boolean; options: string[]; active?: boolean; }
export interface CatalogAttributeValue { definition_id: number; value: string; }
export interface CategoryMetadataDetail { target: typeof CATALOG_TARGET.CATEGORY; entity_id: number; name: string; activity: CatalogMaintenanceRecord["activity"]; revision: number; attribute_definitions: CatalogAttributeDefinition[]; }
export interface ProductMetadataDetail { target: typeof CATALOG_TARGET.PRODUCT; entity_id: number; category_id: number; category_revision: number; sku: string; name: string; purchase_price_centavos: number | null; sale_price_centavos: number; minimum_sale_price_centavos: number; low_stock_threshold: number; primary_location_id: number | null; activity: CatalogMaintenanceRecord["activity"]; revision: number; attribute_definitions: CatalogAttributeDefinition[]; attribute_values: CatalogAttributeValue[]; }
export type CatalogMetadataDetail = CategoryMetadataDetail | ProductMetadataDetail;
export interface CatalogDetailInput { target: CatalogMaintenanceRecord["target"]; entity_id: number; }
export interface ProductImageInput { product_id: number; expected_revision: number; }
export type ProductImageMutationResponse = { kind: "success"; product_id: number; revision: number } | { kind: "cancelled" } | CatalogMaintenanceError;
export type ProductImageThumbnailResponse = { kind: "success"; product_id: number; revision: number; src: string } | CatalogMaintenanceError;
export type SalesProductThumbnailResponse = { kind: "success"; product_id: number; src: string } | { kind: "unavailable" } | { kind: "error"; code: "persistence_failure"; message: string };
export type SalesProductOriginalResponse = SalesProductThumbnailResponse;
export type SalesProductOriginalLoader = (product_id: number) => Promise<SalesProductOriginalResponse>;
export interface CategoryEditInput { target: typeof CATALOG_TARGET.CATEGORY; entity_id: number; expected_revision: number; name: string; }
export interface ProductEditInput { target: typeof CATALOG_TARGET.PRODUCT; entity_id: number; expected_revision: number; expected_category_revision: number; sku: string; name: string; purchase_price_centavos: number; sale_price_centavos: number; minimum_sale_price_centavos: number; low_stock_threshold?: number; attribute_values: CatalogAttributeValue[]; }
export interface CategorySchemaFieldInput { definition_id: number | null; label: string; field_type: CatalogAttributeDefinition["field_type"]; required: boolean; options: string[]; }
export interface EditCategorySchemaInput { category_id: number; expected_revision: number; fields: CategorySchemaFieldInput[]; }
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
const searchProduct = (value: unknown, includePurchaseCost = false): ProductSearchResult | null => {
  if (!responseRecord(value)) return null;
  const hasSale = Object.hasOwn(value, "sale_price_centavos");
  const hasLegacyList = Object.hasOwn(value, "list_price_centavos");
  const sale = hasSale ? value.sale_price_centavos : hasLegacyList ? value.list_price_centavos : value.catalog_unit_price_centavos;
  const minimum = Object.hasOwn(value, "minimum_sale_price_centavos") ? value.minimum_sale_price_centavos : sale;
  const purchase = value.purchase_price_centavos;
  const validPrices = (!hasSale || positiveSafeInteger(value.sale_price_centavos)) && (!hasLegacyList || positiveSafeInteger(value.list_price_centavos)) && (!Object.hasOwn(value, "catalog_unit_price_centavos") || positiveSafeInteger(value.catalog_unit_price_centavos)) && (purchase === undefined || purchase === null || positiveSafeInteger(purchase));
  return (!Object.hasOwn(value, "revision") || nonNegativeSafeInteger(value.revision)) && positiveSafeInteger(value.product_id) && typeof value.sku === "string" && typeof value.name === "string" && typeof value.category_name === "string" && nonNegativeSafeInteger(value.available_quantity) && positiveSafeInteger(sale) && positiveSafeInteger(minimum) && validPrices && minimum <= sale
    ? { product_id: value.product_id, ...(nonNegativeSafeInteger(value.revision) ? { revision: value.revision } : {}), sku: value.sku, name: value.name, category_name: value.category_name, available_quantity: value.available_quantity, purchase_price_centavos: includePurchaseCost && purchase !== undefined ? purchase as number | null : null, sale_price_centavos: sale, list_price_centavos: sale, catalog_unit_price_centavos: sale, minimum_sale_price_centavos: minimum }
    : null;
};
const searchResults = (value: unknown): ProductSearchResult[] | null => {
  if (!Array.isArray(value)) return null;
  const decoded = value.map((item) => searchProduct(item));
  return decoded.every((item): item is ProductSearchResult => item !== null) ? decoded : null;
};
const browseAttribute = (value: unknown): ProductBrowseAttribute | null => responseRecord(value) && hasOnlyKeys(value, ["definition_id", "label", "value"]) && positiveSafeInteger(value.definition_id) && typeof value.label === "string" && typeof value.value === "string"
  ? { definition_id: value.definition_id, label: value.label, value: value.value }
  : null;
const MAX_SALES_BROWSE_ATTRIBUTES = 32;
const MAX_SALES_ATTRIBUTE_LABEL_CHARS = 128;
const MAX_SALES_ATTRIBUTE_VALUE_CHARS = 256;
const salesBrowseAttribute = (value: unknown): ProductBrowseAttribute | null => {
  const attribute = browseAttribute(value);
  return attribute && Array.from(attribute.label).length <= MAX_SALES_ATTRIBUTE_LABEL_CHARS && Array.from(attribute.value).length <= MAX_SALES_ATTRIBUTE_VALUE_CHARS ? attribute : null;
};
const browseProduct = (value: unknown): ProductBrowseResult | null => {
  const product = searchProduct(value, true);
  if (!product || !responseRecord(value) || !nonNegativeSafeInteger(value.revision) || !positiveSafeInteger(value.category_id) || (value.primary_location_code !== undefined && value.primary_location_code !== null && typeof value.primary_location_code !== "string") || !Array.isArray(value.attribute_values)) return null;
  const attributeValues = value.attribute_values.map(browseAttribute);
  return attributeValues.every((item): item is ProductBrowseAttribute => item !== null)
    ? { ...product, revision: value.revision, category_id: value.category_id, primary_location_code: typeof value.primary_location_code === "string" && value.primary_location_code.trim() ? value.primary_location_code : null, attribute_values: attributeValues }
    : null;
};
const browsePage = (value: unknown): ProductBrowsePage | null => {
  if (!responseRecord(value) || value.kind !== RESPONSE_KIND.SUCCESS || !Array.isArray(value.products) || !Array.isArray(value.categories)) return null;
  const products = value.products.map(browseProduct);
  const categories = value.categories.map((item): ProductBrowseCategory | null => responseRecord(item) && positiveSafeInteger(item.category_id) && typeof item.name === "string" ? { category_id: item.category_id, name: item.name } : null);
  return safeInteger(value.page) && value.page > 0 && safeInteger(value.page_size) && value.page_size > 0 && value.page_size <= 50 && nonNegativeSafeInteger(value.total) && nonNegativeSafeInteger(value.total_pages) && products.every((item): item is ProductBrowseResult => item !== null) && categories.every((item): item is ProductBrowseCategory => item !== null) ? { products: products as ProductBrowseResult[], categories: categories as ProductBrowseCategory[], page: value.page, page_size: value.page_size, total: value.total, total_pages: value.total_pages } : null;
};
const failure = (message: string): CatalogMaintenanceError => ({ kind: RESPONSE_KIND.ERROR, code: ERROR_CODE.PERSISTENCE, message });
const error = (value: RecordValue, fallback: string): CatalogMaintenanceError => {
  if (value.code === ERROR_CODE.DUPLICATE_SKU) {
    if (!hasOnlyKeys(value, ["kind", "code", "message"]) || typeof value.message !== "string") return failure(fallback);
    return { kind: RESPONSE_KIND.ERROR, code: ERROR_CODE.DUPLICATE_SKU, message: "The SKU already exists. Use another SKU." };
  }
  return Object.values(ERROR_CODE).includes(value.code as CatalogMaintenanceError["code"]) ? { kind: RESPONSE_KIND.ERROR, code: value.code as CatalogMaintenanceError["code"], message: value.code === ERROR_CODE.STALE ? "This catalog record changed. Reload and try again." : value.code === ERROR_CODE.STALE_SCHEMA ? "Category fields changed. Reload before saving this product." : value.code === ERROR_CODE.VALIDATION ? "Review the catalog values and try again." : value.code === ERROR_CODE.LIFECYCLE ? "This lifecycle change is not allowed." : value.code === ERROR_CODE.UNAVAILABLE ? "This catalog record is unavailable." : fallback } : failure(fallback);
};
const maintenanceRecord = (value: unknown): CatalogMaintenanceRecord | null => record(value) && positiveSafeInteger(value.entity_id) && (value.target === CATALOG_TARGET.CATEGORY || value.target === CATALOG_TARGET.PRODUCT) && typeof value.label === "string" && (value.activity === "active" || value.activity === "archived") && nonNegativeSafeInteger(value.revision) ? { entity_id: value.entity_id, target: value.target, label: value.label, activity: value.activity, revision: value.revision } : null;
const categoryMaintenanceRecord = (value: unknown): CatalogCategoryMaintenanceRecord | null => {
  const item = maintenanceRecord(value);
  return item?.target === CATALOG_TARGET.CATEGORY && responseRecord(value) && nonNegativeSafeInteger(value.active_product_count)
    ? { ...item, target: CATALOG_TARGET.CATEGORY, active_product_count: value.active_product_count }
    : null;
};
const attribute = (value: unknown): CatalogAttributeDefinition | null => record(value) && positiveSafeInteger(value.definition_id) && typeof value.label === "string" && Object.values(ATTRIBUTE_FIELD_TYPE).includes(value.field_type as CatalogAttributeDefinition["field_type"]) && typeof value.required === "boolean" && Array.isArray(value.options) && value.options.every((option) => typeof option === "string") ? { definition_id: value.definition_id, label: value.label, field_type: value.field_type as CatalogAttributeDefinition["field_type"], required: value.required, options: [...value.options], ...(typeof value.active === "boolean" ? { active: value.active } : {}) } : null;
const attributeValue = (value: unknown): CatalogAttributeValue | null => record(value) && positiveSafeInteger(value.definition_id) && typeof value.value === "string" ? { definition_id: value.definition_id, value: value.value } : null;
const detail = (value: unknown): CatalogMetadataDetail | null => {
  if (!responseRecord(value) || !positiveSafeInteger(value.entity_id) || typeof value.name !== "string" || (value.activity !== "active" && value.activity !== "archived") || !nonNegativeSafeInteger(value.revision) || !Array.isArray(value.attribute_definitions) || !value.attribute_definitions.every(attribute)) return null;
  const definitions = value.attribute_definitions.map((item) => attribute(item) as CatalogAttributeDefinition);
  if (value.target === CATALOG_TARGET.CATEGORY) return { target: value.target, entity_id: value.entity_id, name: value.name, activity: value.activity, revision: value.revision, attribute_definitions: definitions };
  const sale = positiveSafeInteger(value.sale_price_centavos) ? value.sale_price_centavos : value.list_price_centavos;
  const purchase = value.purchase_price_centavos;
  return value.target === CATALOG_TARGET.PRODUCT && positiveSafeInteger(value.category_id) && nonNegativeSafeInteger(value.category_revision) && typeof value.sku === "string" && positiveSafeInteger(sale) && (purchase === undefined || purchase === null || positiveSafeInteger(purchase)) && positiveSafeInteger(value.minimum_sale_price_centavos) && value.minimum_sale_price_centavos <= sale && (value.low_stock_threshold === undefined || positiveSafeInteger(value.low_stock_threshold)) && (value.primary_location_id === null || positiveSafeInteger(value.primary_location_id)) && Array.isArray(value.attribute_values) && value.attribute_values.every(attributeValue)
    ? { target: value.target, entity_id: value.entity_id, category_id: value.category_id, category_revision: value.category_revision, sku: value.sku, name: value.name, purchase_price_centavos: purchase === undefined ? null : purchase as number | null, sale_price_centavos: sale, minimum_sale_price_centavos: value.minimum_sale_price_centavos, low_stock_threshold: value.low_stock_threshold === undefined ? 1 : value.low_stock_threshold, primary_location_id: value.primary_location_id as number | null, activity: value.activity, revision: value.revision, attribute_definitions: definitions, attribute_values: value.attribute_values.map((item) => attributeValue(item) as CatalogAttributeValue) }
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
export function createSalesBrowseProductsCommand(command: Invoke) {
  return async (input: ProductBrowseInput = {}): Promise<SalesBrowsePage> => {
    const request = { query: input.query?.trim() || null, category_id: input.category_id ?? null, stock_state: "all", activity: "active", page: input.page ?? 1, page_size: input.page_size ?? 20 };
    try {
      const value = await command("browse_sale_products_command", { request });
      if (!responseRecord(value) || !Array.isArray(value.products) || !Array.isArray(value.categories)) throw new Error("invalid sales browse response");
      const products = value.products.map((item): SalesBrowseProduct | null => {
        if (!responseRecord(item) || !hasOnlyKeys(item, ["product_id", "category_id", "sku", "name", "category_name", "available_quantity", "purchase_price_centavos", "sale_price_centavos", "minimum_sale_price_centavos", "primary_location_code", "attribute_values"]) || !positiveSafeInteger(item.product_id) || !positiveSafeInteger(item.category_id) || typeof item.sku !== "string" || typeof item.name !== "string" || typeof item.category_name !== "string" || !nonNegativeSafeInteger(item.available_quantity) || (item.purchase_price_centavos !== null && !positiveSafeInteger(item.purchase_price_centavos)) || !positiveSafeInteger(item.sale_price_centavos) || !positiveSafeInteger(item.minimum_sale_price_centavos) || item.minimum_sale_price_centavos > item.sale_price_centavos || (item.primary_location_code !== null && typeof item.primary_location_code !== "string") || !Array.isArray(item.attribute_values) || item.attribute_values.length > MAX_SALES_BROWSE_ATTRIBUTES) return null;
        const attributes = item.attribute_values.map(salesBrowseAttribute);
        if (!attributes.every((attribute): attribute is ProductBrowseAttribute => attribute !== null)) return null;
        return { product_id: item.product_id, category_id: item.category_id, sku: item.sku, name: item.name, category_name: item.category_name, available_quantity: item.available_quantity, purchase_price_centavos: item.purchase_price_centavos as number | null, sale_price_centavos: item.sale_price_centavos, list_price_centavos: item.sale_price_centavos, catalog_unit_price_centavos: item.sale_price_centavos, minimum_sale_price_centavos: item.minimum_sale_price_centavos, primary_location_code: item.primary_location_code as string | null, attribute_values: attributes };
      });
      const categories = value.categories.map((item): ProductBrowseCategory | null => responseRecord(item) && hasOnlyKeys(item, ["category_id", "name"]) && positiveSafeInteger(item.category_id) && typeof item.name === "string" ? { category_id: item.category_id, name: item.name } : null);
      if (!safeInteger(value.page) || value.page < 1 || !safeInteger(value.page_size) || value.page_size < 1 || value.page_size > 50 || !nonNegativeSafeInteger(value.total) || !nonNegativeSafeInteger(value.total_pages) || !products.every((item) => item !== null) || !categories.every((item) => item !== null)) throw new Error("invalid sales browse response");
      return { products: products as SalesBrowseProduct[], categories: categories as ProductBrowseCategory[], page: value.page, page_size: value.page_size, total: value.total, total_pages: value.total_pages };
    } catch { throw new Error("The product catalog could not be loaded."); }
  };
}

export function createInventoryBrowseProductsCommand(command: Invoke) {
  return async (input: ProductBrowseInput = {}): Promise<InventoryBrowsePage> => {
    const request = { query: input.query?.trim() || null, category_id: input.category_id ?? null, stock_state: input.stock_state ?? "all", activity: "active", page: input.page ?? 1, page_size: input.page_size ?? 20 };
    try {
      const value = await command("browse_inventory_products_command", { request });
      if (!responseRecord(value) || !Array.isArray(value.products) || !Array.isArray(value.categories)) throw new Error("invalid inventory browse response");
      const products = value.products.map((item): InventoryBrowseProduct | null => responseRecord(item) && hasOnlyKeys(item, ["product_id", "category_id", "sku", "name", "category_name", "available_quantity", "sale_price_centavos", "minimum_sale_price_centavos", "primary_location_code"]) && positiveSafeInteger(item.product_id) && positiveSafeInteger(item.category_id) && typeof item.sku === "string" && typeof item.name === "string" && typeof item.category_name === "string" && nonNegativeSafeInteger(item.available_quantity) && positiveSafeInteger(item.sale_price_centavos) && positiveSafeInteger(item.minimum_sale_price_centavos) && item.minimum_sale_price_centavos <= item.sale_price_centavos && (item.primary_location_code === null || typeof item.primary_location_code === "string") ? { product_id: item.product_id, category_id: item.category_id, sku: item.sku, name: item.name, category_name: item.category_name, available_quantity: item.available_quantity, sale_price_centavos: item.sale_price_centavos, minimum_sale_price_centavos: item.minimum_sale_price_centavos, primary_location_code: item.primary_location_code as string | null } : null);
      const categories = value.categories.map((item): ProductBrowseCategory | null => responseRecord(item) && hasOnlyKeys(item, ["category_id", "name"]) && positiveSafeInteger(item.category_id) && typeof item.name === "string" ? { category_id: item.category_id, name: item.name } : null);
      if (!safeInteger(value.page) || value.page < 1 || !safeInteger(value.page_size) || value.page_size < 1 || value.page_size > 50 || !nonNegativeSafeInteger(value.total) || !nonNegativeSafeInteger(value.total_pages) || !products.every((item) => item !== null) || !categories.every((item) => item !== null)) throw new Error("invalid inventory browse response");
      return { products: products as InventoryBrowseProduct[], categories: categories as ProductBrowseCategory[], page: value.page, page_size: value.page_size, total: value.total, total_pages: value.total_pages };
    } catch { throw new Error("The product catalog could not be loaded."); }
  };
}

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
const salesThumbnail = (value: unknown): SalesProductThumbnailResponse => {
  if (responseRecord(value) && value.kind === "unavailable" && hasOnlyKeys(value, ["kind"])) return { kind: "unavailable" };
  if (responseRecord(value) && value.kind === "success" && hasOnlyKeys(value, ["kind", "product_id", "mime_type", "encoding", "bytes"]) && positiveSafeInteger(value.product_id) && value.mime_type === "image/jpeg" && value.encoding === "base64" && typeof value.bytes === "string" && value.bytes.length <= 4 * Math.ceil(PRODUCT_IMAGE_MAX_BYTES / 3) && canonicalBase64(value.bytes) && value.bytes.startsWith("/9j/")) {
    const decodedLength = value.bytes.length / 4 * 3 - (value.bytes.endsWith("==") ? 2 : value.bytes.endsWith("=") ? 1 : 0);
    if (decodedLength > 0 && decodedLength <= PRODUCT_IMAGE_MAX_BYTES) return { kind: "success", product_id: value.product_id, src: `data:image/jpeg;base64,${value.bytes}` };
  }
  if (responseRecord(value) && value.kind === "error" && hasOnlyKeys(value, ["kind", "code", "message"]) && value.code === "persistence_failure" && typeof value.message === "string") return { kind: "error", code: "persistence_failure", message: "The product image could not be loaded." };
  return { kind: "error", code: "persistence_failure", message: "The product image could not be loaded." };
};
const salesOriginal = (value: unknown, productId: number): SalesProductOriginalResponse => {
  const invalid: SalesProductOriginalResponse = { kind: "error", code: "persistence_failure", message: "The product image could not be loaded." };
  if (responseRecord(value) && value.kind === "unavailable" && hasOnlyKeys(value, ["kind"])) return { kind: "unavailable" };
  if (!responseRecord(value) || value.kind !== "success" || !hasOnlyKeys(value, ["kind", "product_id", "mime_type", "encoding", "bytes"]) || !positiveSafeInteger(value.product_id) || value.product_id !== productId || value.encoding !== "base64" || typeof value.bytes !== "string" || !value.bytes.length || value.bytes.length > 4 * Math.ceil(PRODUCT_IMAGE_MAX_BYTES / 3) || !canonicalBase64(value.bytes)) return invalid;
  const length = value.bytes.length / 4 * 3 - (value.bytes.endsWith("==") ? 2 : value.bytes.endsWith("=") ? 1 : 0);
  if (length > PRODUCT_IMAGE_MAX_BYTES) return invalid;
  // Decode only the signature prefix; the complete bounded payload stays encoded.
  const prefix = atob(value.bytes.slice(0, 16));
  const matches = value.mime_type === "image/jpeg" ? prefix.startsWith("\xff\xd8\xff")
    : value.mime_type === "image/png" ? prefix.startsWith("\x89PNG\r\n\x1a\n")
    : value.mime_type === "image/webp" && prefix.startsWith("RIFF") && prefix.slice(8, 12) === "WEBP";
  return matches ? { kind: "success", product_id: value.product_id, src: `data:${value.mime_type};base64,${value.bytes}` } : invalid;
};
export function createCatalogProductImageCommands(command: Invoke) {
  const updateFailure = "The product image could not be updated.";
  return {
    choose: (input: ProductImageInput): Promise<ProductImageMutationResponse> => command("choose_product_image_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision } }).then((value) => imageMutation(value, updateFailure)).catch(() => failure(updateFailure)),
    remove: (input: ProductImageInput): Promise<ProductImageMutationResponse> => command("remove_product_image_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision } }).then((value) => imageMutation(value, updateFailure)).catch(() => failure(updateFailure)),
    thumbnail: (input: ProductImageInput): Promise<ProductImageThumbnailResponse> => command("catalog_product_image_thumbnail_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision } }).then(imageThumbnail).catch(() => failure("The product image could not be loaded.")),
    salesOriginal: async (product_id: number): Promise<SalesProductOriginalResponse> => {
      if (!positiveSafeInteger(product_id)) return { kind: "unavailable" };
      try { return salesOriginal(await command("sales_product_image_original_command", { request: { product_id } }), product_id); }
      catch { return { kind: "error", code: "persistence_failure", message: "The product image could not be loaded." }; }
    },
    salesThumbnail: (product_id: number): Promise<SalesProductThumbnailResponse> => command("sales_product_image_thumbnail_command", { request: { product_id } }).then(salesThumbnail).catch(() => ({ kind: "error", code: "persistence_failure", message: "The product image could not be loaded." })),
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
    edit: (input: CatalogEditInput) => command("edit_catalog_command", { request: input.target === CATALOG_TARGET.CATEGORY ? { target: input.target, entity_id: input.entity_id, expected_revision: input.expected_revision, name: input.name } : { target: input.target, entity_id: input.entity_id, expected_revision: input.expected_revision, expected_category_revision: input.expected_category_revision, sku: input.sku, name: input.name, purchase_price_centavos: input.purchase_price_centavos, sale_price_centavos: input.sale_price_centavos, minimum_sale_price_centavos: input.minimum_sale_price_centavos, ...(input.low_stock_threshold === undefined ? {} : { low_stock_threshold: input.low_stock_threshold }), attribute_values: input.attribute_values.map((value) => ({ definition_id: value.definition_id, value: value.value })) } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
    editCategorySchema: (input: EditCategorySchemaInput) => command("edit_category_schema_command", { request: { category_id: input.category_id, expected_revision: input.expected_revision, fields: input.fields.map((field) => ({ definition_id: field.definition_id, label: field.label, field_type: field.field_type, required: field.required, options: [...field.options] })) } }).then((value): CatalogMaintenanceResponse => { const item = maintenanceRecord(value); return record(value) && value.kind === RESPONSE_KIND.SUCCESS && item?.target === CATALOG_TARGET.CATEGORY ? { kind: RESPONSE_KIND.SUCCESS, entity_id: item.entity_id, target: item.target, label: item.label, activity: item.activity, revision: item.revision } : record(value) && value.kind === RESPONSE_KIND.ERROR ? error(value, maintainFailure) : failure(maintainFailure); }).catch(() => failure(maintainFailure)),
  };
}
export const searchProducts = createSearchProductsCommand(invoke as Invoke);
export const browseProducts = createBrowseProductsCommand(invoke as Invoke);
export const browseInventoryProducts = createInventoryBrowseProductsCommand(invoke as Invoke);
export const browseSalesProducts = createSalesBrowseProductsCommand(invoke as Invoke);
export const catalogMaintenanceCommands = createCatalogMaintenanceCommands(invoke as Invoke);
export const catalogProductImageCommands = createCatalogProductImageCommands(invoke as Invoke);

export interface ProductLocationSegment { id: number; label: string; position: number; }
export interface ProductLocationSchema { revision: number; segments: ProductLocationSegment[]; }
export interface ProductLocationRecord { location_id: number; code: string; values: string[]; active: boolean; revision: number; }
export type ProductLocationErrorCode = "catalog_access_required" | "validation_error" | "location_schema_in_use" | "duplicate_location_code" | "location_unavailable" | "location_inactive" | "location_in_use" | "stale_location" | "persistence_failure";
export type ProductLocationError = { kind: "error"; code: ProductLocationErrorCode; message: string };
export type ProductLocationResponse = { kind: "schema_success"; schema: ProductLocationSchema } | { kind: "locations_success"; locations: ProductLocationRecord[] } | { kind: "location_success"; location: ProductLocationRecord } | { kind: "assignment_success"; product_id: number; location_id: number | null; revision: number } | { kind: "deleted" } | ProductLocationError;
export interface SaveProductLocationSchemaInput { expected_revision: number; segments: string[]; }
export interface ProductLocationLifecycleInput { location_id: number; expected_revision: number; }
export interface AssignProductLocationInput { product_id: number; expected_revision: number; location_id: number | null; }

const LOCATION_ERROR_CODES: readonly ProductLocationErrorCode[] = ["catalog_access_required", "validation_error", "location_schema_in_use", "duplicate_location_code", "location_unavailable", "location_inactive", "location_in_use", "stale_location", "persistence_failure"];
const productLocationSegment = (value: unknown): ProductLocationSegment | null => responseRecord(value) && positiveSafeInteger(value.id) && typeof value.label === "string" && value.label.trim().length > 0 && nonNegativeSafeInteger(value.position) ? { id: value.id, label: value.label, position: value.position } : null;
const productLocationSchema = (value: unknown): ProductLocationSchema | null => {
  if (!responseRecord(value) || !nonNegativeSafeInteger(value.revision) || !Array.isArray(value.segments)) return null;
  const segments = value.segments.map(productLocationSegment);
  if (!segments.every((segment): segment is ProductLocationSegment => segment !== null) || segments.some((segment, index) => segment.position !== index)) return null;
  return { revision: value.revision, segments };
};
const productLocationRecord = (value: unknown): ProductLocationRecord | null => responseRecord(value) && positiveSafeInteger(value.location_id) && typeof value.code === "string" && value.code.length > 0 && Array.isArray(value.values) && value.values.every((item) => typeof item === "string") && typeof value.active === "boolean" && nonNegativeSafeInteger(value.revision) ? { location_id: value.location_id, code: value.code, values: [...value.values], active: value.active, revision: value.revision } : null;
const productLocationError = (value: RecordValue): ProductLocationError | null => LOCATION_ERROR_CODES.includes(value.code as ProductLocationErrorCode) && typeof value.message === "string" ? { kind: "error", code: value.code as ProductLocationErrorCode, message: value.code === "persistence_failure" ? "The location change could not be completed." : value.message } : null;
const decodeProductLocationResponse = (value: unknown): ProductLocationResponse | null => {
  if (!responseRecord(value) || typeof value.kind !== "string") return null;
  if (value.kind === "error") return productLocationError(value);
  if (value.kind === "schema_success") { const schema = productLocationSchema(value.schema); return schema ? { kind: value.kind, schema } : null; }
  if (value.kind === "locations_success" && Array.isArray(value.locations)) { const locations = value.locations.map(productLocationRecord); return locations.every((item): item is ProductLocationRecord => item !== null) ? { kind: value.kind, locations } : null; }
  if (value.kind === "location_success") { const location = productLocationRecord(value.location); return location ? { kind: value.kind, location } : null; }
  if (value.kind === "assignment_success" && positiveSafeInteger(value.product_id) && (value.location_id === null || positiveSafeInteger(value.location_id)) && nonNegativeSafeInteger(value.revision)) return { kind: value.kind, product_id: value.product_id, location_id: value.location_id as number | null, revision: value.revision };
  if (value.kind === "deleted") return { kind: "deleted" };
  return null;
};
const locationFailure = (): ProductLocationError => ({ kind: "error", code: "persistence_failure", message: "The location change could not be completed." });
export function createProductLocationCommands(command: Invoke) {
  const invokeLocation = (name: string, payload?: Record<string, unknown>): Promise<ProductLocationResponse> => command(name, payload).then(decodeProductLocationResponse).then((decoded) => decoded ?? locationFailure()).catch(locationFailure);
  return {
    schema: () => invokeLocation("location_schema_command"),
    saveSchema: (input: SaveProductLocationSchemaInput) => invokeLocation("save_location_schema_command", { request: { expected_revision: input.expected_revision, segments: [...input.segments] } }),
    list: (include_inactive = false) => invokeLocation("list_product_locations_command", { request: { include_inactive } }),
    create: (values: string[]) => invokeLocation("create_product_location_command", { request: { values: [...values] } }),
    activate: (input: ProductLocationLifecycleInput) => invokeLocation("activate_product_location_command", { request: { location_id: input.location_id, expected_revision: input.expected_revision } }),
    deactivate: (input: ProductLocationLifecycleInput) => invokeLocation("deactivate_product_location_command", { request: { location_id: input.location_id, expected_revision: input.expected_revision } }),
    delete: (input: ProductLocationLifecycleInput) => invokeLocation("delete_product_location_command", { request: { location_id: input.location_id, expected_revision: input.expected_revision } }),
    assignPrimary: (input: AssignProductLocationInput) => invokeLocation("assign_product_primary_location_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision, location_id: input.location_id } }),
  };
}
export function createOnboardingProductLocationCommands(command: Invoke) {
  const invokeLocation = (name: string, payload?: Record<string, unknown>): Promise<ProductLocationResponse> => command(name, payload).then(decodeProductLocationResponse).then((decoded) => decoded ?? locationFailure()).catch(locationFailure);
  return {
    schema: () => invokeLocation("onboarding_location_schema_command"),
    list: () => invokeLocation("onboarding_list_product_locations_command"),
    assignPrimary: (input: AssignProductLocationInput) => invokeLocation("onboarding_assign_product_primary_location_command", { request: { product_id: input.product_id, expected_revision: input.expected_revision, location_id: input.location_id } }),
  };
}
export const productLocationCommands = createProductLocationCommands(invoke as Invoke);
export const onboardingProductLocationCommands = createOnboardingProductLocationCommands(invoke as Invoke);

export type CatalogAccessStatus = "setup_required" | "locked" | "unlocked" | "unavailable";
export type CatalogAccessResponse = { kind: "status"; status: CatalogAccessStatus } | { kind: "recovery_code"; recovery_code: string } | { kind: "success" } | { kind: "error"; code: string; message: string };
const catalogAccessFailure = (): CatalogAccessResponse => ({ kind: "error", code: "access_unavailable", message: "No se pudo acceder a la configuración local del catálogo." });
const CATALOG_ACCESS_ERROR_CODES = ["license_required", "validation_error", "invalid_credentials", "already_configured", "setup_required", "setup_pending", "access_unavailable"] as const;
const decodeCatalogAccess = (value: unknown): CatalogAccessResponse => {
  if (!responseRecord(value) || typeof value.kind !== "string") return catalogAccessFailure();
  if (value.kind === "status" && hasOnlyKeys(value, ["kind", "status"]) && ["setup_required", "locked", "unlocked", "unavailable"].includes(String(value.status))) return { kind: "status", status: value.status as CatalogAccessStatus };
  if (value.kind === "recovery_code" && hasOnlyKeys(value, ["kind", "recovery_code"]) && typeof value.recovery_code === "string" && /^[A-F0-9]{48}$/.test(value.recovery_code)) return { kind: "recovery_code", recovery_code: value.recovery_code };
  if (value.kind === "success" && hasOnlyKeys(value, ["kind"])) return { kind: "success" };
  if (value.kind === "error" && hasOnlyKeys(value, ["kind", "code", "message"]) && typeof value.code === "string" && CATALOG_ACCESS_ERROR_CODES.includes(value.code as (typeof CATALOG_ACCESS_ERROR_CODES)[number]) && typeof value.message === "string") return { kind: "error", code: value.code, message: "No se pudo completar la solicitud de acceso al catálogo." };
  return catalogAccessFailure();
};
export function createCatalogAccessCommands(command: Invoke) {
  const call = (name: string, payload?: Record<string, unknown>) => command(name, payload).then(decodeCatalogAccess).catch(catalogAccessFailure);
  return {
    status: () => call("catalog_access_status_command"),
    beginSetup: (password: string) => call("catalog_access_begin_setup_command", { request: { password } }),
    finishSetup: () => call("catalog_access_finish_setup_command", { request: { confirmed: true } }),
    unlock: (password: string) => call("catalog_access_unlock_command", { request: { password } }),
    lock: () => call("catalog_access_lock_command"),
    changePassword: (current_password: string, new_password: string) => call("catalog_access_change_password_command", { request: { current_password, new_password } }),
    beginRecovery: (recovery_code: string, new_password: string) => call("catalog_access_begin_recovery_command", { request: { recovery_code, new_password } }),
    finishRecovery: () => call("catalog_access_finish_recovery_command", { request: { confirmed: true } }),
  };
}
export const catalogAccessCommands = createCatalogAccessCommands(invoke as Invoke);
