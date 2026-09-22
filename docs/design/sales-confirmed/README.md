# Confirmed sale visual references

These files preserve the approved confirmed-sale mockups as static visual references only. The [confirmed-sale Figma handoff](../sales-confirmed-figma-handoff.md) is the authority for behavior and accessibility.

## Screenshots

- [`desktop-typical.png`](desktop-typical.png) — Frame A: a typical desktop sale with three or four lines, mixed cash and QR payment, and non-zero change.
- [`desktop-long.png`](desktop-long.png) — Frame B: a long desktop sale with many lines, long product facts, large values, and natural document scrolling.
- [`compact-typical.png`](compact-typical.png) — Frame C: a typical compact-width sale using stacked records and one-column DOM order.
- [`compact-long.png`](compact-long.png) — Frame D: a long compact-width sale with complete historical facts and one document scroll surface.
- [`edge-states.png`](edge-states.png) — Frame E: edge facts including zero change, mixed and QR-only payments, an unavailable date, long identifiers, large totals, and an empty-payment proposal explicitly pending product approval.

## HTML sources

- [`reference-source/desktop-typical.html`](reference-source/desktop-typical.html) — Frame A source for the typical desktop reference.
- [`reference-source/desktop-long.html`](reference-source/desktop-long.html) — Frame B source for the long desktop reference.
- [`reference-source/compact-typical.html`](reference-source/compact-typical.html) — Frame C source for the typical compact-width reference.
- [`reference-source/compact-long.html`](reference-source/compact-long.html) — Frame D source for the long compact-width reference.
- [`reference-source/edge-states.html`](reference-source/edge-states.html) — Frame E source for the edge-facts reference.

## Production constraints

Designer `DESIGN.md` files were intentionally excluded and are not authoritative.

Do not import or run the mockup HTML in production. Do not copy its Tailwind CDN setup, Google Fonts, Material Symbols, scripts, or prototype behavior into production code. Production must use the existing React/Tauri code, `src/ui/styles.css`, and system fonts.
