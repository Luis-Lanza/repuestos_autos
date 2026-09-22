import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { DashboardScreen } from "./dashboard-screen.ts";

const dashboardStyles = readFileSync(new URL("../styles.css", import.meta.url), "utf8");

test("defines the approved desktop composition and compact labeled-record CSS contract", () => {
  assert.match(dashboardStyles, /\[data-ui-dashboard-supporting\]\s*\{[^}]*grid-template-columns:\s*repeat\(2,\s*minmax\(0,\s*1fr\)\)/s);
  assert.match(dashboardStyles, /\[data-ui-dashboard-recent-sales\]\s*\{[^}]*grid-column:\s*1\s*\/\s*-1/s);
  assert.match(dashboardStyles, /@media\s*\(max-width:\s*960px\)[\s\S]*\[data-ui-dashboard-supporting\]\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)/);
  assert.match(dashboardStyles, /@media\s*\(max-width:\s*960px\)[\s\S]*\[data-ui-dashboard\]\s+\[data-ui-aligned-data\]\s+td::before\s*\{[^}]*content:\s*attr\(data-label\)/);
  assert.match(dashboardStyles, /@media\s*\(max-width:\s*960px\)[\s\S]*\[data-ui-dashboard-stock-alerts\]\s+\[data-ui-action\]\s*\{[^}]*inline-size:\s*100%[^}]*min-block-size:\s*var\(--size-control-default\)/);
  assert.doesNotMatch(dashboardStyles, /\[data-ui-dashboard(?:-[^\]]+)?\][^{]*\{[^}]*overflow-y\s*:/s);
});

test("renders the approved hierarchy and complete report facts before opening stock alerts", async () => {
  mockIPC((command) => {
    assert.equal(command, "dashboard_command");
    return { kind: "success", report: { today: { metrics: { effective_sale_count: 1, effective_total_centavos: 2500, net_units_out: 1, cancelled_sale_count: 0 } }, month: { metrics: { effective_sale_count: 2, effective_total_centavos: 5000, net_units_out: 2, cancelled_sale_count: 1 } }, top_products: [{ product_id: 1, sku: "FLT-001", product_name: "Filtro", net_units_out: 2 }], payment_distribution: [{ method: "cash", amount_applied_centavos: 5000 }], recent_sales: [{ sale_id: 2, confirmed_at: "2024-03-10 12:00:00", status: "confirmed", total_centavos: 2500 }], stock_alerts: [{ product_id: 3, sku: "BEL-001", product_name: "Correa", quantity: 0, classification: "out_of_stock" }] } };
  });
  const user = userEvent.setup({ document });
  let opened = false;
  render(createElement(DashboardScreen, { onOpenInventoryAlerts: () => { opened = true; } }));

  const main = await screen.findByRole("main");
  assert.equal(within(main).getAllByRole("heading", { name: "Dashboard" }).length, 1);
  assert.ok(within(main).getByText("Resumen operativo de ventas e inventario."));
  await waitFor(() => assert.equal(main.querySelectorAll("[data-ui-dashboard-metric]").length, 8));

  const today = within(main).getByRole("region", { name: "Hoy" });
  assert.equal(within(today).getByText("Ventas efectivas").nextElementSibling?.textContent, "1");
  assert.equal(within(today).getByText("Total efectivo").nextElementSibling?.textContent, "Bs 25,00");
  const month = within(main).getByRole("region", { name: "Este mes" });
  assert.equal(within(month).getByText("Ventas canceladas").nextElementSibling?.textContent, "1");

  const stock = within(main).getByRole("region", { name: "Alertas de stock" });
  const top = within(main).getByRole("region", { name: "Productos más vendidos · Este mes" });
  const payments = within(main).getByRole("region", { name: "Distribución de pagos · Este mes" });
  const recent = within(main).getByRole("region", { name: "Ventas recientes · Todas las fechas" });
  assert.ok(today.compareDocumentPosition(month) & 4);
  assert.ok(month.compareDocumentPosition(stock) & 4);
  assert.ok(stock.compareDocumentPosition(top) & 4);
  assert.ok(top.compareDocumentPosition(payments) & 4);
  assert.ok(payments.compareDocumentPosition(recent) & 4);

  assert.ok(within(stock).getByRole("cell", { name: "3" }));
  assert.ok(within(stock).getByRole("cell", { name: "Correa" }));
  assert.ok(within(stock).getByRole("cell", { name: "BEL-001" }));
  assert.ok(within(stock).getByRole("cell", { name: "0" }));
  assert.ok(within(stock).getByText("Sin stock"));
  assert.ok(within(top).getByRole("cell", { name: "1" }));
  assert.ok(within(top).getByRole("cell", { name: "Filtro" }));
  assert.ok(within(top).getByRole("cell", { name: "FLT-001" }));
  assert.ok(within(top).getByRole("cell", { name: "2" }));
  assert.ok(within(payments).getByRole("cell", { name: "Efectivo" }));
  assert.ok(within(payments).getByRole("cell", { name: "Bs 50,00" }));
  assert.ok(within(recent).getByRole("cell", { name: "2" }));
  assert.ok(within(recent).getByRole("cell", { name: "2024-03-10 12:00:00" }));
  assert.ok(within(recent).getByText("Confirmada"));
  assert.ok(within(recent).getByRole("cell", { name: "Bs 25,00" }));

  const button = within(stock).getByRole("button", { name: "Ver en Inventario" });
  await user.click(button);
  assert.equal(opened, true);
});

