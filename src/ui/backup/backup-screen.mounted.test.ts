import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { createElement } from "react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BackupScreen } from "./backup-screen.ts";

const prepared = { kind: "prepared", token: "restore-token", size_bytes: 2048, schema_version: 6 };
const created = { kind: "created", path: "C:\\copias\\backup.db", created_at_unix_seconds: 1_755_172_920, size_bytes: 4096, schema_version: 6 };

function mountIPC(overrides: Record<string, unknown> = {}) {
  const value = (key: string, fallback: unknown) => typeof overrides[key] === "function" ? (overrides[key] as () => unknown)() : overrides[key] ?? fallback;
  mockIPC((command, payload) => {
    if (command === "choose_backup_destination_command") return value("backupPicker", { kind: "selected", path: "C:\\copias" });
    if (command === "choose_restore_source_command") return value("restorePicker", { kind: "selected", path: "C:\\copias\\backup.db" });
    if (command === "create_backup_command") return value("create", created);
    if (command === "prepare_restore_command") return value("prepare", prepared);
    if (command === "confirm_restore_command") return value("restore", { kind: "restored" });
    throw new Error(`Unexpected command: ${command} ${JSON.stringify(payload)}`);
  });
}

test("renders Spanish continuity regions and persisted backup facts", async () => {
  mountIPC();
  const user = userEvent.setup({ document });
  render(createElement(BackupScreen));
  assert.ok(screen.getByRole("heading", { name: "Copia y restauración", level: 1 }));
  assert.equal(screen.getByRole("main").getAttribute("data-ui-backup"), "true");
  assert.ok(screen.getByRole("region", { name: "Copia de seguridad" }));
  assert.ok(screen.getByRole("region", { name: "Restauración" }));
  await user.click(screen.getByRole("button", { name: "Elegir destino de la copia" }));
  const summary = screen.getByRole("region", { name: "Última copia creada" });
  assert.equal(within(summary).getByText("C:\\copias\\backup.db").textContent, "C:\\copias\\backup.db");
  assert.ok(within(summary).getByText("14/08/2025, 12:02")); assert.ok(within(summary).getByText("4096 bytes")); assert.ok(within(summary).getByText("6"));
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
  resolvePicker({ kind: "selected", path: "C:\\copias" }); await Promise.resolve();
  assert.equal(document.body.textContent?.includes("Copia creada correctamente."), false);
});
