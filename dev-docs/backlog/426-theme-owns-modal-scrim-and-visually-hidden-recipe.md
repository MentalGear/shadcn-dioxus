---
id: 426
title: "Theme owns the no-top-layer modal scrim and the visually-hidden recipe"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P1)."
size: "S"
area: components
updated: 2026-10-25
---

# Row 426: Theme owns the no-top-layer scrim and the visually-hidden recipe

## Item

The modal scrim and the "visible to assistive tech only" recipe are theme concerns, but five component sheets still paint the scrim for the no-top-layer arm (three of them with keyframe duplicates of the theme's fade pair), and the sr-only block is hand-written 12 times in two spellings. Row 145 moved the top-layer `::backdrop` into the theme; this moves the wrapper scrim and the sr-only block too. Construction: the five wrappers carry `data-dx-scrim` and one theme rule paints the scrim; the sr-only block is one `[data-dx-visually-hidden]` rule. After it lands, a sheet cannot paste either one, because the gate rejects it.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P1).

## Size

S

## Detail

Source: [final sweep note, P1](../research/final-sweep-2026-10-25.md).

- **Scrim, declared per component** (under `preview/src/components/`): `dialog/style.css:15-24`, `alert_dialog/style.css:~19-28` (`:23`), `command/style.css:52-60` (`:55`), `drawer/style.css:27-35` (`:30`), `sheet/style.css:26-34` (`:29`). Keyframes byte-identical to the theme's `dx-fade-in/-out` (`dx-components-theme.css:1720,1730`) at `dialog/style.css:43,57`, `alert_dialog/style.css:47,61`, `command/style.css:79,93`. `check-duplicate-keyframes.sh:44-51` allowlists exactly those three.
- **Visually hidden, hand-written 12 times:** `pagination/style.css:163`, `data_table/style.css:131`, `code_block/style.css:130`, `command/style.css:35`, `breadcrumb/style.css:81`, `sidebar/style.css:42,709`, `calendar/style.css:465`, `chart/style.css:417`, `message_scroller/style.css:172`, `assets/main.css:2295,2426`. Spellings: `clip: rect(0, 0, 0, 0)` and `clip-path: inset(50%)`.
- **Construction:** `"data-dx-scrim": ""` on each wrapper (the pattern `data-dx-root` already follows). One zero-specificity `:where([data-dx-scrim])` rule carries the base block, `:has(> dialog)`, `[data-dx-overlay="off"]` and the closed state through `dx-fade-out`. One `:where([data-dx-visually-hidden])` rule; components set the attribute.
- **Subsumes:** the three keyframe allowlist entries; 11 of the 12 sr-only blocks (the docs site's `.dx-charts-sr-only` in `main.css` can use the attribute too). **Does not subsume:** the `::backdrop` rule (already one) and sheet-specific panel animation.
- **Gate:** extend `check-theme-baseline.sh` as it already guards `::backdrop`: no component sheet may declare `background: var(--dx-overlay-scrim)` or the sr-only recipe, and both theme rules must exist. Browser half: the computed-style snapshot (`playwright/computed-style-states.ts`) before and after, the method row 145 used to prove no computed-style difference.
- **Check first:** that all five wrappers render `data-state` on the element that carries the class (true for alert dialog, `alert_dialog.rs:~140`; the others were not read to that depth).
