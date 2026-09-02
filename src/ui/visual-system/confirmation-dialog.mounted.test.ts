import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { StrictMode, createElement, createRef } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { ConfirmationDialog } from "./confirmation-dialog.ts";

const style = document.createElement("style");
style.textContent = `${await readFile(new URL("../styles.css", import.meta.url), "utf8")} .fixture-none{display:none}.fixture-invisible{visibility:hidden}`;
document.head.append(style);

const dialog = (extra: Partial<Parameters<typeof ConfirmationDialog>[0]> = {}) => createElement(ConfirmationDialog, {
  open: true, purpose: "cancellation", title: "Cancelar venta #184",
  description: "Esta acción corrige el inventario.", confirmLabel: "Cancelar venta",
  onCancel: () => undefined, onConfirm: () => undefined, ...extra,
});

function opener(label = "Abrir") {
  const button = document.createElement("button");
  button.textContent = label;
  document.body.append(button);
  button.focus();
  return button;
}

// Formal RED 1: the snapshot candidate restores the opener during StrictMode effect replay.
test("StrictMode replay keeps focus in an open dialog", () => {
  const invoker = opener();
  const view = render(createElement(StrictMode, null, dialog()));
  assert.ok(document.activeElement === document.querySelector('[data-ui-dialog-actions] button'), "focus escaped during StrictMode replay");
  view.unmount();
  invoker.remove();
});

// Formal RED 2: a hidden first endpoint must not defeat forward wrapping.
test("hidden endpoint is excluded from the live focus loop", () => {
  render(dialog({ children: createElement("button", { hidden: true }, "Oculto") }));
  const actions = document.querySelectorAll<HTMLButtonElement>('[data-ui-dialog-actions] button');
  actions[1].focus();
  fireEvent.keyDown(actions[1], { key: "Tab" });
  assert.ok(document.activeElement === actions[0], "hidden endpoint blocked wrapping to Volver");
});

// Formal RED 3: initially pending dialogs need a focusable static containment target.
test("initially pending dialog focuses its named container", () => {
  render(dialog({ purpose: "restore", title: "Restaurar datos", pending: true }));
  assert.ok(document.activeElement === document.querySelector('[role="dialog"]'), "pending dialog lacked internal focus");
});

test("both purposes expose an accessible name and description; valid explicit focus wins", () => {
  const target = createRef<HTMLInputElement>();
  const view = render(dialog({ initialFocusRef: target, children: createElement("input", { ref: target, "aria-label": "Motivo" }) }));
  const cancellation = screen.getByRole("dialog", { name: "Cancelar venta #184" });
  assert.equal(cancellation.getAttribute("data-ui-purpose"), "cancellation");
  assert.equal(document.getElementById(cancellation.getAttribute("aria-describedby")!)?.textContent, "Esta acción corrige el inventario.");
  assert.ok(document.activeElement === screen.getByRole("textbox", { name: "Motivo" }));
  view.rerender(dialog({ purpose: "restore", title: "Restaurar datos locales", description: "Reemplaza los datos actuales." }));
  assert.equal(screen.getByRole("dialog", { name: "Restaurar datos locales" }).getAttribute("data-ui-purpose"), "restore");
});

test("invalid, hidden, disabled, outside, and stale explicit targets fall back to Volver", () => {
  const outside = opener("Fuera");
  const stale = document.createElement("button");
  for (const current of [outside, stale]) {
    const view = render(dialog({ initialFocusRef: { current } }));
    assert.ok(document.activeElement === screen.getByRole("button", { name: "Volver" }));
    view.unmount();
  }
  outside.remove();
  for (const props of [{ hidden: true }, { disabled: true }]) {
    const target = createRef<HTMLButtonElement>();
    const view = render(dialog({ initialFocusRef: target, children: createElement("button", { ...props, ref: target }, "Inválido") }));
    assert.ok(document.activeElement === screen.getByRole("button", { name: "Volver" }));
    view.unmount();
  }
});

