import { invoke } from "@tauri-apps/api/core";

export const FIELD_TYPE = {
  TEXT: "text",
  NUMBER: "number",
  OPTION: "option",
} as const;

export type FieldType = (typeof FIELD_TYPE)[keyof typeof FIELD_TYPE];

export interface CategoryFieldInput {
  label: string;
  field_type: FieldType;
  required: boolean;
  options: string[];
}

export interface CreateCategoryInput {
  name: string;
  fields: CategoryFieldInput[];
}

export interface CategoryField {
  definition_id: number;
  label: string;
  field_type: FieldType;
  required: boolean;
  options: string[];
}

export interface Category {
  category_id: number;
  name: string;
  fields: CategoryField[];
}

export interface AttributeValueInput {
  definition_id: number;
  value: string;
}

export interface CreateProductInput {
  sku: string;
  name: string;
  category_id: number;
  list_price_centavos: number;
  minimum_sale_price_centavos: number;
  opening_quantity: number;
  attribute_values: AttributeValueInput[];
}

export interface CreatedProduct {
  product_id: number;
  sku: string;
  name: string;
  category_id: number;
  category_name: string;
  list_price_centavos: number;
  minimum_sale_price_centavos: number;
  available_quantity: number;
  active: boolean;
}

export interface OnboardingError {
  kind: "error";
  code: string;
  message: string;
}

export type CreateCategoryResponse =
  | ({ kind: "success" } & Category)
  | OnboardingError;

export type CreateProductResponse =
  | ({ kind: "success" } & CreatedProduct)
  | OnboardingError;

export interface ListCategoriesSuccess {
  kind: "success";
  categories: Category[];
}

export type ListCategoriesResponse = ListCategoriesSuccess | OnboardingError;

type Invoke = (command: string, payload?: unknown) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null && !Array.isArray(value);
const safeInteger = (value: unknown): value is number => typeof value === "number" && Number.isSafeInteger(value);
const decodedArray = <T>(value: unknown, decode: (item: unknown) => T | null): T[] | null => Array.isArray(value) && value.every((item) => decode(item) !== null) ? value.map((item) => decode(item) as T) : null;
const categoryField = (value: unknown): CategoryField | null => record(value) && safeInteger(value.definition_id) && typeof value.label === "string" && (value.field_type === FIELD_TYPE.TEXT || value.field_type === FIELD_TYPE.NUMBER || value.field_type === FIELD_TYPE.OPTION) && typeof value.required === "boolean" && Array.isArray(value.options) && value.options.every((option) => typeof option === "string") ? { definition_id: value.definition_id, label: value.label, field_type: value.field_type, required: value.required, options: [...value.options] } : null;
const category = (value: unknown): Category | null => record(value) && safeInteger(value.category_id) && typeof value.name === "string" && decodedArray(value.fields, categoryField) !== null ? { category_id: value.category_id, name: value.name, fields: decodedArray(value.fields, categoryField) as CategoryField[] } : null;
const createdProduct = (value: unknown): CreatedProduct | null => record(value) && safeInteger(value.product_id) && typeof value.sku === "string" && typeof value.name === "string" && safeInteger(value.category_id) && typeof value.category_name === "string" && safeInteger(value.list_price_centavos) && safeInteger(value.minimum_sale_price_centavos) && safeInteger(value.available_quantity) && typeof value.active === "boolean" ? { product_id: value.product_id, sku: value.sku, name: value.name, category_id: value.category_id, category_name: value.category_name, list_price_centavos: value.list_price_centavos, minimum_sale_price_centavos: value.minimum_sale_price_centavos, available_quantity: value.available_quantity, active: value.active } : null;
const generic = (message: string): OnboardingError => ({ kind: "error", code: "persistence_failure", message });
const errorMessages: Record<string, string> = {
  invalid_category: "Category name is required.", invalid_field_definition: "Category field definitions are invalid.", duplicate_category: "Category name already exists.",
  invalid_product: "SKU and product name are required.", missing_category: "The selected category was not found.", duplicate_sku: "SKU already exists.", invalid_list_price: "List price must be a positive whole number of centavos.", invalid_minimum_sale_price: "Minimum sale price must be a positive whole number of centavos.", minimum_sale_price_exceeds_list_price: "Minimum sale price cannot exceed list price.", invalid_opening_quantity: "Opening stock must be a positive whole number.", missing_required_field: "A required category field is missing.", invalid_attribute_value: "A category field value is invalid.", persistence_failure: "The operation could not be persisted.",
};
const boundedError = (value: unknown, codes: string[], fallback: string): OnboardingError => record(value) && typeof value.code === "string" && typeof value.message === "string" && codes.includes(value.code) ? { kind: "error", code: value.code, message: value.code === "persistence_failure" ? fallback : errorMessages[value.code] } : generic(fallback);
const listResponse = (value: unknown): ListCategoriesResponse => record(value) && value.kind === "success" && decodedArray(value.categories, category) !== null ? { kind: "success", categories: decodedArray(value.categories, category) as Category[] } : record(value) && value.kind === "error" ? boundedError(value, ["persistence_failure"], "Categories could not be loaded.") : generic("Categories could not be loaded.");
const categoryResponse = (value: unknown, codes: string[], fallback: string): CreateCategoryResponse => record(value) && value.kind === "success" && category(value) ? { kind: "success", ...(category(value) as Category) } : record(value) && value.kind === "error" ? boundedError(value, codes, fallback) : generic(fallback);
const productResponse = (value: unknown): CreateProductResponse => record(value) && value.kind === "success" && createdProduct(value) ? { kind: "success", ...(createdProduct(value) as CreatedProduct) } : record(value) && value.kind === "error" ? boundedError(value, ["invalid_product", "missing_category", "duplicate_sku", "invalid_list_price", "invalid_minimum_sale_price", "minimum_sale_price_exceeds_list_price", "invalid_opening_quantity", "missing_required_field", "invalid_attribute_value", "persistence_failure"], "The product could not be persisted.") : generic("The product could not be persisted.");

export function createListCategoriesCommand(command: Invoke) {
  return async (): Promise<ListCategoriesResponse> => { try { return listResponse(await command("list_categories_command")); } catch { return generic("Categories could not be loaded."); } };
}

export function createCreateCategoryCommand(command: Invoke) {
  return async (request: CreateCategoryInput): Promise<CreateCategoryResponse> => { try { return categoryResponse(await command("create_category_command", { request }), ["invalid_category", "invalid_field_definition", "duplicate_category", "persistence_failure"], "The category could not be persisted."); } catch { return generic("The category could not be persisted."); } };
}

export function createCreateProductCommand(command: Invoke) {
  return async (request: CreateProductInput): Promise<CreateProductResponse> => { try { return productResponse(await command("create_product_command", { request })); } catch { return generic("The product could not be persisted."); } };
}

export const listCategories = createListCategoriesCommand(invoke as Invoke);
export const createCategory = createCreateCategoryCommand(invoke as Invoke);
export const createProduct = createCreateProductCommand(invoke as Invoke);
