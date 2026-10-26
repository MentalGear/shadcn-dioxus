---
id: 433
title: "Slot variables derived from the sheet, not documented by hand"
status: "DONE 2026-10-26 (lane slot-docs-433). Owner approved 2026-10-25 (row 365 sweep, P8)."
size: "S-M"
area: docs
updated: 2026-10-26
---

# Row 433: Slot variables are derived from the sheet, not documented by hand

## Item

The slot variables a component reads (`var(--dx-label-gap, ...)`, the composition contract of row 170) are kept three ways: a fallback in the CSS, a "Slot variables" table in `docs.md` (present for only 10 components), and an allowlist in `check-css-vars-defined.sh`. Construction: the table is generated at build time from the sheet, as the Props reference already is. A read of a component's own prefix with a fallback, which the sheet does not declare, is a slot; a `/* slot: ... */` comment supplies its description. A slot with no description fails the gate, so a slot cannot be read yet undocumented, or documented yet unread.

## Status / trigger

DONE 2026-10-26 (lane `slot-docs-433`). Owner approved 2026-10-25 (row 365 sweep, P8).

## Size

S-M

## Detail

Source: [final sweep note, P8](../../research/final-sweep-2026-10-25.md). A scanner beside `preview/props_scan.rs`, `docs.md` edits, one gate.

- **Measured (heuristic, a lower bound):** a scan of `preview/src/components/*/style.css` for own-prefix `var(--dx-<component>-<name>, fallback)` reads that the sheet itself does not declare found 60 slot reads, 11 with no mention in the component's `docs.md`: `calendar` (`--dx-calendar-background`, `-border`), `chart` (`--dx-chart-center-primary-max`, `-secondary-max`, `--dx-chart-legend-item-basis`), `kbd` (`--dx-kbd-background`, `-color`), `drawer` (`--dx-drawer-drag`), `dropdown_menu` (`--dx-dropdown-menu-trigger-focus-ring`), `select` (`--dx-select-scroll-button-height`), `toggle_group` (`--dx-toggle-group-spacing`). Some are set by script, which the scan cannot tell.
- **Second hand list:** only 10 `docs.md` files have a "Slot variables" section (`label/docs.md:14-22` is the model). `check-css-vars-defined.sh:72-96` ALLOWLIST holds 19 slot names with reasons such as "slot, label/docs.md", plus three names written by JS.
- **Construction:** each row is `name`, `fallback`, and the property the `var()` feeds. A variable set by JS is marked `/* slot-internal */`.
- **Subsumes:** the 11 undocumented reads, the ten hand tables, and the 19 allowlist entries. **Does not subsume:** slots in the theme's own tokens (the theme documents itself) and the custom properties set by JS, which stay as `slot-internal` entries.
- **Gate:** in the style of `check-props-docs.sh`, a slot read with no description fails, and a `docs.md` may not declare its own `## Slot variables`, as it may not declare `## Props reference`.
- **Inferred:** the 11 is a sizing figure, not a defect count. A slot read through another prefix (Field setting `--dx-label-*`) is counted under label.

## Outcome

- **Scanner:** `preview/slot_scan.rs` (std only, included by `#[path]` like `props_scan.rs`). A slot is a custom property read under the component's own prefix (`--dx-<dir with dashes>-`) with a fallback, never declared by the same sheet; its table row is the name, the first fallback and the properties the reads feed (the declaration the `var()` sits in, comments blanked first). `scripts/slot-scan/main.rs` is the binary the gates compile with a bare `rustc` (`scripts/lib/slot-scan.sh`).
- **Page:** `preview/build.rs` `render_slots_section` adds "Styling (slot variables)" (`docs-styling-slot-variables`) to every component page whose sheet has a public slot, before "Props reference"; `style.css` is registered with `rerun-if-changed`. Row 422 groups it later.
- **Description:** a `/* slot: --name  text */` comment in the sheet (a variable a script or the component itself writes says `/* slot-internal: --name  who writes it */` and stays off the page). The scan found **63 slots** in 20 sheets (the 60 of the sweep was a lower bound), all now described: 59 on the pages of 19 components, 4 internal (`--dx-drawer-drag`, `--dx-chart-center-primary-max` and `-secondary-max`, `--dx-toggle-group-spacing`).
- **Hand lists gone:** the 10 `## Slot variables` sections (45 table rows) are deleted from `docs.md` (their prose survives as descriptions and one generic intro); the 11 undocumented reads are now rows; `check-css-vars-defined.sh` takes the derived slot names as defined, so 10 of its 22 allowlist entries went (12 remain: the 9 theme effect-class slots of `dx-effects.css`, which the theme documents itself, and the 3 JS-written toast names).
- **Gate:** `scripts/check-slot-docs.sh` (in `run-gates.sh`, group Theme and component CSS; `gates.md` regenerated), strict with no allowlist: every slot has exactly one non-empty note; a note naming a variable that is not a slot of its sheet (declared there, read without a fallback, another prefix, or not read) fails; a `docs.md` may not declare `## Slot variables` or the generated heading (`build.rs` asserts the same). Its fixture self-test requires the exact report for one slot of each shape (documented, two-line, bare, internal, empty, declared, bare read, foreign prefix, duplicate, unread, a string that looks like a note).
- **Not subsumed (as planned):** the theme's own tokens, and JS-written names read under a computed name (3 allowlist entries).
