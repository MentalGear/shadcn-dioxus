---
id: 259
title: "Popup root: one theme rule resets every inherited property at a popover or dialog (typography half of row 218)"
status: "LANDED 2026-10-21 (lane popup-root-reset: Popup root theme rule + check-popup-root-reset gate; spec…"
size: "M"
area: overlays
updated: 2026-10-21
---

# Row 259: Popup root: one theme rule resets every inherited property at a popover or dialog (typography half of row 218)

## Item

A popover or dialog stays a DOM descendant of its trigger (the top layer changes painting, not inheritance), so every
inherited property of whatever holds the trigger leaks into the panel: Select's scroll buttons were 14px inside a
`dx-card-footer` and 16px alone, and the toast region took the 14px of `dx-component-card-preview`
(`popup-style-isolation.spec.ts` site-home, chart, data_table, message_scroller, area_chart, pie_chart, site-charts).

## Status / trigger

**LANDED 2026-10-21 (lane popup-root-reset: Popup root theme rule + check-popup-root-reset gate; spec 592 s -> 150 s)**

## Size

M

## Detail

Split out of [row 218](../218-popup-style-isolation-top-layer-popups-stay-dom.md) (owner-approved typography-reset half; the
variant-slot gate half stays open there). Descendant selectors were already closed by `check-popup-boundary.sh`; this is the
inheritance half, by construction.

**Construction.** `preview/assets/dx-components-theme.css`, section "Popup root": ONE rule,
`:where([popover][class^="dx-"], [popover][class*=" dx-"], dialog[class^="dx-"], dialog[class*=" dx-"])`, zero specificity,
that gives the popup root the theme's ROOT value of 98 inherited properties: `font` (normal normal 400 normal
`var(--dx-text-base)` / normal `var(--dx-font-sans)`), `color: var(--dx-foreground)`, `text-align: start`, `cursor: auto`,
`user-select: text` (`auto` would resolve to a `none` parent), `white-space: normal`, `pointer-events`, `list-style`, the
table/SVG/font-feature/ruby/hyphenation families, and so on; the rest take their initial value. It leaves out `direction` (RTL
must reach the panel), every custom property (slots keep composing), and the document-level settings `color-scheme`,
`text-size-adjust`, `text-rendering`, `accent-color`, `-webkit-tap-highlight-color`. The property list is the inherited
longhands of mdn-data 2.27.1 (`css/properties.json`, `inherited: true`), kept in the gate.

**Two stray inheritance sources the construction needed removing.** `.dx-toast-container[popover] { color: inherit }`
(toast sheet; now `var(--dx-foreground)`) and the two JS-injected zero-specificity `color: inherit` rules in
`primitives/src/top_layer.rs` (`:where([popover]), :where(dialog)`) and `primitives/src/toast.rs` (the raw toast region), which
come later in the document and so out-ranked the theme rule: both now read `var(--dx-foreground, inherit)`, so only an app
without the theme inherits.

**Audit of deliberate inheritance.** No popup sheet relied on its host for typography: every popup root that declares its own
size/colour/alignment (popover, hover card, tooltip, command, combobox, select items, menu items via `--dx-menu-item-*`
slots) keeps winning (any specificity beats the rule); the roots that declared none (dropdown/context menu, select and
combobox lists, dialog, alert dialog, sheet, drawer, navigation menu, navbar, menubar) rendered 16px alone and now render
16px everywhere (the old spec's `ALLOWED` font-size entries for DropdownMenu and Select were exactly this). The one explicit
inheritance on a popup root was the toast region's `color: inherit` (above). Custom-property slots (`--dx-menu-item-font-size`,
`--dx-select-scroll-button-height`, ...) are untouched.

**Gate.** `scripts/check-popup-root-reset.sh` (in `run-gates.sh`): exactly one popup-root rule, a single `:where()`, all four
selector forms, every listed property declared (a covering shorthand counts), no `direction`/`color-scheme`/custom property,
mdn-data cross-check when installed (a newly inherited platform property fails until declared or excluded with a reason), and
a built-in self-test that mutates the rule five ways (a property dropped, the `font` shorthand dropped, `direction` added, a
custom property added, specificity raised) and requires each to turn the gate red.

**Spec.** `playwright/popup-style-isolation.spec.ts` shrank from every popup on every one of 85 pages (592.4 s) to one
representative popup per popup component, each opened as the page renders it and again in a hostile host (every inherited
property set absurdly on the popup's parent), plus the two input-group-addon pages (the nesting row 218 was found in) and the
site pages' cards: 22 tests, 150.4 s. A page lists the kinds it must open; a kind never opened fails. Also fixed there:
`reset()` reloaded the page for the always-open toast region on every trigger.

**Evidence.** New spec 22/22 on a debug SSG; the same spec against the same build with the popup-root rule cut out of the
CSS is red on 19 of 22 (tooltip, select, ... show word-spacing, text-align, font-weight, cursor, ... leaking); the OLD spec
from f127233a passes 89/89 on the new build, also with its `ALLOWED` table emptied (89/89), including site-home.
