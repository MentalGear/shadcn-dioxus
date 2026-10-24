---
id: 13
title: "Phase 6 — RTL (dq direction.rs + key-flip concept)"
status: "Landed 2026-09-19 (batch 3); its open follow-ups (the optional RTL for…"
size: "M"
area: a11y
updated: 2026-10-05
---

# Row 13: Phase 6 — RTL (dq direction.rs + key-flip concept)

## Item

Phase 6 — RTL (`dq direction.rs` + key-flip concept)

## Status / trigger

**Landed 2026-09-19 (batch 3); its open follow-ups (the optional RTL for `navigation_menu`/`drawer`/`command`/`date_picker`, the not-reactive provider, a key-handling gate) MERGED 2026-10-05 into row 137, "RTL by construction"**

## Size

M

## Detail

**The Rust half.** New `primitives/src/direction.rs` (unit-tested): a `Direction` enum, `DirectionProvider` and
`use_direction(local: Option<Direction>)` (a local prop wins, else the nearest provider, default `Ltr`), and
`Direction::resolve_horizontal(self, key)` as the single key-resolution site every per-family flip routes through. It ports the
key-flip concept of `dq direction.rs` (83 lines), not the 708-line file, as `plan.md` Phase 6 framed it. Per family: roving-focus
arrows in `tabs`, `radio_group`, `toggle_group`, `toolbar`, `menubar`, `navbar`; submenu open/close key and side in `menu_sub.rs`;
`Slider` mirrors its pointer, keyboard and visual mapping; `Select` and `ScrollArea` carry a `dir` attribute only; `Resizable`
gained a `text_direction` prop that negates the handle-drag delta (`flex-direction` stays unconditional, retracting an earlier CSS
handoff); `Calendar`'s day grid flips through `horizontal_day_step`. Each flip cites the Radix source it mirrors.

**The fixture decision.** One `rtl` variant page per RTL-aware component (13: calendar, context_menu, dropdown_menu, menubar,
navbar, radio_group, resizable, scroll_area, select, slider, tabs, toggle_group, toolbar), each wrapping its primitives in
`DirectionProvider`, with an unmodified `main` LTR control. Filed tier 3 (Radix-parity, labelled opinion) in
`playwright/oracle/tier3-radix/rtl.spec.ts`: 25 rules, 25/25. `Resizable` and `Calendar` have no Radix original (shadcn wraps
`react-resizable-panels` and `react-day-picker`), so their rules are labelled an extrapolation.

**The CSS half.** 29 stylesheets converted from physical to logical properties by category; screen-geometry facts (`data-side`, a
drawer or sheet's own `side`) and transform-paired centring (`left: 50%` plus `translate(-50%, ...)`) stay physical, marked
`/* rtl-physical: <reason> */`; non-zero `translateX` and keyframe values got a `[dir="rtl"]` counterpart. New gate
`scripts/check-css-logical-properties.sh`: 99 hits on the base tree (`cbae695`) to 0, after an audit of the 8 sheets it had
excluded (commit `3ce62f5`, 16 hits; `EXCLUDE` is now empty). LTR value preservation was proven, not assumed:
`computed-style-snapshot.spec.ts` before against after, 27/29 zero diffs, the other 2 unrelated. The Switch thumb no longer
collapses in RTL.

**Findings.** shadcn's own `new-york-v4` stylesheets carry essentially no RTL support (zero logical utilities, `rtl:` in 2 of 62
files); the real RTL behaviour lives in Radix's JS, which confirms the direction-aware-Rust plus logical-CSS split.
`navigation_menu`, `drawer`, `command` and `date_picker` have no direction-aware keyboard logic: not a shadcn parity gap (all four
already have logical CSS; `Command`'s list is vertical and `Drawer`'s `side` is physical geometry), kept as an optional
beyond-parity item. Whether `top_layer.rs`'s JS anchor engine is itself direction-aware was not audited. A pre-existing,
unrelated bug found on the way is row 85. The follow-ups merged into row 137, "RTL by construction".

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/013-phase-6-rtl-dq-direction-rs-key-flip-concept.md`.*
