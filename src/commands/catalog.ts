import { invoke } from "@tauri-apps/api/core";

export type ProductSearchResult = {
  product_id: number;
  sku: string;
  name: string;
  category_name: string;
  available_quantity: number;
  catalog_unit_price_centavos: number;
};

type Invoke = (command: string, payload: unknown) => Promise<unknown>;

export function createSearchProductsCommand(command: Invoke) {
  return (query: string) =>
    command("search_products_command", { request: { query } }) as Promise<
      ProductSearchResult[]
    >;
}

export const searchProducts = createSearchProductsCommand(invoke as Invoke);
