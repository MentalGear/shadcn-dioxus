---
id: 381
title: "Interactive-element cursor is one theme option, data-dx-interactive-element-cursor"
status: "LANDED in batch 14. preview/assets/dx-components-theme.css (\"Interactive-element cursor\") has one…"
size: "M."
area: theme
updated: 2026-10-24
---

# Row 381: one cursor option for every interactive element

## Item

The owner asked for a global, consistent cursor on interactive elements: Select, Tabs, Switch and the calendar cells showed `cursor: pointer`, while Button, Combobox's options, Command's items and the Alert Dialog's buttons showed `cursor: default`, so one press looked different in each component.

## Status / trigger

LANDED in batch 14. `preview/assets/dx-components-theme.css` ("Interactive-element cursor") has one zero-specificity rule giving every pressable `dx-` element `cursor: var(--dx-interactive-element-cursor, default)`. The default is shadcn v4's arrow; `data-dx-interactive-element-cursor="pointer"` on `<html>` or any container opts in to the hand, and a nested `="default"` turns it off again. The theme picker's "Cursor" control sets the attribute. `scripts/check-cursor-baseline.sh` keeps per-sheet cursor declarations from growing back. The masked-versus-overlay fade question raised alongside it was left as is.

## Size

M.

## Detail

See the Status section and the batch 14 commits of the lane.
