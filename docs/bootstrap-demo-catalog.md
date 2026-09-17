# Development demo catalog bootstrap

`bootstrap-demo` is a development-only Rust command. It is not exposed through the production UI or Tauri IPC.

## Safety prerequisite

Before running it, use the app's **Copia y restauración** screen to create and verify an in-app backup. Keep that backup until the command has completed and the catalog has been checked. The command never deletes, truncates, replaces, or repairs a database.

The command opens an existing, current-schema database and refuses missing, invalid, unexpected, user-populated, partial, or incomplete databases without mutation.

## Windows invocation

From the repository root, run:

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin bootstrap-demo -- --database "$env:APPDATA\com.repuestosautos.app\repuestos-autos.sqlite3"
```

The explicit path is optional. Without `--database`, the command resolves the application data directory and targets:

```text
%APPDATA%\com.repuestosautos.app\repuestos-autos.sqlite3
```

`--help` prints the syntax without opening a database:

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin bootstrap-demo -- --help
```

## Outcomes

- **Loaded demo catalog:** preserves `FLT-001` and archived `BUJ-001`, reactivates `BUJ-001`, adds 8 deterministic automotive categories and 98 products, and commits exactly 10 active categories and 100 active products.
- **Already complete:** reports success without changes when the recognized demo state is already complete.
- **Refused:** reports a refusal for any partial, unexpected, user-populated, or non-pristine database. Refusal does not mutate the database.
- **Persistence failure:** the single transaction rolls back categories, products, stock, opening movements, FTS rows, reactivation, and audit facts together.

Each generated product is validated for unique SKU/name, positive centavo prices with minimum price no greater than list price, positive opening stock, searchable FTS presence, and an immutable positive `opening_stock` movement. The shipped records remain unchanged except for the explicit `BUJ-001` reactivation required by the bootstrap contract.
