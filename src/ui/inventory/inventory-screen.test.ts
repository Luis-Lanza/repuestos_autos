import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { createSearchProductsCommand } from "../../commands/catalog.ts";
import { createInventoryCatalogInteraction, InventoryOperationChoices, inventoryScreenDescription } from "./inventory-screen.ts";
import { createInventoryFlow, initialInventoryState } from "./inventory-flow.ts";

test("describes stock entry and physical-count workflows for accessible navigation", () => {
  assert.match(inventoryScreenDescription, /stock entry/i);
  assert.match(inventoryScreenDescription, /physical count/i);
});

test("runs Catalog search, Select, and renders both inventory operation choices", async () => {
  const product = { product_id: 1, sku: "FLT", name: "Filter", available_quantity: 8, category_name: "Filters", catalog_unit_price_centavos: 2500 };
  const calls: unknown[] = [];
  const searchActiveProducts = createSearchProductsCommand(async (_command, payload) => { calls.push(payload); return [product]; });
  const interaction = createInventoryCatalogInteraction(searchActiveProducts);
  const [result] = await interaction.search("filter");
  const selected = createInventoryFlow(initialInventoryState, interaction.select(result));
  const screen = renderToStaticMarkup(createElement(InventoryOperationChoices, { operation: selected.operation, onChange: () => undefined }));
  assert.deepEqual(calls, [{ request: { query: "filter" } }]);
  assert.equal(selected.product?.product_id, product.product_id);
  assert.match(screen, /Stock entry/);
  assert.match(screen, /Physical count/);
});
