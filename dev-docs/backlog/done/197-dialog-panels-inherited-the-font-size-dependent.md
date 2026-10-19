---
id: 197
title: "Dialog panels inherited the font-size-dependent UA padding: 1em"
status: "LANDED 2026-10-14"
size: "S (done)"
area: overlays
updated: 2026-10-14
---

# Row 197: Dialog panels inherited the font-size-dependent UA padding: 1em

## Item

Dialog panels inherited the font-size-dependent UA `padding: 1em`

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

Phase C's `font-size: text-sm` on Sheet/Drawer silently moved their inset 16 -> 14px (snapshot S2 U1). Theme: one `:where(dialog[class^="dx-"])` `padding: 0` reset (asserted by `check-theme-baseline.sh`); Sheet/Drawer declare `var(--dx-space-4)`; the top_layer fixture popovers that leaned on the UA value take the floating surface. **Closed 2026-10-14 (edge lane 0579af5, row 207): Sheet panel padding is now 0 (Nova: title 16px from the edge, was 32); Drawer keeps 16px through `--dx-edge-padding` because `theme.rs`'s picker drawer and `main.css`'s chart code drawer rely on it (they can pad themselves later, then Drawer can follow).** UA `max-height: calc(100% - 6px - 2em)` is the one em-based UA value left on dialog/alert_dialog/command.
