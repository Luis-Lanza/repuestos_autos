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
  catalog_unit_price_centavos: number;
  opening_quantity: number;
  attribute_values: AttributeValueInput[];
}

export interface CreatedProduct {
  product_id: number;
  sku: string;
  name: string;
  category_id: number;
  category_name: string;
  catalog_unit_price_centavos: number;
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

export function createListCategoriesCommand(command: Invoke) {
  return () =>
    command("list_categories_command") as Promise<ListCategoriesResponse>;
}

export function createCreateCategoryCommand(command: Invoke) {
  return (request: CreateCategoryInput) =>
    command("create_category_command", { request }) as Promise<CreateCategoryResponse>;
}

export function createCreateProductCommand(command: Invoke) {
  return (request: CreateProductInput) =>
    command("create_product_command", { request }) as Promise<CreateProductResponse>;
}

export const listCategories = createListCategoriesCommand(invoke as Invoke);
export const createCategory = createCreateCategoryCommand(invoke as Invoke);
export const createProduct = createCreateProductCommand(invoke as Invoke);
