---
name: react-desktop-quality
description: "Trigger: React desktop UI, interaction flow, async race, stale response, accessibility, screen test. Harden this project's React desktop interactions and observable UI behavior."
license: Apache-2.0
metadata:
  author: gentleman-programming
  version: "1.0"
---

## Activation Contract

Use this skill for React screen, flow, command-consumer, async interaction, and accessibility work under `src/ui/`. Do not use it to define Tauri IPC or Rust persistence contracts.

## Hard Rules

- Keep deterministic state transitions in flow modules and side effects in screen or interaction adapters.
- Guard async searches and loads against stale responses, duplicate submission, unmounted updates, and out-of-order completion.
- Preserve the pending, success, empty, validation, and failure states users can observe.
- Use semantic elements, labels, keyboard-operable controls, focus movement, and live status messaging.
- Decode native responses in `src/commands/`; never trust an `invoke` generic as runtime validation.
- Test behavior through rendered output, callbacks, and state transitions; avoid implementation-detail assertions.

## Decision Gates

| Situation | Action |
| --- | --- |
| Pure transition or validation | Change the matching `*-flow.ts` and its test |
| Native payload shape changes | Activate `tauri-ipc-contracts` first |
| Competing async requests | Track request identity and ignore stale completion |
| Destructive or irreversible action | Require explicit confirmation and recoverable feedback |
| New visual control | Verify name, role, keyboard use, focus, pending, and error behavior |

## Execution Steps

1. Read the matching screen, flow, command adapter, and focused tests.
2. Specify the user-observable transition and race behavior.
3. Implement the smallest state and interaction change.
4. Exercise success, failure, duplicate action, and stale completion where relevant.
5. Verify accessible markup and feedback in rendered-output tests.

## Output Contract

Return changed files, focused checks, covered interaction states, accessibility evidence, and residual race risks.

## References

- `references/project-quality-map.md` — project React seams and focused test locations.
