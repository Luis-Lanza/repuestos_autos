import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ActionNotificationProvider } from "../visual-system/action-notifications.ts";
import { BackupScreen } from "./backup-screen.ts";

const prepared = { kind: "prepared", token: "restore-token", size_bytes: 2048, schema_version: 6 };
const created = { kind: "created", file_name: "backup-20250814.sqlite3", created_at_unix_seconds: 1_755_172_920, size_bytes: 4096, schema_version: 6, durability_warning: false, cleanup_warning: false };

function mountIPC(overrides: Record<string, unknown> = {}) {
  const value = (key: string, fallback: unknown) => typeof overrides[key] === "function" ? (overrides[key] as () => unknown)() : overrides[key] ?? fallback;
  mockIPC((command, payload) => {
    if (command === "choose_backup_destination_command") return value("backupPicker", { kind: "selected", token: "destination-token" });
    if (command === "choose_restore_source_command") return value("restorePicker", { kind: "selected", token: "source-token" });
    if (command === "create_backup_command") return value("create", created);
    if (command === "prepare_restore_command") return value("prepare", prepared);
    if (command === "confirm_restore_command") return value("restore", { kind: "restored" });
    throw new Error(`Unexpected command: ${command} ${JSON.stringify(payload)}`);
  });
}

test("shared backup and restore outcomes preserve summaries and recovery warnings", async () => {
  mountIPC();
  const user = userEvent.setup({ document });
  render(createElement(ActionNotificationProvider, null, createElement(BackupScreen)));
  const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  assert.ok(await within(host).findByText("Copia creada correctamente."));
  assert.equal(within(screen.getByRole("main")).queryByText("Copia creada correctamente."), null);
  await user.click(within(host).getByRole("button", { name: /Cerrar/ }));
  assert.ok(screen.getByRole("region", { name: "Última copia creada" }));
  mountIPC({ create: { kind: "error", code: "storage_unavailable" } });
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  assert.ok(await within(host).findByText("El almacenamiento de copias no está disponible. Reintentá."));
  await user.click(within(host).getByRole("button", { name: /Cerrar/ }));
  mountIPC({ create: { ...created, durability_warning: true } });
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  assert.ok(within(screen.getByRole("main")).getByText(/durabilidad del directorio final/));
  mountIPC();
  await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
  await user.click(screen.getByRole("button", { name: "Revisar restauración" }));
  const dialog = screen.getByRole("dialog");
  await user.click(within(dialog).getByRole("checkbox"));
  await user.click(within(dialog).getByRole("button", { name: "Restaurar datos" }));
  assert.ok(await within(host).findByText("Restauración completada correctamente."));
});

test("shared restore preparation and confirmation errors are results, not discarded candidates", async () => {
 const user = userEvent.setup({ document });
 mountIPC({ prepare: { kind: "error", code: "storage_unavailable", message: "bounded" } });
 render(createElement(ActionNotificationProvider, null, createElement(BackupScreen)));
 const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
 await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
 assert.ok(within(host).getByText("El almacenamiento local no está disponible. Reintentá."));
 await user.click(within(host).getByRole("button", { name: /Cerrar/ }));
 mountIPC({ restore: { kind: "error", code: "restore_failed", message: "bounded" } });
 await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
 await user.click(screen.getByRole("button", { name: "Revisar restauración" }));
 await user.click(screen.getByRole("checkbox"));
 await user.click(screen.getByRole("button", { name: "Restaurar datos" }));
 assert.ok(within(host).getByText("No se pudo restaurar la información local. Reintentá."));
 await user.click(within(host).getByRole("button", { name: /Cerrar/ }));
 assert.ok(screen.getByRole("region", { name: "Candidato de restauración" }));
});

test("invalid restore candidates retain inline guidance after preparation outcomes are dismissed", async () => {
  const user = userEvent.setup({ document });
  render(createElement(ActionNotificationProvider, null, createElement(BackupScreen)));
  const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
  const restoration = screen.getByRole("region", { name: "Restauración" });
  for (const [code, message] of [
    ["token_invalid", "El candidato de restauración ya no es válido. Elegí el archivo nuevamente."],
    ["invalid_backup", "El archivo de respaldo no es válido."],
    ["token_expired", "La preparación de restauración venció. Elegí el archivo nuevamente."],
  ]) {
    mountIPC({ prepare: { kind: "error", code, message: "native detail" } });
    await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
    for (const close of within(host).queryAllByRole("button", { name: /Cerrar/ })) await user.click(close);
    assert.equal((await within(restoration).findByRole("alert")).textContent, message);
    assert.equal(screen.queryByRole("region", { name: "Candidato de restauración" }), null);
    assert.equal(screen.queryByRole("dialog"), null);
  }
});

