---
id: 187
title: "Demo upgrades: 25 components whose shadcn demo is clearly richer than ours"
status: "CLOSED 2026-10-19 (duplicate of row 188)"
size: "S mostly (20 of 25 S; 5 M: combobox…"
area: docs
updated: 2026-10-19
---

# Row 187: Demo upgrades: 25 components whose shadcn demo is clearly richer than ours

## Item

**Demo upgrades: 25 components whose shadcn demo is clearly richer than ours**

## Status / trigger

**CLOSED 2026-10-19 (duplicate of row 188)**

## Size

S mostly (20 of 25 S; 5 M: combobox, date_picker, input_group, dropdown_menu, field)

## Detail

Full table and the equal-or-better list: [`research/shadcn-demo-gaps-2026-10-11.md`](../../research/shadcn-demo-gaps-2026-10-11.md). **Top items** (most visible, cheapest): checkbox, input, switch and radio_group each have one bare unlabelled control where shadcn shows a labelled Field with description, disabled and invalid states, a choice-card or a group; tabs shows "Tab 1 Content" where shadcn has real Overview/Analytics/Reports/Settings panels and a disabled tab; accordion's main demo is lorem ipsum (reuse the FAQ from the `until_found` variant and add `multiple`); empty/avatar lack the team avatar group; dropdown_menu and context_menu lack icons, shortcuts and a destructive item; input_otp is a bare passcode where shadcn has a "Verify your login" card. Small ones: spinner in a Button and Badge, separator settings list, tooltip with Kbd and on a disabled button, skeleton table/form, collapsible file tree, textarea composer, progress with a percent readout, scroll_area release list, button_group split button, card event and terms cards. **Dependencies on row 155 parts (a demo that shows the part cannot be written before the part exists):** `AvatarGroup`/`AvatarGroupCount`/`AvatarBadge` (group 9) for empty #13 and avatar #14; menu `Shortcut`/`Group`/`Separator`/`variant="destructive"` (group 2) for dropdown_menu #9 and context_menu #10; `ProgressLabel`/`ProgressValue` (group 9) for progress #24; the accordion "Multiple" demo is the same demo as group 9's, so do it once; Calendar `caption_layout` (group 8) for date_picker #7's month/year dropdowns; Switch `size` (group 7), Tabs vertical (group 5) and the invalid ring (group 1) for the "sizes", "vertical" and "invalid" extras of #3, #5 and #1/#2/#4/#22; InputGroup states (row 133) for #8. **Not in row 155 at all:** combobox #6's multi-select chips and clear button need a multi-select Combobox primitive (none today; at best one of the 53 "Add later" items of the published comparison), so #6 needs its own owner decision before it is scheduled. The base demos (labels, descriptions, disabled states, realistic copy) wait on nothing. **Care:** the Playwright specs that assert on the old demo text move with the demos (grep `playwright/` for `Tab 1 Content`, `option1`, `One thing` first); forms must be `DemoForm`, images bundled with `asset!` (the demo gates); and the invalid-ring and Tabs-padding rows (145, 132) restyle the same controls, so write those demos after them or expect a second touch.

## Update 2026-10-19 (item 6, combobox multi-select)

Landed: `ComboboxMulti` (primitive `primitives/src/combobox/multi.rs`, preview wrapper in
`preview/src/components/combobox/component.rs`) with `ComboboxChips`, `ComboboxClear`, chip remove buttons, `name`
form serialization (one hidden input per value, `<form reset>` restores `default_values`), `aria-multiselectable` on
the list, popup open after each pick, Backspace-on-empty removes the last chip, direction-aware chip navigation.
Construction: `ComboboxContext.model` is an enum (`Single` | `Multi { remove, clear }`) fixed by which root renders,
and the roots differ in type (`Option<T>` vs `Vec<T>`), so single-select paths cannot emit or manipulate a list; the
chips/clear/Backspace operations exist only inside `Multi`. New demos: `multiple`, `multiple_clear`, `multiple_form`,
`rtl`. Spec: `playwright/combobox-multiple.spec.ts`. Remaining for #6: groups inside the multi demo and an
input-group addon variant (shadcn's other demos), and a `name` for the single `Combobox`.

**Update 2026-10-19 (parts):** the card event/terms cards, spinner-in-button/badge and the input_group block addons are demoed (see row 188's parts update); the remaining 187 items are outside the parts lane's components.
