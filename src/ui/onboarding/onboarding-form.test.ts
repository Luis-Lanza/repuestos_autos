import assert from "node:assert/strict";
import test from "node:test";

import type { Category } from "../../commands/onboarding.ts";
import { NAVIGATION_ACTION, SCREEN, screenAfter } from "../app.ts";
import { attributeValuesFor } from "./onboarding-form.ts";

test("keeps required blanks for backend validation and omits optional blanks", () => {
  const category: Category = {
    category_id: 1,
    name: "Belts",
    fields: [
      {
        definition_id: 10,
        label: "Length",
        field_type: "number",
        required: true,
        options: [],
      },
      {
        definition_id: 11,
        label: "Material",
        field_type: "option",
        required: false,
        options: ["Rubber"],
      },
    ],
  };

  assert.deepEqual(attributeValuesFor(category, { 10: "", 11: "" }), [
    { definition_id: 10, value: "" },
  ]);
});

test("navigates from sales to onboarding and back without a router", () => {
  assert.equal(
    screenAfter(SCREEN.SALES, NAVIGATION_ACTION.START_ONBOARDING),
    SCREEN.ONBOARDING,
  );
  assert.equal(
    screenAfter(SCREEN.ONBOARDING, NAVIGATION_ACTION.RETURN_TO_SALES),
    SCREEN.SALES,
  );
});
