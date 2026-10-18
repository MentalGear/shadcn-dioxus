---
id: 192
title: "Tabs size (sm/default/lg) on a shared control-height scale"
status: "LANDED 2026-10-13 (owner decision: the scale, default unchanged)"
size: "S (done)"
area: inputs
updated: 2026-10-13
---

# Row 192: Tabs size (sm/default/lg) on a shared control-height scale

## Item

Tabs `size` (sm/default/lg) on a shared control-height scale

## Status / trigger

**LANDED 2026-10-13 (owner decision: the scale, default unchanged)**

## Size

S (done)

## Detail

The owner found default tab triggers small next to buttons: the list is 32px like a default Button/Input/Select (Nova `h-8`), the trigger inside ~25px. Decision: a size scale, default = today (shadcn parity), `lg` for the bigger look. **Construction:** `--dx-control-height-xs/-sm/-default/-lg` (24/28/32/36px) in the theme; Button sizes, Input, Select trigger, Combobox input and the Input Group addon button read them (zero diff); `TabsSize { Sm, Default, Lg }` -> `data-size` on the Tabs root sets four slot variables (list height 28/32/36, trigger font, inline padding, icon size), re-declared per Tabs so nesting never inherits. Default computed-style identical (harness: 27 elements, light and dark, 0 differences). Demo `tabs/variants/sizes`. **Follow-ups:** Toggle/ToggleGroup `size` (row 155 group 3) and Switch `size` (group 7) must reuse these tokens; the Combobox input is 36px (`-lg`) where Nova uses 32px, a phase-C question.
