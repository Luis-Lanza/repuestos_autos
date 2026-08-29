# Technical Recommendation

## Decision

Build a local Windows desktop application with **Tauri 2 + React + TypeScript + SQLite**.

## Implementation status

The selected stack is in use for catalog onboarding and maintenance, fixed-price POS, operational inventory, read-only sales history, and backup/restore. Backup/restore has Fedora evidence; Windows task 4.1 evidence remains deferred. Returns, cancellation, and reporting remain planned requirements and have no current Rust application/command or UI implementation.

## Why this fits the validated scope

| Requirement | Technical fit |
| --- | --- |
| One Windows computer, offline operation | Tauri packages a locally installed Windows desktop application. |
| One local source of truth | SQLite stores catalog, sales, movements, and backup data in one local database file; reporting remains planned. |
| Reliable stock and sales | Execute implemented confirmation and adjustment logic inside database transactions; use the same approach when planned cancellation and return work is implemented. |
| Dynamic category fields and global search | Model field definitions and values relationally; index base fields and searchable attribute values. |
| USB backup/restore | The desktop shell can explicitly export and import a validated database backup file; Windows task 4.1 evidence remains deferred. |

Tauri's official SQL plugin supports SQLite on Windows, and its migrations execute inside a transaction. Tauri can bundle Windows installers as an MSI or NSIS setup executable. [Tauri SQL plugin](https://v2.tauri.app/plugin/sql/) · [Tauri Windows installers](https://tauri.app/distribute/windows-installer/)

## Proposed stack

- **Desktop shell:** Tauri 2 (Rust host).
- **User interface:** React + TypeScript.
- **Local database:** SQLite, accessed through a narrow repository/service layer.
- **Schema changes:** Versioned SQL migrations.
- **Validation:** TypeScript form validation plus database constraints; the backend/database transaction remains the final authority for stock, catalog-price resolution, and derived payment values.
- **Installer:** Tauri NSIS setup executable for Windows; use an MSI only if deployment policy requires it.

## Important implementation rules

1. Store money as integer centavos, never floating-point numbers.
2. Commit a confirmed sale, its payment lines, and its stock movements in one transaction.
3. Never update stock without a corresponding movement record.
4. Do not let the UI directly implement business rules alone; resolve the current catalog price at confirmation, persist it as the sale-line snapshot, derive cash applied and change from tendered cash and the cart total, and validate stock availability and payment integrity in the application service/database transaction.
5. Backup via a database-safe export flow; do not rely on copying a database file while a write may be in progress.

## Deliberately not recommended for v1

- A cloud-first web app: adds internet, hosting, authentication, and synchronization complexity that the validated one-computer offline operation does not need.
- Electron: workable, but unnecessary for this small local Windows application when Tauri meets the packaging and SQLite requirements.
- A server database such as PostgreSQL: useful for multiple devices or branches, but operationally excessive for one computer.

## Future evolution

If the business later needs multiple computers or branches, introduce a central API and server database as a separate evolution. Do not prematurely add synchronization or subscription enforcement to the v1 inventory domain.
