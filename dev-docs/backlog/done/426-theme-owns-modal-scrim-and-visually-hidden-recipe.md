---
id: 426
title: "Theme owns the no-top-layer modal scrim and the visually-hidden recipe"
status: "DONE 2026-10-26. Owner approved 2026-10-25 (row 365 sweep, P1)."
size: "S"
area: components
updated: 2026-10-26
---

# Row 426: Theme owns the no-top-layer scrim and the visually-hidden recipe

## Item

The modal scrim and the "visible to assistive tech only" recipe are theme concerns, but five component sheets still paint the scrim for the no-top-layer arm (three of them with keyframe duplicates of the theme's fade pair), and the sr-only block is hand-written 12 times in two spellings. Row 145 moved the top-layer `::backdrop` into the theme; this moves the wrapper scrim and the sr-only block too. Construction: the five wrappers carry `data-dx-scrim` and one theme rule paints the scrim; the sr-only block is one `[data-dx-visually-hidden]` rule. After it lands, a sheet cannot paste either one, because the gate rejects it.

## Status / trigger

DONE 2026-10-26. Owner approved 2026-10-25 (row 365 sweep, P1).

## Size

S

## Detail

Source: [final sweep note, P1](../../research/final-sweep-2026-10-25.md).

- **Scrim, declared per component** (under `preview/src/components/`): `dialog/style.css:15-24`, `alert_dialog/style.css:~19-28` (`:23`), `command/style.css:52-60` (`:55`), `drawer/style.css:27-35` (`:30`), `sheet/style.css:26-34` (`:29`). Keyframes byte-identical to the theme's `dx-fade-in/-out` (`dx-components-theme.css:1720,1730`) at `dialog/style.css:43,57`, `alert_dialog/style.css:47,61`, `command/style.css:79,93`. `check-duplicate-keyframes.sh:44-51` allowlists exactly those three.
- **Visually hidden, hand-written 12 times:** `pagination/style.css:163`, `data_table/style.css:131`, `code_block/style.css:130`, `command/style.css:35`, `breadcrumb/style.css:81`, `sidebar/style.css:42,709`, `calendar/style.css:465`, `chart/style.css:417`, `message_scroller/style.css:172`, `assets/main.css:2295,2426`. Spellings: `clip: rect(0, 0, 0, 0)` and `clip-path: inset(50%)`.
- **Construction:** `"data-dx-scrim": ""` on each wrapper (the pattern `data-dx-root` already follows). One zero-specificity `:where([data-dx-scrim])` rule carries the base block, `:has(> dialog)`, `[data-dx-overlay="off"]` and the closed state through `dx-fade-out`. One `:where([data-dx-visually-hidden])` rule; components set the attribute.
- **Subsumes:** the three keyframe allowlist entries; 11 of the 12 sr-only blocks (the docs site's `.dx-charts-sr-only` in `main.css` can use the attribute too). **Does not subsume:** the `::backdrop` rule (already one) and sheet-specific panel animation.
- **Gate:** extend `check-theme-baseline.sh` as it already guards `::backdrop`: no component sheet may declare `background: var(--dx-overlay-scrim)` or the sr-only recipe, and both theme rules must exist. Browser half: the computed-style snapshot (`playwright/computed-style-states.ts`) before and after, the method row 145 used to prove no computed-style difference.
- **Check first:** that all five wrappers render `data-state` on the element that carries the class (true for alert dialog, `alert_dialog.rs:~140`; the others were not read to that depth).

## Outcome

- **Theme** (`dx-components-theme.css`, after "The modal scrim, once"): five zero-specificity rules on `:where([data-dx-scrim])` (base, `:has(> dialog)`, `[data-dx-overlay="off"]`, `[data-state="closed"]` and `"open"` through the shared `dx-fade-out/-in`) and one `:where([data-dx-visually-hidden])` rule (the `clip-path: inset(50%)` spelling, padding/border 0, margin -1px).
- **Scrim**: `dx-dialog-backdrop`, `dx-alert-dialog-backdrop`, `dx-sheet-root`, `dx-drawer-root` and `dx-command-dialog-backdrop` carry `"data-dx-scrim": ""` (`dialog`, `alert_dialog`, `sheet`, `drawer` `component.rs`; the Command wrapper is rendered by `primitives/src/command.rs`, so the attribute is there). Their sheets lost the wrapper block and the three `dx-*-animate-*`/`backdrop-*` keyframe pairs (the classes stay as hooks). All five wrappers render `data-state` on the element that carries the class (the primitive's `DialogRoot`/`AlertDialogRoot`/`Drawer` root), so the "check first" item holds.
- **Visually hidden**: 11 of the 12 pasted blocks are gone (pagination, data_table, code_block, command, breadcrumb, sidebar x1, calendar, chart, message_scroller, the docs site's `.dx-charts-sr-only`); each element sets `data-dx-visually-hidden` (components, the three sidebar demos, the Command demos, `charts_gallery.rs`, and the chart primitive's `chart-data-wrapper`). `.dx-calendar-sr-only` stays as a class too because three specs select it. Two rules keep the recipe's shape and sit on `SR_ONLY_ALLOW` in the gate with a reason, because the element is picked by a selector, not rendered by the component: the sidebar's collapsed-rail label hiding (`> :not(:first-child)` of a caller's button) and the docs navbar's brand text inside a media query. Demo-only inline copies (`style:` strings in `dialog` share, `virtual_grid` selectable, `site/block_demo.rs`, the `virtual_list` reorder demo's `<style>`) are not scanned and were left.
- **Gate**: `check-theme-baseline.sh` arm 6 (and 6b): no component sheet or `main.css` rule reads `var(--dx-overlay-scrim)` (outside `::backdrop`, which arm 2 owns) or has the sr-only shape (`position: absolute` + `width: 1px` + `height: 1px` + `clip`/`clip-path`); the six theme rules must exist with their declarations; the five wrapper sources must emit `data-dx-scrim`. Mutations checked by hand: a pasted sr-only rule, a pasted scrim rule, the theme's sr-only rule renamed, the attribute removed from Sheet each turn it red. `check-duplicate-keyframes.sh`'s allowlist is empty now (its three entries were the three scrim keyframe pairs).
- **Computed styles, before and after** (same probe on two `lane-ssg` builds, `queue/batch-27` `b26f7341` and this branch; per component page: the wrapper of the open modal on the web arm, the same wrapper with `overlay: false`, and a shallow clone without its `<dialog>` (the no-top-layer arm) in open, closed, open+off and closed+off, under `prefers-reduced-motion: reduce` too; 19 scrim properties each; plus every sr-only element on 14 pages, 20 properties and its box): **scrim 0 differences** (animation name aside: `dx-dialog-backdrop-animate-in/out`, `dx-alert-animate-*`, `dx-command-dialog-backdrop-in/out` are now `dx-fade-in/out`, the byte-identical keyframes); **sr-only: 17 of 85 elements differ**, exactly those that had the `clip: rect(0, 0, 0, 0)` spelling (data table 7, breadcrumb 4, calendar 6): `clip` `rect(0,0,0,0)` -> `auto`, `clip-path` `none` -> `inset(50%)`, margin 0 -> -1px. Every one is a 1x1 clipped box either way (width/height identical, nothing painted); this is what unifying the two spellings costs.
- **Latent bug found, not changed**: the per-sheet `@media (prefers-reduced-motion: reduce) { .dx-sheet-root { animation-duration: ... } }` never applied (it loses to `.dx-sheet-root[data-state="open"]`, higher specificity), so the wrapper scrim's reduced-motion duration was 0.2s in the baseline too. The theme rule does not re-create the dead block, so reduced motion stays as it was. A `@media (prefers-reduced-motion: reduce) { :where([data-dx-scrim][data-state]) { animation-duration: var(--dx-motion-duration-reduced) } }` after the state rules would fix it; that is a behaviour change, so it is a decision, not part of this row.
