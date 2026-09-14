import type { BackupResponse, BackupSummary, RestoreSummary } from "../../commands/backup.ts";

export type BackupStatus = "idle" | "pending" | "success" | "failure" | "unavailable";
export type RestoreStatus = "idle" | "prepared" | "invalid" | "expired" | "unavailable" | "pending" | "success" | "failure" | "recovery";
export interface BackupState { backup_status: BackupStatus; restore_status: RestoreStatus; summary: RestoreSummary | null; confirmed: boolean; backup: BackupSummary | null; feedback: string | null; }
export const initialBackupState: BackupState = { backup_status: "idle", restore_status: "idle", summary: null, confirmed: false, backup: null, feedback: null };
export type BackupAction =
  | { type: "backup_started" } | { type: "backup_cancelled" } | { type: "backup_succeeded"; summary: BackupSummary }
  | { type: "restore_prepared"; summary: RestoreSummary } | { type: "restore_confirmation_changed"; confirmed: boolean }
  | { type: "restore_started" } | { type: "restore_confirm_started" } | { type: "restore_succeeded" } | { type: "backup_failed"; status?: "failure" | "unavailable"; message: string }
  | { type: "restore_failed"; status: Exclude<RestoreStatus, "idle" | "prepared" | "pending" | "success">; message: string }
  | { type: "restore_cancelled" } | { type: "failed"; message: string };

export function createBackupFlow(state: BackupState, action: BackupAction): BackupState {
  switch (action.type) {
    case "backup_started": return { ...state, backup_status: "pending", feedback: null };
    case "backup_cancelled": return { ...state, backup_status: "idle", feedback: "Selección de copia cancelada." };
    case "backup_succeeded": return { ...state, backup_status: "success", backup: action.summary, feedback: "Copia creada correctamente." };
    case "restore_prepared": return { ...state, restore_status: "prepared", summary: action.summary, confirmed: false, feedback: null };
    case "restore_confirmation_changed": return { ...state, confirmed: action.confirmed };
    case "restore_started": return { ...state, restore_status: "pending", feedback: null };
    case "restore_confirm_started": return { ...state, restore_status: "pending", feedback: null };
    case "restore_succeeded": return { ...state, restore_status: "success", summary: null, confirmed: false, feedback: "Restauración completada correctamente." };
    case "backup_failed": return { ...state, backup_status: action.status ?? "failure", feedback: action.message };
    case "restore_failed": return { ...state, restore_status: action.status, feedback: action.message };
    case "restore_cancelled": return { ...state, restore_status: state.summary ? "prepared" : "idle", feedback: null };
    case "failed": return state.backup_status === "pending" ? { ...state, backup_status: "failure", feedback: action.message } : { ...state, restore_status: "failure", feedback: action.message };
  }
}
export const canConfirmRestore = (state: BackupState) => state.summary !== null && state.restore_status === "prepared" && state.confirmed;

export function backupFeedback(code: string, restore = false) {
  if (restore) return code === "invalid_backup" ? "El archivo de respaldo no es válido." : code === "token_expired" ? "La preparación de restauración venció. Elegí el archivo nuevamente." : code === "token_invalid" ? "El candidato de restauración ya no es válido. Elegí el archivo nuevamente." : code === "database_unavailable" || code === "storage_unavailable" ? "El almacenamiento local no está disponible. Reintentá." : code === "recovery_failed" ? "No se pudo recuperar la restauración. Reintentá." : code === "restore_failed" ? "No se pudo restaurar la información local. Reintentá." : "No se pudo preparar la restauración. Reintentá.";
  return code === "destination_exists" ? "Ya existe una copia en ese destino. Elegí otro." : code === "storage_unavailable" ? "El almacenamiento de copias no está disponible. Reintentá." : "No se pudo crear la copia. Reintentá.";
}
export function restoreErrorStatus(code: string): Exclude<RestoreStatus, "idle" | "prepared" | "pending" | "success"> { return code === "invalid_backup" || code === "unsupported_schema" ? "invalid" : code === "token_expired" ? "expired" : code === "storage_unavailable" || code === "database_unavailable" ? "unavailable" : code === "recovery_failed" ? "recovery" : "failure"; }

export interface BackupCommands { chooseBackupDestination: () => Promise<{ kind: "cancelled" } | { kind: "selected"; path: string } | { kind: "error"; code: string }>; chooseRestoreSource: () => Promise<{ kind: "cancelled" } | { kind: "selected"; path: string } | { kind: "error"; code: string }>; createBackup: (path: string) => Promise<BackupResponse>; prepareRestore: (path: string) => Promise<BackupResponse>; confirmRestore: (token: string) => Promise<BackupResponse>; }
export function createBackupInteraction(commands: BackupCommands, dispatch: (action: BackupAction) => void) {
  let mounted = true; let backupBusy = false; let restoreBusy = false; let backupId = 0; let restoreId = 0;
  const backup = async () => { if (!mounted || backupBusy) return; backupBusy = true; const id = ++backupId; dispatch({ type: "backup_started" }); const selected = await commands.chooseBackupDestination(); if (!mounted || id !== backupId) return void (backupBusy = false); if (selected.kind === "cancelled") { backupBusy = false; return dispatch({ type: "backup_cancelled" }); } if (selected.kind === "error") { backupBusy = false; return dispatch({ type: "backup_failed", status: selected.code === "storage_unavailable" ? "unavailable" : "failure", message: backupFeedback(selected.code) }); } const result = await commands.createBackup(selected.path); if (!mounted || id !== backupId) return void (backupBusy = false); backupBusy = false; return result.kind === "created" ? dispatch({ type: "backup_succeeded", summary: result.summary }) : dispatch({ type: "backup_failed", status: result.kind === "error" && result.code === "storage_unavailable" ? "unavailable" : "failure", message: backupFeedback(result.kind === "error" ? result.code : "storage_unavailable") }); };
  const prepareRestore = async () => { if (!mounted || restoreBusy) return; restoreBusy = true; const id = ++restoreId; dispatch({ type: "restore_started" }); const selected = await commands.chooseRestoreSource(); if (!mounted || id !== restoreId) return void (restoreBusy = false); if (selected.kind === "cancelled") { restoreBusy = false; return dispatch({ type: "restore_cancelled" }); } if (selected.kind === "error") { restoreBusy = false; return dispatch({ type: "restore_failed", status: restoreErrorStatus(selected.code), message: backupFeedback(selected.code, true) }); } const result = await commands.prepareRestore(selected.path); if (!mounted || id !== restoreId) return void (restoreBusy = false); restoreBusy = false; return result.kind === "prepared" ? dispatch({ type: "restore_prepared", summary: result.summary }) : dispatch({ type: "restore_failed", status: result.kind === "error" ? restoreErrorStatus(result.code) : "failure", message: backupFeedback(result.kind === "error" ? result.code : "storage_unavailable", true) }); };
  const restore = async (token: string) => { if (!mounted || restoreBusy) return; restoreBusy = true; const id = ++restoreId; dispatch({ type: "restore_confirm_started" }); const result = await commands.confirmRestore(token); if (!mounted || id !== restoreId) return void (restoreBusy = false); restoreBusy = false; return result.kind === "restored" ? dispatch({ type: "restore_succeeded" }) : dispatch({ type: "restore_failed", status: result.kind === "error" ? restoreErrorStatus(result.code) : "failure", message: backupFeedback(result.kind === "error" ? result.code : "restore_failed", true) }); };
  return { backup, prepareRestore, restore, dispose: () => { mounted = false; backupId++; restoreId++; } };
}
