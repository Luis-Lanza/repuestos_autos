import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { App, NAVIGATION_ACTION, SCREEN, screenAfter } from "./app.ts";
import { AppShell } from "./app-shell.ts";

const destinations = [
  ["Métricas", SCREEN.DASHBOARD, NAVIGATION_ACTION.OPEN_DASHBOARD],
  ["Ventas", SCREEN.SALES, NAVIGATION_ACTION.RETURN_TO_SALES],
  ["Inventario", SCREEN.INVENTORY, NAVIGATION_ACTION.OPEN_INVENTORY],
  ["Catálogo", SCREEN.CATALOG, NAVIGATION_ACTION.OPEN_CATALOG],
  ["Alta de productos", SCREEN.ONBOARDING, NAVIGATION_ACTION.START_ONBOARDING],
  ["Historial de ventas", SCREEN.SALES_HISTORY, NAVIGATION_ACTION.OPEN_SALES_HISTORY],
  ["Reportes", SCREEN.REPORTS, NAVIGATION_ACTION.OPEN_REPORTS],
  ["Copia y restauración", SCREEN.BACKUP, NAVIGATION_ACTION.OPEN_BACKUP],
] as const;

const expectedDestination = {
  [NAVIGATION_ACTION.OPEN_DASHBOARD]: SCREEN.DASHBOARD,
  [NAVIGATION_ACTION.START_ONBOARDING]: SCREEN.ONBOARDING,
  [NAVIGATION_ACTION.RETURN_TO_SALES]: SCREEN.SALES,
  [NAVIGATION_ACTION.OPEN_INVENTORY]: SCREEN.INVENTORY,
  [NAVIGATION_ACTION.OPEN_BACKUP]: SCREEN.BACKUP,
  [NAVIGATION_ACTION.OPEN_CATALOG]: SCREEN.CATALOG,
  [NAVIGATION_ACTION.OPEN_SALES_HISTORY]: SCREEN.SALES_HISTORY,
  [NAVIGATION_ACTION.OPEN_REPORTS]: SCREEN.REPORTS,
} as const;

test("AppShell exposes identity and the dashboard-first Spanish navigation", async () => {
  const actions: string[] = [];
  const user = userEvent.setup({ document });
  const view = render(createElement(AppShell, {
    screen: SCREEN.DASHBOARD,
    onNavigate: (action) => actions.push(action),
  }, createElement("main", null, "Contenido")));

  assert.ok(screen.getByText("Repuestos Autos"));
  const navigation = screen.getByRole("navigation", { name: "Navegación principal" });
  const buttons = within(navigation).getAllByRole("button");
  assert.deepEqual(buttons.map((button) => button.textContent), destinations.map(([label]) => label));
  assert.deepEqual(buttons.filter((button) => button.getAttribute("aria-current") === "page").map((button) => button.textContent), ["Métricas"]);

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
  assert.equal(inventory.children.length, 1);
  assert.equal(inventory.getAttribute("aria-describedby"), null);
  assert.doesNotMatch(inventory.outerHTML, /badge|count|status|alert|warning|stock|dot/i);
});

test("App keeps the global Inventory alert count across screens and opens the alert filter", async () => {
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "active" };
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [{ product_id: 7, product_name: "Correa", quantity: 0, classification: "out_of_stock" }] };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(App));
  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  const inventory = within(navigation).getByRole("button", { name: "Inventario" });
  assert.equal(inventory.textContent, "Inventario");
  await user.click(inventory);
  await waitFor(() => assert.match(inventory.textContent ?? "", /⚠ 1 alerta de stock/));
  await user.click(within(navigation).getByRole("button", { name: "Ventas" }));
  assert.match(inventory.textContent ?? "", /⚠ 1 alerta de stock/);
  await user.click(inventory);
  assert.ok(await screen.findByRole("combobox", { name: "Estado del stock" }));
  assert.equal((screen.getByRole("combobox", { name: "Estado del stock" }) as HTMLSelectElement).value, "all");
});

test("opens the unified Reports interface and keeps sidebar focus and active state", async () => {
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "active" };
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "dashboard_command") return { kind: "error", code: "persistence_failure", message: "unavailable" };
    if (command === "gross_profit_report_command") return { kind: "success", report: { amount_centavos: 0, missing_cost_line_count: 0 } };
    if (command === "list_movement_ledger_product_options_command") return { kind: "success", products: [] };
    if (command === "list_movement_ledger_command") return { kind: "success", rows: [], page: 1, page_size: 50, has_more: false };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document }); render(createElement(App));
  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  const reports = within(navigation).getByRole("button", { name: "Reportes" });
  await user.click(reports);
  assert.ok(await screen.findByRole("heading", { level: 1, name: "Reportes" }));
  assert.ok(await screen.findByRole("combobox", { name: "Tipo de movimiento" }));
  assert.equal(screen.queryByRole("region", { name: "Ganancia bruta" }), null);
  assert.equal(document.querySelectorAll("main").length, 1);
  assert.equal(reports.getAttribute("aria-current"), "page");
  assert.equal(document.activeElement, reports);
  assert.equal(screen.queryByText(/Próximamente|Otros informes/), null);
});

