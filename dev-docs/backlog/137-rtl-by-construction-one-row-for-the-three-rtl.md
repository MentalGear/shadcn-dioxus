---
id: 137
title: "RTL by construction -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for…"
status: "IN PROGRESS 2026-10-19 (lane rtl: a-e built, see Update 2026-10-19)"
size: "M (the docs page alone is S; parts a…"
area: docs
updated: 2026-10-05
---

# Row 137: RTL by construction -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for…

## Item

**RTL by construction** -- one row for the three RTL threads: row 13's open follow-ups (optional RTL for `navigation_menu`/`drawer`/`command`/`date_picker`), the old row 137 (`Direction` has no user-facing docs page or `component.json`) and shadcn's "RTL example on every page" gap

## Status / trigger

**BACKLOG, MERGED 2026-10-05 (owner decision); the owner then CONFIRMED the solve-by-construction version the same day: a reactive `DirectionProvider`, a preview-level LTR/RTL toggle, the `scripts/check-direction-keys.sh` gate, logical-properties CSS and an RTL oracle (pieces a-e below)**; scheduled in the next-session order (features, after 138, 146-148, 161, 162 and 165; it shares `preview/src/main.rs` with 148, 161 and 162, so it goes after them). Was: **BACKLOG** -- recorded 2026-10-04 from the catalog audit (§3.7); low value, optional

## Size

M (the docs page alone is S; parts a, b, c and e are S-M each)

## Detail

**Facts (2026-10-05).** `primitives/src/direction.rs` has `Direction`, `DirectionProvider`, `use_direction` and `Direction::resolve_horizontal`; 17 primitives use it. **Five files handle `ArrowLeft`/`ArrowRight` WITHOUT it:** `tag_group`, `date_picker`, `accordion`, `navigation_menu` and `move_interaction` (the last covers swipe and drag gestures). `DirectionProvider` is deliberately NOT reactive (the direction is captured at mount). 15 per-component RTL demo variants exist (14 `variants/rtl` pages plus the carousel's `virtual_loop_rtl`; `oracle/tier3-radix/rtl.spec.ts` drives them), and layout already mirrors through CSS logical properties, enforced by `check-css-logical-properties.sh` (row 13). shadcn shows an RTL example on every component page; we show one on 15 components, each hand-built. **The construction (one rule, so "a component forgot RTL" cannot recur):** (a) make `DirectionProvider` reactive (signal-backed), so the direction can change live; (b) a preview-level LTR/RTL toggle that wraps every demo in `DirectionProvider` and sets `dir` on the demo container. It replaces the per-component RTL variants as the way to SEE RTL (keep them as fixtures for `rtl.spec.ts`) and gives every page an RTL example, which is what shadcn has; (c) a gate `scripts/check-direction-keys.sh`: horizontal arrow-key and gesture handling in `primitives/` must go through `Direction::resolve_horizontal`, or sit on an allowlisted vertical-only case with a stated reason (the shape of the other `check-*.sh` markers); it catches the five holdouts, and must be added to `scripts/run-gates.sh` and to CLAUDE.md's gate sentence; (d) logical-properties CSS: layout needs nothing new, because it already mirrors through CSS logical properties and `check-css-logical-properties.sh` keeps it that way (the toggle in (b) is what would expose a physical-property leak, which then gets fixed in CSS, not worked around per component); (e) an oracle that runs every component's keyboard and positioning smoke under RTL via the toggle, which also answers row 13's open question whether `top_layer.rs`'s JS anchor positioning is direction-aware. **Subsumes:** row 13's follow-ups (`drawer`'s `side` is a physical-geometry fact and `command`'s list is a vertical stack, so the gate's allowlist may record "nothing to flip" for them instead of code), this row's old scope (below) and the shadcn RTL gap. **The `Direction` docs page becomes the explainer for all of it** (what the provider does, that it is reactive, the toggle, the logical-CSS rule, the gate). **Does not cover:** `message_scroller`'s own RTL pass (row 146; it can use the toggle once it exists) and the anchor-positioning audit if (e) is not built. **Old row 137 text (2026-10-04, kept as history):** shadcn's `direction` page documents `DirectionProvider`/`useDirection` (a 6-line re-export of Base UI's). The primitive already exists here: `primitives/src/direction.rs` (`Direction`, `DirectionProvider`, `use_direction`, `Direction::resolve_horizontal`), every RTL-aware root takes a `dir` prop and 11 components ship an `rtl` demo variant (allowlisted in `check-preview-composition.sh`); users already reach it as `dioxus_primitives::direction`. **Only a docs page is missing:** `preview/src/components/direction/{component.json,docs.md,variants/main/mod.rs}` plus a `examples!` entry and a spec -- no new markup, no oracle.

## Update 2026-10-19 (lane rtl, round 5)

- (a) `DirectionProvider` is signal-backed (`Signal<Direction>` context, written from the prop each render); `use_direction` reads it, so consumers follow a live change. New `Direction::ltr_equivalent` (built on `resolve_horizontal`) for handlers that match `ArrowLeft` = previous.
- (c) `scripts/check-direction-keys.sh` (in `run-gates.sh`): a primitives file naming `Key::ArrowLeft/Right` or `delta_x` outside comments/tests must read the direction or sit on a reasoned allowlist (`move_interaction`: physical deltas mirrored by slider/resizable; `color_picker`: physical plane; drawer and command have no horizontal key code, nothing to flip). Holdouts fixed: tag_group, date_picker (segments), accordion (horizontal), navigation_menu (trigger and link).
- (b) Component pages (Normal kind) have an RTL toggle (`#dx-direction-toggle`, `PREVIEW_DIRECTION` global in `preview/src/main.rs`): `dir` on the demo frame plus a `DirectionProvider` around each demo. Block demos are iframes and are not covered. New `direction` component page (docs explain all of it; demo flips tabs live).
- (e) Row 13's question answered: `top_layer.rs` was NOT direction-aware (`align` start/end were the physical left/right edge). Fixed in both the CSS anchor rules (`:dir(rtl)` overrides for top/bottom sides) and the JS positioner (`hAlign`). `oracle/tier3-radix/rtl-toggle.spec.ts` covers reactivity, the holdouts, popup align and a sweep of every component under the toggle.
- (d) Physical-property leaks the sweep exposes: see the lane report.
