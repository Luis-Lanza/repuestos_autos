import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { DashboardScreen } from "./dashboard-screen.ts";

test("renders dashboard sections and stock navigation from one snapshot", async () => {
  mockIPC((command) => {
    assert.equal(command, "dashboard_command");
    return { kind: "success", report: { today: { metrics: { effective_sale_count: 1, effective_total_centavos: 2500, net_units_out: 1, cancelled_sale_count: 0 } }, month: { metrics: { effective_sale_count: 2, effective_total_centavos: 5000, net_units_out: 2, cancelled_sale_count: 1 } }, top_products: [{ product_id: 1, sku: "FLT-001", product_name: "Filtro", net_units_out: 2 }], payment_distribution: [{ method: "cash", amount_applied_centavos: 5000 }], recent_sales: [{ sale_id: 2, confirmed_at: "2024-03-10 12:00:00", status: "confirmed", total_centavos: 2500 }], stock_alerts: [{ product_id: 3, sku: "BEL-001", product_name: "Correa", quantity: 0, classification: "out_of_stock" }] } };
  });
  const user = userEvent.setup({ document });
  let opened = false;
  render(createElement(DashboardScreen, { onOpenInventoryAlerts: () => { opened = true; } }));
  assert.ok(await screen.findByRole("heading", { name: "Dashboard" }));
  assert.ok(await screen.findByText("Filtro"));
  const button = screen.getByRole("button", { name: "Ver en Inventario" });
  await user.click(button);
  assert.equal(opened, true);
  await waitFor(() => assert.equal(screen.getByRole("heading", { name: "Dashboard" }).textContent, "Dashboard"));
  assert.equal(within(screen.getByRole("main")).getAllByText("Ventas efectivas").length, 2);
  const stockPanel = screen.getByRole("region", { name: "Alertas de stock" });
  assert.ok(within(stockPanel).getByRole("columnheader", { name: "Stock actual" }));
  assert.equal(within(stockPanel).queryByRole("columnheader", { name: "Unidades netas" }), null);
});

test("keeps dashboard sections loading instead of presenting loading as empty", async () => {
  let resolveIPC: ((value: unknown) => void) | undefined;
  mockIPC(() => new Promise((resolve) => { resolveIPC = resolve; }));
  render(createElement(DashboardScreen, { onOpenInventoryAlerts: () => undefined }));
  const topPanel = await screen.findByRole("region", { name: "Productos más vendidos" });
  assert.ok(within(topPanel).getByText("Cargando productos más vendidos…"));
  assert.equal(within(topPanel).queryByText("No hay productos vendidos en este período."), null);
  resolveIPC?.({ kind: "success", report: { today: { metrics: { effective_sale_count: 0, effective_total_centavos: 0, net_units_out: 0, cancelled_sale_count: 0 } }, month: { metrics: { effective_sale_count: 0, effective_total_centavos: 0, net_units_out: 0, cancelled_sale_count: 0 } }, top_products: [], payment_distribution: [], recent_sales: [], stock_alerts: [] } });
  assert.ok(await screen.findByText("No hay productos vendidos en este período."));
});

test("keeps dashboard sections in error state instead of presenting failure as empty", async () => {
  mockIPC(() => { throw new Error("native failure"); });
  render(createElement(DashboardScreen, { onOpenInventoryAlerts: () => undefined }));
  const recentPanel = await screen.findByRole("region", { name: "Ventas recientes" });
  assert.ok(within(recentPanel).getByText("No se pudieron cargar las ventas recientes."));
  assert.equal(within(recentPanel).queryByText("No hay ventas recientes."), null);
});
