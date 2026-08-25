import assert from "node:assert/strict";
import { test } from "node:test";

import type { ProductSearchResult } from "../../commands/catalog.ts";
import { catalogResultDetails } from "./catalog-result";

const brakePad: ProductSearchResult = {
  product_id: 1,
  sku: "BP-100",
  name: "Brake Pad",
  category_name: "Brakes",
  available_quantity: 4,
  catalog_unit_price_centavos: 2_500,
};

test("presents catalog price as draft guidance from the typed search result", () => {
  assert.equal(
    catalogResultDetails(brakePad),
    "BP-100 — Brake Pad — Brakes — catalog price guidance Bs 25.00 (4 in stock)",
  );
});
