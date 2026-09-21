---
name: Warm Tactile POS
colors:
  surface: '#FFFCF7'
  surface-dim: '#ded9d1'
  surface-bright: '#fef9f0'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f8f3ea'
  surface-container: '#f2ede4'
  surface-container-high: '#ece8df'
  surface-container-highest: '#e7e2d9'
  on-surface: '#1d1c16'
  on-surface-variant: '#4b454b'
  inverse-surface: '#32302a'
  inverse-on-surface: '#f5f0e7'
  outline: '#7c757c'
  outline-variant: '#cdc4cb'
  surface-tint: '#68596d'
  primary: '#271b2c'
  on-primary: '#ffffff'
  primary-container: '#3d3042'
  on-primary-container: '#a997ad'
  inverse-primary: '#d3c0d7'
  secondary: '#994626'
  on-secondary: '#ffffff'
  secondary-container: '#fe956e'
  on-secondary-container: '#762c0e'
  tertiary: '#142312'
  on-tertiary: '#ffffff'
  tertiary-container: '#293926'
  on-tertiary-container: '#90a38a'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#f0dcf4'
  primary-fixed-dim: '#d3c0d7'
  on-primary-fixed: '#231728'
  on-primary-fixed-variant: '#504255'
  secondary-fixed: '#ffdbcf'
  secondary-fixed-dim: '#ffb59b'
  on-secondary-fixed: '#380d00'
  on-secondary-fixed-variant: '#7a2f11'
  tertiary-fixed: '#d5e8cc'
  tertiary-fixed-dim: '#b9ccb1'
  on-tertiary-fixed: '#101f0e'
  on-tertiary-fixed-variant: '#3b4b37'
  background: '#fef9f0'
  on-background: '#1d1c16'
  surface-variant: '#e7e2d9'
  surface-subtle: '#EEE7DC'
  surface-elevated: '#FFFFFF'
  text-base: '#29252B'
  text-muted: '#625A63'
  text-disabled: '#817982'
  border-default: '#E4DDD2'
  border-strong: '#8D8177'
  primary-hover: '#312535'
  warning-edge: '#C7923E'
  danger: '#A33F46'
  danger-hover: '#873139'
typography:
  display-total:
    fontFamily: Segoe UI
    fontSize: 24px
    fontWeight: '650'
    lineHeight: 30px
    letterSpacing: -0.01em
  headline-1:
    fontFamily: Segoe UI
    fontSize: 24px
    fontWeight: '650'
    lineHeight: 30px
    letterSpacing: -0.01em
  headline-2:
    fontFamily: Segoe UI
    fontSize: 18px
    fontWeight: '650'
    lineHeight: 24px
  headline-3:
    fontFamily: Segoe UI
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 22px
  body-default:
    fontFamily: Segoe UI
    fontSize: 15px
    fontWeight: '400'
    lineHeight: 22px
  body-money:
    fontFamily: Segoe UI
    fontSize: 15px
    fontWeight: '500'
    lineHeight: 20px
  label-default:
    fontFamily: Segoe UI
    fontSize: 13px
    fontWeight: '600'
    lineHeight: 18px
  caption-default:
    fontFamily: Segoe UI
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
  mono-sku:
    fontFamily: Cascadia Mono
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
    letterSpacing: 0.02em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  gutter: 1.25rem
  gutter-compact: 1rem
  margin: 1.5rem
  margin-compact: 1rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.25rem
  space-2xl: 1.5rem
---

## Brand & Style

This design system drives a retail Point of Sale (POS) environment tailored for high-frequency counter operations. The operational context demands immediate legibility, zero-latency feedback, and high density without visual claustrophobia. The brand aesthetic merges utilitarian mechanical precision with a warm, grounded earth-tone palette—rejecting the sterile blue/cool-gray conventions of standard enterprise software in favor of an artisanal yet hyper-functional workshop aesthetic.

