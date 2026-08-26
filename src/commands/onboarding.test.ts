import assert from "node:assert/strict";
import test from "node:test";

import {
  createCreateCategoryCommand,
  createCreateProductCommand,
  createListCategoriesCommand,
} from "./onboarding.ts";

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
    catalog_unit_price_centavos: 4_500,
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
          catalog_unit_price_centavos: 4_500,
          opening_quantity: 6,
          attribute_values: [{ definition_id: 9, value: "Rubber" }],
        },
      },
    },
  ]);
  assert.deepEqual(categories, { kind: "success", categories: [] });
});
