# Project Rust Integrity Map

Use these paths as the local architecture map:

- `src-tauri/src/domain/money.rs` — non-negative centavos and checked addition/multiplication.
- `src-tauri/src/domain/quantity.rs` — quantity validity.
- `src-tauri/src/domain/request_id.rs` — request identity parsing.
- `src-tauri/src/domain/sales/` — sale and post-sale policy.
- `src-tauri/src/application/sales/confirm_sale.rs` — sale transaction orchestration and request replay.
- `src-tauri/src/application/sales/post_sale.rs` — post-sale application seam.
- `src-tauri/src/application/sales/repository.rs` — repository interfaces used by application logic.
- `src-tauri/src/infrastructure/sqlite/post_sale_transaction.rs` — SQLite transaction adapter for post-sale operations.
- `src-tauri/src/infrastructure/sqlite/post_sale_repository.rs` — canonical payload and request persistence implementation.
- `src-tauri/src/commands/` — transport translation; keep domain policy out of this directory.

## Integrity Review

- Construct domain values before opening or mutating a transaction when possible.
- Keep all facts for one use case inside the same commit decision.
- Verify rollback after every repository or invariant failure that can occur mid-operation.
- Use `MoneyCentavos` and `Quantity` checked operations instead of primitive arithmetic.
- Define errors by recoverable meaning, not by `rusqlite` text.
- For idempotent mutations, produce a deterministic canonical payload with an explicit version, hash it, and compare both operation kind and digest on replay.
- Return the original persisted result only for an exact replay; reject the same request ID with different semantics.