test("invalid confirmation token retains inline recovery and cannot confirm the obsolete candidate", async () => {
  let confirmations = 0;
  mountIPC({ restore: () => { confirmations++; return { kind: "error", code: "token_invalid", message: "native detail" }; } });
  const user = userEvent.setup({ document });
  render(createElement(ActionNotificationProvider, null, createElement(BackupScreen)));
  await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
  await user.click(screen.getByRole("button", { name: "Revisar restauración" }));
  await user.click(screen.getByRole("checkbox"));
  await user.click(screen.getByRole("button", { name: "Restaurar datos" }));
  const host = screen.getByRole("region", { name: "Notificaciones de acciones" });
  for (const close of within(host).queryAllByRole("button", { name: /Cerrar/ })) await user.click(close);
  const restoration = screen.getByRole("region", { name: "Restauración" });
  assert.equal((await within(restoration).findByRole("alert")).textContent, "El candidato de restauración ya no es válido. Elegí el archivo nuevamente.");
  assert.equal((within(restoration).getByRole("button", { name: "Revisar restauración" }) as HTMLButtonElement).disabled, true);
  const confirm = screen.queryByRole("button", { name: "Restaurar datos" });
  if (confirm) await user.click(confirm);
  assert.equal(confirmations, 1);
  assert.equal(screen.queryByText("Restauración completada correctamente."), null);
});

test("surviving backup host rejects duplicate and unmounted creation", async () => {
 let finish!: (value: unknown) => void; let calls = 0;
 mountIPC({ create: () => { calls++; return new Promise(resolve => { finish = resolve; }); } });
 const view = render(createElement(ActionNotificationProvider, null, createElement(BackupScreen)));
 const picker = screen.getByRole("button", { name: "Elegir destino de la copia" });
 fireEvent.click(picker); fireEvent.click(picker);
 await waitFor(() => assert.equal(calls, 1));
 view.rerender(createElement(ActionNotificationProvider));
 await act(async () => { finish(created); });
 assert.equal(document.querySelectorAll('[data-ui-action-notice]').length, 0);
});

test("renders Spanish continuity regions and a path-free backup summary", async () => {
  mountIPC();
  const user = userEvent.setup({ document });
  render(createElement(BackupScreen));
  assert.ok(screen.getByRole("heading", { name: "Copia y restauración", level: 1 }));
  assert.equal(screen.getByRole("main").getAttribute("data-ui-backup"), "true");
  assert.ok(screen.getByRole("region", { name: "Copia de seguridad" }));
  assert.ok(screen.getByRole("region", { name: "Restauración" }));
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  const summary = screen.getByRole("region", { name: "Última copia creada" });
  assert.ok(within(summary).getByText("backup-20250814.sqlite3"));
  assert.equal(within(summary).queryByText(/C:\\\\copias/), null);
  assert.ok(within(summary).getByText("14/08/2025, 12:02")); assert.ok(within(summary).getByText("4096 bytes")); assert.ok(within(summary).getByText("6"));
});

test("announces durability uncertainty and internal cleanup failure as separate accessible warnings", async () => {
  mountIPC({ create: { ...created, durability_warning: true, cleanup_warning: true } });
  const user = userEvent.setup({ document }); render(createElement(BackupScreen));
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  assert.match((await screen.findByRole("alert")).textContent ?? "", /durabilidad del directorio final/);
  assert.match(screen.getByRole("status").textContent ?? "", /limpieza de un archivo interno temporal/);
  assert.equal(screen.queryByText("Copia creada correctamente."), null);
});

test("keeps picker cancellation harmless and bounds invalid, expired, and unavailable feedback", async () => {
  let restoreError: unknown = { kind: "cancelled" };
  mountIPC({ backupPicker: { kind: "cancelled" }, restorePicker: { kind: "cancelled" } });
  const user = userEvent.setup({ document }); render(createElement(BackupScreen));
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  assert.equal(screen.queryByRole("alert"), null);
  await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
  assert.equal(screen.queryByRole("alert"), null);
  for (const code of ["invalid_backup", "token_expired", "storage_unavailable", "restore_failed", "recovery_failed"]) {
    restoreError = { kind: "error", code, message: "native detail" };
    mountIPC({ prepare: restoreError });
    await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
    const copy = await screen.findByRole("alert");
    assert.ok(copy.textContent?.includes(code === "invalid_backup" ? "válido" : code === "token_expired" ? "venció" : code === "recovery_failed" ? "recuperar" : code === "restore_failed" ? "restaurar" : "disponible"));
  }
});

