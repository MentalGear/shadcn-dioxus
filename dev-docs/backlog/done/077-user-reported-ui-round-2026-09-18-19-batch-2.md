---
id: 77
title: "User-reported UI round (2026-09-18/19, batch 2): Command palette focus, Data Table filter, Tag Group sizing…"
status: "Landed 2026-09-18/19 (batch 2) — six independent user reports against components rows 66-76 had…"
size: "M"
area: menus
updated: 2026-09-18
---

# Row 77: User-reported UI round (2026-09-18/19, batch 2): Command palette focus, Data Table filter, Tag Group sizing…

## Item

User-reported UI round (2026-09-18/19, batch 2): Command palette focus, Data Table filter, Tag Group sizing, Sheet, Resizable, Navigation Menu reflow, Drawer ×3

## Status / trigger

Landed 2026-09-18/19 (batch 2) — six independent user reports against components rows 66-76 had just shipped

## Size

M

## Detail

Six user reports against components rows 66-76 had just shipped, each root-caused and fixed independently (batch 2; the lane
hand-backs and integration report were session scratchpad, not repo artifacts).

- **Command palette first-open focus (`1e6ae0f`).** `command.rs` keeps `autofocus` and adds an `onmounted` then
  `spawn(set_focus(true))` backstop (the pattern `toggle`/`slider`/`dropdown_menu` use) plus a forwarding `onmounted` prop. Root
  cause unconfirmed: it never reproduced on the CSR dev server in 40+ trials, the report was against the deployed SSG build, so the
  backstop covers the class.
- **Data Table filter question: no defect.** The predicate already matches TanStack's `includesString` (trimmed, case-insensitive
  substring); `0c0f627` named it (`includes_string`, with 8 unit tests). Closed in batch 3 (`b22a222`): the dashboard email
  client's `message_matches_search` now routes through it, which also fixed a latent bug (`tag.label().contains(&query)` never
  lowercased the label).
- **Tag Group sizing (`090d9f1`).** Not a token-migration regression (an audit of the 41 stylesheets row 31b touched found 363
  exact matches, 130 equal-value rewrites, 0 mismatches); the real gap was `text-sm` sizing plus `padding-block` and `min-height`
  under `border-box`, now a 28 px tag (was 20 px).
- **Sheet (`12a8f86`).** A shadcn v4 translation pass: the overlay tint moved onto the `<dialog>`'s own `::backdrop` (the UA
  backdrop opacity was stacking with the root tint, ~55% instead of 50%), `--dx-shadow-lg`, `sm:max-w-sm` at `width >= 40rem`, a
  500/300 ms slide, close-button sizes, plus the Drawer-class centering fix (row 78).
- **Resizable (`47c6291`).** The handle and grip read the wrong token pair (a foreground-ish gray, not the border pair every other
  divider uses); handle width 4 px to 1 px with a 4 px hit area, grip 16x24 to 12x16, demo container constrained like shadcn's 450 px.
  Deliberate deviation: shadcn's fixed `h-[200px]` is not translated, because this primitive's panels are pure-CSS flex-basis
  percentages over `overflow: hidden` and a fixed height clipped the nested panel (caught by screenshot).
- **Navigation Menu "width reflow" (`08a14d8`).** Not a width transition (rAF sampling showed constant width): the panel-swap
  animation was dead, so differently sized panels popped. Two causes: a `transition` at the normal cascade tier cannot outrank the
  inline `transform: none` that `use_anchor_position_fallback` sets on every render, and `use_popover_sync` calls `hidePopover()`
  synchronously on close. Fixed with `@keyframes`/`animation`, `use_popover_shown_while_mounted` (as Tooltip, HoverCard, Popover,
  `SelectList`, `ComboboxList` use, row 19), `width: max-content`, gated by reduced motion. The same commit added the keyboard
  surface row 72 had left out (Home/End, panel Up/Down stopping at the ends, ArrowDown opening a closed trigger) and fixed three
  latent bugs: content children mounting in reverse source order (now an explicit `content_index` prop), native Tab focus never
  syncing into the roving-focus collection, and a stale-value no-op in the Escape-refocus handler.
- **Drawer, three fixes (`7310b89`, `2bf6489`).** (1) The native `dialog:modal` UA sheet centres the panel as a floating card (the
  same class as Sheet's bug, shared construction in row 78); the overdrag gap is covered by a solid `filter: drop-shadow` 40 px
  past the edge. (2) `user-select: none` on the handle and on content while dragging; the handle's `touch-action` corrected from
  `none` to a per-side `pan-*`. (3) `DrawerClose` was unstyled (UA `ButtonFace`, 1.28:1 contrast in dark mode), now styled like the
  outline button; `2bf6489` also reads `active_pointer_id` with `.peek()` (row 79).
- **Navbar sub-item (audit batch 2026-09-26, `7a2f68e`).** Found dead by rAF sampling (open to closed in one frame; the entrance too);
  fixed by the same Navigation Menu construction. `navbar.spec.ts` regression test red 3/3 on main, green 3/3 fixed. Accepted
  limit: a close forced by the browser's own `popover="auto"` light dismiss still skips the animation.

All six were verified live against the shared dev server during batch-2 integration.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/077-user-reported-ui-round-2026-09-18-19-batch-2.md`.*
