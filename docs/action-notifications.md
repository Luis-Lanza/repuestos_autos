# Action outcome notifications

## Shared contract

`src/ui/visual-system/action-notifications.ts` exports:

- `ActionNotificationProvider`: mount once above activation and the application shell. It owns the queue and host; do not mount another host per screen.
- `useActionNotifications(): ActionNotifications | null`: returns a stable API inside the provider, explicitly `null` outside it. Isolated screens must retain an inline outcome fallback rather than crash or silently discard outcomes. Wrap mounted producer tests in the provider when asserting shared notifications.
- `ActionNotifications.publish({ severity, message }): string`: adds a distinct notice on every call and returns its dismissal ID. Supported severities are `success`, `error`, `warning`, and `info`; producer messages and visible severity labels are Spanish.
- `ActionNotifications.dismiss(id): void`: removes only a notice, never domain, retry, receipt, or recovery state.

Publish directly from an **accepted operation completion**, after mounted/request-generation guards. Do not watch feedback strings in effects. The provider deliberately does not deduplicate equal messages: identical repeated operations are distinct outcomes. Producers own submission locks, request acceptance and stale/unmounted suppression. Never publish both from a screen and its parent for the same completion.

## Lifetime and queue policy

At most three cards are visible, oldest first. Further notices remain in an in-memory FIFO queue with an explicit visible waiting count; there is no eviction or silent dropping, including unresolved errors/warnings. Every publication updates its stable live announcement channel, even when its card is waiting. A waiting card has no expiry clock until it becomes visible.

Success/info expire after six visible seconds. Hover and focus independently pause the remaining time; leaving hover while focus remains does not resume it. Error/warning cards require explicit dismissal. The host scrolls within the viewport if messages are long. Close buttons are keyboard accessible and publication never moves focus.

The retained queue is intentionally not capped in memory: bounding retained unresolved notices would require either loss or a separate durable notification inbox. This is transient session UI, not a persisted audit/recovery record. Closing the app clears it. Recovery information must remain inline or persisted, not rely on this queue.

## Position and modal accessibility

The portal normally lives under `document.body`, outside shell scrolling/clipping, with a fixed top-right host and semantic severity colors plus visible text labels. There are always two separate live regions, polite for success/info and assertive for error/warning. Distinct notice IDs make repeated equal messages produce different announcements. Cards themselves are not additional live regions.

A DOM observer reparents the **same portal node** inside the active visible `aria-modal="true"` dialog, using the focused dialog when available and the last visible modal otherwise. A focus listener tracks nested modal changes. This keeps announcements and close controls within the modal accessibility subtree and existing focus containment, rather than relying only on z-index. Closing the modal returns the node to the remaining modal or body without remounting its live regions/cards or resetting notice lifetime. The portal wrapper has no layout box.

Future modal integrations must retain the existing visible `aria-modal` contract; native top-layer dialogs, transformed containing blocks, and actual assistive-technology behavior need desktop verification before introducing different modal primitives.

## Initial producer rollout (N1)

| Producer | Shared outcome owner | Kept inline |
| --- | --- | --- |
| Inventory stock entry / physical count | Accepted success and ordinary `persistence_failure`, once per unlocked completion | All other domain validation/recovery error codes, field/password validation, request-conflict guidance, stale stock advisory, read-only alert loading/empty/unavailable states |
| App activation handoff | App callback publishes activation success before switching to the shell | Identity/access/recovery and installation-code guidance remain inline; N3 owns import failure/cancellation below |

Inventory keeps retry identity and entered fields in its existing flow. Ordinary failure leaves a screen-local `Reintentar` control after notification dismissal. With no provider, existing inline success/error Feedback remains the compatibility fallback. A successful mutation followed by failed alert refresh still publishes success; the unavailable alert guidance remains inline, not a second mutation failure.

This is a global rollout, not an inventory-only scope. N2 covers sales/onboarding/catalog below; N3 covers backup/history/reports/licensing. Do not migrate loading, empty, validation, stale, partial-save, durability/recovery guidance or persisted receipts indiscriminately.

## Sales and catalog administration (N2)

