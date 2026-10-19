---
id: 94
title: "Demo wrappers that set max-width + margin: 0 auto never reach that width — they shrink-wrap instead, so every…"
status: "CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code"
size: "S per wrapper (add width: 100%); all…"
area: charts
updated: 2026-10-19
---

# Row 94: Demo wrappers that set max-width + margin: 0 auto never reach that width — they shrink-wrap instead, so every…

## Item

Demo wrappers that set `max-width` + `margin: 0 auto` never reach that width — they shrink-wrap instead, so every affected demo renders narrower than its author intended

## Status / trigger

**CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code**

## Size

S per wrapper (add `width: 100%`); **all DONE 2026-09-26 (`650db75`)**: the carousel's five in round 5, the other 12 in the audit batch, guarded by `check-demo-wrapper-width.sh`

## Detail

**2026-09-26 (audit batch): FIXED** in `650db75` -- measured on every page: 12 affected (chart_tooltip ×9 via one shared rule, combobox disabled/controlled/dynamic), each now exactly its max-width; `form`/`toolbar` fixed too (latent: content happens to fill the cap today). Guarded by construction: new `scripts/check-demo-wrapper-width.sh` (in CLAUDE.md's gate list), zero false positives today; limits stated in its header. **The real mechanism, measured rather than reasoned:** `.dx-component-preview-frame` (`preview/assets/main.css`) is `display: flex; flex-direction: column`, so its cross axis is horizontal. A wrapper's own `margin: 0 auto` makes BOTH cross-axis margins `auto`, and per flexbox auto cross-axis margins take precedence over `align-self: stretch` — so the item's cross size stays `auto` and it shrink-wraps its content. **`align-items: stretch` therefore does not fix it**: measured on the carousel page, the five wrappers stayed at 176/250/137/22/137px both before and after injecting that rule. An explicit `width: 100%` does fix it — the same five went to 320/448/320/192/320px, each exactly its own `max-width`. **And `align-items: stretch` would have been actively harmful:** probing all 66 component pages (123 preview frames) with that rule injected widened **80 direct children that have no `max-width` at all**, e.g. `alert_dialog`'s bare `<button>` 159px -> 682px, `button`'s demos 113px -> 682px, `chart`'s `.dx-chart` 300px -> 682px. Evidence: `$S/round5/align-probe.json` plus `align-probe.js`/`fix-probe.js` beside it. **Scope of the class:** 24 direct children across the 66 pages carry a `max-width` and are candidates; carousel's five are fixed (each variant's wrapper now leads with `width: 100%`, with the mechanism documented in `variants/main/mod.rs` so it is not "cleaned up" later as redundant). The other ~19 (`combobox` confirmed: three wrappers at 200/200/236px against a 320px `max-width`) were NOT fixed as of round 5 (fixed 2026-09-26, `650db75`, see the first sentence) — each needed its own author to confirm the intended width is really the `max-width` value rather than the shrink-wrapped one that has been shipping. **Why this matters beyond tidiness:** the carousel's main demo was rendering at 176px against shadcn's 320px, which was a larger visual difference from the reference than the Previous/Next placement this round set out to fix.
