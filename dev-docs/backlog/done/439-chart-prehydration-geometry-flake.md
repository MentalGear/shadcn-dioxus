---
id: 439
title: "chart-prehydration geometry flake: the two loads race font-display: optional"
status: "DONE 2026-10-25 (lane chart-prehydration)."
size: "S"
area: testing
updated: 2026-10-25
---

# Row 439: chart-prehydration geometry flake

## Item

`playwright/chart-prehydration.spec.ts` failed twice in the release suite and passed on a rerun: batch 22 on `/charts/line/` (cards 14 px taller before hydration) and batch 26 on `/charts/` (58 diffs: `card.h 321 -> 341`, `header0.h 46 -> 66`, `chart0.y 78 -> 98`, `footer0.y 250 -> 270`, the hydrated side taller). Markup and CSS were identical and a fresh build passed, so both were taken for a flake. They are one defect.

## Status / trigger

DONE 2026-10-25 (lane `chart-prehydration`).

## Size

S

## Detail

Cause (measured, not a hydration shift): Geist is self-hosted with `font-display: optional` (`preview/src/site/head.rs`), so a document uses it only if it lands inside the ~100 ms block window and otherwise not at all. The spec loads the page twice, JavaScript off and hydrated, and each load races the font on its own. Under load one gets Geist and the other the fallback face, and a card description or footer line wraps to a second line on that side only: 20 px is one `text-sm` line of the 6-month description (`/charts/`: header 46 -> 66), 14 px one `text-xs` footer line (`/charts/line/`, `/charts/pie/`, `/charts/bar/`: card 321 -> 335).

- The state is not selectable by waiting. A woff2 delayed 600 ms ends `Geist: loaded` in `document.fonts` with `fonts.ready` resolved, and the text is still laid out in the fallback (probe: `/charts/` card 1 header 66 px with the font blocked or delayed, 46 px with it in time). The product is right: `optional` never swaps inside a document, so nothing shifts within one load; only two loads can disagree.
- Deterministic repro: the old spec with the woff2 aborted in the hydrated context only fails 7/7 routes, `1440px /charts/ #1 Area Chart card.h: 321 -> 341 (20.00)` and `header0.h: 46 -> 66`, the batch 26 numbers exactly; `/charts/radial/` and `/charts/bar/` show the 14 px footer variant.
- Natural repro (release build of `queue/batch-26`, 4 workers, 16 busy loops, 56 runs per round): old spec 5/56 failed (`/charts/line/` x3, `/charts/pie/`, `/charts/tooltip/`), in BOTH directions (`footer0.h 85 -> 71` and `71 -> 85`), so neither side is the slow one. On the older `release-template` tree: 1/56 twice. Without load: 0 in 21 runs.
- The same race had already been found and fixed in one copy: `first-paint-styles.spec.ts` ("198 differing boxes on /component/tabs/", `noWebFonts`), and `computed-style-snapshot.spec.ts` pins it its own way. Three occurrences, so the class is "a two-load comparison without a pinned font state".

## Outcome

- `playwright/web-fonts.ts`: `noWebFonts(page|context)` (one font-request pattern, moved out of `first-paint-styles.spec.ts`) and `expectNoWebFont(page, label)`, which fails naming the font state, not as 14/20 px box diffs, if the pin stops matching.
- `chart-prehydration.spec.ts` pins both contexts to the fallback face and asserts it on both sides. Fallback rather than Geist because the fallback needs no race to win and is identical on every machine; "wait for `document.fonts.ready` and use Geist on both" is not possible under `optional` (above), and making the product keep its line count across a font swap is not needed, the product never swaps. `fonts-self-hosted.spec.ts` still owns "the font really loads".
- Gate `scripts/check-spec-font-state.sh` (in `run-gates.sh`, group Playwright specs; `gates.md` regenerated): a spec that opens a `javaScriptEnabled:` context and reads boxes or styles must call `noWebFonts(` within 8 lines or carry `font-ok: <reason>` / a reasoned ALLOW entry (accordion and questionnaire: one markup-only JavaScript-off load each), and no spec may carry its own `woff2` route. Self-test mutations inline; removing the pin from the chart spec turns it red.
- After, same release build and load: new spec 0/112 failed (2 rounds of 56), `chart-prehydration` and `first-paint-styles` green through `test:changed`.
- Not covered, noted: `computed-style-snapshot.spec.ts` keeps its own inline pin (works; a later cleanup can switch it to the helper). Single-load specs (`layout-stability`, `charts-hydration-stability`) compare frames inside one document, whose font state cannot change, so they cannot hit this.
