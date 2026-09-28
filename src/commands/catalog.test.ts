import assert from "node:assert/strict";
import test from "node:test";

import { CATALOG_INTENT, CATALOG_TARGET, createBrowseProductsCommand, createCatalogMaintenanceCommands, createSearchProductsCommand, createCatalogProductImageCommands, createProductLocationCommands } from "./catalog.ts";

const searchProduct = { product_id: 1, sku: "FLT-1", name: "Filter", category_name: "Engine", available_quantity: 4, purchase_price_centavos: 1800, sale_price_centavos: 2500, list_price_centavos: 2500, catalog_unit_price_centavos: 2500, minimum_sale_price_centavos: 2500, revision: 2 };
const browsePage = { kind: "success", products: [{ ...searchProduct, category_id: 9, primary_location_code: null, attribute_values: [] }], categories: [{ category_id: 9, name: "Engine" }], page: 1, page_size: 20, total: 1, total_pages: 1 };

test("decodes the paged browse contract and sends optional filters in its request envelope", async () => {
  const calls: unknown[] = [];
  const browse = createBrowseProductsCommand(async (command, payload) => { calls.push({ command, payload }); return { ...browsePage, products: [{ ...browsePage.products[0], original_image_base64: "/9j/secret", source_path: "/private/image.jpg" }] }; });
  assert.deepEqual(await browse({ query: "  filter ", category_id: 9, stock_state: "low_stock", page: 2, page_size: 20 }), { products: [{ ...searchProduct, category_id: 9, primary_location_code: null, attribute_values: [] }], categories: [{ category_id: 9, name: "Engine" }], page: 1, page_size: 20, total: 1, total_pages: 1 });
  assert.deepEqual(calls, [{ command: "browse_products_command", payload: { request: { query: "filter", category_id: 9, stock_state: "low_stock", activity: "active", page: 2, page_size: 20 } } }]);
});

test("decodes ordered compact browse attributes including empty values without projection drift", async () => {
  const attributes = [{ definition_id: 2, label: "Material", value: "" }, { definition_id: 8, label: "Diameter", value: "50" }];
  const browse = createBrowseProductsCommand(async () => ({ ...browsePage, products: [{ ...browsePage.products[0], primary_location_code: "A1-SHELF2", attribute_values: attributes, internal: "not projected" }] }));
  assert.deepEqual((await browse()).products[0], { ...searchProduct, category_id: 9, primary_location_code: "A1-SHELF2", attribute_values: attributes });
  assert.equal(Object.hasOwn((await browse()).products[0], "internal"), false);
});

test("rejects malformed paged browse responses without exposing native details", async () => {
  for (const response of [null, { ...browsePage, products: [{ ...searchProduct, category_id: 0, attribute_values: [] }] }, { ...browsePage, products: [{ ...browsePage.products[0], attribute_values: [{ definition_id: 1, value: "missing label" }] }] }, { ...browsePage, products: [{ ...browsePage.products[0], attribute_values: [{ definition_id: 1, label: "Size", value: "M", internal: true }] }] }, { ...browsePage, total: -1 }, { ...browsePage, page_size: 51 }]) {
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

test("decodes legacy list-price aliases as sale prices and projects purchase cost", async () => {
  const search = createSearchProductsCommand(async () => [{
    product_id: 1, sku: "FLT-1", name: "Filter", category_name: "Engine", available_quantity: 4,
    catalog_unit_price_centavos: 5000, list_price_centavos: 5000, purchase_price_centavos: 2_000, minimum_sale_price_centavos: 2500, revision: 2,
  }]);
  assert.deepEqual(await search("filter"), [{
    product_id: 1, sku: "FLT-1", name: "Filter", category_name: "Engine", available_quantity: 4,
    purchase_price_centavos: 2_000, sale_price_centavos: 5_000, list_price_centavos: 5_000, catalog_unit_price_centavos: 5_000, minimum_sale_price_centavos: 2500, revision: 2,
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
  const commands = createCatalogMaintenanceCommands(async (command) => { calls.push(command); return { kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0, active_product_count: 3, product_count: 999 }] }; });
  assert.deepEqual(await commands.listCategories(), { kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0, active_product_count: 3 }] });
  assert.deepEqual(calls, ["list_catalog_categories_command"]);
});

test("decodes authoritative active-product counts only on category metadata records", async () => {
  const commands = createCatalogMaintenanceCommands(async () => ({ kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0, active_product_count: 4 }] }));
  assert.deepEqual(await commands.listCategories(), { kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0, active_product_count: 4 }] });
  for (const count of [-1, 1.5, Number.MAX_SAFE_INTEGER + 1, undefined]) {
    const malformed = createCatalogMaintenanceCommands(async () => ({ kind: "success", records: [{ entity_id: 2, target: "category", label: "Filters", activity: "active", revision: 0, active_product_count: count }] }));
    assert.deepEqual(await malformed.listCategories(), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
  }
});