test("clears the sidebar count while a refresh fails instead of retaining stale alert state", async () => {
  let calls = 0;
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "active" };
    if (command !== "list_inventory_alerts_command") throw new Error(`Unexpected command: ${command}`);
    calls += 1;
    return calls === 1 ? { kind: "alerts", alerts: [{ product_id: 7, product_name: "Correa", quantity: 0, classification: "out_of_stock" }] } : Promise.reject(new Error("refresh failed"));
  });
  const user = userEvent.setup({ document });
  render(createElement(App));
  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  const inventory = within(navigation).getByRole("button", { name: /Inventario/ });
  await waitFor(() => assert.match(inventory.textContent ?? "", /1 alerta de stock/));
  await user.click(within(navigation).getByRole("button", { name: "Ventas" }));
  await waitFor(() => assert.doesNotMatch(inventory.textContent ?? "", /alerta de stock/));
  assert.ok(calls >= 2);
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
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "active" };
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "dashboard_command") return { kind: "error", code: "persistence_failure", message: "unavailable" };
    if (command === "gross_profit_report_command") return { kind: "success", report: { amount_centavos: 0, missing_cost_line_count: 0 } };
    if (command === "list_movement_ledger_product_options_command") return { kind: "success", products: [] };
    if (command === "list_movement_ledger_command") return { kind: "success", rows: [], page: 1, page_size: 50, has_more: false };
    if (command === "choose_backup_destination_command") return { kind: "cancelled" };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(App));

  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  assert.ok(screen.getByRole("heading", { level: 1, name: "Métricas" }));
  assert.equal(within(navigation).getByRole("button", { name: "Métricas" }).getAttribute("aria-current"), "page");

  const backup = within(navigation).getByRole("button", { name: "Copia y restauración" });
  await user.click(backup);
  assert.equal(screen.getByRole("navigation", { name: "Navegación principal" }), navigation);
  assert.ok(screen.getByRole("heading", { level: 1, name: "Copia y restauración" }));
  assert.equal(backup.getAttribute("aria-current"), "page");
  assert.equal(document.activeElement, backup);

  const sales = within(navigation).getByRole("button", { name: "Ventas" });
  await user.click(sales);
  assert.equal(screen.getByRole("navigation", { name: "Navegación principal" }), navigation);
  assert.ok(screen.getByRole("heading", { level: 1, name: "Ventas" }));
  assert.equal(sales.getAttribute("aria-current"), "page");
  assert.equal(document.activeElement, sales);
});

test("license import success remains announced after App transitions to active navigation", async () => {
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "license_missing" };
    if (command === "license_installation_code_command") return { kind: "code", code: "c".repeat(64) };
    if (command === "choose_license_file_command") return { kind: "selected" };
    if (command === "import_license_command") return { kind: "imported", status: "active" };
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(App));

  await user.click(await screen.findByRole("button", { name: "Importar archivo de licencia" }));
  const notice = await screen.findByText("Licencia activada correctamente.");
  assert.equal(notice.getAttribute("role"), "status");
  assert.equal(notice.getAttribute("aria-live"), "polite");
  assert.ok(screen.getByRole("navigation", { name: "Navegación principal" }));
  assert.equal(screen.queryByRole("heading", { name: "Activá Repuestos Autos" }), null);
});

test("unlicensed startup defaults to activation and recovery exposes only safe navigation and backup creation", async () => {
  let restores = 0; let backups = 0;
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "license_missing" };
    if (command === "license_installation_code_command") return { kind: "code", code: "c".repeat(64) };
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "dashboard_command") return { kind: "error", code: "persistence_failure", message: "unavailable" };
    if (command === "list_movement_ledger_product_options_command") return { kind: "success", products: [] };
    if (command === "list_movement_ledger_command") return { kind: "success", rows: [], page: 1, page_size: 50, has_more: false };
    if (command === "choose_backup_destination_command") return { kind: "selected", token: "recovery-destination-token" };
    if (command === "create_backup_command") { backups++; return { kind: "created", file_name: "backup-recovery.sqlite3", created_at_unix_seconds: 1, size_bytes: 32, schema_version: 6, durability_warning: true, cleanup_warning: false }; }
    if (command === "choose_restore_source_command" || command === "prepare_restore_command" || command === "confirm_restore_command") { restores++; throw new Error("restore IPC must not run"); }
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document }); render(createElement(App));
  assert.ok(await screen.findByRole("heading", { name: "Activá Repuestos Autos" }));
  await user.click(screen.getByRole("button", { name: "Continuar en modo de recuperación" }));
  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  assert.deepEqual(within(navigation).getAllByRole("button").map((button) => button.textContent), ["Métricas", "Historial de ventas", "Reportes", "Copia y restauración"]);
  await user.click(within(navigation).getByRole("button", { name: "Copia y restauración" }));
  const restore = screen.getByRole("button", { name: "Elegir archivo de respaldo" }) as HTMLButtonElement;
  assert.equal(restore.disabled, true);
  assert.ok(screen.getByText("La restauración está disponible con una licencia activa."));
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  await waitFor(() => assert.equal(backups, 1));
  assert.ok(await screen.findByText("backup-recovery.sqlite3"));
  assert.match((await screen.findByRole("alert")).textContent ?? "", /durabilidad del directorio final/);
  assert.equal(restores, 0);
});

