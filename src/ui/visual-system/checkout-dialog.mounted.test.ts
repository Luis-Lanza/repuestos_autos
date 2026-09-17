import assert from "node:assert/strict";
import test from "node:test";
import { createElement, createRef } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CheckoutDialog } from "./checkout-dialog.ts";

function opener() {
  const button = document.createElement("button");
  button.textContent = "Revisar y cobrar";
  document.body.append(button);
  button.focus();
  return button;
}

function view(extra: Partial<Parameters<typeof CheckoutDialog>[0]> = {}) {
  return render(createElement(CheckoutDialog, {
    open: true,
    title: "Revisar y cobrar",
    description: "Revisá la venta antes de confirmar.",
    confirmLabel: "Confirmar venta",
    onCancel: () => undefined,
    onConfirm: () => undefined,
    ...extra,
  }));
}

test("routine checkout has a safe primary action and restores focus after Volver", async () => {
  const trigger = opener();
  const first = createRef<HTMLInputElement>();
  let rendered: ReturnType<typeof render>;
  rendered = view({ initialFocusRef: first, children: createElement("input", { ref: first, "aria-label": "Precio" }), onCancel: () => rendered.rerender(createElement(CheckoutDialog, { open: false, title: "Revisar y cobrar", description: "Revisá la venta antes de confirmar.", confirmLabel: "Confirmar venta", onCancel: () => undefined, onConfirm: () => undefined })) });
  const dialog = screen.getByRole("dialog", { name: "Revisar y cobrar" });
  assert.equal(dialog.getAttribute("data-ui-checkout-dialog"), "true");
  assert.equal(dialog.getAttribute("data-ui-dialog-layout"), "checkout");
  assert.equal(dialog.getAttribute("data-ui-confirmation-dialog"), null);
  assert.equal(screen.getByRole("button", { name: "Confirmar venta" }).getAttribute("data-ui-action"), "primary");
  assert.equal(document.activeElement, first.current);
  await userEvent.setup({ document }).click(screen.getByRole("button", { name: "Volver" }));
  assert.equal(document.activeElement, trigger);
  rendered.unmount();
  trigger.remove();
});

test("pending checkout contains focus and blocks Escape and activation", async () => {
  let cancels = 0;
  let confirms = 0;
  view({ pending: true, pendingLabel: "Confirmando…", onCancel: () => { cancels += 1; }, onConfirm: () => { confirms += 1; }, children: createElement("button", null, "Editar") });
  const dialog = screen.getByRole("dialog", { name: "Revisar y cobrar" });
  assert.equal(document.activeElement, dialog);
  fireEvent.keyDown(dialog, { key: "Escape" });
  const user = userEvent.setup({ document });
  await user.click(screen.getByRole("button", { name: "Volver" }));
  await user.click(screen.getByRole("button", { name: "Confirmando…" }));
  assert.deepEqual([cancels, confirms], [0, 0]);
  assert.equal(dialog.getAttribute("aria-busy"), "true");
});