test("projects successful maintenance records without backend-only fields", async () => {
  const commands = createCatalogMaintenanceCommands(async (command) => command === "list_catalog_maintenance_command" ? { kind: "success", records: [{ entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 0, sql: "hidden" }] } : { kind: "success", entity_id: 1, target: "product", label: "Filter", activity: "archived", revision: 1, sql: "hidden" });
  assert.deepEqual(await commands.list(), { kind: "success", records: [{ entity_id: 1, target: "product", label: "Filter", activity: "active", revision: 0 }] });
  assert.deepEqual(await commands.maintain({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, intent: CATALOG_INTENT.ARCHIVE, expected_revision: 0 }), { kind: "success", entity_id: 1, target: "product", label: "Filter", activity: "archived", revision: 1 });
});

test("allowlists metadata detail and edit payloads with typed pricing values", async () => {
  const calls: unknown[] = [];
  const commands = createCatalogMaintenanceCommands(async (command, payload) => {
    calls.push({ command, payload });
    return command === "catalog_metadata_detail_command"
      ? { target: "product", entity_id: 1, category_id: 2, category_revision: 6, sku: "FLT", name: "Filter", list_price_centavos: 3000, purchase_price_centavos: 1200, minimum_sale_price_centavos: 2500, primary_location_id: null, activity: "archived", revision: 3, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option", required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }], sql: "hidden" }
      : { kind: "success", entity_id: 1, target: "product", label: "", activity: "archived", revision: 4, sql: "hidden" };
  });
  const detail = await commands.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, ignored: true } as never);
  const edited = await commands.edit({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, expected_revision: 3, expected_category_revision: 2, sku: "FLT", name: "Filter", purchase_price_centavos: 1200, sale_price_centavos: 3000, minimum_sale_price_centavos: 2500, attribute_values: [{ definition_id: 4, value: "Paper", ignored: true }], ignored: true } as never);
  assert.deepEqual(calls, [
    { command: "catalog_metadata_detail_command", payload: { request: { target: "product", entity_id: 1 } } },
    { command: "edit_catalog_command", payload: { request: { target: "product", entity_id: 1, expected_revision: 3, expected_category_revision: 2, sku: "FLT", name: "Filter", purchase_price_centavos: 1200, sale_price_centavos: 3000, minimum_sale_price_centavos: 2500, attribute_values: [{ definition_id: 4, value: "Paper" }] } } },
  ]);
  assert.deepEqual(detail, { kind: "success", detail: { target: "product", entity_id: 1, category_id: 2, category_revision: 6, sku: "FLT", name: "Filter", purchase_price_centavos: 1200, sale_price_centavos: 3000, minimum_sale_price_centavos: 2500, primary_location_id: null, activity: "archived", revision: 3, attribute_definitions: [{ definition_id: 4, label: "Material", field_type: "option", required: true, options: ["Paper"] }], attribute_values: [{ definition_id: 4, value: "Paper" }] } });
  assert.deepEqual(edited, { kind: "success", entity_id: 1, target: "product", label: "", activity: "archived", revision: 4 });
});

test("projects category detail and rejects malformed detail payloads", async () => {
  const category = createCatalogMaintenanceCommands(async () => ({ target: "category", entity_id: 2, name: "Filters", activity: "active", revision: 1, attribute_definitions: [], sql: "hidden" }));
  const malformed = createCatalogMaintenanceCommands(async () => ({ target: "product", entity_id: 1, name: "Filter" }));
  assert.deepEqual(await category.detail({ target: CATALOG_TARGET.CATEGORY, entity_id: 2 }), { kind: "success", detail: { target: "category", entity_id: 2, name: "Filters", activity: "active", revision: 1, attribute_definitions: [] } });
  assert.deepEqual(await malformed.detail({ target: CATALOG_TARGET.PRODUCT, entity_id: 1 }), { kind: "error", code: "persistence_failure", message: "The catalog could not be loaded." });
});