test("keeps dashboard sections loading instead of presenting loading as empty", async () => {
  let resolveIPC: ((value: unknown) => void) | undefined;
  mockIPC(() => new Promise((resolve) => { resolveIPC = resolve; }));
  render(createElement(DashboardScreen, { onOpenInventoryAlerts: () => undefined }));
  const topPanel = await screen.findByRole("region", { name: "Productos más vendidos · Este mes" });
  assert.ok(within(topPanel).getByText("Cargando productos más vendidos…"));
  assert.equal(within(topPanel).queryByText("No hay productos vendidos en este período."), null);
  assert.equal(screen.getAllByRole("status").length, 1);
  resolveIPC?.({ kind: "success", report: { today: { metrics: { effective_sale_count: 0, effective_total_centavos: 0, net_units_out: 0, cancelled_sale_count: 0 } }, month: { metrics: { effective_sale_count: 0, effective_total_centavos: 0, net_units_out: 0, cancelled_sale_count: 0 } }, top_products: [], payment_distribution: [], recent_sales: [], stock_alerts: [] } });
  assert.ok(await screen.findByText("No hay productos vendidos en este período."));
  assert.ok(screen.getByText("No hay pagos registrados en este período."));
  assert.ok(screen.getByText("No hay ventas recientes."));
  assert.ok(screen.getByText("No hay alertas de stock."));
  assert.equal(document.querySelectorAll("[data-ui-dashboard-metric] dd").length, 8);
  assert.ok([...document.querySelectorAll("[data-ui-dashboard-metric] dd")].every((value) => value.textContent === "0" || value.textContent === "Bs 0,00"));
});

test("keeps dashboard sections atomically unavailable and retries the whole report", async () => {
  let attempts = 0;
  mockIPC(() => { attempts += 1; throw new Error("native failure"); });
  const user = userEvent.setup({ document });
  render(createElement(DashboardScreen, { onOpenInventoryAlerts: () => undefined }));
  const recentPanel = await screen.findByRole("region", { name: "Ventas recientes · Todas las fechas" });
  assert.ok(within(recentPanel).getByText("No se pudieron cargar las ventas recientes."));
  assert.equal(within(recentPanel).queryByText("No hay ventas recientes."), null);
  assert.equal(screen.getAllByRole("alert").length, 1);
  assert.equal(screen.queryByRole("button", { name: "Ver en Inventario" }), null);
  await user.click(screen.getByRole("button", { name: "Reintentar" }));
  await waitFor(() => assert.equal(attempts, 2));
});
