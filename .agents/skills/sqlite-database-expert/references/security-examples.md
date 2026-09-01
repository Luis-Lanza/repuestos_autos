# Secure SQLite Patterns

## Bind Values

Use placeholders for every value that can vary at runtime:

```rust
use rusqlite::{params, Connection};

fn rename_product(connection: &Connection, product_id: i64, name: &str) -> rusqlite::Result<usize> {
    connection.execute(
        "UPDATE products SET name = ?1 WHERE id = ?2",
        params![name, product_id],
    )
}
```

Treat wildcard input as data. If `%` and `_` must be literal, escape them and declare the escape character:

```rust
fn like_pattern(input: &str) -> String {
    format!("%{}%", input.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
}

let pattern = like_pattern(query);
let mut statement = connection.prepare(
    "SELECT id, name FROM products WHERE name LIKE ?1 ESCAPE '\\'",
)?;
```

## Restrict Dynamic SQL

SQLite parameters cannot bind table names, column names, keywords, or sort direction. Convert a closed enum to fixed SQL instead of accepting arbitrary strings:

```rust
enum ProductOrder { Name, Sku }

let order = match requested_order {
    ProductOrder::Name => "name COLLATE NOCASE, id",
    ProductOrder::Sku => "sku COLLATE NOCASE, id",
};
let sql = format!("SELECT id, sku, name FROM products ORDER BY {order}");
```

The interpolation is safe only because every output is a code-owned literal. Reject unknown tokens; do not sanitize and continue.

## Decode Explicitly

- Select explicit columns instead of `SELECT *`.
- Decode each column into the narrowest Rust type.
- Convert `QueryReturnedNoRows`, constraint failures, and unavailable storage into bounded repository or application errors.
- Do not return raw SQL, paths, or `rusqlite` messages through IPC.

## Protect Writes

- Start one transaction for all facts in a use case.
- Check affected-row counts for optimistic concurrency and guarded inventory changes.
- Keep `PRAGMA foreign_keys = ON` on every opened connection.
- Prefer database constraints and immutable triggers for facts that must survive all callers.
- Test injection-shaped strings as ordinary bound values and verify schema/data remain intact.

Project examples: `src-tauri/src/application/sales/confirm_sale.rs`, `src-tauri/src/infrastructure/sqlite/catalog_repository.rs`, and `src-tauri/src/infrastructure/sqlite/post_sale_repository.rs`.
