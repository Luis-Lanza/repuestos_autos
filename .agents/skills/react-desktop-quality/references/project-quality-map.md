# Project React Quality Map

Use these repository paths as the source of truth:

- `src/ui/app.ts` — top-level React navigation and screen selection.
- `src/ui/sales/sale-flow.ts` — deterministic sale state transitions, pending confirmation, stale-price acknowledgement, and persisted summaries.
- `src/ui/sales/sale-screen.ts` — sale interactions and user-visible feedback.
- `src/ui/inventory/inventory-flow.ts` and `src/ui/inventory/inventory-screen.ts` — inventory transitions and rendered controls.
- `src/commands/catalog.ts` — native invocation adapter with runtime response decoding.
- `src/ui/inventory/inventory-screen.test.ts` — rendered semantic output and interaction-adapter testing pattern.
- `src/ui/sales/sale-flow.test.ts` — reducer-level transition testing pattern.

## Review Prompts

- Can an older request overwrite a newer query or screen state?
- Can repeated clicks submit the same mutation while pending?
- Does every command failure become stable, actionable UI feedback?
- Are controls named, keyboard operable, and associated with labels?
- Does focus move predictably after validation, confirmation, or navigation?
- Do tests observe behavior rather than hook or component internals?
