const KIND = { CANCELLED: "cancelled", SELECTED: "selected", CREATED: "created", PREPARED: "prepared", RESTORED: "restored", ERROR: "error" } as const;
const ERROR_CODE = { STORAGE_UNAVAILABLE: "storage_unavailable", DESTINATION_EXISTS: "destination_exists", INVALID_BACKUP: "invalid_backup", UNSUPPORTED_SCHEMA: "unsupported_schema", CONFIRMATION_REQUIRED: "confirmation_required", TOKEN_INVALID: "token_invalid", TOKEN_EXPIRED: "token_expired", RESTORE_FAILED: "restore_failed", RECOVERY_FAILED: "recovery_failed", DATABASE_UNAVAILABLE: "database_unavailable" } as const;

export interface BackupSummary { path: string; created_at_unix_seconds: number; size_bytes: number; schema_version: number; }
export interface RestoreSummary { token: string; size_bytes: number; schema_version: number; }
export type PathSelection = { kind: typeof KIND.CANCELLED } | { kind: typeof KIND.SELECTED; path: string };
export type BackupResponse = { kind: typeof KIND.CREATED; summary: BackupSummary } | { kind: typeof KIND.PREPARED; summary: RestoreSummary } | { kind: typeof KIND.RESTORED } | BackupError;
export interface BackupError { kind: typeof KIND.ERROR; code: (typeof ERROR_CODE)[keyof typeof ERROR_CODE]; message: string; }
type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const record = (value: unknown): value is RecordValue => typeof value === "object" && value !== null;
const error = (value: unknown): BackupError => ({ kind: KIND.ERROR, code: record(value) && typeof value.code === "string" && Object.values(ERROR_CODE).includes(value.code as BackupError["code"]) ? value.code as BackupError["code"] : ERROR_CODE.STORAGE_UNAVAILABLE, message: record(value) && typeof value.code === "string" && value.code === ERROR_CODE.DESTINATION_EXISTS ? "A backup already exists at that destination." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.INVALID_BACKUP ? "The selected backup is invalid." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.UNSUPPORTED_SCHEMA ? "The selected backup schema is unsupported." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.CONFIRMATION_REQUIRED ? "Restore confirmation is required." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.TOKEN_INVALID ? "The restore confirmation is invalid." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.TOKEN_EXPIRED ? "The restore confirmation has expired." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.RESTORE_FAILED ? "The restore could not be completed." : record(value) && typeof value.code === "string" && value.code === ERROR_CODE.DATABASE_UNAVAILABLE ? "The database is unavailable." : "Backup storage is unavailable." });
const selection = (value: unknown): PathSelection | BackupError => record(value) && value.kind === KIND.SELECTED && typeof value.path === "string" ? { kind: KIND.SELECTED, path: value.path } : record(value) && value.kind === KIND.CANCELLED ? { kind: KIND.CANCELLED } : error(value);
const response = (value: unknown): BackupResponse => record(value) && value.kind === KIND.CREATED && ["path"].every((key) => typeof value[key] === "string") && ["created_at_unix_seconds", "size_bytes", "schema_version"].every((key) => typeof value[key] === "number") ? { kind: KIND.CREATED, summary: { path: value.path as string, created_at_unix_seconds: value.created_at_unix_seconds as number, size_bytes: value.size_bytes as number, schema_version: value.schema_version as number } } : record(value) && value.kind === KIND.PREPARED && typeof value.token === "string" && ["size_bytes", "schema_version"].every((key) => typeof value[key] === "number") ? { kind: KIND.PREPARED, summary: { token: value.token, size_bytes: value.size_bytes as number, schema_version: value.schema_version as number } } : record(value) && value.kind === KIND.RESTORED ? { kind: KIND.RESTORED } : error(value);

export function createBackupCommands(invoke: Invoke) {
  const call = (command: string, payload: Record<string, unknown>) => invoke(command, payload).then(response).catch(error);
  return {
    chooseBackupDestination: () => invoke("choose_backup_destination_command", {}).then(selection).catch(error),
    chooseRestoreSource: () => invoke("choose_restore_source_command", {}).then(selection).catch(error),
    createBackup: (destination: string) => call("create_backup_command", { request: { destination } }),
    prepareRestore: (source: string) => call("prepare_restore_command", { request: { source } }),
    confirmRestore: (token: string) => call("confirm_restore_command", { request: { token, confirmed: true } }),
  };
}
const tauriInvoke: Invoke = async (command, payload) => (await import("@tauri-apps/api/core")).invoke(command, payload);
export const backupCommands = createBackupCommands(tauriInvoke);
