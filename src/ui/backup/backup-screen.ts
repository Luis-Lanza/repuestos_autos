import { createElement, useReducer } from "react";

import { backupCommands, type BackupResponse } from "../../commands/backup.ts";
import { canConfirmRestore, createBackupFlow, initialBackupState } from "./backup-flow.ts";

const describeError = (response: BackupResponse) => response.kind === "error" ? `${response.code}: ${response.message}` : "storage_unavailable: Backup storage is unavailable.";
export function BackupScreen() {
  const [state, dispatch] = useReducer(createBackupFlow, initialBackupState);
  const backup = async () => { dispatch({ type: "backup_started" }); const selection = await backupCommands.chooseBackupDestination(); if (selection.kind === "error") return dispatch({ type: "failed", message: describeError(selection) }); if (selection.kind === "cancelled") return dispatch({ type: "backup_cancelled" }); const response = await backupCommands.createBackup(selection.path); dispatch(response.kind === "created" ? { type: "backup_succeeded", summary: response.summary } : { type: "failed", message: describeError(response) }); };
  const prepare = async () => { dispatch({ type: "restore_started" }); const selection = await backupCommands.chooseRestoreSource(); if (selection.kind === "error") return dispatch({ type: "failed", message: describeError(selection) }); if (selection.kind === "cancelled") return dispatch({ type: "failed", message: "Restore selection was cancelled." }); const response = await backupCommands.prepareRestore(selection.path); dispatch(response.kind === "prepared" ? { type: "restore_prepared", summary: response.summary } : { type: "failed", message: describeError(response) }); };
  const restore = async () => { if (!state.summary) return; dispatch({ type: "restore_started" }); const response = await backupCommands.confirmRestore(state.summary.token); dispatch(response.kind === "restored" ? { type: "restore_succeeded" } : { type: "failed", message: describeError(response) }); };
  return createElement("main", { "aria-labelledby": "backup-heading" },
    createElement("h1", { id: "backup-heading" }, "Backup and restore"),
    createElement("p", null, "Create a local backup or restore a validated backup file."),
    createElement("button", { type: "button", disabled: state.backup_status === "pending", onClick: backup }, state.backup_status === "pending" ? "Creating backup…" : "Choose backup destination"),
    state.backup ? createElement("p", { role: "status" }, `Backup created at ${state.backup.path} on ${new Date(state.backup.created_at_unix_seconds * 1000).toISOString()}. ${state.backup.size_bytes} bytes, schema version ${state.backup.schema_version}.`) : null,
    createElement("h2", null, "Restore"),
    createElement("button", { type: "button", disabled: state.restore_status === "pending", onClick: prepare }, state.restore_status === "pending" ? "Preparing restore…" : "Choose backup file"),
    state.summary ? createElement("section", { "aria-labelledby": "restore-summary-heading" }, createElement("h3", { id: "restore-summary-heading" }, "Restore candidate"), createElement("p", null, `${state.summary.size_bytes} bytes, schema version ${state.summary.schema_version}.`), createElement("p", null, "This will replace the current local data."), createElement("label", null, createElement("input", { type: "checkbox", checked: state.confirmed, onChange: (event) => dispatch({ type: "restore_confirmation_changed", confirmed: event.target.checked }) }), " I understand that restore replaces local data."), createElement("button", { type: "button", disabled: !canConfirmRestore(state), onClick: restore }, state.restore_status === "pending" ? "Restoring…" : "Confirm restore")) : null,
    state.feedback ? createElement("p", { role: state.feedback.includes(":") ? "alert" : "status" }, state.feedback) : null,
  );
}
