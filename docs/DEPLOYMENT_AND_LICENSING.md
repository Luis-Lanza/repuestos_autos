# Deployment and licensing

## Decision

The app uses a **perpetual license for one Windows PC**, activated manually and entirely offline. There are no subscriptions, expiry dates, grace periods, renewals, online checks, or remote revocation. Licensing is a practical commercial boundary, not perfect DRM; it cannot prevent a determined person with full control of the PC from modifying the application.

## Quick activation path

1. On the customer PC, open activation and copy the **installation code** shown by the app.
2. The customer sends that code to the vendor through the agreed support channel.
3. On a vendor-controlled system, use the local signing CLI to sign a `.lic` file with the private Ed25519 key.
4. Return the `.lic` file to the customer. In the app, choose **Import license** and select the file.
5. Confirm the app reports an active license. No internet connection is used to activate or verify it.

The code is a machine-binding hash, not a signing secret. Each accepted license is permanent for the PC it was issued for.

## What the license binds and verifies

| Part | Maintainer rule |
| --- | --- |
| PC identity | On Windows, the app reads `MachineGuid`, normalizes it, and derives a SHA-256 hash bound to this product. The raw Windows identifier is not stored or logged. The app stores the hash, not the raw identifier. |
| License | The vendor signs the license with Ed25519. The app contains the public verification key and checks the signature and PC binding locally. |
| Key custody | The private signing key stays on vendor-controlled storage, outside Git, the app build inputs, installer, and customer PC. Never copy a real public key or key ID into this guide or support messages. |
| File handling | Customers import the vendor-provided `.lic` file through the app. Rejected imports do not replace an existing valid license. |

## Unlicensed recovery and access

A missing, invalid, or PC-mismatched license must not hold customer data hostage.

| Allowed without a valid license | Blocked without a valid license |
| --- | --- |
| Read existing records and history | Sales and other business mutations |
| View reports and export data | Catalog, stock, location, and configuration changes |
| Create backups | Corrections and operator-requested restore |
| Open activation and import a license | Ordinary restore workflow |
| Automatic startup crash recovery | — |

Startup crash recovery is independent of licensing so the app can protect durable data after a crash. It is not an operator restore path. A valid license is required for a user-selected restore.

## PC replacement and reissue

A license is for one PC and cannot be self-transferred. A license for a different PC requires vendor-managed reissue and is normally paid. After a PC replacement, the vendor may, at its discretion, waive the reissue fee after reviewing reasonable replacement evidence. There is no remote revocation: an offline PC cannot receive a revocation command.

## Deployment and data practices

- Build a signed Windows NSIS installer (`.exe`). When needed, include the WebView2 offline installer so setup does not depend on internet access.
- Prefer per-machine installation for a shared store PC; it requires administrator approval. Per-user installation remains an option for a single Windows account.
- Keep SQLite data and backups in the dedicated local data directory, outside the application install directory. Installers and updates must never overwrite that directory.
- Deliver updates as signed installers. Before an update that runs database migrations, make and verify a backup.
- Test backup creation, startup recovery, export, and operator restore separately; licensing must not disable the first three recovery/access paths.

## Vendor key-handling reminders

- Keep the private key and its protected backup under vendor control, outside Git and outside all customer-facing artifacts. Restrict access and verify that the backup can be recovered.
- The vendor CLI is intentionally outside the application package and build graph; it is never included in the customer installer. Use its interactive prompts for private-key location and `.lic` output location. Never put private-key bytes or a private-key path in command arguments or environment variables.
- Never print, log, paste into tickets, or commit private-key material. Do not store customer records or issued `.lic` files in this repository.
- If the signing key is lost, licenses for that key cannot be issued. If it is compromised, treat it as a vendor security incident; rotation requires an application trust update and customer rollout.

## Pre-release Windows verification checklist

- [ ] On a clean supported Windows PC, install the signed NSIS installer; verify the WebView2 offline setup path where applicable and the intended per-machine/per-user behavior.
- [ ] Confirm first launch shows an installation code and works offline.
- [ ] Import a vendor-signed `.lic` file; verify activation succeeds only on the intended PC and remains active after restart with the network disconnected.
- [ ] Verify malformed, altered, unsupported, unknown-key, and different-PC license files fail closed; a rejected import must leave a previously valid license intact.
- [ ] Without a valid license, verify reads/history/reports, export, and backup creation work; sales and other business mutations, corrections, and operator restore are blocked.
- [ ] Verify startup crash recovery still runs without a valid license, and verify a valid license permits the normal operator restore flow.
- [ ] Update over an existing installation; confirm application data and backups are preserved and migrations are preceded by a verified backup.
- [ ] Audit the release artifacts and repository for signing private-key material, customer records, and issued license files; none may be present.
