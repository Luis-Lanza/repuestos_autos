# 05 — Rust catalog and strict Tauri command contract

Status: approved

## Dependencies

04 SQLite persistence.

## Scope

Expose catalog price as `catalog_unit_price_centavos`. Replace the confirmation command DTO with the reduced request: request ID, product/quantity lines, nullable tender, and nullable QR. Reject unknown legacy authority fields before use-case execution and return persisted authoritative summaries with stable typed errors.

## Expected path groups

- `src-tauri/src/commands/catalog.rs`
- `src-tauri/src/commands/confirm_sale.rs`
- `src-tauri/src/application/catalog/mod.rs`
- `src-tauri/src/lib.rs` only if registration requires it
- `src-tauri/tests/catalog_search.rs`
- `src-tauri/tests/command_seam.rs`

## Verification evidence

Run `cargo test --manifest-path src-tauri/Cargo.toml --test command_seam`, `cargo test --manifest-path src-tauri/Cargo.toml --test catalog_search`, and the Rust suite. Prove strict rejection of negotiated price/payment rows/applied cash/change, nullable inputs, stable error mapping, and persisted cash/QR/mixed/retry summaries.

## Rollback

Revert Rust command/catalog DTOs and seam tests together. Do not deploy a reduced Rust IPC contract with a legacy TypeScript caller.

## Cumulative-history boundary warning

Keep TypeScript and React changes out of this Rust-only work unit. Verify the actual predecessor diff and replan before 400 authored changed lines.

## Key Learnings

The command adapter converts shapes and errors; it must not recalculate prices or payments.
