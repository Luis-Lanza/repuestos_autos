import type { ProductSearchResult } from "../../commands/catalog.ts";

function formatBolivianos(centavos: number): string {
  return `Bs ${(centavos / 100).toFixed(2)}`;
}

export function catalogResultDetails(product: ProductSearchResult): string {
  return `${product.sku} — ${product.name} — ${product.category_name} — catalog ${formatBolivianos(product.catalog_unit_price_centavos)} (${product.available_quantity} in stock)`;
}
