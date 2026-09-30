export const LICENSE_STATUS = {
  ACTIVE: "active",
  ACTIVATION_REQUIRED: "activation_required",
  LICENSE_MISSING: "license_missing",
  FILE_INVALID: "license_file_invalid",
  UNSUPPORTED_VERSION: "license_unsupported_version",
  UNKNOWN_KEY: "license_unknown_key",
  SIGNATURE_INVALID: "license_signature_invalid",
  WRONG_PRODUCT: "license_wrong_product",
  MACHINE_MISMATCH: "license_machine_mismatch",
  IDENTITY_UNAVAILABLE: "machine_identity_unavailable",
  STORAGE_UNAVAILABLE: "license_storage_unavailable",
} as const;
type LicenseStatusCode = (typeof LICENSE_STATUS)[keyof typeof LICENSE_STATUS];
export type LicenseStatusResponse = { kind: "status"; code: LicenseStatusCode } | { kind: "error"; code: "license_storage_unavailable" };
export type InstallationCodeResponse = { kind: "code"; code: string } | { kind: "error"; code: "machine_identity_unavailable" };
export type LicenseSelectionResponse = { kind: "selected" } | { kind: "cancelled" } | { kind: "error"; code: "license_file_invalid" };
export type LicenseImportResponse = { kind: "imported"; status: "active" } | { kind: "cancelled" } | { kind: "error"; code: LicenseStatusCode };

type Invoke = (command: string, payload: Record<string, unknown>) => Promise<unknown>;
type RecordValue = Record<string, unknown>;
const isRecord = (value: unknown): value is RecordValue => typeof value === "object" && value !== null && !Array.isArray(value);
const isStatusCode = (value: unknown): value is LicenseStatusCode => typeof value === "string" && Object.values(LICENSE_STATUS).includes(value as LicenseStatusCode);
const status = (value: unknown): LicenseStatusResponse => isRecord(value) && value.kind === "status" && isStatusCode(value.code) ? { kind: "status", code: value.code } : { kind: "error", code: "license_storage_unavailable" };
const installationCode = (value: unknown): InstallationCodeResponse => isRecord(value) && value.kind === "code" && typeof value.code === "string" && /^[a-f0-9]{64}$/.test(value.code) ? { kind: "code", code: value.code } : isRecord(value) && value.kind === "error" && value.code === LICENSE_STATUS.IDENTITY_UNAVAILABLE ? { kind: "error", code: LICENSE_STATUS.IDENTITY_UNAVAILABLE } : { kind: "error", code: LICENSE_STATUS.IDENTITY_UNAVAILABLE };
const selection = (value: unknown): LicenseSelectionResponse => isRecord(value) && value.kind === "selected" ? { kind: "selected" } : isRecord(value) && value.kind === "cancelled" ? { kind: "cancelled" } : { kind: "error", code: "license_file_invalid" };
const imported = (value: unknown): LicenseImportResponse => isRecord(value) && value.kind === "imported" && value.status === LICENSE_STATUS.ACTIVE ? { kind: "imported", status: "active" } : isRecord(value) && value.kind === "cancelled" ? { kind: "cancelled" } : isRecord(value) && value.kind === "error" && isStatusCode(value.code) ? { kind: "error", code: value.code } : { kind: "error", code: LICENSE_STATUS.FILE_INVALID };

export function createLicenseCommands(invoke: Invoke) {
  return {
    status: () => invoke("license_status_command", {}).then(status).catch(() => ({ kind: "error", code: LICENSE_STATUS.STORAGE_UNAVAILABLE } as const)),
    installationCode: () => invoke("license_installation_code_command", {}).then(installationCode).catch(() => ({ kind: "error", code: LICENSE_STATUS.IDENTITY_UNAVAILABLE } as const)),
    chooseFile: () => invoke("choose_license_file_command", {}).then(selection).catch(() => ({ kind: "error", code: LICENSE_STATUS.FILE_INVALID } as const)),
    importFile: () => invoke("import_license_command", {}).then(imported).catch(() => ({ kind: "error", code: LICENSE_STATUS.FILE_INVALID } as const)),
  };
}
const tauriInvoke: Invoke = async (command, payload) => (await import("@tauri-apps/api/core")).invoke(command, payload);
export const licenseCommands = createLicenseCommands(tauriInvoke);
