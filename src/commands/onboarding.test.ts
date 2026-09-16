import assert from "node:assert/strict";
import test from "node:test";

import {
  createCreateCategoryCommand,
  createCreateProductCommand,
  createListCategoriesCommand,
} from "./onboarding.ts";

test("projects valid categories and products while dropping native fields", async () => {
  const category = { category_id: 3, name: "Belts", fields: [{ definition_id: 9, label: "Material", field_type: "option", required: true, options: ["Rubber"], hidden: true }], hidden: true };
  const product = { product_id: 8, sku: "BEL-1", name: "Accessory belt", category_id: 3, category_name: "Belts", list_price_centavos: 4500, minimum_sale_price_centavos: 3500, available_quantity: 6, active: true, hidden: true };
  assert.deepEqual(await createListCategoriesCommand(async () => ({ kind: "success", categories: [category] }))(), { kind: "success", categories: [{ category_id: 3, name: "Belts", fields: [{ definition_id: 9, label: "Material", field_type: "option", required: true, options: ["Rubber"] }] }] });
  assert.deepEqual(await createCreateCategoryCommand(async () => ({ kind: "success", ...category }))({ name: "Belts", fields: [] }), { kind: "success", category_id: 3, name: "Belts", fields: [{ definition_id: 9, label: "Material", field_type: "option", required: true, options: ["Rubber"] }] });
  assert.deepEqual(await createCreateProductCommand(async () => ({ kind: "success", ...product }))({ sku: "BEL-1", name: "Accessory belt", category_id: 3, list_price_centavos: 4500, minimum_sale_price_centavos: 3500, opening_quantity: 6, attribute_values: [] }), { kind: "success", product_id: 8, sku: "BEL-1", name: "Accessory belt", category_id: 3, category_name: "Belts", list_price_centavos: 4500, minimum_sale_price_centavos: 3500, available_quantity: 6, active: true });
});

test("rejects malformed onboarding data atomically and checks numeric transport", async () => {
  const invalid = [
    { kind: "success", categories: [{ category_id: 1, name: "x", fields: [{ definition_id: 1, label: "x", field_type: "unknown", required: true, options: [] }] }] },
    { kind: "success", categories: [{ category_id: 1, name: "x", fields: [{ definition_id: Number.MAX_SAFE_INTEGER + 1, label: "x", field_type: "text", required: true, options: [] }] }] },
    { kind: "success", categories: [{ category_id: 1, name: "x", fields: [{ definition_id: 1, label: "x", field_type: "text", required: true, options: ["ok", 2] }] }] },
  ];
  for (const response of invalid) assert.deepEqual(await createListCategoriesCommand(async () => response)(), { kind: "error", code: "persistence_failure", message: "Categories could not be loaded." });
  const malformedProduct = createCreateProductCommand(async () => ({ kind: "success", product_id: 1, sku: "x", name: "x", category_id: 1, category_name: "x", list_price_centavos: 1.5, minimum_sale_price_centavos: 1, available_quantity: 1, active: true }));
  assert.deepEqual(await malformedProduct({ sku: "x", name: "x", category_id: 1, list_price_centavos: 1, minimum_sale_price_centavos: 1, opening_quantity: 1, attribute_values: [] }), { kind: "error", code: "persistence_failure", message: "The product could not be persisted." });
});

test("maps recognized and malformed onboarding errors to fixed safe messages", async () => {
  const native = { kind: "error", code: "duplicate_category", message: "SQL /panic native text" };
  assert.deepEqual(await createCreateCategoryCommand(async () => native)({ name: "x", fields: [] }), { kind: "error", code: "duplicate_category", message: "Category name already exists." });
  assert.deepEqual(await createCreateProductCommand(async () => ({ kind: "error", code: "unknown", message: "native" }))({ sku: "x", name: "x", category_id: 1, list_price_centavos: 1, minimum_sale_price_centavos: 1, opening_quantity: 1, attribute_values: [] }), { kind: "error", code: "persistence_failure", message: "The product could not be persisted." });
  assert.deepEqual(await createListCategoriesCommand(async () => { throw new Error("SQL path"); })(), { kind: "error", code: "persistence_failure", message: "Categories could not be loaded." });
});

test("adapts category and product onboarding through narrow IPC payloads", async () => {
  const calls: unknown[] = [];
  const invoke = async (command: string, payload?: unknown) => {
    calls.push({ command, payload });
    if (command === "list_categories_command") {
      return { kind: "success", categories: [] };
    }
    return { kind: "error", code: "duplicate_sku", message: "SKU already exists." };
  };

  const categories = await createListCategoriesCommand(invoke)();
  await createCreateCategoryCommand(invoke)({
    name: "Belts",
    fields: [
      {
        label: "Material",
        field_type: "option",
        required: true,
        options: ["Rubber"],
      },
    ],
  });
  await createCreateProductCommand(invoke)({
    sku: "BEL-1",
    name: "Accessory belt",
    category_id: 3,
    list_price_centavos: 4_500,
    minimum_sale_price_centavos: 3_500,
    opening_quantity: 6,
    attribute_values: [{ definition_id: 9, value: "Rubber" }],
  });

  assert.deepEqual(calls, [
    { command: "list_categories_command", payload: undefined },
    {
      command: "create_category_command",
      payload: {
        request: {
          name: "Belts",
          fields: [
            {
              label: "Material",
              field_type: "option",
              required: true,
              options: ["Rubber"],
            },
          ],
        },
      },
    },
    {
      command: "create_product_command",
      payload: {
        request: {
          sku: "BEL-1",
          name: "Accessory belt",
          category_id: 3,
          list_price_centavos: 4_500,
          minimum_sale_price_centavos: 3_500,
          opening_quantity: 6,
          attribute_values: [{ definition_id: 9, value: "Rubber" }],
        },
      },
    },
  ]);
  assert.deepEqual(categories, { kind: "success", categories: [] });
});