The design movement is **Tactile Warm Functionalism**. It pairs solid, purposeful surface tiers with deliberate structural borders, subtle surface elevations, and warm, readable typography. Physical feedback is translated directly into digital ergonomics: large touch targets (minimum 44px), distinct tabular numeric hierarchies, and high-visibility focus indicators designed for fast keyboard navigation and barcode-driven sales workflows.

## Colors

The color palette is built upon a warm linen canvas with high-contrast plum-slate and terracotta accents.

- **Canvas & Surfaces**: The base canvas (`#F4EFE6`) provides a low-strain, warm background. Content panels use `#FFFCF7` to cleanly separate logical groups. Secondary containers, table headers, and prefix chips use `#EEE7DC`, while elevated dialogs and active inputs use pristine `#FFFFFF`.
- **Primary & Interactive**: The primary brand action color is `#3D3042` (deep aubergine slate), transitioning to `#312535` on interaction.
- **Focus & Accent**: The secondary accent is `#C56845` (warm copper/terracotta), reserved for focus rings, highlights, and active operational callouts.
- **Feedback Semantics**:
  - **Success**: `#52634D` (olive moss) provides a measured, grounded confirmation state.
  - **Warning**: `#C7923E` (amber ochre) indicates threshold alerts like low stock.
  - **Danger**: `#A33F46` (brick crimson) handles removals, invalid values, and fatal errors, shifting to `#873139` on press.
- **Text & Contrast**: Primary body text is set in deep charcoal (`#29252B`), ensuring strict WCAG AAA compliance against warm surfaces. Secondary metadata is styled in `#625A63`.

## Typography

The type system prioritizes computational clarity and scanning velocity. It relies on system-grade humanist proportions (`Segoe UI`) for structural UI and strict monospaced geometry (`Cascadia Mono`) for part tracking and SKU codes.

All currency numbers (`body-money`, `display-total`) must be styled with `font-variant-numeric: tabular-nums lining-nums` to ensure exact column alignment across transactional lines. SKU displays enforce strict uppercase rendering with fixed-width mono characters to prevent reading mistakes during warehouse and counter handoffs.

## Layout & Spacing

The layout uses a multi-tier structural grid optimized for fixed desktop POS terminals while collapsing predictably onto touch/tablet surfaces.

- **Desktop (`min-width: 961px`)**:
  - Global Shell: Fixed 208px sidebar paired with an auto-scrolling main viewport padded at `margin: 1.5rem` (`24px`).
  - Sales Layout: Asymmetric 2-column grid (`minmax(0, 1fr)` for product catalog, `minmax(260px, 320px)` for checkout summaries) separated by a `1.25rem` (`20px`) gutter. The summary panel is pinned `sticky` at the block top.
- **Compact (`max-width: 960px`)**:
  - Sidebar compresses to 176px with `1rem` (`16px`) padding.
  - Sales layout collapses into a single vertical stack (`minmax(0, 1fr) max-content`) where the sales summary sits under the product list without sticky positioning.
- **Internal Component Rhythm**:
  - Form field rows gap at `0.75rem` (`12px`).
  - Search and result rows gap at `0.75rem` (`12px`) with `0.5rem` (`8px`) vertical row padding.
  - Cart rows use `0.75rem` (`12px`) internal padding and gaps.

## Elevation & Depth

This design system avoids decorative drop-shadows on standard surfaces, opting for **tactile borders and tonal layers** to express spatial hierarchy:

1. **Level 0 (Canvas Base)**: `#F4EFE6` fills the background viewport. Flat and non-elevated.
2. **Level 1 (Structural Panels)**: Surface `#FFFCF7` outlined by a solid `1px` border in `#E4DDD2`. No box-shadow. Structural containment is established through crisp boundaries.
3. **Level 2 (Inlaid Elements & Recessed States)**: `#EEE7DC` creates indented fields, table header bands, and prefix adornments. Outlined with `#E4DDD2`.
4. **Level 3 (Interactive Modals & Checkout Overlays)**: Surface `#FFFFFF` sits above an ambient dimming backdrop. Elevation is established via a diffused warm ambient shadow: `0 12px 32px rgba(41, 37, 43, 0.18)` paired with a `1px` border in `#8D8177`.

