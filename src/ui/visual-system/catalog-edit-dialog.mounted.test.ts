import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CatalogEditDialog } from "./catalog-edit-dialog.ts";

const record = { entity_id: 4, target: "category" as const, label: "Filtros", activity: "active" as const, revision: 2 };
const detail = { target: "category" as const, entity_id: 4, name: "Filtros", activity: "active" as const, revision: 2, attribute_definitions: [] };
const form = { name: "Filtros", attribute_values: {} };

function view(recoveryRequired: boolean, onReload = () => undefined) {
  return render(createElement(CatalogEditDialog, {
    record,
    detail,
    form,
    loading: false,
    pending: false,
    feedback: null,
    lifecycleFeedback: recoveryRequired ? "No se pudieron actualizar los registros del catálogo. Reintentá." : null,
    recoveryRequired,
    fieldErrors: {},
    onChange: () => undefined,
    onSubmit: () => undefined,
    onLifecycle: () => undefined,
    onReload,
    onCancel: () => undefined,
  }));
}

test("current purchase price is exposed as required in the rendered product form", () => {
  render(createElement(CatalogEditDialog, {
    record: { ...record, target: "product", label: "Filtro" },
    detail: { ...detail, target: "product", name: "Filtro", category_id: 2, attribute_definitions: [] },
    form: { ...form, sku: "FLT", purchase_price_centavos: "2000", sale_price_centavos: "3000", minimum_sale_price_centavos: "2500" },
    loading: false,
    pending: false,
    feedback: null,
    lifecycleFeedback: null,
    recoveryRequired: false,
    fieldErrors: {},
    onChange: () => undefined,
    onSubmit: () => undefined,
    onLifecycle: () => undefined,
    onReload: () => undefined,
    onCancel: () => undefined,
  }));

  const dialog = screen.getByRole("dialog", { name: "Editar Filtro" });
  assert.equal(within(dialog).getByRole("textbox", { name: "Precio actual de compra (Bs)" }).getAttribute("aria-required"), "true");
});

test("recovery preserves the detail while disabling incompatible actions and exposing retry", async () => {
  let reloads = 0;
  const rendered = view(true, () => { reloads += 1; });
  const dialog = screen.getByRole("dialog", { name: "Editar Filtros" });
  assert.equal((within(dialog).getByRole("textbox", { name: "Nombre de la categoría" }) as HTMLInputElement).disabled, true);
  assert.equal((within(dialog).getByRole("button", { name: "Archivar" }) as HTMLButtonElement).disabled, true);
  assert.equal((within(dialog).getByRole("button", { name: "Guardar metadatos" }) as HTMLButtonElement).disabled, true);
  assert.ok(within(dialog).getByRole("alert"));
  await userEvent.click(within(dialog).getByRole("button", { name: "Reintentar actualización" }));
  assert.equal(reloads, 1);
  rendered.rerender(createElement(CatalogEditDialog, {
    record: { ...record, activity: "archived", revision: 3 },
    detail: { ...detail, activity: "archived", revision: 3 },
    form,
    loading: false,
    pending: false,
    feedback: null,
    lifecycleFeedback: null,
    recoveryRequired: false,
    fieldErrors: {},
    onChange: () => undefined,
    onSubmit: () => undefined,
    onLifecycle: () => undefined,
    onReload: () => undefined,
    onCancel: () => undefined,
  }));
  assert.equal((within(screen.getByRole("dialog", { name: "Editar Filtros" })).getByRole("button", { name: "Reactivar" }) as HTMLButtonElement).disabled, false);
});
