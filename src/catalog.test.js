import assert from "node:assert/strict";
import test from "node:test";

import { createSearchProductsCommand } from "./commands/catalog.ts";

test("returns catalog prices through the catalog command seam", async () => {
  const searchProducts = createSearchProductsCommand(
    async (command, payload) => {
      assert.equal(command, "search_products_command");
      assert.deepEqual(payload, { request: { query: "filter" } });
      return [
        {
          product_id: 1,
          sku: "FIL-1",
          name: "Oil filter",
          category_name: "Filters",
          available_quantity: 4,
          catalog_unit_price_centavos: 2_500,
        },
      ];
    },
  );

  const products = await searchProducts("filter");

  assert.equal(products[0].catalog_unit_price_centavos, 2_500);
  assert.equal("minimum_unit_price_centavos" in products[0], false);
});
