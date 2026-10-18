---
id: 133
title: "InputGroup has no invalid or disabled styling"
status: "LANDED -- the wrapper draws the shared invalid recipe and --dx-opacity-disabled through slot…"
size: "S"
area: inputs
updated: 2026-10-13
---

# Row 133: InputGroup has no invalid or disabled styling

## Item

`InputGroup` has no invalid or disabled styling

## Status / trigger

**LANDED** -- the wrapper draws the shared invalid recipe and `--dx-opacity-disabled` through slot variables (`input_group/style.css`), the bare control draws neither; `playwright/input_group.spec.ts` "Invalid and disabled states (row 133)" asserts both (2026-10-13)

## Size

S

## Detail

`input/style.css` styles `.dx-input[aria-invalid="true"]` (`:56`) and `.dx-input:disabled` (`:65`); `input_group/style.css` has only `:hover:not(:disabled)` rules (`:16`, `:32`), so a wrapper whose control is invalid or disabled looks like a healthy one (no invalid ring, no disabled opacity on the addons). shadcn's `InputGroup` carries both (`has-[[data-slot][aria-invalid=true]]`, `data-disabled`). **Fix:** `:has(.dx-input-group-control[aria-invalid="true"])` and `:has(.dx-input-group-control:disabled)` rules on the wrapper reusing Input's tokens; add the two states to the input_group demo and a computed-style assertion. **2026-10-05:** the invalid half is an instance of the cross-cutting invalid-state ring (row 155, group 1, built inside row 145's state recipe); do not write a second one here.
