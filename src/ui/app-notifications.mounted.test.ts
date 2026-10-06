import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./app.ts";

function installActivation(importResult: unknown = { kind: "imported", status: "active" }) {
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "activation_required" };
    if (command === "license_installation_code_command") return { kind: "code", code: "a".repeat(64) };
    if (command === "choose_license_file_command") return { kind: "selected" };
    if (command === "import_license_command") return importResult;
    if (command === "list_inventory_alerts_command") return { kind: "alerts", alerts: [] };
    return { kind: "error", code: "persistence_failure", message: "Unavailable" };
  });
}

test("App host spans activation and shell; handoff has one success owner and survives navigation", async () => {
  installActivation();
  render(createElement(App));
  const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
  const user = userEvent.setup({ document });
  await screen.findByRole("textbox", { name: "Tu código de instalación" });
  await user.click(screen.getByRole("button", { name: "Importar archivo de licencia" }));
  assert.ok(await within(host).findByText("Licencia activada correctamente."));
  const navigation = await screen.findByRole("navigation", { name: "Navegación principal" });
  assert.equal(document.querySelector("[data-ui-activation-notice]"), null);
  assert.equal(host.closest("[data-ui-shell-content]"), null);
  await user.click(within(navigation).getByRole("button", { name: /^Inventario/ }));
  await screen.findByRole("heading", { name: "Inventario" });
  assert.equal(screen.getByRole("region", { name: "Notificaciones de acciones" }), host);
  assert.ok(within(host).getByText("Licencia activada correctamente."));
});

test("activation import failure belongs to the shared host without hiding recovery", async () => {
  installActivation({ kind: "error", code: "license_machine_mismatch" });
  render(createElement(App));
  const user = userEvent.setup({ document });
  await screen.findByRole("textbox", { name: "Tu código de instalación" });
  await user.click(screen.getByRole("button", { name: "Importar archivo de licencia" }));
  const main = screen.getByRole("main", { name: "Activá Repuestos Autos" });
  const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
  assert.ok(await within(host).findByText("El archivo de licencia no corresponde a este equipo."));
  assert.equal(within(main).queryByText("El archivo de licencia no corresponde a este equipo."), null);
  assert.ok(within(main).getByRole("button", { name: "Continuar en modo de recuperación" }));
  assert.equal(document.querySelectorAll("[data-ui-action-notice]").length, 1);
});
