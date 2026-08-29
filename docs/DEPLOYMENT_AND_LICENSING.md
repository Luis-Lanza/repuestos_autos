# Deployment and Offline Licensing Proposal

## Recommendation

Distribute the application as a signed Windows NSIS installer. Monthly or annual subscriptions enforced with a signed offline license file are a future commercial-control option; they are not part of v1.

V1 must not validate licenses or block sales, stock changes, or configuration changes because of subscription status. Any future licensing mechanism remains separate from inventory and sales domain logic.

## Deployment

```text
Vendor build
  -> signed Windows installer (.exe)
  -> install on the store computer
  -> application files
  -> local SQLite data directory
  -> first-run installation code
```

| Area | Decision |
| --- | --- |
| Installer | Tauri NSIS setup executable (`.exe`). |
| Offline installation | Embed the WebView2 offline installer when the target computer may lack WebView2. |
| Installation scope | Prefer `perMachine` for a shared store computer; it requires an administrator during installation. |
| Data location | Keep SQLite data and backups outside the application install directory, in a dedicated local data directory. The installer/updater must never overwrite this directory. |
| Trust | Sign production installers with a Windows code-signing certificate to reduce SmartScreen warnings. |
| Updates | Deliver a new signed installer manually in v1. Database migrations run on launch and must be backed up first. |

Tauri supports NSIS Windows setup executables and an offline WebView2 installer mode. Its default per-user installation avoids administrator privileges; `perMachine` installation is appropriate when the same store computer may have different Windows accounts. [Tauri Windows installers](https://tauri.app/distribute/windows-installer/)

## Offline subscription flow

```text
1. First run: app generates an Installation Code.
2. Customer pays monthly or annually.
3. Vendor generates a signed license for that Installation Code and expiry date.
4. Customer imports the license file from USB, email attachment, or messaging app.
5. App verifies the signature locally and enables write operations until expiry.
6. On renewal, the vendor sends a new license file; the customer imports it.
```

### License payload

The file contains a JSON payload plus an Ed25519 signature. The vendor keeps the private signing key; the application contains only the public verification key.

```json
{
  "license_id": "lic_...",
  "installation_id": "install_...",
  "customer_name": "...",
  "plan": "monthly",
  "issued_at": "2026-08-19T00:00:00Z",
  "expires_at": "2026-09-19T00:00:00Z",
  "features": ["inventory", "pos", "reports"]
}
```

The app accepts a license only when its signature is valid, its installation ID matches, and it has not expired.

## Expiry policy

| State | Application behavior |
| --- | --- |
| Active | All functions available. |
| Grace period (7 days) | Full function with a prominent renewal warning. |
| Expired | Block new sales, stock changes, and configuration changes. Preserve read-only access to history, reports, and backup export. |
| Renewed | Importing a valid newer license restores normal operation immediately. |

Never encrypt, delete, or hold the customer’s operational data hostage. Subscription expiry controls future write operations, not access to existing records or the ability to make a backup.

## Security boundary

An offline license is a **commercial deterrent**, not perfect DRM. A user with full control of the computer can change the system clock or restore an old disk/database snapshot.

Mitigations for a future licensing release:

- Store the last accepted wall-clock time locally; if the clock moves substantially backwards, require license renewal before writes.
- Store license state separately from ordinary database backups when possible.
- Validate signature, installation ID, expiry, and monotonic last-seen time on every application launch and before write operations.
- Make the license code easy to operate administratively; avoid fragile hardware fingerprints that create support incidents after a Windows or hardware repair.

Perfect enforcement requires periodic contact with a server. If a future version allows occasional internet access, add signed online renewal/validation with a grace period; do not add cloud inventory synchronization merely for billing.

## Vendor operations checklist

- [ ] Keep the private signing key outside the application and outside the source repository.
- [ ] Record license ID, customer, installation ID, plan, issue date, and expiry in a vendor-controlled register.
- [ ] Send a renewed signed license only after confirming payment.
- [ ] Test expiry, grace, renewal, clock rollback, and backup/restore behavior before release.
- [ ] Test installation and restore on a clean Windows target device.
