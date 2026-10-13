# Row 111 phase C -- implementation plan (2026-10-13)

Scoping only: a static read at `main` `26b7a01` (row 145 landed), plus the **dirty tree another agent was editing while this was written** (see 0). Nothing was built or run. Every Nova value below that is not quoted from a repo comment is recalled from `shadcn-ui/ui@295a1f1` `style-nova.css`, and each lane must re-read its own `.cn-<component>` blocks at that SHA and cite them in the sheet (no copy is checked out here; fetch through the proxy). Contrast ratios in 2 and 3 were computed here from the hex values.

## 0. Preconditions and the in-flight tree

The working tree is not clean: `popover/*`, `primitives/src/popover.rs`, `primitives/src/date_picker.rs`, `top_layer/component.rs`, `tabs/*` (new `TabsSize`, `variants/sizes/`), and `button`, `combobox`, `input`, `input_group`, `select` `style.css` (they now read a new `--dx-control-height-xs/sm/default/lg` ladder, 24/28/32/36px, owner decision 2026-10-13), plus `dx-components-theme.css` (the ladder tokens). Rules for this plan:

- Phase C does **not** start until that work is committed: the `before` snapshot cannot be recovered afterwards (CLAUDE.md), and the lanes below would collide on `tabs`, `combobox`, `button`, `dx-components-theme.css`.
- The ladder is decided: lanes read `--dx-control-height-*` for every sized control and never re-type 1.5/1.75/2/2.25rem.
- `popover`, `select`, `input`, `input_group`, `button`, `badge`, `card`, `dialog`, `top_layer` are **held out** of the batch (Nova already, or owned by the in-flight agent). Only their phase-C residue is assigned, after the commit (tabs -> lane C, combobox -> lane F).

## 1. Inventory at the current tree

Criterion: a sheet is "on Nova" when it cites `style-nova.css` / `.cn-*` or its values were matched in `99f00b1`/`0e6d3ac`/row 145; "by value" means the numbers match Nova though no comment cites it (to verify, not to redo).

### 1a. Already on Nova (no phase-C work except the noted residue)

| Component | Evidence | Residue |
|---|---|---|
| button, badge, card, input, dialog, select, tabs | phase B `99f00b1` (8/4/2/2/5/3/1 Nova comments) | tabs: `.dx-tabs-content { padding: var(--dx-space-8) }` (row 132, Nova has none); `.dx-tabs-content-themed` border + card fill; `line`/vertical = parity group 5. Button: the destructive form stays solid (row 134) |
| popover | `style.css:1` "Nova popover", header/title/description, trigger = outline Button | in-flight agent; nothing for C |
| checkbox | `.cn-checkbox` 1rem, focus recipe | none |
| input_group | focus/fill match Input; **invalid and disabled landed with row 145** (`input_group/style.css:42-53`) | row 133's CSS is done; its demo states and the computed-style assertion are not (`playwright/input_group.spec.ts` has no invalid/disabled case). Close the row after adding them |
| switch, toggle | Nova invalid, disabled, pressed (toggle derives pressed from accent + 15% foreground) | toggle height is padding-driven, not the ladder (row 155 group 3) |
| native_select | "a field, not a button": Nova field fill | `option` background `--dx-popover` (palette) |
| hover_card, context_menu, dropdown_menu, menubar, command, alert_dialog | one Nova comment each (`p-2.5`, `pr-8` indicator, `fade-in-0 zoom-in-95`) | only the floating recipe and the indicator slot are Nova; items, triggers, panels are not (1b) |
| message_scroller | built from `message-scroller.tsx` + `.cn-message-scroller-content` | none |
| label | shadcn defaults (14px/500) via slot variables (row 170) | none |
| kbd, table, pagination | match by value (kbd h-5/min-w-5, table head h-10/cell p-2, pagination 2rem) | verify only |

### 1b. Remaining: what "migrate" means (the phase-C scope)

"Migrate" = replace the sheet's own sizes, radii, paddings, fills and edges with Nova's recipe through `--dx-` tokens and the ladder, cite `.cn-*`, delete literals the gates would flag, and remove the sheet's own state code that the row-145 recipes now cover. Current values are read from the sheets; "Nova" is the recalled target (verify).