test("the live loop excludes hidden ancestors, aria-hidden, inert, CSS-hidden, negative, and hidden inputs", () => {
  render(dialog({ children: createElement("div", null,
    createElement("button", { hidden: true }, "hidden"),
    createElement("div", { hidden: true }, createElement("button", null, "hidden ancestor")),
    createElement("div", { "aria-hidden": "true" }, createElement("button", null, "aria hidden")),
    createElement("div", { ref: (node) => { node?.setAttribute("inert", ""); } }, createElement("button", null, "inert")),
    createElement("button", { className: "fixture-none" }, "display none"),
    createElement("button", { className: "fixture-invisible" }, "visibility hidden"),
    createElement("button", { tabIndex: -1 }, "negative"),
    createElement("button", { disabled: true }, "disabled"),
    createElement("input", { type: "hidden" })),
  }));
  const back = screen.getByRole("button", { name: "Volver" });
  const confirm = screen.getByRole("button", { name: "Cancelar venta" });
  confirm.focus(); fireEvent.keyDown(confirm, { key: "Tab" });
  assert.ok(document.activeElement === back);
  fireEvent.keyDown(back, { key: "Tab", shiftKey: true });
  assert.ok(document.activeElement === confirm);
});

test("Tab and Shift+Tab recompute eligible descendants after ref replacement", () => {
  const first = createRef<HTMLButtonElement>();
  const second = createRef<HTMLButtonElement>();
  const view = render(dialog({ children: createElement("button", { ref: first }, "Primero") }));
  view.rerender(dialog({ children: createElement("button", { ref: second }, "Segundo") }));
  assert.equal(first.current, null);
  const confirm = screen.getByRole("button", { name: "Cancelar venta" });
  confirm.focus(); fireEvent.keyDown(confirm, { key: "Tab" });
  assert.ok(document.activeElement === second.current);
  fireEvent.keyDown(second.current!, { key: "Tab", shiftKey: true });
  assert.ok(document.activeElement === confirm);
});

test("idle Escape and Volver cancel, while pending locks every activation", async () => {
  let cancels = 0; let confirms = 0;
  const user = userEvent.setup({ document });
  const view = render(dialog({ onCancel: () => { cancels += 1; }, onConfirm: () => { confirms += 1; } }));
  await user.keyboard("{Escape}");
  await user.click(screen.getByRole("button", { name: "Volver" }));
  await user.click(screen.getByRole("button", { name: "Cancelar venta" }));
  assert.deepEqual([cancels, confirms], [2, 1]);
  view.rerender(dialog({ pending: true, pendingLabel: "Cancelando…", onCancel: () => { cancels += 1; }, onConfirm: () => { confirms += 1; } }));
  await user.keyboard("{Escape}");
  await user.click(screen.getByRole("button", { name: "Volver" }));
  await user.click(screen.getByRole("button", { name: "Cancelando…" }));
  assert.deepEqual([cancels, confirms], [2, 1]);
  assert.ok(screen.getByRole("dialog"));
});

test("connected invokers restore after cancel, parent close, and real unmount", async () => {
  const user = userEvent.setup({ document });
  const invoker = opener();
  let view: ReturnType<typeof render>;
  view = render(dialog({ onCancel: () => view.rerender(dialog({ open: false })) }));
  await user.click(screen.getByRole("button", { name: "Volver" }));
  assert.ok(document.activeElement === invoker);
  invoker.focus(); view.rerender(dialog()); view.rerender(dialog({ open: false }));
  assert.ok(document.activeElement === invoker);
  invoker.focus(); view.rerender(dialog()); view.unmount();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(document.activeElement === invoker);
  invoker.remove();
});

test("replacement dialog ownership prevents stale cleanup focus theft", async () => {
  const oldInvoker = opener("Anterior");
  const old = render(dialog());
  old.unmount();
  const currentInvoker = opener("Actual");
  const current = render(dialog({ title: "Restaurar datos" }));
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(current.getByRole("dialog").contains(document.activeElement));
  current.unmount(); oldInvoker.remove(); currentInvoker.remove();
});

test("document capture contains Tab after focus is displaced externally", () => {
  const outside = opener("Externo");
  render(dialog());
  const [back, confirm] = screen.getAllByRole("button").slice(-2);
  outside.focus(); fireEvent.keyDown(outside, { key: "Tab" });
  assert.ok(document.activeElement === back);
  outside.focus(); fireEvent.keyDown(outside, { key: "Tab", shiftKey: true });
  assert.ok(document.activeElement === confirm);
  outside.remove();
});

test("content seam rejects blank titles and absent or blank descriptions", () => {
  for (const props of [{ title: " " }, { description: "" }, { description: null }, { description: false }]) {
    assert.throws(() => render(dialog(props as never)), /nonblank title|valid description/);
  }
});

test("a disconnected invoker is never restored", async () => {
  const invoker = opener();
  const view = render(dialog());
  invoker.remove();
  view.rerender(dialog({ open: false }));
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(document.activeElement !== invoker);
});
