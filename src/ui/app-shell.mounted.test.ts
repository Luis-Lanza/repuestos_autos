import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { App, NAVIGATION_ACTION, SCREEN, screenAfter } from "./app.ts";
import { AppShell } from "./app-shell.ts";

const destinations = [
  ["Ventas", SCREEN.SALES, NAVIGATION_ACTION.RETURN_TO_SALES],
  ["Inventario", SCREEN.INVENTORY, NAVIGATION_ACTION.OPEN_INVENTORY],
  ["Catálogo", SCREEN.CATALOG, NAVIGATION_ACTION.OPEN_CATALOG],
  ["Alta de productos", SCREEN.ONBOARDING, NAVIGATION_ACTION.START_ONBOARDING],
  ["Historial de ventas", SCREEN.SALES_HISTORY, NAVIGATION_ACTION.OPEN_SALES_HISTORY],
  ["Copia y restauración", SCREEN.BACKUP, NAVIGATION_ACTION.OPEN_BACKUP],
] as const;

const expectedDestination = {
  [NAVIGATION_ACTION.START_ONBOARDING]: SCREEN.ONBOARDING,
  [NAVIGATION_ACTION.RETURN_TO_SALES]: SCREEN.SALES,
  [NAVIGATION_ACTION.OPEN_INVENTORY]: SCREEN.INVENTORY,
  [NAVIGATION_ACTION.OPEN_BACKUP]: SCREEN.BACKUP,
  [NAVIGATION_ACTION.OPEN_CATALOG]: SCREEN.CATALOG,
  [NAVIGATION_ACTION.OPEN_SALES_HISTORY]: SCREEN.SALES_HISTORY,
} as const;

test("AppShell exposes identity and the six existing actions through persistent Spanish navigation", async () => {
  const actions: string[] = [];
  const user = userEvent.setup({ document });
  const view = render(createElement(AppShell, {
    screen: SCREEN.SALES,
    onNavigate: (action) => actions.push(action),
  }, createElement("main", null, "Contenido")));

  assert.ok(screen.getByText("Repuestos Autos"));
  const navigation = screen.getByRole("navigation", { name: "Navegación principal" });
  const buttons = within(navigation).getAllByRole("button");
  assert.deepEqual(buttons.map((button) => button.textContent), destinations.map(([label]) => label));
  assert.deepEqual(buttons.filter((button) => button.getAttribute("aria-current") === "page").map((button) => button.textContent), ["Ventas"]);

  for (const [label, , action] of destinations) {
    await user.click(within(navigation).getByRole("button", { name: label }));
    assert.equal(actions.at(-1), action);
  }

  for (const [, current] of destinations) {
    view.rerender(createElement(AppShell, { screen: current, onNavigate: () => undefined }, createElement("main", null, "Contenido")));
    const currentButtons = within(navigation).getAllByRole("button").filter((button) => button.getAttribute("aria-current") === "page");
    assert.equal(currentButtons.length, 1);
    assert.equal(currentButtons[0].textContent, destinations.find(([, destination]) => destination === current)?.[0]);
  }

  const inventory = within(navigation).getByRole("button", { name: "Inventario" });
  assert.equal(inventory.textContent, "Inventario");
  assert.equal(inventory.children.length, 0);
  assert.equal(inventory.getAttribute("aria-describedby"), null);
  assert.doesNotMatch(inventory.outerHTML, /badge|count|status|alert|warning|stock|dot/i);
});

test("screenAfter preserves the complete transition table and Sales fallback", () => {
  for (const current of Object.values(SCREEN)) {
    for (const action of Object.values(NAVIGATION_ACTION)) {
      assert.equal(screenAfter(current, action), expectedDestination[action], `${current} + ${action}`);
    }
  }
  assert.equal(screenAfter(SCREEN.SALES, NAVIGATION_ACTION.RETURN_TO_SALES), SCREEN.SALES);
});

test("App keeps one shell mounted while safe navigation changes content, active state, and retained focus", async () => {
  const user = userEvent.setup({ document });
  render(createElement(App));

  const navigation = screen.getByRole("navigation", { name: "Navegación principal" });
  assert.ok(screen.getByRole("heading", { level: 1, name: "Confirm sale" }));
  assert.equal(within(navigation).getByRole("button", { name: "Ventas" }).getAttribute("aria-current"), "page");

  const backup = within(navigation).getByRole("button", { name: "Copia y restauración" });
  await user.click(backup);
  assert.equal(screen.getByRole("navigation", { name: "Navegación principal" }), navigation);
  assert.ok(screen.getByRole("heading", { level: 1, name: "Backup and restore" }));
  assert.equal(backup.getAttribute("aria-current"), "page");
  assert.equal(document.activeElement, backup);

  const sales = within(navigation).getByRole("button", { name: "Ventas" });
  await user.click(sales);
  assert.equal(screen.getByRole("navigation", { name: "Navegación principal" }), navigation);
  assert.ok(screen.getByRole("heading", { level: 1, name: "Confirm sale" }));
  assert.equal(sales.getAttribute("aria-current"), "page");
  assert.equal(document.activeElement, sales);
});

test("production CSS declares the desktop and compact shell width contracts", async () => {
  const source = await readFile(new URL("./styles.css", import.meta.url), "utf8");
  assert.match(source, /--size-shell-sidebar:\s*208px/);
  assert.match(source, /grid-template-columns:\s*var\(--size-shell-sidebar\)\s+minmax\(0,\s*1fr\)/);
  assert.match(source, /@media \(max-width: 960px\)[\s\S]*--size-shell-sidebar:\s*176px/);
});