## Shapes

The design system enforces a calibrated, mechanical corner radius scale:
- **Buttons, Inputs, and Select Controls**: `6px` (`0.375rem`). Provides an intentional, tactile shape that is clearly distinguished from surrounding panels.
- **Panels and Group Cards**: `8px` (`0.5rem`). Keeps layout cards crisp and structural without feeling harsh.
- **Modals and Dialog Overlays**: `10px` (`0.625rem`). Softens the prominent transactional layer floating over the POS shell.
- **Badges and Indicators**: `4px` (`0.25rem`). Maintains legibility and compact badge geometries.

## Components

### 1. Buttons & Actions
- **Touch Targets**: Standard interactive controls maintain a minimum height of `44px`; prominent confirmation controls scale to `48px`.
- **Primary Button**: Background `#3D3042`, text `#FFFCF7`, radius `6px`. Hover state: `#312535`. Active/Pressed: `#29252B`. Disabled: Background `#EEE7DC`, text `#817982`.
- **Secondary Button**: Background `#FFFCF7`, border `1px solid #8D8177`, text `#29252B`. Hover state: Background `#EEE7DC`.
- **Tertiary / Destructive Action**: Borderless or faint border; danger variant uses text `#A33F46`, hovering to `#873139` with subtle background `#EEE7DC`.
- **Focus Indicator**: Global focus ring is a solid `3px` outline in `#C56845` with a `2px` offset.

### 2. Input Fields & Currency Inputs
- **Base Input**: Height `44px`, background `#FFFFFF`, border `1px solid #E4DDD2`, radius `6px`, horizontal padding `12px`. Text `#29252B`.
- **Currency Field**: Grouped layout with an integrated prefix adornment (`Bs`) on `#EEE7DC`, bordered by `#E4DDD2`. Numbers are right-aligned, styled with `body-money` (`15px`, tabular figures).
- **Error State**: Border shifts to `2px solid #A33F46`. Attached inline error message rendered in `caption-default` with `role="alert"`.

### 3. Badges & Stock Chips
- **Geometry**: Height `24px`, padding `2px 8px`, radius `4px`, `label-default` font.
- **Available**: Background `#52634D` at 12% opacity, text `#52634D`, border `1px solid #52634D`.
- **Low Stock**: Background `#C7923E` at 14% opacity, text `#C7923E`, border `1px solid #C7923E`.
- **Out of Stock**: Background `#A33F46` at 12% opacity, text `#A33F46`, border `1px solid #A33F46`.

### 4. Product List & Cart Rows
- **List Rows**: Non-table list implementation (`ul`/`li`). Row padding `8px 12px`, separated by `1px solid #E4DDD2`. Hover highlights with `#EEE7DC`.
- **Cart Rows**: Compact grid with product title, Cascadia Mono SKU, unit price, quantity stepper, and dynamic subtotal. Bottom border `1px solid #E4DDD2`.

### 5. Panels & Summary Box
- **Panels**: Background `#FFFCF7`, border `1px solid #E4DDD2`, radius `8px`, internal padding `20px` (desktop) or `16px` (compact).
- **Sticky Summary Box**: Contains unit count, grand total in `display-total` (`24px / 30px / 650`), and primary `Review and checkout` action.

### 6. Checkout & Confirmation Dialogs
- **Backdrop**: `#29252B` with 40% opacity.
- **Modal Window**: Max width `min(1040px, 100%)`, max height `calc(100vh - 48px)`. Radius `10px`, background `#FFFFFF`, border `1px solid #8D8177`, padding `24px`. Cart pane maintains independent vertical scroll while overall dialog remains `overflow: hidden`.