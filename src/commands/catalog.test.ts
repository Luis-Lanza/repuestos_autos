import assert from "node:assert/strict";
import test from "node:test";

import { CATALOG_INTENT, CATALOG_TARGET, createBrowseProductsCommand, createCatalogMaintenanceCommands, createSearchProductsCommand } from "./catalog.ts";

const searchProduct = { product_id: 1, sku: "FLT-1", name: "Filter", category_name: "Engine", available_quantity: 4, catalog_unit_price_centavos: 2500, list_price_centavos: 2500, minimum_sale_price_centavos: 2500, revision: 2 };
const browsePage = { kind: "success", products: [{ ...searchProduct, category_id: 9 }], categories: [{ category_id: 9, name: "Engine" }], page: 1, page_size: 20, total: 1, total_pages: 1 };

test("decodes the paged browse contract and sends optional filters in its request envelope", async () => {
  const calls: unknown[] = [];
  const browse = createBrowseProductsCommand(async (command, payload) => { calls.push({ command, payload }); return browsePage; });
  assert.deepEqual(await browse({ query: "  filter ", category_id: 9, stock_state: "low_stock", page: 2, page_size: 20 }), { products: [{ ...searchProduct, category_id: 9 }], categories: [{ category_id: 9, name: "Engine" }], page: 1, page_size: 20, total: 1, total_pages: 1 });
  assert.deepEqual(calls, [{ command: "browse_products_command", payload: { request: { query: "filter", category_id: 9, stock_state: "low_stock", activity: "active", page: 2, page_size: 20 } } }]);
});

test("rejects malformed paged browse responses without exposing native details", async () => {
  for (const response of [null, { ...browsePage, products: [{ ...searchProduct, category_id: 0 }] }, { ...browsePage, total: -1 }, { ...browsePage, page_size: 51 }]) {
    const browse = createBrowseProductsCommand(async () => response);
    await assert.rejects(browse(), /product catalog/);
  }
});

test("projects search products and strips native fields", async () => {
  const calls: unknown[] = [];
  const search = createSearchProductsCommand(async (command, payload) => { calls.push({ command, payload }); return [{ ...searchProduct, internal: "hidden" }]; });
  assert.deepEqual(await search("filter"), [searchProduct]);
  assert.deepEqual(calls, [{ command: "search_products_command", payload: { request: { query: "filter" } } }]);
});

test("uses the true list price as the compatibility catalog price and keeps the minimum", async () => {
  const search = createSearchProductsCommand(async () => [{
    product_id: 1, sku: "FLT-1", name: "Filter", category_name: "Engine", available_quantity: 4,
    catalog_unit_price_centavos: 5000, list_price_centavos: 5000, minimum_sale_price_centavos: 2500, revision: 2,
  }]);
  assert.deepEqual(await search("filter"), [{
    product_id: 1, sku: "FLT-1", name: "Filter", category_name: "Engine", available_quantity: 4,
    catalog_unit_price_centavos: 5000, list_price_centavos: 5000, minimum_sale_price_centavos: 2500, revision: 2,
  }]);
});

test("rejects malformed search arrays, unsafe numbers, and native rejection text", async () => {
  for (const response of [null, {}, [{ ...searchProduct, name: 4 }], [{ ...searchProduct, available_quantity: Number.MAX_SAFE_INTEGER + 1 }], [{ ...searchProduct, revision: 1.5 }], [{ ...searchProduct }, { ...searchProduct, sku: undefined }]]) {
    const search = createSearchProductsCommand(async () => response);
    await assert.rejects(search("filter"), /product search/);
  }
  const rejected = createSearchProductsCommand(async () => { throw new Error("SQL path /panic details"); });
  await assert.rejects(rejected("filter"), (error: unknown) => error instanceof Error && error.message === "The product search could not be completed." && !error.message.includes("SQL"));
});

test("allowlists maintenance payloads and preserves only stable opaque outcomes", async () => {
  const calls: unknown[] = [];
  const commands = createCatalogMaintenanceCommands(async (command, payload) => {
    calls.push({ command, payload });
    return { kind: "error", code: "stale_catalog_record", message: "SQLite busy: internal details" };
  });
  const result = await commands.maintain({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, intent: CATALOG_INTENT.ARCHIVE, expected_revision: 0, ignored: true } as never);
  assert.deepEqual(calls, [{ command: "maintain_catalog_command", payload: { request: { target: "product", entity_id: 1, intent: "archive", expected_revision: 0 } } }]);
  assert.deepEqual(result, { kind: "error", code: "stale_catalog_record", message: "This catalog record changed. Reload and try again." });
});

