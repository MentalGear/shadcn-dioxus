---
id: 248
title: "Select renders its placeholder until mount, so a Select with a default value shifts layout at hydration"
status: "LANDED 2026-10-21 (already fixed by row 306's item_to_label, fb8c0b65)"
size: "S-M"
area: components
updated: 2026-10-21
---

# Row 248: Select renders its placeholder until mount, so a Select with a default value shifts layout at hydration

## Item

Select renders its placeholder until mount, so a Select with a default value shifts layout at hydration

## Status / trigger

**LANDED 2026-10-21 (already fixed by row 306's item_to_label, fb8c0b65)**

## Size

S-M

## Detail

The Select primitive shows the placeholder until its options register, then the selected label; any Select with a default_value and no placeholder changes width at hydration (the data_table page-size Select caused CLS 0.0131 > 0.01). Fix in the primitive: render the default value's label in SSG and the first client render.

## Resolution (lane ssr-parity, 2026-10-21)

SelectValue's label comes from `item_to_label` over the selected values during render, so a default value's label is in the server HTML. No Select/Combobox demo is in the row-329 parity baseline (select, combobox, data_table all identical before/after the first client frame). Nothing left to change.