| Producer | Shared outcome owner | Kept inline / persisted |
| --- | --- | --- |
| Sale confirmation | `SaleScreen`: accepted confirmation success and ordinary `persistence_failure` / exception | Persisted receipt, draft/retry identity, price/payment/stock/product validation, minimum-price and stale-catalog correction, request-conflict guidance; search/thumbnail read failures |
| Onboarding category creation | `OnboardingScreen`: accepted category success and ordinary persistence failure | Missing/invalid/duplicate category and field-definition validation; category loading and retry |
| Onboarding product creation / initial location | `OnboardingScreen`: full success, ordinary persistence failure, or one warning describing a saved product with failed assignment | Attribute/server validation, duplicate SKU, changed-category guidance and failed pre-save category refresh. Partial creation guidance stays inline; clear the created product's fields and direct correction to Catálogo, never repeat creation to repair the location |
| Catalog setup / unlock / recovery | `CatalogAccessGate`: accepted unlock or acknowledged setup/recovery completion; ordinary `access_unavailable` errors. Beginning setup/recovery publishes only information about the remaining acknowledgement, not final success | Credential/password validation, configuration/license prerequisites, the one-time recovery code and its mandatory acknowledgement; never include the code in a notification |
| Catalog password change | `CatalogPasswordChange`: accepted success and ordinary access failure | Credential/confirmation validation. Preserve entered values after failure and clear them only after accepted success |
| Catalog lock | `CatalogMaintenanceScreen`: accepted lock success and ordinary access failure | Any prerequisite/validation failure; isolated access success falls back inline across gate/screen handoff |
| Product/category archive / reactivate | `CatalogMaintenanceScreen`: accepted lifecycle success or ordinary persistence failure, from either the detail or category list, never both | Archive confirmations, blocked lifecycle, unavailable/stale record correction, refresh/reload gates |
| Product/category metadata, primary location and category schema | `CatalogMaintenanceScreen`: full accepted save success, ordinary persistence failure, or one partial-save warning | Field/server validation, obsolete record/schema guidance, failed refresh and authoritative-detail gates. If metadata saves but location assignment fails, or fields save but renaming fails, retain explicit partial-save guidance and require reload before saving again |
| Product image choose / remove | `CatalogMaintenanceScreen`: accepted success, ordinary mutation failure or mismatched identity failure; picker cancellation is information | Preview loading failures/absence, image/server validation and stale-catalog recovery. `CatalogEditDialog` only renders explicitly classified inline image feedback; it never publishes |
| Location schema / create / activate / deactivate / delete | `LocationManagementScreen`: accepted success and ordinary persistence failure | Schema/value/duplicate-code validation, in-use/inactive/unavailable/stale guidance, frozen-schema advisory, destructive confirmations and read-only load/retry |

Each owner publishes inside the guarded async handler, not an effect observing reducer feedback. Provider-backed screens suppress only the routine result they own; isolated screens retain inline fallback. Partial outcomes publish a warning **and** retain correction inline, so dismissal cannot erase the recovery instructions. A successful mutation publishes before read-only refresh: failure to refresh is not a second mutation failure and must not encourage repeating an already saved operation.

Submission refs serialize requests before React rerenders. Mounted and request-generation checks reject late responses; catalog selection close/lock invalidates its generation. Pending image/metadata/lifecycle dialogs cannot close through Escape or their cancel action. Onboarding serializes category and product creation against their shared mutation generation. Partial-save screen guidance clears after authoritative detail reload (or explicit selection close/lock, where reopening still loads authoritative detail). Existing reducer/domain state, IPC envelopes and safety confirmations remain owned by their existing flows.

Mounted producer tests use the real provider and inspect the host separately from screen-owned inline sections. The host may be inside a modal's subtree: a whole-dialog text query can match a notification and is not evidence of redundant inline feedback. Tests cover repeated equal failures, accepted success/error, validation exceptions, partial persistence, failed refresh, picker cancellation, pending dismissal protection and surviving-provider unmount suppression. Dialog severity is an explicit prop, never inferred from Spanish message text.

## Continuity, corrections, exports and licensing (N3)