test("maps malformed maintenance results and invoke failures to an opaque failure", async () => {
  const malformed = createCatalogMaintenanceCommands(async () => ({ kind: "success", sql: "never expose" }));
  const unavailable = createCatalogMaintenanceCommands(async () => { throw new Error("SQLite details"); });
  assert.deepEqual(await malformed.list(), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
  assert.deepEqual(await unavailable.list(), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
});

test("uses the bounded category-maintenance path separately from product browsing", async () => {
  const calls: string[] = [];
  const commands = createCatalogMaintenanceCommands(async (command) => { calls.push(command); return { kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0, product_count: 999 }] }; });
  assert.deepEqual(await commands.listCategories(), { kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0 }] });
  assert.deepEqual(calls, ["list_catalog_categories_command"]);
});

test("projects successful maintenance records without backend-only fields", async () => {
  const commands = createCatalogMaintenanceCommands(async (command) => command === "list_catalog_maintenance_command" ? { kind: "success", records: [{ entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 0, sql: "hidden" }] } : { kind: "success", entity_id: 1, target: "product", label: "Filter", activity: "archived", revision: 1, sql: "hidden" });
  assert.deepEqual(await commands.list(), { kind: "success", records: [{ entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 0 }] });
  assert.deepEqual(await commands.maintain({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, intent: CATALOG_INTENT.ARCHIVE, expected_revision: 0 }), { kind: "success", entity_id: 1, target: "product", label: "Filter", activity: "archived", revision: 1 });
});

test("allowlists metadata detail and edit payloads with typed values", async () => {
  const calls: unknown[] = [];
  const commands = createCatalogMaintenanceCommands(async (command, payload) => {
    calls.push({ command, payload });
    return command === "catalog_metadata_detail_command"
      ? { target: "product", entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", list_price_centavos: 3000, minimum_sale_price_centavos: 2500, activity: "archived", revision: 3, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option", required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }], sql: "hidden" }
      : { kind: "success", entity_id: 1, target: "product", label: "", activity: "archived", revision: 4, sql: "hidden" };
  });
  const detail = await commands.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, ignored: true } as never);
  const edited = await commands.edit({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, expected_revision: 3, sku: "FLT", name: "Filter", list_price_centavos: 3000, minimum_sale_price_centavos: 2500, attribute_values: [{ definition_id: 4, value: "Paper", ignored: true }], ignored: true } as never);
  assert.deepEqual(calls, [
    { command: "catalog_metadata_detail_command", payload: { request: { target: "product", entity_id: 1 } } },
    { command: "edit_catalog_command", payload: { request: { target: "product", entity_id: 1, expected_revision: 3, sku: "FLT", name: "Filter", list_price_centavos: 3000, minimum_sale_price_centavos: 2500, attribute_values: [{ definition_id: 4, value: "Paper" }] } } },
  ]);
  assert.deepEqual(detail, { kind: "success", detail: { target: "product", entity_id: 1, category_id: 2, sku: "FLT", name: "Filter", list_price_centavos: 3000, minimum_sale_price_centavos: 2500, activity: "archived", revision: 3, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option", required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }] } });
  assert.deepEqual(edited, { kind: "success", entity_id: 1, target: "product", label: "", activity: "archived", revision: 4 });
});

test("projects category detail and rejects malformed detail payloads", async () => {
  const category = createCatalogMaintenanceCommands(async () => ({ target: "category", entity_id: 2, name: "Filters", activity: "active", revision: 1, attribute_definitions: [], sql: "hidden" }));
  const malformed = createCatalogMaintenanceCommands(async () => ({ target: "product", entity_id: 1, name: "Filter" }));
  assert.deepEqual(await category.detail({ target: CATALOG_TARGET.CATEGORY, entity_id: 2 }), { kind: "success", detail: { target: "category", entity_id: 2, name: "Filters", activity: "active", revision: 1, attribute_definitions: [] } });
  assert.deepEqual(await malformed.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: 1 }), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
});

test("rejects unsafe, nonpositive, and inconsistent catalog facts", async () => {
  const valid = { ...searchProduct };
  const invalidProducts = [
    { ...valid, product_id: 0 },
    { ...valid, product_id: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, list_price_centavos: 0 },
    { ...valid, list_price_centavos: undefined },
    { ...valid, list_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, minimum_sale_price_centavos: -1 },
    { ...valid, minimum_sale_price_centavos: undefined },
    { ...valid, minimum_sale_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, catalog_unit_price_centavos: -1 },
    { ...valid, available_quantity: -1 },
    { ...valid, revision: -1 },
    { ...valid, revision: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, list_price_centavos: 4_000, minimum_sale_price_centavos: 5_000 },
  ];
  for (const response of invalidProducts) {
    const search = createSearchProductsCommand(async () => [response]);
    await assert.rejects(search("filter"), /product search/);
  }
});
