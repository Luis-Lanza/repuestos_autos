import assert from "node:assert/strict";
import { test } from "node:test";

import type { ProductSearchResult } from "../../commands/catalog.ts";
import { catalogResultDetails } from "./catalog-result";

const brakePad: ProductSearchResult = {
  product_id: 1,
  sku: "BP-100",
  name: "Pastilla de freno",
  category_name: "Frenos",
  available_quantity: 4,
  catalog_unit_price_centavos: 2_509,
  sale_price_centavos: 2_509,
  list_price_centavos: 2_509,
  minimum_sale_price_centavos: 2_509,
  revision: 0,
};

test("projects typed catalog identity and exact immutable Bs price", () => {
  const result = catalogResultDetails(brakePad);

  assert.deepEqual(result.product, {
    id: 1,
    name: "Pastilla de freno",
    sku: "BP-100",
    category: "Frenos",
  });
  assert.deepEqual(result.price, {
    kind: "immutable",
    centavos: 2_509,
    text: "Bs 25,09",
  });
});

test("prefers canonical sale price while retaining legacy aliases as fallback", () => {
  assert.equal(catalogResultDetails({ ...brakePad, sale_price_centavos: 3_000, list_price_centavos: 2_509, catalog_unit_price_centavos: 2_000 }).price.centavos, 3_000);
  const legacy = { ...brakePad, sale_price_centavos: undefined, list_price_centavos: 2_509 } as unknown as ProductSearchResult;
  assert.equal(catalogResultDetails(legacy).price.centavos, 2_509);
});

test("projects available, low, and zero stock as distinct typed facts", () => {
  assert.deepEqual(catalogResultDetails(brakePad).stock, {
    kind: "available",
    quantity: 4,
    text: "Disponible: 4",
  });
  assert.deepEqual(
    catalogResultDetails({ ...brakePad, available_quantity: 1 }).stock,
    { kind: "low", quantity: 1, text: "Stock bajo: 1" },
  );
  assert.deepEqual(
    catalogResultDetails({ ...brakePad, available_quantity: 0 }).stock,
    { kind: "out", quantity: 0, text: "Sin stock: 0" },
  );
});

test("expresses in-cart and zero-stock availability without inventing actions", () => {
  assert.equal(catalogResultDetails(brakePad).availability, "available");
  assert.equal(
    catalogResultDetails(brakePad, { inCart: true }).availability,
    "in_cart",
  );
  assert.equal(
    catalogResultDetails({ ...brakePad, available_quantity: 0 }, { inCart: true })
      .availability,
    "out_of_stock",
  );
  assert.equal("action" in catalogResultDetails(brakePad), false);
});
