import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryScreen } from "./inventory-screen.ts";

const product = {
  product_id: 1,
  sku: "FLT",
  name: "Filter",
  category_name: "Filters",
  available_quantity: 8,
  catalog_unit_price_centavos: 2500,
  revision: 0,
};

function successfulOperation(requestId: string) {
  return {
    kind: "success",
    request_id: requestId,
    product_id: product.product_id,
    previous_quantity: product.available_quantity,
    quantity_delta: 3,
    resulting_quantity: 11,
    occurred_at: "2025-01-01T00:00:00Z",
    note: null,
  };
}

async function searchAndSelect() {
  const user = userEvent.setup({ document });
  const search = screen.getByRole("textbox", { name: "Search catalog" });
  await user.type(search, "filter{Enter}");
  const select = await screen.findByRole("button", { name: "Select" });
  select.focus();
  await user.keyboard("{Enter}");
  await screen.findByRole("heading", { name: "Filter", level: 2 });
  return user;
}

test("mounts Inventory and supports keyboard catalog search and selection", async () => {
  const commands: Array<{ command: string; payload: unknown }> = [];
  mockIPC((command, payload) => {
    commands.push({ command, payload });
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "search_products_command") return [product];
    throw new Error(`Unexpected command: ${command}`);
  });

  render(createElement(InventoryScreen));

  screen.getByRole("heading", { name: "Inventory", level: 1 });
  await searchAndSelect();
  assert.deepEqual(commands, [
    { command: "list_inventory_alerts_command", payload: {} },
    { command: "search_products_command", payload: { request: { query: "filter" } } },
  ]);
});

test("keeps Saving disabled until confirmation resolves and announces success", async () => {
  let resolveConfirmation!: (value: ReturnType<typeof successfulOperation>) => void;
  const confirmation = new Promise<ReturnType<typeof successfulOperation>>((resolve) => {
    resolveConfirmation = resolve;
  });
  let confirmationRequestId = "";
  mockIPC((command, payload) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "search_products_command") return [product];
    if (command === "confirm_stock_entry_command") {
      confirmationRequestId = String((payload?.request as { request_id?: unknown }).request_id);
      return confirmation;
    }
    throw new Error(`Unexpected command: ${command}`);
  });

  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Quantity" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirm operation" }));

  const saving = screen.getByRole("button", { name: "Saving…" });
  assert.equal((saving as HTMLButtonElement).disabled, true);

  assert.match(confirmationRequestId, /^[0-9a-f-]{36}$/i);
  resolveConfirmation(successfulOperation(confirmationRequestId));
  assert.match((await screen.findByRole("status")).textContent ?? "", /Saved 3; balance is 11\./);
});

test("announces an isolated inventory command failure", async () => {
  mockIPC((command) => {
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    if (command === "search_products_command") return [product];
    if (command === "confirm_stock_entry_command") {
      return { kind: "error", code: "persistence_failure", message: "Native detail" };
    }
    throw new Error(`Unexpected command: ${command}`);
  });

  render(createElement(InventoryScreen));
  const user = await searchAndSelect();
  await user.type(screen.getByRole("spinbutton", { name: "Quantity" }), "3");
  await user.click(screen.getByRole("button", { name: "Confirm operation" }));

  assert.equal(
    (await screen.findByRole("alert")).textContent,
    "persistence_failure: The inventory operation could not be completed.",
  );
});
