# Dashboard Round-1 visual references

This directory preserves the independently approved Round-1 static visual references:

- [`desktop-typical.png`](desktop-typical.png): approved Frame A at 1440×900.
- [`compact-typical.png`](compact-typical.png): approved Frame C at 960×800.

Frames B–G remain future designer deliveries and are not included here.

## Implementation authority

[`../dashboard-figma-handoff.md`](../dashboard-figma-handoff.md) is the behavioral and accessibility authority for production implementation. These images are visual references only.

The HTML under [`reference-source/`](reference-source/) is disposable reference material only. It must not be imported, run, or copied into production. Do not copy prototype CSS, inline SVG or icons, scripts, browser dependencies, or prototype behavior into React or Tauri.

Production implementation must use the existing components, [`src/ui/styles.css`](../../../src/ui/styles.css), system fonts, and the current shell and data contracts.

Designer `DESIGN.md` files are deliberately excluded from this repository reference set and are non-authoritative.
