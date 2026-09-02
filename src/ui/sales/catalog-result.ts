import type { ProductSearchResult } from "../../commands/catalog.ts";
import { formatBs } from "./sale-flow.ts";

export type CatalogStockProjection =
  | { kind: "available"; quantity: number; text: string }
  | { kind: "low"; quantity: 1; text: "Stock bajo: 1" }
  | { kind: "out"; quantity: 0; text: "Sin stock: 0" };

export type CatalogResultProjection = {
  product: {
    id: number;
    name: string;
    sku: string;
    category: string;
  };
  price: {
    kind: "immutable";
    centavos: number;
    text: string;
  };
  stock: CatalogStockProjection;
  availability: "available" | "in_cart" | "out_of_stock";
};

function stockProjection(quantity: number): CatalogStockProjection {
  if (quantity === 0) return { kind: "out", quantity: 0, text: "Sin stock: 0" };
  if (quantity === 1)
    return { kind: "low", quantity: 1, text: "Stock bajo: 1" };
  return { kind: "available", quantity, text: `Disponible: ${quantity}` };
}

export function catalogResultDetails(
  product: ProductSearchResult,
  context: { inCart?: boolean } = {},
): CatalogResultProjection {
  return {
    product: {
      id: product.product_id,
      name: product.name,
      sku: product.sku,
      category: product.category_name,
    },
    price: {
      kind: "immutable",
      centavos: product.catalog_unit_price_centavos,
      text: formatBs(product.catalog_unit_price_centavos),
    },
    stock: stockProjection(product.available_quantity),
    availability:
      product.available_quantity === 0
        ? "out_of_stock"
        : context.inCart
          ? "in_cart"
          : "available",
  };
}
