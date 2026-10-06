import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ActionNotificationProvider } from "../visual-system/action-notifications.ts";
import { ActivationScreen } from "./activation-screen.ts";

test("shared licensing cancellation and failure preserve installation guidance and suppress late imports", async () => {
 let selection: unknown = { kind: "cancelled" };
 let finish!: (value: unknown) => void;
 let picks = 0;
 mockIPC(command => {
  if (command === "license_status_command") return { kind: "status", code: "license_missing" };
  if (command === "license_installation_code_command") return { kind: "code", code: "b".repeat(64) };
  if (command === "choose_license_file_command") { picks++; return selection; }
  throw new Error(command);
 });
 const view = render(createElement(ActionNotificationProvider, null, createElement(ActivationScreen, { onRecovery: () => undefined, onActivated: () => assert.fail("unexpected success") })));
 await screen.findByRole("textbox", { name: "Tu código de instalación" });
 const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
 const user = userEvent.setup({ document });
 await user.click(screen.getByRole("button", { name: "Importar archivo de licencia" }));
 assert.ok(within(host).getByText("No se seleccionó ningún archivo."));
 assert.equal(document.querySelector('[data-ui-action-notice]')?.getAttribute("data-severity"), "info");
 await user.click(within(host).getByRole("button", { name: /Cerrar/ }));
 selection = { kind: "error", code: "license_file_invalid" };
 await user.click(screen.getByRole("button", { name: "Importar archivo de licencia" }));
 assert.equal(document.querySelector('[data-ui-action-notice]')?.getAttribute("data-severity"), "error");
 assert.ok(screen.getByRole("textbox", { name: "Tu código de instalación" }));
 await user.click(within(host).getByRole("button", { name: /Cerrar/ }));
 selection = new Promise(resolve => { finish = resolve; });
 const button = screen.getByRole("button", { name: "Importar archivo de licencia" });
 fireEvent.click(button); fireEvent.click(button);
 await waitFor(() => assert.equal(picks, 3));
 view.rerender(createElement(ActionNotificationProvider));
 await act(async () => { finish({ kind: "cancelled" }); });
 assert.equal(document.querySelectorAll('[data-ui-action-notice]').length, 0);
});

test("activation screen exposes an accessible installation code and recovery action", async () => {
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "activation_required" };
    if (command === "license_installation_code_command") return { kind: "code", code: "a".repeat(64) };
    throw new Error(`Unexpected command: ${command}`);
  });
  let recovery = 0;
  const user = userEvent.setup({ document });
  render(createElement(ActivationScreen, { onRecovery: () => recovery++, onActivated: () => undefined }));
  const input = await screen.findByRole("textbox", { name: "Tu código de instalación" });
  assert.equal((input as HTMLInputElement).readOnly, true);
  assert.equal((input as HTMLInputElement).value, "a".repeat(64));
  assert.ok(screen.getByRole("button", { name: "Importar archivo de licencia" }));
  await user.click(screen.getByRole("button", { name: "Continuar en modo de recuperación" }));
  assert.equal(recovery, 1);
  await waitFor(() => assert.ok(screen.getByRole("heading", { name: "Activá Repuestos Autos" })));
  assert.doesNotMatch(document.body.textContent ?? "", /expir|suscrip|renov|servidor|C:\\|signature/i);
});

test("activation import status is announced and cancellation is not an alert", async () => {
  mockIPC((command) => {
    if (command === "license_status_command") return { kind: "status", code: "license_missing" };
    if (command === "license_installation_code_command") return { kind: "code", code: "b".repeat(64) };
    if (command === "choose_license_file_command") return { kind: "cancelled" };
    throw new Error(`Unexpected command: ${command}`);
  });
  const user = userEvent.setup({ document });
  render(createElement(ActivationScreen, { onRecovery: () => undefined, onActivated: () => undefined }));
  await user.click(await screen.findByRole("button", { name: "Importar archivo de licencia" }));
  assert.equal((await screen.findByText("No se seleccionó ningún archivo.")).getAttribute("aria-live"), "polite");
  assert.equal(screen.queryByRole("alert"), null);
});
