---
id: 292
title: "Tooltip: rich-content demo lays title and body side by side"
status: "OPEN (owner review 2026-10-19)."
size: "S"
area: preview
updated: 2026-10-19
---

# Row 292: Tooltip: rich-content demo lays title and body side by side

## Item

In the "Rich content" tooltip the bold title and the body sit next to each other (your screenshot). They stack correctly only during the fade.

## Status / trigger

OPEN (owner review 2026-10-19).

## Size

S

## Detail

- **Why it changes mid-transition:** `.dx-tooltip-content[data-state="open"] { display: inline-flex }` (`tooltip/style.css:192`) only matches while the tooltip is open. The Nova base also sets `align-items: center` and a gap but no `flex-direction`, so the two `<p>`s become a row.
- During the closing fade `data-state="closed"`, that rule stops matching. The content falls back to block display, so the paragraphs stack, which is the "correct" look you saw.
- **Fix:** wrap the demo's two `<p>`s in a column (`tooltip/variants/main/mod.rs:10-19`).
- Don't change the base rule: icon-plus-text and Kbd tooltips depend on the row layout.
- Also make the closed state keep `inline-flex`, so no tooltip changes layout while it fades out. That is the general bug behind this one.