test("production CSS declares the desktop and compact shell width contracts", async () => {
  const source = await readFile(new URL("./styles.css", import.meta.url), "utf8");
  assert.match(source, /--size-shell-sidebar:\s*208px/);
  assert.match(source, /grid-template-columns:\s*var\(--size-shell-sidebar\)\s+minmax\(0,\s*1fr\)/);
  assert.match(source, /data-ui-shell-sidebar[^}]*color:\s*var\(--color-text\)[^}]*background:\s*var\(--color-surface\)[^}]*border-inline-end:\s*1px solid var\(--color-border\)/s);
  assert.match(source, /data-ui-shell-navigation[^}]*button\[aria-current="page"\][^}]*color:\s*var\(--color-text-inverse\)[^}]*background:\s*var\(--color-action\)/s);
  assert.match(source, /\[data-ui-inventory-cue\]\s*\{[^}]*color:\s*var\(--color-danger\)/s);
  assert.match(source, /\[data-ui-shell-navigation\] button\[aria-current="page"\] \[data-ui-inventory-cue\]\s*\{[^}]*color:\s*inherit/s);
  assert.match(source, /@media \(max-width: 960px\)[\s\S]*--size-shell-sidebar:\s*176px/);
  assert.match(source, /data-ui-inventory-layout[^}]*grid-template-columns:\s*minmax\(0, 1\.85fr\) minmax\(260px,\s*1fr\)/);
  assert.match(source, /\[data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns:\s*minmax\(12rem,\s*2fr\) minmax\(10rem,\s*1fr\) minmax\(9rem,\s*1fr\) auto auto/);
  assert.match(source, /\[data-ui-catalog-workspace\] \[data-ui-catalog-toolbar-item="views"\] \{[^}]*display:\s*flex/);
  assert.match(source, /@media \(max-width: 960px\)[\s\S]*\[data-ui-catalog-workspace\] \[data-ui-product-browser\] > form \{[^}]*grid-template-columns:\s*repeat\(2, minmax\(0, 1fr\)\)/);
  assert.doesNotMatch(source, /data-ui-catalog-layout[^}]*grid-template-columns:\s*minmax\(280px,\s*4fr\) minmax\(0,\s*7fr\)/);
  assert.doesNotMatch(source, /@media \(max-width: 1199px\) and \(min-width: 961px\)/);
  assert.match(source, /@media \(max-width: 960px\)[\s\S]*data-ui-inventory-layout[^}]*grid-template-columns:\s*minmax\(0, 1fr\)/);
  assert.match(source, /data-ui-shell-content[^}]*overflow:\s*auto/);
  assert.match(source, /data-ui-product-browser-list[^}]*overflow-y:\s*auto/);
  assert.match(source, /(?:^|\n)\[data-ui-sale-search\]\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\) auto/s);
  assert.match(source, /@container sales-browse \(min-width:\s*40rem\)[\s\S]*\[data-ui-product-browser="sales"\] \[data-ui-sale-search\]\s*\{[^}]*grid-template-columns:\s*minmax\(12rem,\s*1\.6fr\) minmax\(10rem,\s*1fr\) auto auto/s);
  assert.match(source, /(?:^|\n)\[data-ui-sale-list\] > li\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\) auto/s);
  assert.match(source, /\[data-ui-product-browser="sales"\] \[data-ui-sale-list\] > li\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\) auto/s);
  assert.match(source, /data-ui-product-browser-pages[^}]*flex:\s*0 0 auto/);
  assert.doesNotMatch(source, /data-ui-catalog-search-actions/);
  assert.match(source, /\[data-ui-catalog-workspace\] \[data-ui-product-browser-list\]\[data-ui-catalog-gallery="true"\][^}]*grid-template-columns:\s*repeat\(5, minmax\(0, 1fr\)\)/);
  assert.match(source, /@media \(max-width: 960px\)[\s\S]*\[data-ui-catalog-workspace\] \[data-ui-product-browser-list\]\[data-ui-catalog-gallery="true"\][^}]*grid-template-columns:\s*repeat\(2, minmax\(0, 1fr\)\)/);
  assert.match(source, /\[data-ui-catalog-workspace\] \[data-ui-catalog-toolbar-item="views"\] > button[^}]*min-block-size:\s*var\(--size-control-default\)/);
  assert.doesNotMatch(source, /data-ui-catalog-master[^}]*max-block-size:\s*(?:520|208)px/);
});
