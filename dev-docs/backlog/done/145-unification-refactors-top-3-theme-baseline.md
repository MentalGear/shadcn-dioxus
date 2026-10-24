---
id: 145
title: "Unification refactors, top 3: theme baseline, floating-surface recipe, state recipes (incl. Combobox and…"
status: "LANDED 2026-10-13 (all three lanes; proof: hover/open computed-style snapshot) -- Lane T (5345236)…"
size: "L (three lanes)"
area: theme
updated: 2026-10-13
---

# Row 145: Unification refactors, top 3: theme baseline, floating-surface recipe, state recipes (incl. Combobox and…

## Item

Unification refactors, top 3: theme baseline, floating-surface recipe, state recipes (incl. Combobox and Textarea missing a focus ring)

## Status / trigger

**LANDED 2026-10-13 (all three lanes; proof: hover/open computed-style snapshot)** -- **Lane T** (`5345236`): one zero-specificity `touch-action` rule and one modal `::backdrop` scrim in the theme (50 per-component declarations and 6 scrim blocks deleted), half-step spacing tokens, gate `check-theme-baseline.sh`; snapshot before (`f5e72ab`) vs after: no computed-style difference (the only 24 values were a harness race, now fixed by construction in `computed-style-states.ts`: the open pass waits for the previous overlay to unmount and counts only additions as an open; four captures, two under CPU load, byte-identical). **Lane F**: one floating-surface recipe on `data-dx-surface="floating"` (edge `1px foreground/10`, radius, popover fill and ink, `shadow-md`, padding, one 150 ms open/close timing with ease-out/ease-in, reduced motion for every panel) for the 12 surfaces, shared `dx-fade/pop/drop` keyframes through `--dx-floating-enter/-exit` slots, one menu-item recipe for dropdown, context menu and menubar (disabled items now `opacity .5`, shadcn), gate `check-duplicate-keyframes.sh`. **Lane S**: focus converged on `--dx-ring-field`/`--dx-ring-focus` (+ an inset variant), one invalid recipe from `--dx-ring-invalid*` tokens (row 155 group 1's invalid ring, built here), disabled literals -> `--dx-opacity-disabled`; two accessibility gaps closed (sidebar group/menu actions had no focus indicator; disabled checkbox was unstyled); gate `check-css-state-recipes.sh`. Snapshot T vs F+S: 15,236 differing values, all classified: 15,038 intended or their layout consequences (the +2px panel border, docs-page heights), 256 a correct unlisted change (floating panels set their own ink instead of inheriting muted text), 198 a fixture regression fixed (top_layer's raw `PopoverContent` needed `data-dx-surface`), plus an order-dependent sidebar dropdown-trigger focus ring fixed with the `--dx-dropdown-menu-trigger-focus-ring` slot variable. Full Playwright suite on the F+S debug build: 2695 passed, 6 failed, all in the baseline set. Left for later (not regressions): reduced-motion coverage for the remaining 32 animating sheets, `button { font-family: inherit }` for ~25 button-rendering sheets, Field's `invalid` not propagating to the control (controls carry `aria-invalid`, as in shadcn), the per-component slide distances and placement gaps. **OPEN, OWNER-APPROVED 2026-10-05 for the next new session** (second in the chain 135 -> 145 -> 111; step 0 is the instance-list recount) -- audit 2026-10-04 (`dev-docs/archive/research/unification-audit-2026-10-04.md`; static read at `44a103e`, nothing executed, hypotheses H1-H4 in its section 6)

## Size

L (three lanes)

## Detail

The 2026-10-04 unification audit (`dev-docs/archive/research/unification-audit-2026-10-04.md`) found 15 concerns implemented ad
hoc per component; this row took the top three in order, each as a construction owned by one lane (the Status section above has
the commits and the snapshot proof). Owner-approved 2026-10-05; step 0 was a recount of the audit's instance lists, which ran on
the clean tree at `f5e72ab` on 2026-10-13 and superseded the audit's counts.

1. **Theme baseline (lane T, `5345236`).** `touch-action: manipulation` became ONE zero-specificity theme rule over
   `:where(<role list>[class^="dx-"])` (50 per-component declarations in 33 files deleted; 11 deliberate `none`/`pan-*` stay with
   reasons); the six copies of the modal-scrim `::backdrop` block (dialog, alert dialog, sheet, drawer, command, popover) became
   one theme rule on `dialog[class^="dx-"]:not([popover])`; `--dx-space-0-5/1-5/2-5/3-5` replaced 37 half-step spacing literals;
   gate `check-theme-baseline.sh`. The overlay and menu-indicator tokens and the Command wrapper scrim were already done.
2. **Floating surface (lane F).** One recipe on `data-dx-surface="floating"` for the 12 surfaces (edge, radius, fill, ink,
   shadow, padding, one 150 ms timing, reduced motion), shared `dx-fade/pop/drop` keyframes selected through
   `--dx-floating-enter/-exit`, one menu-item recipe for dropdown, context menu and menubar, gate `check-duplicate-keyframes.sh`;
   every anchored keyframe animates `translate`/`scale`, never `transform` (`check-anchored-keyframes.sh`, inverted; the
   color-picker and date-picker fades were 133 and 144 px off-centre before it).
3. **State recipes (lane S).** Focus converged on `--dx-ring-field`/`--dx-ring-focus`, one invalid recipe from `--dx-ring-invalid*`,
   disabled literals became `--dx-opacity-disabled`; Combobox and Textarea gained a focus ring (`playwright/focus-ring.spec.ts`
   covers 12 controls), the sidebar group and menu actions gained a focus indicator and the disabled checkbox a style; gate
   `check-css-state-recipes.sh`. Row 170's rule applied: state recipes are slot variables, not more tag selectors on another
   component's root.

**Proof.** Computed-style snapshot before (`f5e72ab`) versus after: lane T no difference; T versus F+S 15,236 differing values, all
classified (15,038 intended or their layout consequences, 256 a correct unlisted change, 198 a fixture regression fixed). Full
suite on the F+S debug build: 2695 passed, 6 failed, all in the baseline set. A snapshot-harness race (24 values) was fixed by
construction in `computed-style-states.ts`. Left for later: reduced-motion coverage for the remaining 32 animating sheets,
`button { font-family: inherit }` for ~25 button sheets, Field's `invalid` not propagating to the control, per-component slide
distances.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/145-unification-refactors-top-3-theme-baseline.md`.*
