---
id: 137
title: "RTL by construction -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for…"
status: "LANDED 2026-10-19 (items a-e: reactive DirectionProvider, preview RTL toggle, check-direction-keys…"
size: "M (the docs page alone is S; parts a…"
area: docs
updated: 2026-10-05
---

# Row 137: RTL by construction -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for…

## Item

**RTL by construction** -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for `navigation_menu`/`drawer`/`command`/`date_picker`), the old row 137 (`Direction` has no user-facing docs page or `component.json`) and shadcn's "RTL example on every page" gap

## Status / trigger

**LANDED 2026-10-19 (items a-e: reactive DirectionProvider, preview RTL toggle, check-direction-keys gate, logical popup align).** Was: **BACKLOG, MERGED 2026-10-05 (owner decision); the owner then CONFIRMED the solve-by-construction version the same day: a reactive `DirectionProvider`, a preview-level LTR/RTL toggle, the `scripts/check-direction-keys.sh` gate, logical-properties CSS and an RTL oracle (pieces a-e below)**; scheduled in the next-session order (features, after 138, 146-148, 161, 162 and 165; it shares `preview/src/main.rs` with 148, 161 and 162, so it goes after them). Was: **BACKLOG** -- recorded 2026-10-04 from the catalog audit (§3.7); low value, optional

## Size

M (the docs page alone is S; parts a, b, c and e are S-M each)

## Detail

One row for the three RTL threads (row 13's open follow-ups, the old row 137 "`Direction` has no docs page", shadcn's "RTL example
on every page"), merged 2026-10-05 and confirmed by the owner the same day as a solve-by-construction version.

**Facts at the time.** `primitives/src/direction.rs` had `Direction`, `DirectionProvider`, `use_direction` and
`Direction::resolve_horizontal`, used by 17 primitives; five files handled `ArrowLeft`/`ArrowRight` without it (`tag_group`,
`date_picker`, `accordion`, `navigation_menu`, `move_interaction`); `DirectionProvider` captured the direction at mount; 15 hand-built
per-component RTL variants existed (driven by `oracle/tier3-radix/rtl.spec.ts`), and layout already mirrored through CSS logical
properties (`check-css-logical-properties.sh`, row 13).

**The construction (one rule, so "a component forgot RTL" cannot recur), landed 2026-10-19 (lane rtl, round 5):**

- (a) `DirectionProvider` is signal-backed (`Signal<Direction>` context written from the prop each render); `use_direction` reads
  it. New `Direction::ltr_equivalent` (built on `resolve_horizontal`) for handlers that match `ArrowLeft` as previous. Reactivity
  needed one more piece: ~13 primitives froze a plain `use_direction()` value into a mount-time `use_context_provider` closure
  (found by the toggle spec: tabs ignored a live flip), so their context structs now hold `use_direction_signal(direction)`, a
  `ReadSignal<Direction>`, and read it at use; a new consumer built the same way follows by construction.
- (b) A preview-level LTR/RTL toggle on component pages (`#dx-direction-toggle`, `PREVIEW_DIRECTION` in `preview/src/main.rs`):
  `dir` on the demo frame plus a `DirectionProvider` around each demo, which gives every page an RTL example like shadcn's
  (block demos are iframes and are not covered). The per-component RTL variants stay as fixtures. A new `direction` component
  page explains all of it.
- (c) `scripts/check-direction-keys.sh` (in `run-gates.sh`): a primitives file naming `Key::ArrowLeft/Right` or `delta_x` outside
  comments and tests must read the direction or sit on a reasoned allowlist (`move_interaction`: physical deltas mirrored by
  slider and resizable; `color_picker`: physical plane; drawer and command have no horizontal key code). The holdouts were fixed:
  tag_group, date_picker (segments), accordion (horizontal), navigation_menu (trigger and link).
- (d) Logical-properties CSS needed nothing new: the toggle sweep (every Normal component, RTL, arrows plus Escape, page overflow,
  page errors) exposed no physical-property leak in component CSS.
- (e) Row 13's open question answered: `top_layer.rs` was NOT direction-aware (`align` start/end were the physical edges). Fixed
  in both the CSS anchor rules (`:dir(rtl)` overrides for top/bottom sides) and the JS positioner (`hAlign`).
  `oracle/tier3-radix/rtl-toggle.spec.ts` covers reactivity, the holdouts, popup align and a sweep of every component.

**Evidence.** `rtl-toggle.spec.ts` 85/85 on a lane-ssg build; 936 existing specs green except `rtl.spec.ts` DropdownMenu.Sub
"opens to the LEFT" (0.11 px over its 1 px tolerance), which fails identically on the fonts lane's build. Remaining: block (iframe)
demos are not covered by the toggle; `message_scroller`'s RTL pass (row 146) can use it.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/137-rtl-by-construction-one-row-for-the-three-rtl.md`.*
