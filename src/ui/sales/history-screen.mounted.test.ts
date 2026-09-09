import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { SalesHistoryScreen } from "./history-screen.ts";

const summary = (sale_id: number) => ({
  sale_id,
  confirmed_at: "2024-03-10 05:00:00",
  status: "confirmed",
  total_centavos: 2_500,
  line_count: 1,
  payment_count: 1,
  payment_methods: ["cash"],
  has_corrections: false,
});
const detail = (sale_id: number) => ({
  ...summary(sale_id),
  lines: [{
    sale_line_id: sale_id,
    product_id: 4,
    sku: "FLT",
    product_name: `Filter ${sale_id}`,
    quantity: 1,
    unit_price_centavos: 2_500,
    line_total_centavos: 2_500,
    returned_quantity: 0,
    cancellation_restored_quantity: 0,
    remaining_returnable_quantity: 1,
  }],
  payments: [{
    method: "cash",
    amount_applied_centavos: 2_500,
    amount_tendered_centavos: 2_500,
    change_given_centavos: 0,
  }],
  returns: [],
  cancellation: null,
});
const deferred = <Value,>() => {
  let resolve!: (value: Value) => void;
  const promise = new Promise<Value>((done) => { resolve = done; });
  return { promise, resolve };
};

test("keeps the newer mounted list when an older load finishes late", async () => {
  const lists = [deferred<unknown>(), deferred<unknown>()];
  let listCall = 0;
  mockIPC((command) => {
    if (command === "list_sales_history_command") return lists[listCall++].promise;
    throw new Error(command);
  });
  render(createElement(SalesHistoryScreen));
  const load = screen.getByRole("button", { name: "Cargando…" });

  fireEvent.submit(load.closest("form")!);
  lists[1].resolve({ kind: "success", sales: [summary(72)], has_more: false });
  assert.ok(await screen.findByText("Venta #72"));
  lists[0].resolve({ kind: "success", sales: [summary(71)], has_more: false });
  await waitFor(() => assert.ok(screen.getByText("Venta #72")));
});

test("late correction success cannot replace a newer selected sale", async () => {
  const correction = deferred<unknown>();
  const detailCalls: number[] = [];
  mockIPC((command, payload) => {
    if (command === "list_sales_history_command")
      return { kind: "success", sales: [summary(71), summary(72)], has_more: false };
    if (command === "sale_history_detail_command") {
      const saleId = Number(payload?.saleId);
      detailCalls.push(saleId);
      return { kind: "success", detail: detail(saleId) };
    }
    if (command === "create_sale_return_command") return correction.promise;
    throw new Error(command);
  });
  render(createElement(SalesHistoryScreen));
  const user = userEvent.setup({ document });
  await screen.findByText("Venta #71");
  await user.click(screen.getAllByRole("button", { name: "Ver detalle" })[0]);
  await user.click(await screen.findByRole("button", { name: "Begin item return" }));
  await user.click(screen.getByRole("checkbox", { name: "Include this original sale line" }));
  await user.type(screen.getByRole("spinbutton", { name: "Return quantity" }), "1");
  await user.click(screen.getByRole("button", { name: "Record inventory return" }));
  await user.click(screen.getByRole("button", { name: "Back to history" }));
  await user.click(screen.getAllByRole("button", { name: "Ver detalle" })[1]);
  assert.ok(await screen.findByText(/Sale 72 ·/));

  correction.resolve({
    kind: "success",
    result: {
      request_id: "late-return",
      return_id: 9,
      sale_id: 71,
      status: "confirmed",
      occurred_at: "2024-03-11 05:00:00",
      lines: [{ sale_line_id: 71, product_id: 4, quantity: 1 }],
    },
  });
  await waitFor(() => assert.ok(screen.getByText(/Sale 72 ·/)));
  assert.deepEqual(detailCalls, [71, 72]);
});