| Producer | Shared outcome owner | Kept inline / persisted |
| --- | --- | --- |
| Backup destination / creation | `BackupScreen` accepts guarded interaction completions: ordinary action failure, full success, or warning for published backup with durability/cleanup uncertainty | Last backup summary; durability uncertainty and cleanup guidance remain after dismissal. No safe-retry advice is added for uncertainty |
| Restore source / preparation / confirmation | `BackupScreen`: ordinary failure/unavailable outcome and accepted restore success | Candidate, acknowledgement and destructive confirmation; invalid/expired candidate (`token_invalid`, `invalid_backup`, `unsupported_schema`, `token_expired`) and recovery failure. Typed `restoreErrorStatus` classifies invalid tokens as `invalid`, never ordinary toast-only failure; preparation/confirmation guidance remains inline independently of notice dismissal. Recovery failure explicitly warns that local state is uncertain and requires assistance, not blind retry |
| History return / cancellation | `createSalesHistoryInteraction` accepts matching request/sale identities and calls `SalesHistoryScreen` once, before detail refresh | Quantity/reason/acknowledgement validations, conflicts, correction records and detail recovery. Accepted request IDs cannot be resubmitted even after refresh fails. Reload failure says the correction **was saved**, not that mutation failed; reload stays available while duplicate correction is blocked |
| Movement-ledger PDF | `MovementLedgerScreen`: success, picker cancellation (info), ordinary failure and resource limit | Query/search/loading/empty/filter states. Export submission ref blocks duplicates; changed applied filters/pages invalidate the old export completion |
| Gross-profit PDF | `GrossProfitReportScreen`: success, cancellation (info), failure and resource limit | Query/loading/empty, applied range and missing historical cost disclosure. Changing range during an export retains the original export owner and lock; accepted completion still describes that saved PDF, not the newer query |
| License import | `ActivationScreen`: guarded import failure and cancellation (info) | Identity/access/load failures, installation code and recovery navigation. App remains the sole success owner through activation unmount and shell handoff |

All migrated screens retain isolated inline compatibility. Notifications never observe persistent strings in effects. History's accepted success also has an isolated inline fallback; its persisted-request guard is independent of notice dismissal. History correction failures remain inline because they require authoritative detail recovery rather than blind resubmission. Backup picker cancellation remains harmless under its existing standalone flow; candidate preparation itself is not completion of a restore.

## Global producer checklist

- [x] Inventory entry and physical count.
- [x] Sale confirmation, persisted receipt and onboarding category/product/location outcomes.
- [x] Catalog access/setup/recovery/lock/password; product/category metadata, schema, lifecycle, images and primary location.
- [x] Location schema/create/lifecycle/delete.
- [x] Backup creation and restore preparation/confirmation outcomes, with inline safety exceptions.
- [x] History persisted return/cancellation success before refresh, with independent correction recovery.
- [x] Movement-ledger and gross-profit PDF success/cancel/failure/resource limits.
- [x] License import failure/cancel and App-owned success handoff.

## Manual desktop verification checklist (N4; not yet observed)

- [ ] At desktop and compact window sizes, scroll each screen/dialog: host stays top-right, readable, unclipped and reachable without hiding critical controls.
- [ ] Verify Spanish severity labels/icons, color contrast, long-message wrapping and host scrolling in the actual WebView.
- [ ] With keyboard and a screen reader, confirm distinct repeated announcements, polite/assertive channels, modal containment, nested dialogs and focus-preserving dismissal.
- [ ] Confirm six-second success/info expiry, hover/focus pause, persistent errors/warnings, three-card FIFO overflow and waiting count.
- [ ] Complete and dismiss each producer outcome; navigate away: notices survive navigation while receipt, summaries, correction records, candidate and partial/recovery guidance remain independent.
- [ ] Exercise duplicate clicks and pending dismissal; leave a screen before completion: no stale publication. Change report filters during a pending export and verify the documented ownership policy.
- [ ] Persist a history correction then fail detail refresh: success is announced once, reload remains usable, and duplicate correction cannot be submitted.
- [ ] Check activation failure/cancel without losing installation/recovery guidance, then successful activation: exactly one success survives the handoff.
- [ ] Simulate backup durability/cleanup and restore recovery uncertainty using an isolated fixture environment, never the customer's live database; dismissal must not erase guidance or suggest safe retry.

## Evidence and pending checks

Deterministic public-provider and mounted tests cover duplicate text, FIFO overflow retention, severity lifetime, hover/focus pause, server/isolated compatibility, modal live-region identity, keyboard close controls, activation handoff/navigation persistence, inventory retry state, duplicate/unmounted suppression and successful persistence followed by failed read-only refresh. CSS tests guard positioning and severity selectors, not measured WebView geometry.

Browser/desktop visual checks and assistive-technology announcements are still pending. No visual or screen-reader observation is claimed by jsdom tests.