test("decodes picker cancellation and revision-checked image mutations without paths", async () => {
  const calls: unknown[] = [];
  let result: unknown = { kind: "cancelled" };
  const images = createCatalogProductImageCommands(async (command, payload) => { calls.push({ command, payload }); return result; });
  assert.deepEqual(await images.choose({ product_id: 1, expected_revision: 7 }), { kind: "cancelled" });
  result = { kind: "success", product_id: 1, revision: 8, path: "/private/image.jpg" };
  assert.deepEqual(await images.choose({ product_id: 1, expected_revision: 7 }), { kind: "error", code: "persistence_failure", message: "The product image could not be updated." });
  result = { kind: "success", product_id: 1, revision: 8 };
  assert.deepEqual(await images.remove({ product_id: 1, expected_revision: 7 }), { kind: "success", product_id: 1, revision: 8 });
  assert.deepEqual(calls, [
    { command: "choose_product_image_command", payload: { request: { product_id: 1, expected_revision: 7 } } },
    { command: "choose_product_image_command", payload: { request: { product_id: 1, expected_revision: 7 } } },
    { command: "remove_product_image_command", payload: { request: { product_id: 1, expected_revision: 7 } } },
  ]);
});

test("decodes only bounded canonical JPEG thumbnail data", async () => {
  const images = createCatalogProductImageCommands(async () => ({ kind: "success", product_id: 1, revision: 7, mime_type: "image/jpeg", encoding: "base64", bytes: "/9j/2Q==" }));
  assert.deepEqual(await images.thumbnail({ product_id: 1, expected_revision: 7 }), { kind: "success", product_id: 1, revision: 7, src: "data:image/jpeg;base64,/9j/2Q==" });
  for (const response of [
    { kind: "success", product_id: 1, revision: 7, mime_type: "image/png", encoding: "base64", bytes: "/9j/2Q==" },
    { kind: "success", product_id: 1, revision: 7, mime_type: "image/jpeg", encoding: "base64", bytes: "%%%=" },
    { kind: "success", product_id: 1, revision: 7, mime_type: "image/jpeg", encoding: "base64", bytes: "/9j/" + "A".repeat(2_796_204) },
    { kind: "success", product_id: 1, revision: 7, mime_type: "image/jpeg", encoding: "base64", bytes: "/9j/2R==" },
    { kind: "success", product_id: 1, revision: 7, mime_type: "image/jpeg", encoding: "base64", bytes: "/9j/2Q==", path: "/private/image.jpg" },
  ]) {
    const malformed = createCatalogProductImageCommands(async () => response);
    assert.deepEqual(await malformed.thumbnail({ product_id: 1, expected_revision: 7 }), { kind: "error", code: "persistence_failure", message: "The product image could not be loaded." });
  }
});

test("location command contracts send narrow request envelopes and decode stable outcomes", async () => {
  const calls: unknown[] = [];
  const responses: unknown[] = [
    { kind: "schema_success", schema: { revision: 2, segments: [{ id: 4, label: "Zone", position: 0 }] } },
    { kind: "schema_success", schema: { revision: 3, segments: [{ id: 4, label: "Zone", position: 0 }, { id: 5, label: "Shelf", position: 1 }] } },
    { kind: "location_success", location: { location_id: 7, code: "A1-SHELF", values: ["A1", "Shelf"], active: true, revision: 0 } },
    { kind: "assignment_success", product_id: 3, location_id: null, revision: 8 },
    { kind: "deleted" },
  ];
  const locations = createProductLocationCommands(async (name, payload) => { calls.push({ name, payload }); return responses.shift(); });
  assert.deepEqual(await locations.schema(), { kind: "schema_success", schema: { revision: 2, segments: [{ id: 4, label: "Zone", position: 0 }] } });
  assert.deepEqual(await locations.saveSchema({ expected_revision: 2, segments: ["Zone", "Shelf"] }), { kind: "schema_success", schema: { revision: 3, segments: [{ id: 4, label: "Zone", position: 0 }, { id: 5, label: "Shelf", position: 1 }] } });
  assert.deepEqual(await locations.create(["A1", "Shelf"]), { kind: "location_success", location: { location_id: 7, code: "A1-SHELF", values: ["A1", "Shelf"], active: true, revision: 0 } });
  assert.deepEqual(await locations.assignPrimary({ product_id: 3, expected_revision: 7, location_id: null }), { kind: "assignment_success", product_id: 3, location_id: null, revision: 8 });
  assert.deepEqual(await locations.delete({ location_id: 7, expected_revision: 0 }), { kind: "deleted" });
  assert.deepEqual(calls, [
    { name: "location_schema_command", payload: undefined },
    { name: "save_location_schema_command", payload: { request: { expected_revision: 2, segments: ["Zone", "Shelf"] } } },
    { name: "create_product_location_command", payload: { request: { values: ["A1", "Shelf"] } } },
    { name: "assign_product_primary_location_command", payload: { request: { product_id: 3, expected_revision: 7, location_id: null } } },
    { name: "delete_product_location_command", payload: { request: { location_id: 7, expected_revision: 0 } } },
  ]);
});

