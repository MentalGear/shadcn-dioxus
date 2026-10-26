---
id: 433
title: "Slot variables derived from the sheet, not documented by hand"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P8)."
size: "S-M"
area: docs
updated: 2026-10-25
---

# Row 433: Slot variables are derived from the sheet, not documented by hand

## Item

The slot variables a component reads (`var(--dx-label-gap, ...)`, the composition contract of row 170) are kept three ways: a fallback in the CSS, a "Slot variables" table in `docs.md` (present for only 10 components), and an allowlist in `check-css-vars-defined.sh`. Construction: the table is generated at build time from the sheet, as the Props reference already is. A read of a component's own prefix with a fallback, which the sheet does not declare, is a slot; a `/* slot: ... */` comment supplies its description. A slot with no description fails the gate, so a slot cannot be read yet undocumented, or documented yet unread.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P8).

## Size

S-M

## Detail

Source: [final sweep note, P8](../research/final-sweep-2026-10-25.md). A scanner beside `preview/props_scan.rs`, `docs.md` edits, one gate.

- **Measured (heuristic, a lower bound):** a scan of `preview/src/components/*/style.css` for own-prefix `var(--dx-<component>-<name>, fallback)` reads that the sheet itself does not declare found 60 slot reads, 11 with no mention in the component's `docs.md`: `calendar` (`--dx-calendar-background`, `-border`), `chart` (`--dx-chart-center-primary-max`, `-secondary-max`, `--dx-chart-legend-item-basis`), `kbd` (`--dx-kbd-background`, `-color`), `drawer` (`--dx-drawer-drag`), `dropdown_menu` (`--dx-dropdown-menu-trigger-focus-ring`), `select` (`--dx-select-scroll-button-height`), `toggle_group` (`--dx-toggle-group-spacing`). Some are set by script, which the scan cannot tell.
- **Second hand list:** only 10 `docs.md` files have a "Slot variables" section (`label/docs.md:14-22` is the model). `check-css-vars-defined.sh:72-96` ALLOWLIST holds 19 slot names with reasons such as "slot, label/docs.md", plus three names written by JS.
- **Construction:** each row is `name`, `fallback`, and the property the `var()` feeds. A variable set by JS is marked `/* slot-internal */`.
- **Subsumes:** the 11 undocumented reads, the ten hand tables, and the 19 allowlist entries. **Does not subsume:** slots in the theme's own tokens (the theme documents itself) and the custom properties set by JS, which stay as `slot-internal` entries.
- **Gate:** in the style of `check-props-docs.sh`, a slot read with no description fails, and a `docs.md` may not declare its own `## Slot variables`, as it may not declare `## Props reference`.
- **Inferred:** the 11 is a sizing figure, not a defect count. A slot read through another prefix (Field setting `--dx-label-*`) is counted under label.