| Lane | Component | Current (sheet) | Nova target to verify | Notes |
|---|---|---|---|---|
| M | dropdown_menu, context_menu, menubar | item row `--dx-menu-item-padding: space-2 space-3` (8x12px), radius `calc(lg - sm)` (4px), gap 8px; dropdown trigger `padding: space-2 18px` literal + card fill + inset ring; menubar bar `padding 4px`, `--dx-background` fill, inset ring | item `px-1.5 py-1` (6x4px), `rounded-md` (8px), gap 1.5; menubar `h-8 rounded-lg p-[3px]`, trigger `rounded-sm px-1.5 py-0.5`; trigger = outline Button composition | the item row is **theme-owned** (`dx-components-theme.css` ~`:545`, `:958`, `:1006`): token change lands in T0, sheets only drop leftovers. Rows go ~8px shorter |
| M | navbar, navigation_menu, toolbar | triggers `padding space-2 space-3`, radius `lg - sm`; toolbar buttons same; navigation_menu `min-width 280px`, grids 25-37.5rem | NavigationMenu trigger `h-9 rounded-lg px-2.5 py-1.5`; viewport/links `rounded-lg p-2` | navbar and toolbar are not shadcn components: align to menubar/Toggle values |
| M | tooltip | `font-size: sm`, padding `space-2 space-3` (slot), inverted tone | `rounded-md px-3 py-1.5 text-xs`, `bg-foreground text-background` | font-size is the only likely delta |
| P | alert_dialog | **not Nova** (audit): `border 1px --dx-border`, literal `0 2px 10px rgb(0 0 0/18%)`, radius lg, padding 32/24px, title 20px/700, buttons `space-2 18px` literal, action = solid `--dx-destructive` (not Button's `--dx-destructive-fill`, so it differs from the destructive Button in dark), panel fill `--dx-background` | `rounded-xl p-4 ring-1 ring-foreground/10 bg-popover`-class panel (dialog's recipe), title `text-base font-medium`, footer actions as `Button`s | the largest layout move of the phase; `alert-dialog.spec.ts` + snapshot |
| P | sheet, drawer | panel `--dx-background`, `--dx-shadow-lg/xl`, close button 24px/0.7 opacity (sheet), `padding: space-2 space-4` bordered close (drawer), title 16px/600 vs 18px/600 | `gap-4 p-4 text-sm`, title `text-base font-medium`, close = ghost icon `Button`, ring/shadow per `.cn-sheet-content` | pick **one** modal panel token with Dialog (audit row 2: popover vs background vs card today); under C that is #171717 vs #0a0a0a, an owner-visible choice, defaulting to Dialog's `--dx-popover` |
| P | command | dialog `--dx-card` fill, `border 1px`, `--dx-shadow-2xl`; input 16px with `space-3` padding; item 8x12px | `rounded-xl`, ring not border, input group `h-8`, item `px-2 py-1.5`-class | item row shares the menu token if it is the same `.cn-` rule (verify) |
| F | textarea | `--dx-card` fill by `data-style`, padding `space-2 space-3` (8x12), 16px font | `min-h-16 rounded-lg px-2.5 py-2 bg-transparent dark:bg-input/30`, `text-base md:text-sm` | fill is the row-5 "three fills in one form row" class: one control fill (Input's: transparent / `input 30%`) |
| F | combobox | input `--dx-card` fill, 36px (= ladder `lg`, owner-decided), option 6x8px | field fill like Input; list item `rounded-md py-1 pl-1.5` | sheet dirty today (ladder): after the commit |
| F | native_select, input_otp | native fill ok; OTP slot `2.5x2.75rem`, literal `#FFFFFF26` dark fill (ignores presets), 16px text | OTP slot `size-8`-class group `rounded-lg`, separators, `dark:bg-input/30` | literal hex becomes the Input fill token |
| F | date_picker, color_picker, button_group, form, field | date group `--dx-card` fill + `space-2` padding; color_picker field chrome + literal black thumb shadows; button_group-text `--dx-card`; form literals `0.9rem`, `0.35rem`, `0.85rem`, radius-sm; field `radius-md` card | field fill and `h-8`-class heights from the ladder; form/field: drop literals | form (7) and field (4) carry row 170 baseline sites: use slot variables |
| K | radio_group, slider, progress, resizable, separator, skeleton, spinner, kbd, label, scroll_area, aspect_ratio | slider track **8px**, thumb 16px, inset ring; progress **8px**; radio card fill; resizable grip border | slider track `h-1` (4px), thumb `size-3` (12px) `border-ring bg-white`; progress `h-1`; radio `size-4 border-input dark:bg-input/30` | slider and progress halve in height: visible, intended |
| K | toggle_group | items `min-width 35px`, `padding 10px` literals, `border-radius 0`, joined | items = Toggle (`h-8 min-w-8 rounded-lg px-2.5`) | spacing/variant/size are parity group 3 (same sheet): take it **after** this lane, not alongside |
| C | accordion, collapsible | trigger `padding-block: space-4` (16px), 300 ms chevron | trigger `rounded-lg py-2.5 text-sm font-medium`, content `pt-0 pb-2.5` | chevron size/rotate is audit row 12 |
| C | alert | padding 16px, inset ring in `--dx-input`, card fill, destructive = ring + **text `--dx-destructive`** | `rounded-lg border px-2.5 py-2 text-sm`; destructive text on `--dx-destructive-text` | row 134 site; AlertAction is parity group 9 |
| C | avatar, breadcrumb, empty, item, pagination, table, data_table, tag_group | avatar default **64px** (sm 2, md 3, lg 4rem); breadcrumb gap 4px; item padding 16px; empty padding 48x24px | avatar `size-8` default / `sm size-6` / `lg size-10`; breadcrumb `gap-1.5`; item `px-3 py-2.5`-class; empty `p-6` | **API sizes differ** (sm/md/lg vs sm/default/lg): owner question 6 |
| C | toast | literal `0 4px 12px rgb(0 0 0/15%)` (not dark-aware), height 4rem, padding 12x16 | Sonner-based in shadcn; `shadow-lg`, `rounded-lg`, tokens | rebuild is row 138, not here |
| C | tabs (after commit) | row 132 padding | no panel padding | drop the default, give the two non-Card consumers their own, or document; measure a Card-in-tab first |
| Cal | calendar | literal `0 2px...` shadow (token-fallback), nav 1.75rem, `font-weight 300` day headers, cell 2rem, own border + background | `--cell-size` 8 spacing, `p-2`, no own border inside a popover | caption layout/week numbers are parity group 8 |
| X | attachment, bubble, marker, message | chat kit (new, Nova-era); `color-mix(--dx-destructive ..)` for text-ish inks (bubble `:16-20`, attachment `:7-11`) | audit only | destructive inks to review under the new token |
| X | chart family (chart, chart_tooltip, 6 chart dirs), carousel, drag_and_drop_list, virtual_list | chart 28 shadcn notes (geometry parity); carousel Nova-matched earlier; dnd literal `10px` radius, hard-coded blue | audit only; palette stays multi-hue (owner 2026-10-11) | `chart_*_parity.rs` + chart specs must stay green; `.dx-chart-interactive-toggle` label gets fixed by the palette (3) |
| S | sidebar | 956 lines, own `--dx-sidebar-*` roles, 65 row-170 baseline sites, 2 literal shadows | sidebar tokens only (`.cn-sidebar`) | dark sidebar bg = page under C: owner question 1 |

Closed by earlier work (do not redo): ramp references (`var(--primary-color*)` / `--secondary-color*` outside the theme: 0 real uses, only 3 comments); font stacks; touch-action; scrims; half-step tokens; focus/invalid/disabled recipes.
Leftover from item (c): `test-harness/assets/dx-components-theme.css` is an unrelated 83-line old file, not referenced by `test-harness/src/main.rs` (it loads `main.css`); delete it (lane T0).

## 2. The palette change (option C) and row 134

### 2a. Token edits (all in `preview/assets/dx-components-theme.css`, lane T0)

Change the **role fallbacks**, not the ramp: `--primary-color*` stay (installers' aliases, and `--dx-primary-foreground`, `--contrast-error-color`, `--primary-info-color` read `--primary-color`).

| Role token (`:167-189`) | Now dark (ramp step) | Option C dark | Light | Surfaces that paint it (sheets) |
|---|---|---|---|---|
| `--dx-background` | `#000` (`--primary-color`) | **`#0a0a0a`** (shadcn `oklch(0.145)`) | unchanged | page, main.css, alert_dialog / sheet / drawer panels, menubar, calendar, slider thumb, switch, tabs, bubble, message_scroller, avatar fallback, sidebar mobile sheet |
| `--dx-card` | `#141313` (`-3`) | **`#171717`** (`oklch(0.205)`) | unchanged | alert, attachment, bubble, button_group, card, carousel, chart, combobox, command, date_picker, dnd list, dropdown trigger, field, form, pagination, radio, switch, tabs, textarea, alert_dialog cancel |
| `--dx-popover` | `#262626` (`-5`) | **`#171717`** | unchanged | all 12 floating surfaces (`--dx-floating-background`), dialog, native_select options, navigation_menu viewport |
| `--dx-accent` | `#3e3e3e` (`-7`) | **`#262626`** (`-5`) | unchanged | hover/highlight rows of dropdown, context, menubar, navbar, navigation_menu, select, combobox, command, native_select, calendar cells, pagination, toggle, toggle_group, toolbar, chart toggle, alert_dialog cancel |

Verification that this is exactly shadcn: the `neutral` preset in `theme-presets.css:66-98` already sets background `oklch(0.145)`, card/popover `oklch(0.205)`, accent/muted/secondary `oklch(0.269)`. So the `before` dark capture with `data-theme-base=neutral` is a free cross-check of the new default's fills (ring, border, input differ: we keep blue ring, `#232323` border, `#3e3e3e` input).

Computed ratios (dark; text on surface), old surfaces -> new:

| Pair | page `#000`->`#0a0a0a` | card `#141313`->`#171717` | popover `#262626`->`#171717` | accent `#3e3e3e`->`#262626` |
|---|---|---|---|---|
| muted-foreground `#a1a1a1` | 8.13 -> 7.66 | 7.18 -> 6.94 | 5.86 -> **6.94** | **4.14 -> 5.86** |
| foreground `#d4d4d4` | 14.2 -> 13.4 | 12.5 -> 12.1 | 10.2 -> 12.1 | 7.2 -> 10.2 |
| docs link `#4d94ff` (main.css:34) | 7.0 -> 6.6 | 6.18 -> 5.97 | 5.04 -> 5.97 | 3.56 -> 5.04 |
| destructive as text `#a22e2e` | 2.97 -> 2.80 | 2.62 -> **2.53** | 2.14 -> 2.53 | 1.51 -> 2.14 |

Things that this change creates or leaves:

1. **State ties.** Dark `muted`, `secondary` and `accent` become the same `#262626` (they already are in shadcn). Any element whose rest/active fill is `--dx-muted` and whose hover/highlight is `--dx-accent` loses its hover change: calendar nav buttons (dark rest `--dx-muted`, `calendar/style.css:52`) and today / in-range cells (`:251-305` muted vs accent hover), pagination (`:66-70`), navigation_menu's `color-mix(accent 60%, popover)` (`:352`), command highlighted vs selected, tabs. The toggle already derives pressed from accent + 15% foreground (distinct by construction); the lane that owns each sheet must list its muted/accent pairs and apply the same move (derive the second state from the first plus a fixed step) or accept the tie as Nova does. T0 greps the whole tree for the pairs and hands each lane its list.
2. **Edges get fainter**: `--dx-border` `#232323` on the new card is 1.14:1 (1.18 now); shadcn's `white/10%` would be about 1.32:1. Not part of C; owner question 2.
3. **`--dx-input` fills** (`color-mix(input 30%, transparent)`) are unchanged in formula; they land at roughly `#1a1a1a` on the new page, the same as Nova's `input/30`.
4. The ramp comment blocks that quote old hex values (`chart/style.css:175`, `calendar/style.css:288`, `main.css:24-33`) are stale text: update them in the owning lane.

### 2b. `--dx-destructive-text` (row 134), defined once in the theme

Fill-grade `--dx-destructive` (dark `#a22e2e`) is 2.5-2.8:1 as text on every dark surface. Construction (same as `--dx-primary-ink`, `:208`): clamp the role's OKLCH lightness, hue and chroma kept.

```css
--dx-destructive-text: var(--light, oklch(from var(--dx-destructive) min(l, 0.55) c h))
                       var(--dark,  oklch(from var(--dx-destructive) max(l, 0.70) c h));
```

Candidate values, to be measured not trusted: shadcn's own dark destructive (`oklch(0.704 0.191 22.216)`, about `#ff6467`) is 6.85 / 6.21 / 5.24:1 on page / card+popover / accent; light `#dc2626` is 4.83:1 on white but **4.43:1 on `#f5f5f5`** (muted), so light needs the 0.55 clamp. Under any preset the clamp is a no-op in dark (preset dark `--destructive` is already that salmon) and the same in light, so `theme-preset-contrast` stays the guard. `--dx-destructive-fill` (button/badge) is untouched; toast error and alert_dialog action keep white-ish ink on the fill (5.16:1), but alert_dialog should read `--dx-destructive-fill` like the Button.

Sites that paint destructive as **text** (to move onto the token):

| Site | Today | Owner of the edit |
|---|---|---|
| `alert/style.css:56,60,70,74` (title, description, icon under `data-style="destructive"`) | `color: var(--dx-destructive)` | lane C |
| `field/style.css:104` (`--dx-label-color` slot) and `:138` (`.dx-field-error`) | `var(--dx-destructive)` | lane F |
| `dashboard/views/email_client/email_client.css:881` | `color: var(--dx-destructive)` | lane T0 (single line) |
| `input/variants/required/mod.rs:14` (inline `color: var(--dx-destructive);`) | demo | lane T0 (single line; `input/style.css` is dirty but this file is not) |
| `bubble/style.css:16,20`, `attachment/style.css:7,11` (`color-mix(destructive 80%/45%, foreground)`) | text-ish inks, mixed with the foreground | lane X reviews: measure; probably fine |
| `button/style.css:243`, `alert_dialog:210`, rings and `border`/`background` uses | not text | no change |

Gate (by construction, lane T0): `scripts/check-destructive-text.sh` fails on `color:` / `--dx-*-color:` / inline `color:` whose value is `var(--dx-destructive)` (or a `color-mix(` that leads with it) in `preview/src/**` and `primitives/src/**`; `border`, `background`, `fill`, `stroke`, `box-shadow` stay legal; reasoned allowlist, stale entries fail. Add it to `scripts/run-gates.sh` and CLAUDE.md's gate sentence (main loop edits both).

## 3. The axe contrast pass

What exists:

| Tool | Covers | Gap for phase C |
|---|---|---|
| `playwright/all-routes.spec.ts` | axe (wcag2a/aa/21a/21aa + best-practice) on every `CATALOG` route + `/`, `/docs`, `/demos`, `/charts/`, light and dark, **closed state**, motion off | closed state only; `KNOWN_AXE` has 3 entries that this phase must delete (the spec fails when a listed finding stops reproducing): `DESTRUCTIVE_AS_TEXT` (alert dark, site-home dark), `FIELD_ERROR_TEXT` (site-home dark), `CHART_TOGGLE_LABEL` (bar_chart, line_chart dark; 4.13:1 today, 5.86:1 under option C, so the palette alone clears it) |
| per-component specs (69 files call `expectNoAxeViolations`, `playwright/axe.ts`) | the two-state convention: closed and open (menus, dialogs, select, combobox, calendar popup) | the only place open overlays are scanned; must be re-run on the merged tree, light and dark |
| `playwright/theme-preset-contrast.spec.ts` | computed text vs composited background >= 4.5:1, every base (9) x accent (17) x light/dark, for Badge and Button variants (rest, hover), link Button, Toggle/ToggleGroup, docs accent ink; pressed toggle distinct | covers none of the overlays, menus, alert, field; "none" axis (the library's own look) is where option C lands |
| grandfathered `color-contrast` exclusions in `preview.spec.ts`, `drag_and_drop_list.spec.ts`, `tag_group.spec.ts` | n/a | re-run with them removed once, record which still need to stay |

What the pass must cover (the previous round never ran the colour-contrast half, row 111 (b)):

1. Baseline now (before T0): `all-routes` (light+dark) and `theme-preset-contrast` on the committed tip, to know the red set before anything moves; save the failing list.
2. After T0 and the lanes: `all-routes` green with the three `KNOWN_AXE` entries deleted; every `<component>.spec.ts` axe scan (closed + open) in light and dark; `theme-preset-contrast` green.
3. **Hover and highlight states are not seen by axe** (it reads the settled state). Option C's whole point is the highlighted row, so add one spec (lane-less, main loop or T0 adds `playwright/dark-surface-contrast.spec.ts`, modelled on `theme-preset-contrast`'s canvas compositing): open Dropdown, Select, Combobox, Command, NavigationMenu, Calendar and Pagination hover; measure item text, shortcut text and muted text on the highlighted row in both modes; assert >= 4.5:1 and that the highlight differs from its menu by at least the existing `DISTINCT_HOVER` step (this is the "1.00:1 invisible highlight" failure option C exists to avoid).
4. The known suspects, each with a measurement before and after in the row 134/111 addendum: destructive text (alert, field, dashboard, demo), dark `--dx-primary` `#e5e5e5` as ink (item (d); 14-16:1 on every dark surface, so low risk), the docs link `#4d94ff` (above), muted-foreground on accent (5.86), chart toggle label, toast error and alert_dialog action ink (5.16 on the dark fill), and destructive Button/Badge (already swept by the preset spec).
5. Exit: zero axe violations on all routes, both modes, `KNOWN_AXE` empty (or each remaining entry with a row), preset sweep green; an exception list with reasons otherwise.

## 4. Proof plan (computed-style snapshot, format 3, with focus / hover / open)

Tool: `playwright/computed-style-snapshot.spec.ts` + `computed-style-states.ts` (capture with `DX_SNAPSHOT=1 CSS_SNAPSHOT_OUT=... -g capture`, compare with `CSS_SNAPSHOT_A/_B ... -g compare`; both schemes; `<OUT>.coverage.json` says what the hover/open passes actually reached). A functional snapshot may run on a debug SSG build; build only through `scripts/build-ssg.sh` into an isolated absolute `CARGO_TARGET_DIR` (`scripts/lane-target.sh create <name>`, `CARGO_INCREMENTAL=0`).

| Capture | Tree | Purpose |
|---|---|---|
| `S0` + `S0'` | committed tip after the in-flight work lands (0) | the `before`; A/A byte-identical or the harness is not trustworthy (diff `S0` vs `S0'` must be empty) |
| `S1` | tip + T0 only (own commit, own build) | palette attribution: S0 -> S1 is the option C diff alone, plus `--dx-destructive-text` consumers (none yet) and the menu-item density |
| `S2` | merged final tree | S1 -> S2 is the sum of the lane diffs; each difference is bucketed by selector class (`dx-<component>`) to a lane, so no lane's diff is polluted by the palette |

Classification: `compare` names mode/page/element/property; a small script (main loop, scratchpad) buckets `<after>.diff.txt` by `dx-` class -> lane and by property (`background-color`, `height`, `padding-*`, `border-radius`, `box-shadow`, `color`). Chrome routes (`/`, `/docs`, `/demos`, `/charts/`, dashboard) aggregate every lane: classify them by class too. Anything unbucketed is a failure to explain, not a footnote. Light mode must show **only** the listed deltas.

Intended differences (the lists lanes are held to; each entry old -> new):

| Lane | Intended differences |
|---|---|
| T0 (S0 -> S1) | dark only: `background-color` `#000 -> #0a0a0a` (page and background-role panels), `#141313 -> #171717` (card role), `#262626 -> #171717` (popover role, all floating panels), `#3e3e3e -> #262626` (accent role: every hover/highlight/active) plus the `color-mix` values derived from them (navigation_menu, native_select option, input fills); `box-shadow` rings that embed those; `color`/`background` of the chart toggle label pair. Menu item row (both modes): padding `8x12 -> 6x4`, radius `4 -> 8`, gap `8 -> 6` on dropdown, context menu, menubar items (and their heights, panel heights, submenu offsets). No light colour change. |
| M | menubar bar `height/padding/background` and trigger radius, dropdown trigger 18px literal -> Button, navigation_menu/navbar/toolbar trigger padding and radius, tooltip font-size; panel heights follow the item row |
| P | alert_dialog: padding `32/24 -> 16`, radius, title `20px/700 -> 16px/500`, border + literal shadow -> ring, buttons -> Button geometry, action fill `--dx-destructive -> --dx-destructive-fill`; sheet/drawer/command: panel background token (the one owner-visible choice), shadow/ring, close button geometry, title size, item padding |
| F | textarea / combobox / date_picker / button_group / OTP / native_select fills (card or literal -> the Input fill), padding, heights from the ladder, OTP slot geometry, form/field literals gone, field error text colour |
| K | slider track `8 -> 4px`, thumb `16 -> 12px`, progress `8 -> 4px`, radio/checkbox fills, toggle_group item geometry (Toggle's), resizable grip |
| C | accordion trigger padding, alert padding + destructive text colour, avatar sizes (if the API question is approved), breadcrumb gap, item/empty padding, pagination, table, toast shadow, tabs-content padding (row 132) |
| Cal | calendar cell/nav sizes, border/shadow inside a popover |
| X | none expected beyond the palette; any other difference is a finding |
| S | sidebar roles (if question 1 is yes): background `#0a0a0a -> #171717`; literal shadows -> tokens |

Beyond the snapshot: the full Playwright suite on a release no-base-path SSG build diffed against the pre-phase baseline (the F+S debug run: 2695 passed / 6 failed, all in the known baseline set) -- specs that assert the old values update in the owning lane (the only literals found: `all-routes.spec.ts:150`, `drawer.spec.ts:192,587`); `top-layer.spec.ts` and menu specs that measure item heights will move with the density change. `scripts/run-gates.sh` ALL GREEN with exit 0, its tree line quoted; `scripts/registry-install-smoke.sh` (required: `component.json`-adjacent `style.css` change on dozens of components); chart parity tests green (they compare geometry, not colour); the debug build says nothing about timing specs.

## 5. Lane split (ONE batch, disjoint file sets) and what is serial

Serial, and why (CLAUDE.md: only for a named reason):

| Step | What | Reason |
|---|---|---|
| 0 | wait for the in-flight commit; run `run-gates.sh` and the contrast baselines; capture `S0`, `S0'`; lane targets created | the `before` cannot be recovered; `tabs`, `combobox`, `button`, theme file collide with the in-flight edits |
| 1 | **one batch** of T0 + 8 lanes. T0 commits first | T0's diff is the attribution baseline for everyone; the lanes edit concurrently against the token **names** given below, but a lane's snapshot/axe step waits for T0's commit |
| 2 | main loop: merge, `S1`/`S2`, classification, axe pass, gates, install smoke, backlog | one dev server per port / one cargo lock: edits are parallel, builds and tests are serial |

Main-loop-only files (no lane touches them): `CLAUDE.md`, `scripts/run-gates.sh`, `scripts/check-cross-component-overrides.baseline.tsv` (shrink with `--update-baseline` only), `scripts/check-*.sh` allowlists, `playwright/all-routes.spec.ts` (delete the three `KNOWN_AXE` entries when the fixes land), `dev-docs/backlog.md`, `preview/src/components/mod.rs`, `preview/build.rs`.

Token contract given to every lane at dispatch: `--dx-destructive-text`, `--dx-menu-item-*` (new values), the four dark roles, `--dx-control-height-*`, and "read Input's fill, do not re-type it".

| Lane | Owns (edit) | Must not touch | Specs it updates |
|---|---|---|---|
| **T0** theme | `preview/assets/dx-components-theme.css`; `preview/assets/dx-utilities.css` (regenerate; `generate-dx-utilities.js --check`); `scripts/check-destructive-text.sh` (new); delete `test-harness/assets/dx-components-theme.css`; `email_client.css` and `input/variants/required/mod.rs` (one line each); new `playwright/dark-surface-contrast.spec.ts` | any component `style.css` | `theme-preset-contrast.spec.ts` if the default look needs new probes |
| **M** menus | `components/{dropdown_menu,context_menu,menubar,navbar,navigation_menu,toolbar,tooltip,hover_card}/` | `primitives/src/menu_item.rs` and the same dirs' parity group 2 work (run it after M, not alongside) | `dropdown-menu`, `context-menu`, `menubar`, `navbar`, `navigation_menu`, `toolbar`, `tooltip`, `hover-card`, `menu-indicator-gap` |
| **P** panels | `components/{alert_dialog,sheet,drawer,command}/` | `dialog/` (Nova; parity group 4 owns its close button) | `alert-dialog`, `sheet`, `drawer`, `command`, `modal-exit` assertions that quote old values |
| **F** fields | `components/{textarea,combobox,native_select,input_otp,date_picker,color_picker,button_group,form,field}/` | `input/`, `input_group/`, `select/`, `popover/` | `textarea`, `combobox`, `native_select`, `input_otp`, `date-picker`, `color-picker`, `button_group`, `field`, `focus-ring`, `input_group` (adds the row 133 invalid/disabled cases) |
| **K** ranges | `components/{radio_group,slider,progress,switch,checkbox,toggle,toggle_group,resizable,separator,skeleton,spinner,kbd,label,scroll_area,aspect_ratio}/` | parity groups 3 and 7 (Toggle/Switch props) | `radio-group`, `slider`, `switch`, `checkbox`, `toggle`, `toggle_group`, `resizable`, `kbd`, `spinner` |
| **C** content | `components/{accordion,collapsible,alert,avatar,breadcrumb,empty,item,pagination,table,data_table,tag_group,toast,tabs}/` (tabs after the in-flight commit) | parity groups 5, 6, 9 | `accordion`, `alert`, `avatar`, `breadcrumb`, `empty`, `pagination`, `table`, `data_table`, `tag_group`, `toast`, `tabs` |
| **Cal** | `components/calendar/` | `date_picker/` (lane F) | `calendar` |
| **X** audit | `components/{attachment,bubble,marker,message,message_scroller,chart,chart_tooltip,area_chart,bar_chart,line_chart,pie_chart,radar_chart,radial_chart,carousel,drag_and_drop_list,virtual_list}/` | `preview/src/*_parity.rs` fixtures | chart/carousel/dnd specs only if a value they quote moved |
| **S** sidebar | `components/sidebar/` | row 170's other sheets | `sidebar` |

Row 170 interaction: lanes add slot variables where they touch a baseline sheet (form, field, command, input_group, sidebar, toolbar, chart); the baseline only shrinks, and the main loop runs `--update-baseline`. The burn-down itself is a follow-up, not part of this batch.
After this batch: parity groups 2, 3, 4, 5, 6, 7, 8, 9 land per sheet, one lane each, on the now-Nova sheets.

## 6. Open owner questions

| # | Question | Default if unanswered | Recommendation |
|---|---|---|---|
| 1 | Dark **sidebar** background: it reads the ramp (`--primary-color-2` = `#0a0a0a`) and will equal the new page. shadcn's sidebar is `#171717` (`oklch(0.205)`), accent `#262626`. Take it with option C? | unchanged (sidebar blends into the page) | yes, same pairing logic as C |
| 2 | Dark **border / input**: keep `#232323` / `#3e3e3e` (edge 1.14:1 on the new card, 1.18 today) or move to shadcn's `white 10% / 15%` (about 1.32:1)? Changes every hairline, table row, separator, input edge. | unchanged (not part of option C) | measure in S1 first, then ask with the picture |
| 3 | **Avatar** (and Item, Empty) sizes: ours are sm 2 / md 3 / lg 4rem with a 64px default; shadcn is 24 / 32 / 40. Keep our API and re-map values, or follow shadcn's names? | keep API, Nova values for sm/default/lg | follow shadcn (default 32); it is a visible shrink |
| 4 | Row 132: drop `.dx-tabs-content`'s default 2rem padding (Nova has none) and give the two non-Card consumers their own? | drop it after measuring a nested Card | yes |
| 5 | Now that `--dx-destructive-text` exists, is Nova's tinted destructive Button / Badge wanted (row 134: "if a tinted style is wanted")? | no, solid fill stays | no |
| 6 | One modal panel fill for Dialog, AlertDialog, Sheet, Drawer, Command: `--dx-popover` (Dialog today; #171717 in dark) or `--dx-background` (#0a0a0a)? | Dialog's `--dx-popover` | popover |
| 7 | `--dx-primary-foreground` dark `#000` -> `#171717` (shadcn)? Only affects ink on the light `#e5e5e5` primary fill. | unchanged | low value; leave |

Nothing else blocks: chart palette and accent dark are decided (2026-10-11); toggle-group spacing and calendar caption are decided (2026-10-10); the control-height ladder is decided (2026-10-13).

## 7. Size estimate

| Lane | Size | Main risk |
|---|---|---|
| T0 theme + gate + new contrast spec | S-M | state ties (2a.1) and the menu-item density moving every menu spec; 15k-class snapshot diff, all dark |
| M menus | M | menu/top-layer specs that measure heights; parity group 2 shares the dirs |
| P panels | M | AlertDialog re-skin moves layout; panel-fill choice |
| F fields | M | three fills collapse into one; field/form carry row 170 sites; combobox dirty until commit |
| K ranges | S-M | slider/progress halving; toggle_group vs parity group 3 |
| C content | M | most components; avatar API question; tabs after commit |
| Cal | S | cell sizes inside the date picker popup |
| X audit | S-M | chart parity tests must stay green; mostly "no difference" proofs |
| S sidebar | S-M | 956 lines, 65 baseline sites; demos in iframes (not captured by the open/hover passes) |
| Main loop: captures, classification, axe, gates, smoke, backlog | M | four snapshot captures plus one release suite run; the hover/open passes do not reach iframe state |
| **Total** | **L** (as the row says) | ~9 lanes in one batch, then one serial verify |
