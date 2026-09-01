---
name: tauri-ipc-contracts
description: "Trigger: Tauri IPC, invoke contract, command registration, runtime decoding, capability permission, native response. Secure and verify this project's frontend-to-Rust command seam."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

## Activation Contract

Use this skill when adding or changing Tauri command names, request envelopes, response variants, frontend adapters, registration, or capabilities. Do not move domain policy into the IPC seam.

## Hard Rules

- Treat every `invoke` result as untrusted `unknown`; decode required fields and discriminants at runtime.
- Keep TypeScript payload keys aligned with Serde behavior and the Rust command parameter envelope.
- Return stable, typed success and error variants; map internal failures to bounded public codes.
- Register every production command in `desktop_command_builder`; register testable commands in `command_builder` when applicable.
- Add only the minimum Tauri capability or plugin permission required by the main window.
- Keep commands thin: validate transport input, call the application seam, and translate output.
- Never expose filesystem paths, SQL details, panic text, or unrestricted native operations over IPC.

## Decision Gates

| Situation | Action |
| --- | --- |
| Response is external to TypeScript | Add or update a runtime decoder |
| Command is added or renamed | Update frontend name, Rust handler, and registration together |
| Plugin or native operation is introduced | Review `src-tauri/capabilities/default.json` for least privilege |
| Business rule changes | Activate `rust-application-integrity` |
| Persistent operation changes | Activate `sqlite-database-expert` |

## Execution Steps

1. Trace the frontend adapter, Rust command request/response, application call, handler registration, and capability.
2. Define one explicit request envelope and one decodable response union.
3. Reject malformed payloads and preserve stable public error codes.
4. Test malformed responses in TypeScript and registered/excluded commands in Rust.
5. Verify production and test command surfaces do not drift.

## Output Contract

Return changed contract surfaces, command names, decoding evidence, registration checks, permission changes, and residual exposure.

## References

- `references/project-ipc-map.md` — local IPC seam, registration, and capability map.
