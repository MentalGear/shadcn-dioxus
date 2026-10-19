---
id: 176
title: "Reduced motion has a gap: the infinite loops of Skeleton (pulse), Progress (indeterminate sweep) and Avatar…"
status: "LANDED 2026-10-10 -- was: OPEN, found while writing dev-docs/motion.md"
size: "S (done)"
area: theme
updated: 2026-10-10
---

# Row 176: Reduced motion has a gap: the infinite loops of Skeleton (pulse), Progress (indeterminate sweep) and Avatar…

## Item

Reduced motion has a gap: the infinite loops of Skeleton (pulse), Progress (indeterminate sweep) and Avatar (loading pulse) have no `prefers-reduced-motion` rule, and the shared layer only reaches interactive roots

## Status / trigger

**LANDED 2026-10-10** -- was: OPEN, found while writing `dev-docs/motion.md`

## Size

S (done)

## Detail

**Landed construction:** ONE rule in `dx-components-theme.css` -- under `prefers-reduced-motion: reduce`, `animation: none !important` on every `[data-dx-motion-key]` host and everything under it (`::before`/`::after` included). `use_motion` already marks exactly the elements that loop, so skeleton, spinner, indeterminate progress and loading avatar are covered and the next looping component is covered by calling `use_motion`. `!important` so it wins over a component's `animation:` shorthand in any stylesheet order. A loop outside that convention writes its own block (OTP caret, `.dx-shimmer`). **Two things found on the way:** (1) the old spinner and OTP-caret reduced rules set only `animation-duration: var(--dx-motion-duration-reduced)` on an `infinite` animation, which does not stop it -- 0.01 ms per cycle samples a random phase each frame and flickers; both removed (spinner, via the theme rule) or replaced with `animation: none` (caret), and the gate now rejects a duration-only override. (2) A still frame of the indeterminate bar is a half-width bar that reads as 50%, so `progress/style.css` draws diagonal stripes across the whole track under reduced motion (a still ring, block and placeholder carry "loading" for the others, with `role="status"` / `aria-busy` unchanged; WCAG 2.2.2 / 2.3.3 essential state stays visible). **Gate:** `scripts/check-infinite-animations.sh` now fails an `infinite` animation that is not (a) inside `no-preference`, (b) stopped by a same-selector reduced rule (`animation: none` / finite replacement), or (c) in a component sheet whose component calls `use_motion` while the theme carries the host rule. **Spec:** `motion-gating.spec.ts` "reduced motion stops every loop" emulates the preference and asserts `animation-name: none` (and no running animation) on spinner, skeleton, progress and avatar, plus the full-width still progress bar. Docs: `dev-docs/motion.md` and the `/docs` Motion section (`motion_docs.rs`) say it is covered.
