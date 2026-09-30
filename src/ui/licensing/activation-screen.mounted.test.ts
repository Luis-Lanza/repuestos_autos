import assert from "node:assert/strict";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ActivationScreen } from "./activation-screen.ts";

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
