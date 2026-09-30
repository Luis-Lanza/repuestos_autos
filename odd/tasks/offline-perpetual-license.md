# Offline Perpetual Per-PC License

## Objective
Add an offline perpetual, single-PC licensing boundary that activates from a vendor-signed license file and protects commercial use without holding customer data hostage.

## Approved Product Policy
- Commercial model: one-time, perpetual license for one PC.
- Activation: fully offline. The application displays an installation code; the vendor privately generates and returns a signed `.lic` file.
- Vendor tool: a private local CLI, outside the application package/build graph. Its private signing key never enters the repository, installer, client PC, or application build inputs.
- Binding: signed license contains a hash of the application identity plus Windows machine identity; the application stores only the hash, not the raw machine identifier.
- Valid license: all features available offline, with no expiry, renewal, grace period, or subscription checks.
- No valid license: activation/import is available; existing data remains readable and exportable; creating backups is allowed; sales, catalog/configuration changes, stock operations, corrections, and ordinary restore are blocked.
- Startup crash recovery remains available independently of license state. Operator-initiated restore requires a valid license.
- The public verification key may ship in the app. The private signing key must remain vendor-controlled.

## Constraints
- Do not add a hosted activation service, account system, or cloud inventory synchronization.
- Do not implement hardware fingerprints that create fragile support incidents; isolate Windows identity access behind a provider.
- Enforce write policy in authoritative Rust application/command seams, never only in React UI.
- Do not globally block `DatabaseState::with_read` or `with_write`; use explicit allowlists so recovery and backup behavior stay intentional.
- Do not commit, push, or open a PR without explicit user authorization.

## Tasks
- [x] T1 Write and verify a change-local protocol/design: license envelope/version, Ed25519 signing/verification, installation-code derivation, storage, error states, allowed recovery operations, private CLI boundary, and reactivation policy. Route: delegated design/research. Evidence: read-only mapping plus protocol handoff completed.
- [ ] T2 Create vendor-only signing CLI outside the app package, with deterministic test vectors and no committed private key. Route: delegated writer.
- [ ] T3 Implement application-side machine identity, license verifier/storage/import, typed IPC, and activation UI. Route: delegated vertical slices.
- [ ] T4 Gate authoritative business writes and ordinary restore while preserving reads, backup creation, and startup recovery. Route: delegated vertical slices.
- [ ] T5 Independently verify valid, invalid, tampered, mismatched-machine, missing-license, backup/recovery, and Windows installation paths. Route: delegated verifier.

## Acceptance Criteria
- A vendor-signed perpetual license validates only on its intended Windows installation.
- An altered license, unknown key, unsupported version, or machine mismatch fails closed without exposing internal details.
- The app contains only public verification material; no private signing material is committed or shipped.
- Unlicensed/rejected state allows activation, read access, export, and backup creation, but blocks business writes and operator restore.
- Startup recovery still protects durable data without a valid license.
- The vendor CLI can generate a license from an installation code without a server.
- The user can import a license through an accessible local activation flow.

## T1 Protocol
- **License v1 payload**: canonical compact UTF-8 JSON with fixed fields `version`, `license_id`, `key_id`, `product_id`, `machine_hash`, and `issued_at`; `version` is `1`, `product_id` is `com.repuestosautos.app`, and all values are ASCII. Reject duplicate, unknown, reordered, whitespace-altered, or unsupported fields.
- **Envelope**: fixed JSON fields `format: "repuestos-autos-license"`, base64url canonical `payload`, and base64url 64-byte Ed25519 `signature`. Sign `repuestos-autos-license\0v1\0` plus the exact canonical payload bytes.
- **Machine binding**: isolate Windows `MachineGuid` registry access behind a provider; normalize it to canonical lowercase GUID form; hash `repuestos-autos-machine-binding\0v1\0`, product ID, and canonical GUID with SHA-256. Never persist or log the raw GUID.
- **Key custody**: the app embeds only public verification keys indexed by `key_id`, defined as lowercase SHA-256 hex of the 32-byte public key. The vendor-only CLI lives under `tools/license-cli/` in this public repository but outside the application package/build graph. It contains no private key, key path/default, customer registry, or operational secret. Private key material is loaded only from vendor-controlled protected local storage or an interactive secret, never command arguments, environment, logs, repository, app build inputs, or customer machines. A local private-key file is exactly the 32-byte Ed25519 seed as lowercase hex.
- **Storage/import**: accepted envelope is `<app_data>/license.lic`, separate from SQLite. Validate fully before staging/syncing/atomically replacing; a rejected import preserves the prior valid file. Cap input size and expose only bounded error codes. `machine_hash`/installation code is exactly 64 lowercase hex; `issued_at` is UTC RFC3339 at second precision; `license_id`/`key_id` are 1–64 ASCII letters, digits, dots, underscores, or hyphens.
- **States**: `active`, `activation_required`, `license_missing`, `license_file_invalid`, `license_unsupported_version`, `license_unknown_key`, `license_signature_invalid`, `license_wrong_product`, `license_machine_mismatch`, `machine_identity_unavailable`, and `license_storage_unavailable`.
- **Enforcement**: allow activation, reads, exports, backup creation, and startup crash recovery without a license. Block sales, stock operations, catalog/configuration/location changes, corrections, and both operator restore stages. Enforce explicitly at authoritative Rust use-case/command seams; do not globally gate database read/write wrappers.
- **Reissue**: every PC needs a vendor-issued license. Reissue is normally paid; any replacement-PC exception is a vendor-managed support decision. No self-service transfer or remote revocation exists offline.

## Progress
- 2026-09-29: User selected perpetual single-PC licensing, fully offline activation, and a private local vendor CLI.
- 2026-09-29: User selected unlicensed recovery policy: allow reading/export/backup creation; block ordinary restore and all business writes.
- 2026-09-29: User selected paid, vendor-managed reissue for another PC; no self-service transfer.
- 2026-09-29: User confirmed the vendor CLI may live under `tools/license-cli/` in this public repository; private signing material and customer records stay outside Git and application artifacts.
- 2026-09-29: Vendor provisioned production public verification key `e869bd20891d1bf5c56007619f4d6d49d5d678395742659f6c5f5d1c52703f02` with key ID `24cd3b399059804e5ea6cfc43ff785c364dc1593bf7783e676e8defd4daa995d`; private key remains outside Git.

## Evidence
- Work-unit commit: `671c165 feat(licensing): add offline perpetual activation`.
- Read-only mapping: `docs/DEPLOYMENT_AND_LICENSING.md`, `src-tauri/src/lib.rs`, backup/restore commands, SQLite startup, Tauri command builder, app shell, and Cargo boundaries.
- Protocol research: Ed25519 verification/signing, Windows registry MachineGuid access, and Tauri non-blocking license-file import patterns.

## Next Step
- Implement T3 application-side verification, activation/import, and enforcement using the provisioned public key; do not create, read, or store private signing material.