test("sends explicit null for authorized unassignment and retains revision-checked response decoding", async () => {
  const calls: unknown[] = [];
  const locations = createProductLocationCommands(async (name, payload) => {
    calls.push({ name, payload });
    return { kind: "assignment_success", product_id: 3, location_id: null, revision: 8 };
  });

  assert.deepEqual(await locations.assignPrimary({ product_id: 3, expected_revision: 7, location_id: null }), {
    kind: "assignment_success", product_id: 3, location_id: null, revision: 8,
  });
  assert.deepEqual(calls, [{
    name: "assign_product_primary_location_command",
    payload: { request: { product_id: 3, expected_revision: 7, location_id: null } },
  }]);

  const malformed = createProductLocationCommands(async () => ({ kind: "assignment_success", product_id: 3, location_id: null, revision: -1 }));
  assert.deepEqual(await malformed.assignPrimary({ product_id: 3, expected_revision: 7, location_id: null }), {
    kind: "error", code: "persistence_failure", message: "The location change could not be completed.",
  });
});

test("rejects malformed location contracts and hides native failure details", async () => {
  for (const response of [null, { kind: "schema_success", schema: { revision: -1, segments: [] } }, { kind: "locations_success", locations: [{ location_id: 0, code: "A", values: [], active: true, revision: 0 }] }, { kind: "location_success", location: { location_id: 1, code: "A", values: [], active: "yes", revision: 0 } }, { kind: "error", code: "sql_error", message: "private SQL" }]) {
    const locations = createProductLocationCommands(async () => response);
    assert.deepEqual(await locations.schema(), { kind: "error", code: "persistence_failure", message: "The location change could not be completed." });
  }
  const rejected = createProductLocationCommands(async () => { throw new Error("SQL path /private"); });
  assert.deepEqual(await rejected.create(["A"]), { kind: "error", code: "persistence_failure", message: "The location change could not be completed." });
});

test("rejects unsafe, nonpositive, and inconsistent catalog facts", async () => {
  const valid = { ...searchProduct };
  const invalidProducts = [
    { ...valid, product_id: 0 },
    { ...valid, product_id: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, sale_price_centavos: 0 },
    { ...valid, sale_price_centavos: undefined },
    { ...valid, sale_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, purchase_price_centavos: 0 },
    { ...valid, purchase_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, minimum_sale_price_centavos: -1 },
    { ...valid, minimum_sale_price_centavos: undefined },
    { ...valid, minimum_sale_price_centavos: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, catalog_unit_price_centavos: -1 },
    { ...valid, available_quantity: -1 },
    { ...valid, revision: -1 },
    { ...valid, revision: Number.MAX_SAFE_INTEGER + 1 },
    { ...valid, sale_price_centavos: 4_000, minimum_sale_price_centavos: 5_000 },
  ];
  for (const response of invalidProducts) {
    const search = createSearchProductsCommand(async () => [response]);
    await assert.rejects(search("filter"), /product search/);
  }
});

test("edits category schemas through a typed envelope and decodes recoverable stale-schema errors", async () => {
  const calls: unknown[] = [];
  let response: unknown = { kind: "success", entity_id: 2, target: "category", label: "", activity: "active", revision: 4 };
  const commands = createCatalogMaintenanceCommands(async (command, payload) => { calls.push({ command, payload }); return response; });
  assert.deepEqual(await commands.editCategorySchema({ category_id: 2, expected_revision: 3, fields: [
    { definition_id: 5, label: "Material", field_type: "option", required: true, options: ["Steel", "Paper"] },
    { definition_id: null, label: "Length", field_type: "number", required: false, options: [] },
  ] }), { kind: "success", entity_id: 2, target: "category", label: "", activity: "active", revision: 4 });
  response = { kind: "error", code: "stale_category_schema", message: "native internal text" };
  assert.deepEqual(await commands.edit({ target: CATALOG_TARGET.PRODUCT, entity_id: 1, expected_revision: 8, expected_category_revision: 3, sku: "FLT", name: "Filter", purchase_price_centavos: 1200, sale_price_centavos: 3000, minimum_sale_price_centavos: 2500, attribute_values: [] }), { kind: "error", code: "stale_category_schema", message: "Category fields changed. Reload before saving this product." });
  assert.deepEqual(calls, [
    { command: "edit_category_schema_command", payload: { request: { category_id: 2, expected_revision: 3, fields: [
      { definition_id: 5, label: "Material", field_type: "option", required: true, options: ["Steel", "Paper"] },
      { definition_id: null, label: "Length", field_type: "number", required: false, options: [] },
    ] } } },
    { command: "edit_catalog_command", payload: { request: { target: "product", entity_id: 1, expected_revision: 8, expected_category_revision: 3, sku: "FLT", name: "Filter", purchase_price_centavos: 1200, sale_price_centavos: 3000, minimum_sale_price_centavos: 2500, attribute_values: [] } } },
  ]);
});