test("ignores mounted list completion after unmount", async () => {
  const list = deferred<unknown>();
  mockIPC((command) => {
    if (command === "list_sales_history_command") return list.promise;
    throw new Error(command);
  });
  const view = render(createElement(SalesHistoryScreen));
  assert.ok(screen.getByText("Cargando historial de ventas…"));
  view.unmount();
  list.resolve({ kind: "success", sales: [summary(71)], has_more: false });
  await list.promise;
  assert.equal(document.body.textContent, "");
});

test("renders the bounded Spanish history list as scannable semantic data", async () => {
  mockIPC((command) => {
    if (command === "list_sales_history_command") return {
      kind: "success",
      sales: [
        summary(71),
        { ...summary(72), confirmed_at: "private native timestamp", status: "cancelled", payment_count: 2, payment_methods: ["cash", "qr"] },
      ],
      has_more: true,
    };
    if (command === "sale_history_detail_command") return { kind: "success", detail: detail(Number(71)) };
    throw new Error(command);
  });
  render(createElement(SalesHistoryScreen));
  const user = userEvent.setup({ document });

  assert.equal(screen.getByRole("heading", { level: 1 }).textContent, "Historial de ventas");
  assert.ok(screen.getByLabelText("Desde"));
  assert.ok(screen.getByLabelText("Hasta"));
  const table = await screen.findByRole("table", { name: "Ventas del período" });
  for (const heading of ["Venta", "Fecha y hora", "Estado", "Artículos", "Pagos", "Total", "Acción"])
    assert.ok(screen.getByRole("columnheader", { name: heading }));
  assert.match(table.textContent ?? "", /Venta #71.*10\/03\/2024, 05:00.*Confirmada.*1 artículo.*Efectivo.*Bs 25,00/s);
  assert.match(table.textContent ?? "", /Venta #72.*Fecha no disponible.*Cancelada.*2 pagos · Efectivo y QR/s);
  assert.ok(screen.getByText("Hay más ventas. Reducí el rango de fechas."));
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /data-ui-history-filters[^}]*repeat\(2, minmax\(10rem, 1fr\)\)[^}]*auto/);
  assert.match(css, /max-width: 960px[\s\S]*data-ui-history-filters[^}]*repeat\(2, minmax\(0, 1fr\)\)/);
  assert.match(css, /data-ui-history-filters[^}]*> button[^}]*grid-column: 1 \/ -1/);
  assert.match(css, /max-width: 960px[\s\S]*data-ui-aligned-data[^}]*tbody[^}]*display: block/);

  await user.click(screen.getAllByRole("button", { name: "Ver detalle" })[0]);
  assert.ok(await screen.findByText(/Sale 71 ·/));
});

test("preserves inclusive date commands and shows bounded empty and error recovery copy", async () => {
  const requests: unknown[] = [];
  let attempt = 0;
  mockIPC((command, payload) => {
    if (command !== "list_sales_history_command") throw new Error(command);
    requests.push(payload?.request);
    attempt += 1;
    if (attempt === 1) return { kind: "success", sales: [], has_more: false };
    return { kind: "error", code: "persistence_failure", message: "private sqlite path and stack" };
  });
  render(createElement(SalesHistoryScreen));
  const user = userEvent.setup({ document });

  assert.ok(await screen.findByText("No hay ventas en este rango."));
  await user.clear(screen.getByLabelText("Desde"));
  await user.type(screen.getByLabelText("Desde"), "2024-03-10");
  await user.clear(screen.getByLabelText("Hasta"));
  await user.type(screen.getByLabelText("Hasta"), "2024-03-11");
  await user.click(screen.getByRole("button", { name: "Cargar historial" }));

  assert.ok(await screen.findByRole("alert"));
  assert.ok(screen.getByText("No se pudo cargar el historial de ventas."));
  assert.equal(screen.queryByText(/sqlite|stack/i), null);
  assert.ok(screen.getByRole("button", { name: "Reintentar historial" }));
  assert.equal((requests[1] as { from_utc: string }).from_utc.slice(0, 10), "2024-03-10");
  assert.equal(new Date((requests[1] as { to_exclusive_utc: string }).to_exclusive_utc).getDate(), 12);
});