test("shows license denial after restore prepare without entering the confirmation flow", async () => {
  mountIPC({ prepare: { kind: "error", code: "license_required", message: "native detail" } });
  const user = userEvent.setup({ document });
  render(createElement(BackupScreen));
  await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
  assert.match((await screen.findByRole("alert")).textContent ?? "", /licencia válida/);
  assert.equal(screen.queryByRole("dialog"), null);
  assert.equal(screen.queryByRole("region", { name: "Candidato de restauración" }), null);
});

test("shows license denial after confirmation without reporting restore success", async () => {
  mountIPC({ restore: { kind: "error", code: "license_required", message: "native detail" } });
  const user = userEvent.setup({ document });
  render(createElement(BackupScreen));
  await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
  await user.click(screen.getByRole("button", { name: "Revisar restauración" }));
  const dialog = screen.getByRole("dialog", { name: "Restaurar datos locales" });
  await user.click(within(dialog).getByRole("checkbox"));
  await user.click(within(dialog).getByRole("button", { name: "Restaurar datos" }));
  assert.match((await screen.findByRole("alert")).textContent ?? "", /licencia válida/);
  assert.equal(screen.queryByText("Restauración completada correctamente."), null);
});

test("gates acknowledgement, uses the exact restore dialog, contains focus, and locks pending", async () => {
  let resolveRestore!: (value: unknown) => void; let restores = 0;
  mountIPC({ restore: () => { restores++; return new Promise((resolve) => { resolveRestore = resolve; }); } });
  const user = userEvent.setup({ document }); render(createElement(BackupScreen));
  await user.click(screen.getByRole("button", { name: "Elegir archivo de respaldo" }));
  await user.click(screen.getByRole("button", { name: "Revisar restauración" }));
  const dialog = screen.getByRole("dialog", { name: "Restaurar datos locales" });
  assert.equal(dialog.getAttribute("aria-describedby") !== null, true);
  assert.ok(within(dialog).getByText("Esta acción reemplazará los datos locales actuales"));
  const confirm = within(dialog).getByRole("button", { name: "Restaurar datos" }) as HTMLButtonElement;
  await user.click(confirm); assert.equal(restores, 0);
  const acknowledgement = within(dialog).getByRole("checkbox", { name: /reemplaza los datos locales/ });
  await user.click(acknowledgement); await user.click(confirm); await user.click(confirm);
  assert.equal(restores, 1); assert.equal((acknowledgement as HTMLInputElement).disabled, true);
  assert.equal(screen.getByRole("dialog").getAttribute("aria-busy"), "true");
  await user.keyboard("{Tab}"); assert.equal(document.activeElement?.closest("[role=dialog]"), dialog);
  await user.keyboard("{Shift>}{Tab}{/Shift}"); assert.equal(document.activeElement?.closest("[role=dialog]"), dialog);
  resolveRestore({ kind: "restored" });
  await waitFor(() => assert.ok(screen.getByText("Restauración completada correctamente.")));
  assert.equal(screen.queryByRole("dialog"), null);
});

test("keeps backup panels top-aligned without changing their semantic regions", async () => {
  const css = await readFile(new URL("../styles.css", import.meta.url), "utf8");
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*data-ui-backup\][^}]*align-content:\s*start/);
  assert.match(css, /@media \(min-width: 961px\)[\s\S]*data-ui-backup-layout\][^}]*align-content:\s*start/);
});

test("does not duplicate backup picker activation", async () => {
  let resolvePicker!: (value: unknown) => void; let calls = 0;
  mountIPC({ backupPicker: () => { calls++; return new Promise((resolve) => { resolvePicker = resolve; }); } });
  const user = userEvent.setup({ document }); render(createElement(BackupScreen));
  const picker = screen.getByRole("button", { name: "Elegir destino de la copia" });
  await user.click(picker); await user.click(picker); assert.equal(calls, 1);
  resolvePicker({ kind: "cancelled" });
});

test("ignores completion after the backup screen unmounts", async () => {
  let resolvePicker!: (value: unknown) => void;
  mountIPC({ backupPicker: () => new Promise((resolve) => { resolvePicker = resolve; }) });
  const user = userEvent.setup({ document }); const view = render(createElement(BackupScreen));
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" })); view.unmount();
  resolvePicker({ kind: "selected", token: "destination-token" }); await Promise.resolve();
  assert.equal(document.body.textContent?.includes("Copia creada correctamente."), false);
});
