# Sales Product Browser Parity

## Goal
Make Sales product discovery visually rich enough to identify parts quickly in its constrained left panel: compact table by default, an optional two-column gallery, thumbnail images, two category-defined non-empty attribute values, and a quick detail view with all attributes.

## Decisions
- Sales owns selection, stock eligibility, cart state, pricing, and checkout; it reuses browse presentation, not Catalog maintenance actions.
- Default Sales mode is a compact table; gallery is optional and uses two columns in the Sales panel.
- Browse summaries show up to the first two non-empty category-defined attributes in configured order.
- Quick detail shows all available attributes before adding a product.
- Images appear in both modes with existing bounded thumbnail behavior and a missing-image fallback.
- No per-product detail IPC requests; browse attributes must arrive through a bounded paged projection.
- Current zero-stock behavior remains disabled for adding.

## Tasks
- [x] T1 Map and extend paged browse contracts for ordered product attribute summaries and complete detail data. Evidence: extend paged browse only; provide all page attributes ordered by `attribute_definitions.id`, derive first-two non-empty in the presentation, and load them with one page-bounded query. No migration or per-product detail IPC.
- [x] T2 Add bounded attribute projection and repository/IPC tests without per-product fetches. Evidence: browse now loads ordered `{definition_id,label,value}` attributes with one repository query scoped to selected page IDs; Rust repository/command and strict TypeScript decoder tests cover empty values, page isolation, malformed data, and projection shape.
- [x] T3 Build Sales-specific compact table/gallery presentation with thumbnail lifecycle and add-state behavior. Evidence: Sales now defaults to a persisted Sales-only compact table, offers an optional two-column gallery, loads revision-checked bounded thumbnails for the current browse page, and displays at most two ordered non-empty attribute summaries; Add/Added and zero-stock disabling remain Sales-owned.
- [x] T4 Add accessible quick product detail with all attributes and preserve checkout flow behavior. Evidence: Sales identity buttons open a focus-managed, dismissible read-only dialog built solely from the current browse snapshot (including image fallback, stock and all prices, and ordered attributes with explicit “Sin dato” handling); table/gallery and mounted tests confirm focus restoration, unchanged browse requests, and unchanged Add flow.
- [x] T5 Validate Rust/TypeScript contracts, desktop density, image fallbacks, attribute ordering, and Sales interactions. Full validation and independent verification pass; final tracker evidence is committed locally.
- [x] T6 Repair Sales browse table/gallery density. Evidence: table identity shrinks while price/Add stay non-wrapping in a protected trailing area; Sales gallery uses auto-fitting cards with a 15rem minimum track, stable 8rem image frames, price/stock hierarchy, and full-width actions; result lists scroll internally with compact-width height bounds while search/status/pagination and summary remain outside. Rendered mode/action assertions and CSS contract checks pass; Add/checkout and Catalog presentation tests remain green. TypeScript retains exactly the 32 documented unrelated baseline diagnostics.

## Delivery
- Delivery strategy: stacked to main, selected by the user after the branch reached ~492 changed lines; split review slices before PR creation.
- TDD mode: standard (not strict), from `openspec/config.yaml` (`sdd.strict_tdd: false`).
- Rust runner: `cargo test --manifest-path src-tauri/Cargo.toml`.

## Evidence
- T2: Rust browse/command checks pass (16/16), strict TypeScript decoder checks pass (16/16), and `git diff --check` passes. Independent verification passed. Committed locally; no push was authorized.
- T3: shared ProductBrowser/Sales mounted checks pass (45/45) and `git diff --check` passes. Table/gallery preference, two-column Sales gallery, thumbnail stale fallback, ordered summaries, zero-stock disablement, and Catalog presentation boundaries were independently verified. `tsc --noEmit` retains the clean-HEAD-confirmed 32-diagnostic unrelated baseline; it is not a passing gate. Committed locally; no push was authorized.
- T4: shared ProductBrowser/Sales mounted checks pass (47/47) and `git diff --check` passes. Sales quick detail is browse-snapshot-only, preserves Add/checkout behavior, renders all ordered attributes with fallbacks, restores focus on dismiss, and traps Tab/Shift+Tab. Independent verification passed. Committed locally; no push was authorized.
- T5: full Rust suite passes (259/259) and full frontend suite passes (295/295), including all 11 W9 audits. Exact typecheck comparison against clean `64cad07` confirms the unchanged 32-diagnostic baseline; it is not a passing gate. Automated UI coverage passed; native desktop geometry and runtime behavior were not exercised. Committed locally; no push was authorized.
- T6: focused ProductBrowser/Sales/shell checks pass (54/54) and `git diff --check` passes. Exact typecheck comparison against clean `1969be7` proves the same 32 baseline diagnostics. Table/gallery geometry is CSS/test verified; Windows runtime visual confirmation remains pending. Independently verified and committed locally; push is authorized.
