# Project IPC Map

Use these paths when tracing an IPC contract:

- `src/commands/catalog.ts` — `invoke` adapters, request envelopes, runtime guards, response unions, and safe fallback messages.
- `src/commands/confirm-sale.ts` — sale confirmation request and response mapping.
- `src/commands/backup.ts` — native dialog, backup, and restore adapters.
- `src-tauri/src/lib.rs` — Tauri command functions, managed state, production `desktop_command_builder`, test `command_builder`, and command-surface tests.
- `src-tauri/src/commands/` — transport request/response types and translation into application calls.
- `src-tauri/src/application/` — application seams commands should call.
- `src-tauri/capabilities/default.json` — main-window capability allowlist.

## Contract Checklist

For each command, verify:

1. The TypeScript command string exactly matches the registered Rust name.
2. The JavaScript payload nesting matches the Rust command parameter, usually `{ request: ... }`.
3. Rust request types deserialize all required fields without relying on ambiguous defaults.
4. TypeScript receives `unknown` and validates objects, arrays, discriminants, and required scalar fields.
5. Public errors use stable codes and user-safe messages.
6. Both production and test handler lists contain the intended command, unless the operation is intentionally desktop-only.
7. `src-tauri/capabilities/default.json` grants no unrelated plugin operation.

Use the command-surface tests in `src-tauri/src/lib.rs` to prove registration and exclusion. Use focused command adapter tests under `src/commands/*.test.ts` to prove malformed native values cannot enter UI state.
