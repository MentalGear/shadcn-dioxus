# Next round plan (2026-10-14): owner findings after phase C, plus the approved queue

Status: **BATCH 1 LANDED 2026-10-14** (items 1-11, rows 193, 197, 203-212; see "Outcome" at the end; batch 2 and rows 213-216 are next). Originally: PLANNED (owner: "we tackle these in a new chat"). Backlog rows 193 and
203-212 point here. Main at the time of writing: `dc44637` (phase C, PR #96; deploy PR #97).

Read first in the new chat: `CLAUDE.md` (lanes, gates, isolated target dirs), this file, then the rows.

## 0. Owner decisions already taken (2026-10-14)

| # | Decision | Where |
|---|---|---|
| D1 | Dark `--dx-border` / `--dx-input` follow shadcn: `oklch(1 0 0 / 10%)` / `oklch(1 0 0 / 15%)` (row 111 decision 2; side-by-side pictures were shown: ours #232323 1.14:1 vs 1.32:1 on the #171717 card; input 1.68 vs 1.58) | row 203 |
| D2 | Date picker: **Apply/Cancel footer (`confirm`) is the DEFAULT**; a no-footer mode stays available | row 209 |
| D3 | Drawer: **`DrawerSide::Auto` is the DEFAULT** (bottom on narrow viewports, side on wide ones) | row 208 |
| D4 | Close-on-scroll helper (row 193) goes ahead, merged with item 7 below | row 193 |
| D5 | Parity wave (row 155 groups 2-9) goes ahead after the overlapping lanes | row 155 |

## 1. Baselines to capture BEFORE the first batch (they cannot be recovered afterwards)

- Computed-style snapshot `N0` of `main` (debug SSG, the harness as of `dc44637`; A/A twice, byte-identical),
  procedure in `dev-docs/research/phase-c-plan-2026-10-13.md` sec. 4 and `/home/user/tgt/snap/cap.sh` if the
  container still has it (it will not in a new container: re-derive from `playwright/computed-style-snapshot.spec.ts`).
- Full Playwright suite on the same build -> `pw-n0.fails` (the reference set; at `dc44637` the known reds are
  the r3 set: hydration-parity selftest (by design), first-paint-styles data_table CLS 0.0131, theme-picker-phone x4,
  drag_and_drop_list:696 (flake)).
- For item 1 (layout flicker): a frame-by-frame recording at the iPad widths BEFORE anything changes.

## 2. The items

Each item: problem -> evidence -> decision -> construction (the class it removes) -> files -> proof.

### Item 1 -- Layout flicker of the top demo group at iPad-landscape widths (row 204)

- **Problem (owner):** the top demo group's elements flicker (re-layout) at a certain viewport width, seen on
  iPad landscape.
- **Evidence:** none yet. Candidates: a breakpoint whose content changes height across the width (e.g. a media
  query at exactly ~1024-1194px toggling a column count, the sidebar collapsing, or a scrollbar appearing and
  crossing the breakpoint -- `html { scrollbar-gutter: stable }` was meant to stop that class); a ResizeObserver /
  container-query feedback loop; `content-visibility: auto` cards re-laying-out; the Extra explainer line or
  wrapping header text changing height.
- **Plan:** a Sonnet debug lane (read-only first): reproduce on a release SSG build at 1024x768, 1180x820,
  1194x834 and 1366x1024, both orientations, with a recorder (rAF loop logging `getBoundingClientRect` of the
  top group's children + `ResizeObserver` entries + `matchMedia` changes for ~3 s with no input). Bisect the
  oscillating rule. Report the root cause with the recording.
- **Construction:** a **layout-stability spec**: sweep widths 320..1440 step 8 px (both schemes), load each
  route group, and fail if any element's box changes after `load + fonts.ready + 2 frames` without input (a
  second change = oscillation). This subsumes the class at every width, not just the iPad one.
- **Files:** whatever the root cause names (likely `preview/assets/main.css` or one component sheet) + new
  `playwright/layout-stability.spec.ts`.
- **Proof:** the recording before/after; the sweep green; snapshot diff limited to the fixed rule.

### Item 2 -- No sticky hover on touch; press feedback as a switchable utility (row 205)

- **Problem (owner):** touch devices show hover/active styles (sticky `:hover` after a tap).
- **Decision (recommended, owner OK with "go"):** hover styles exist only for devices that really hover; a
  short, transitioned **press feedback** (`:active`) is the touch affordance, and it is a system-wide utility
  that can be switched on/off, demoed in the theme/palette popover.
- **Construction:**
  1. Every `:hover` rule in a component `style.css` / `main.css` lives inside
     `@media (hover: hover) and (pointer: fine)`. New gate `scripts/check-hover-media.sh` (allowlist with reasons,
     stale entries fail; register in `run-gates.sh` and the CLAUDE.md sentence). Mechanical sweep of all sheets.
  2. Theme "Press feedback" section: tokens `--dx-press-scale` (e.g. 0.98), `--dx-press-duration`,
     `--dx-press-fill` (a `color-mix` step), applied by ONE zero-specificity rule on the same interactive set the
     theme's `touch-action` rule already enumerates, gated by `html:not([data-dx-press-feedback="off"])` and
     `@media (prefers-reduced-motion: no-preference)` for the transition.
  3. A toggle "Press feedback" in the theme/palette popover (`preview/src/main.rs` theme picker), persisted like the
     other theme settings, setting `data-dx-press-feedback` on `<html>`.
- **Watch:** `:hover` rules that also style `:focus-visible` (split them); hover-intent menus (`hover_intent.rs`)
  are pointer-event driven, not CSS hover -- they already filter nothing by `pointerType`; decide whether
  hover-open should require `pointerType === "mouse"` (Radix does) -- recommended yes, same lane.
- **Proof:** gate green; a spec with `hasTouch`/`isMobile` emulation asserting no hover fill after tap and a press
  fill during `pointerdown`; snapshot: only hover-state records under touch change.

### Item 3 -- Modal initial focus depends on how it was opened (row 206)

- **Problem (owner):** opening a demo dialog/drawer focuses its input, so the mobile keyboard pops up.
- **Decision:** keep focus moving INTO the dialog (WAI-ARIA dialog pattern; screen readers and keyboard users need
  it), but choose the target by the opening modality:
  - opened by **keyboard** (Enter/Space/shortcut on the trigger): the first relevant focusable (today's behaviour);
  - opened by **pointer/touch**: the dialog container itself (`tabindex="-1"`, no visible ring);
  - an explicit `autofocus` on a field always wins.
- **Construction:** ONE place in the shared dialog / focus-trap primitive (`primitives/src/dialog.rs`; check
  `alert_dialog.rs`, `drawer.rs`, Command dialog, modal Popover use it) records the modality of the event that
  opened it (last `pointerdown` vs `keydown`, the same signal `:focus-visible` uses) and picks the initial target.
  Subsumes Dialog, AlertDialog, Sheet, Drawer, Command, modal Popover.
- **Proof:** spec per surface: open by click -> `document.activeElement` is the panel, no input focused; open by
  keyboard -> first field focused; `autofocus` honoured; Escape/Tab trap unchanged; axe unchanged.

### Item 4 + 5 -- One edge-panel system for Sheet and Drawer; Drawer like shadcn's demo (rows 207, 208)

- **Fact (2026-10-14):** both sit on the same modal `dialog` primitive, but each has its own side enum
  (`SheetSide`, `DrawerSide`), its own sheet and its own slide animations; Drawer adds drag-to-dismiss and the
  handle (`primitives/src/drawer.rs`).
- **Construction (row 207):** one edge-panel recipe in the theme (side, slide-in/out keyframes via the shared
  `--dx-floating-enter/-exit`-style slots, inset, radius per side) and one side type; Sheet = edge panel;
  Drawer = edge panel + drag + handle. Fixes stop being made twice.
- **Drawer like shadcn (row 208; owner D3):**
  - `DrawerSide::Auto` = **default**: bottom below the `md` breakpoint, right at/above it (document it; an
    explicit side means exactly that side).
  - Rounded: bottom drawer rounds its top edge; a side drawer floats inset from the viewport edge with all
    corners rounded (shadcn/Vaul demo look; verify against the live shadcn demo page first).
  - **Nested drawers:** a stack context; opening a child scales/offsets the parent (Vaul `NestedRoot`), closing
    restores; Escape and drag close only the top-most; scroll lock and inert stay single (the scroll-lock
    refcount already nests).
  - The parity wave's Sheet `side` typed prop (row 155 group 7) folds in here.
- **Open:** Nova panel padding 0 (row 197) -- decide inside this lane, since the edge-panel recipe owns the inset.
- **Proof:** drawer/sheet specs extended (Auto at 390 vs 1280; nested open/close/Escape/drag order); snapshot.

### Item 6 -- Date picker: verify shadcn, `confirm` default, natural language (row 209)

- **Owner D2:** the Apply/Cancel footer (`confirm`) is the **default**: selections stage inside the panel;
  Apply commits and closes, Cancel/Escape/outside-click restores the previous value and closes. A no-footer
  mode remains (`confirm: false`): single closes on select, range closes when both ends are picked.
- **Natural language:** the owner believes shadcn's original never offers an input. As far as known here,
  shadcn's "Natural Language Picker" uses `chrono-node`, which parses typed text, i.e. it has a text input, and
  shows "Your post will be published on ...". This could not be verified in-session (the shadcn repo is not
  attached; the docs fetch showed no demo code). **First step of the lane: verify on the live demo / source.**
  If shadcn has no input, remove ours; if it has one, match its exact behaviour (placeholder, parse on input,
  the sentence under it, the calendar button).
- **Also:** which shadcn date demos keep the overlay open after a selection (the uncontrolled basic one does) --
  record per demo in the lane report; our demos mirror the source per demo, with `confirm` as the library default.
- **Files:** `primitives/src/date_picker.rs` (staging state), `preview/src/components/date_picker/**`,
  `calendar` only if the footer needs a slot. Specs: `date-picker.spec.ts` (apply/cancel/escape restore; range).

### Item 7 -- Close overlays on scroll (merged into row 193)

- **Decision:** one helper, two policies:
  - **close on any scroll:** Tooltip, HoverCard, Select/Combobox lists, menus when `scroll_lock` is off;
  - **close when the anchor leaves the viewport:** Popover, DatePicker (they can hold half-entered input).
- **Mobile caveat (must):** opening the virtual keyboard scrolls/resizes the *visual* viewport; ignore
  visual-viewport-only scroll and focus-caused scroll, listen only to document / scroll-container scroll.
  Otherwise the Combobox closes the moment it is tapped.
- **Construction:** in the top-layer code next to `viewportMetrics()`; listeners through `js_listener.rs`
  (`check-js-listeners.sh`), passive (`check-blocking-scroll-listeners.sh`); a spec per surface (open, scroll a
  little: policy-dependent; scroll anchor out: closed; keyboard-open simulation: stays open).

### Item 8 -- iOS Combobox list covers its input (row 210; highest priority)

- **Problem (owner):** on iOS the opened combobox list covers the input. Must never happen.
- **Research first:** how does shadcn do it? shadcn's classic Combobox is a **Button** that opens a Popover with
  a `CommandInput` at the top of the panel -- the input is *inside* the panel, so the list cannot cover it.
  shadcn v4's newer Base UI Combobox has an input trigger with a positioner (collision avoidance). Record which
  one our demo mirrors and how that positioner avoids the input (side flip vs shift, `collisionAvoidance`,
  visual-viewport handling).
- **Likely cause:** when the keyboard opens, the visual viewport shrinks/pans; placement reasons in layout-viewport
  terms, and flip/shift lands the list over the input.
- **Construction:** anchored placement treats the anchor's box as an **exclusion zone**: the list goes fully
  below or fully above it inside the *visual* viewport, shrinking its max-height rather than overlapping; in the
  shared placement (`primitives/src/top_layer.rs`, `use_anchor_position_fallback`; check whether the CSS
  anchor-positioning arm's `position-try` can overlap and give it the same guarantee).
- **Regression test (geometry, not visual):** list box never intersects the input box and stays inside the
  visual viewport, swept over: viewports 390x844, 390x664, 390x500, 390x400 (keyboard-reduced), 768x1024,
  1024x768; input at top/middle/bottom of the page; several scroll offsets; visual-viewport zoom/pan (CDP
  `Emulation.setPageScaleFactor` / `setVisibleSize` where available); light/dark; RTL; typing that shrinks the
  list. Assert again after the list's own scroll and after typing.
- **Limits to state honestly:** Playwright cannot open the real iOS keyboard, and only Chromium is installed in the
  container (no WebKit). The spec simulates the keyboard via viewport height + visual-viewport offsets; a
  real-device check stays a manual owner step (write the exact steps into the PR).

### Item 9 -- Chart load animation flicker (row 211)

- **Cause (confirmed in `preview/src/components/chart/style.css` ~273):** the SSG paints the final chart;
  `Chart` sets `data-animate="true"` on its first measured render in the browser (hydration), which restarts the
  bars/lines from zero: full -> gone -> animate.
- **Decision:** not shadcn's "hide until in view" (shadcn is client-rendered and has nothing painted before JS;
  hiding SSG charts would paint empty charts without JS and on first paint). Instead:
  - a chart visible in the **first paint never animates**;
  - a chart **below the fold animates once when it first scrolls into view**, through the visibility-gated
    motion of `primitives/src/activity.rs` (`use_motion`; no new listeners);
  - no JS or `prefers-reduced-motion`: the final drawing, always.
- **Files:** `preview/src/components/chart/component.rs` + `style.css` (all chart kinds share them).
- **Proof:** spec: load a charts page, assert the above-the-fold chart's bars have their final transform at every
  sampled frame from first paint (no scale(0) frame); scroll a below-fold chart in: one animation; reduced motion:
  none.

### Item 10 -- Small-control demos look like shadcn's: labelled, not a lone dot in a big frame (row 212)

- **Problem (owner):** checkbox/toggle-style demos show a small control alone in a big preview frame; shadcn's
  demos pair them with a label (and often a description).
- **Scope:** checkbox, radio_group, switch, toggle, toggle_group, slider, progress, label, kbd, badge, spinner,
  separator, skeleton (audit each against its shadcn demo page; rows 187/188 already did many demos -- check them
  first so nothing is redone).
- **Construction:** demos compose the real `Label` / `Field` parts (no hand-rolled label markup), matching the
  shadcn demo's text; a cheap check in the demo-parity spec that every control demo has an accessible name from a
  visible `label` (not only `aria-label`), allowlist with reasons.

### Item 11 -- Dark border/input tokens (row 203; owner D1)

- Two dark-arm token changes in `preview/assets/dx-components-theme.css` (`--dx-border`, `--dx-input`). Its own
  commit first in the batch so the snapshot attributes it alone (expected: dark-only border/input colours,
  `color-mix` values derived from `--dx-input` such as `--dx-field-fill`, rings embedding them). Re-run
  `dark-surface-contrast.spec.ts` and axe dark.

## 3. Lanes and order (CLAUDE.md: one batch, disjoint files; serialize only for a named reason)

**Batch 1 (concurrent):**

| Lane | Rows | Owns | Must not touch |
|---|---|---|---|
| T (theme) | 203, 205 | theme css, new `check-hover-media.sh` (+run-gates, CLAUDE.md), every sheet's `:hover` wrapping (mechanical), theme-picker toggle in `main.rs` | primitives |
| O (overlays) | 193, 210 | `primitives/src/top_layer.rs`, combobox/popover/tooltip/hover_card/select primitives, their specs, new combobox geometry spec | component sheets (T owns them this batch) |
| F (focus) | 206 | `primitives/src/dialog.rs`, `alert_dialog.rs` focus parts, focus specs | drawer.rs (E) |
| E (edge panels) | 207, 208 | `primitives/src/drawer.rs`, sheet + drawer components/sheets, their specs | dialog.rs focus code (F) -- coordinate through F's API |
| D (date) | 209 | `primitives/src/date_picker.rs`, date_picker/calendar components, date specs | top_layer.rs |
| C (charts) | 211 | chart component + sheet, chart specs | activity.rs (read-only use) |
| L (layout debug) | 204 | read-only, then the root-cause file + new layout-stability spec | -- |
| Demo | 212 | `variants/**` of the listed components | component.rs / style.css |

Named conflicts: T sweeps every `style.css` for `:hover`, so E/D/C/O sheet edits would collide -> T does the
`:hover` sweep LAST within batch 1 (after the others' sheet edits land) or the other lanes leave `:hover` lines to
T. Decide at dispatch; the simplest is T's sweep as a follow-up step after batch 1's sheet edits merge.

**Batch 2:** parity wave row 155 groups 2-9 (menu item parts in `menu_item.rs`; Toggle/ToggleGroup
variant/size/spacing; Dialog close button + Header/Footer (after F); Tabs line + vertical; Badge ghost/link +
`r#as`; sizes on Switch/Select/NativeSelect (+ Sheet side -> already in E); Calendar caption layouts + week
numbers (after D)).

**Proof per batch:** N0 -> N1 (T0-style theme commit alone) -> N2 (all lanes), each diff classified against the
lanes' intended-differences tables; full suite vs `pw-n0.fails`; `run-gates.sh` ALL GREEN; registry smoke;
release deploy, link posted with the main commit.

## 4. Outcome (batch 1, landed 2026-10-14)

Per item (details and spec counts in the backlog rows):

- Item 1, layout flicker (row 204): the reported oscillation was not reproduced at any iPad size or in a 320-1440 sweep;
  two real re-layouts fixed (Avatar SSR != hydrated; `content-visibility` on the multicol masonry cards) and a new
  `layout-stability.spec.ts` (red on main, green on the fix). Follow-ups: row 214 (masonry column assignment depends
  on parse progress; the likeliest remaining cause of the iPad flicker), row 215 (`/charts/` re-lays out at hydration).
- Item 2, hover and press feedback (rows 205, 163): all `:hover` under `(hover: hover) and (pointer: fine)`, new gate
  `check-hover-media.sh`, switchable press feedback (tokens, one zero-specificity rule, theme-picker switch),
  hover-intent mouse-only. Open: drift check between the press-feedback and touch-action lists.
- Item 3, modal initial focus (row 206): one open-time rule in `use_dialog_open_driver`, 35-test spec.
- Items 4 + 5, edge panels (rows 207, 208, 197, 155 group 7): one `EdgeSide` and theme recipe, `Auto` default,
  floating rounded side drawers, nested drawers, typed Sheet `side`.
- Item 6, date picker (row 209): `confirm` default with one staging hook, natural-language demo rewritten to shadcn's.
- Item 7, close on scroll (row 193) and item 8, Combobox never covers its input (row 210): one placement fix for all
  ten `use_anchor_position_fallback` callers, `--dx-available-height`, `use_close_on_scroll`; specs 59/59 and 26/26.
- Item 9, chart entry (row 211): a hydrated chart that was painted in view never re-animates.
- Item 10, small-control demos (row 212): eight demos labelled like shadcn's plus `demo-labels.spec`.
- Item 11, dark border/input (row 203): white 10% / 15% with the derived fills lightened.

**Two research corrections** (found while verifying against shadcn source, before building): shadcn's Natural
Language date picker DOES have a text input, and NO shadcn date demo has Apply/Cancel (the `confirm` default is the
owner's own design, not parity).

**Deviations from the plan as written:**
- Sheet panel padding is 0 (Nova), Drawer keeps 16px through `--dx-edge-padding` (the theme picker drawer and the chart
  code drawer rely on it).
- The Command palette keeps its input autofocus on tap (an explicit `autofocus` wins, as in shadcn), so the phone
  keyboard still pops for the palette by design.
- The focus construction changed from a page-level modality tracker to reading `:focus-visible` on the opener at open
  time (the tracker had an install-order dependency).

**Next:** batch 2 = the parity wave, row 155 groups 2-9 together with row 156 (Drawer snap points, non-modal mode);
then row 213 (Message Scroller parity, pending the owner's markdown decision), rows 214 and 215, and the infra row 216.
