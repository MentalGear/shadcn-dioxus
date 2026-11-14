---
id: 519
title: "Every non-menu overlay re-assembles the anchored panel recipe: one `top_layer::use_anchored_panel`, and the hover timers in their own module"
status: "PROPOSED 2026-11-14 from the overlay layering audit; before row 449 or as its first step."
size: "M (inferred, 8 to 10 h), one lane…"
area: primitives
updated: 2026-11-14
---

# Row 519: one `top_layer::use_anchored_panel`, and the hover timers in their own module

## Item

Eight overlays each assemble the same anchored panel by hand: `use_popover_shown_while_mounted`,
`use_anchor_position_fallback`, a `merge_attributes` for `role`, `popover`, `data-side`, `data-align` and the open
state, and `anchored_content_attributes`. The copies are Tooltip (`tooltip.rs:490-540`), HoverCard
(`hover_card.rs:648-695`), Popover non-modal (`popover.rs:732-760`), Navbar (`navbar.rs:924-1000`), NavigationMenu
(`navigation_menu.rs:1002-1050`), Select list (`select/components/list.rs:356-460`), Combobox list
(`combobox/components/list.rs:145-185`) and Toast (`toast.rs:882-925`), about 25 code lines each plus repeated
comments (inferred). The menu family already generalises the recipe: `use_menu_panel` (`menu_root.rs:262-330`) serves
Dropdown, Context, Menubar and submenus. The hover timers (`use_hover_timers`, `HoverTimers`, `use_hover_dismiss`,
`hover_card.rs:44-190`, `:613`) are shared by Tooltip and HoverCard but live in HoverCard's file.

## Status / trigger

PROPOSED 2026-11-14 from [the overlay layering audit](../research/overlay-layering-audit-2026-11-14.md); before
[row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) or as its first step.

## Size

M (inferred, 8 to 10 h), one lane, medium risk.

## Detail

- **The construction.** One hook and one config struct in `top_layer/`: `id`, `open`, `set_open`, `kind` (Manual or
  Auto), `sync` (while-mounted or immediate), the anchor id, `side`, `align`, the gap and the owned attributes. It
  returns the attribute list to spread, as `use_menu_panel` does. The 8 sites above move onto it, and `use_menu_panel`
  becomes a thin caller, so the three menu hosts share it too. It removes about 200 code lines of copies against about
  80 for the hook (120 to 180 net, inferred) and the repeated comments.
- **Per-site differences it carries.** Manual or Auto; while-mounted or immediate (`popover_sync.rs`); the gaps (Tooltip
  4 px, Popover 8, HoverCard 10, Select and Combobox 4); HoverCard's dormant `force_mount` instance
  (`hover_card.rs:567-585`, `hidden` on the panel); Select's item-aligned case, which carries no engine attribute
  (`select/components/list.rs:430-460`); Toast's no-op native-close callback.
- **The free precursor (inferred 0.5 h, no behaviour change).** Move `use_hover_timers`, `HoverTimers` and
  `use_hover_dismiss` out of `hover_card.rs` into their own module; `tooltip.rs:3` imports it from there.
- **Why before row 449.** Row 449 edits the spread line and the `onbeforetoggle` handler at 13 call sites. With the
  recipe in one hook its 13 call-site edits become one.
- **What pins behaviour** (filenames only; their content is unread): `playwright/tooltip.spec.ts`,
  `hover-card.spec.ts`, `hover-card-force-mount.spec.ts`, `popover.spec.ts`, `navigation_menu.spec.ts`,
  `anchored-close-clip.spec.ts`, `dismiss-parity.spec.ts`, `escape-gap.spec.ts`,
  `oracle/tier2-html/top-layer.spec.ts`; unit tests `tooltip.rs:641` (an SSR open tooltip with one folded `style`),
  `hover_card.rs:784`, `top_layer/tests.rs`. The lane reads them before it moves a site.
- **Further candidates**, one line each:
  - `HoverSurface`: Tooltip and HoverCard as one root, trigger and content with a config struct; about 250 lines
    (inferred), about 5 h, low risk, right after this row.
  - One hover-intent engine under `HoverTimers`, `HoverIntent` and `SubMenuState` (one touch rule, one set of delay
    constants); medium risk, after [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md).
  - `use_modal_dialog` for the dialog quartet repeated in `dialog.rs:544-600`, `alert_dialog.rs:292-340` and
    `popover.rs:561-620`; about 60 lines, low risk.
- **Proof.** The specs and unit tests above stay green on a release build; a gate or test that the 8 sites no longer
  call the three hooks directly.
