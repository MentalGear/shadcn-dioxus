---
id: 292
title: "Tooltip: rich-content demo lays title and body side by side"
status: "LANDED 2026-10-20 (lane controls, 782baae0, spec fixes 839c07c4)."
size: "S"
area: preview
updated: 2026-10-20
---

# Row 292: Tooltip: rich-content demo lays title and body side by side

## Item

In the "Rich content" tooltip the bold title and the body sit next to each other (your screenshot). They stack correctly only during the fade.

## Status / trigger

LANDED 2026-10-20 (lane controls, 782baae0, spec fixes 839c07c4).

## Size

S

## Detail

- **Why it changes mid-transition:** `.dx-tooltip-content[data-state="open"] { display: inline-flex }` (`tooltip/style.css:192`) only matches while the tooltip is open. The Nova base also sets `align-items: center` and a gap but no `flex-direction`, so the two `<p>`s become a row.
- During the closing fade `data-state="closed"`, that rule stops matching. The content falls back to block display, so the paragraphs stack, which is the "correct" look you saw.
- **Fix:** wrap the demo's two `<p>`s in a column (`tooltip/variants/main/mod.rs:10-19`).
- Don't change the base rule: icon-plus-text and Kbd tooltips depend on the row layout.
- Also make the closed state keep `inline-flex`, so no tooltip changes layout while it fades out. That is the general bug behind this one.

## Landed

- **Root cause:** `.dx-tooltip-content[data-state="open"] { display: inline-flex }` was the only thing that made the content a flex row, so the closing fade (`data-state="closed"`) fell back to the UA block layout. A bare `display` on the class was not an option: author `display` beats the UA's `[popover]:not(:popover-open) { display: none }`.
- **Fix by construction:** the rule now covers both states, `:is([data-state="open"], [data-state="closed"]:is(:popover-open, :not([popover])))`, so a tooltip lays out the same open and closing and is still hidden when not shown. The demo wraps title and body in a column `div` (base row layout kept for icon and Kbd tooltips).
- **Spec:** `playwright/tooltip.spec.ts` "Rich content layout (row 292)": paragraphs stacked at rest, and every sampled frame of the close fade has the open display. Both red on the pre-fix build, green after.
