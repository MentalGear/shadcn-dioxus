---
id: 71
title: "Rule 11 self-overlap regression, baseline group D"
status: "Found + fixed 2026-09-18; live re-run against the integration server confirmed round 2"
size: "S"
area: tests
updated: 2026-09-18
---

# Row 71: Rule 11 self-overlap regression, baseline group D

## Item

Rule 11 self-overlap regression, baseline group D

## Status / trigger

Found + fixed 2026-09-18; live re-run against the integration server confirmed round 2

## Size

S

## Detail

Found in the 2026-09-18 baseline (4 cases: Select/Combobox "no self-overlap once open", Navbar simulated-keyboard, DropdownMenu simulated-no-anchor-engine) — a discrepancy from `conformance-harness.md`'s own last-recorded-good state (Rule 11 landed 2026-09-02, file "passing" after Rule 15 landed 2026-09-04), not a previously-accepted flake. Root cause, bisected by git archaeology plus an instrumented reproduction (Lane J): commit `daebe40` (2026-09-14ish, PR #54, sidebar TOC) added a global `html { scroll-behavior: smooth }` to `preview/assets/main.css` for legitimate in-page TOC-link navigation. `top-layer.spec.ts`'s `pinNearTop` fixture helper calls `scrollIntoView`/`window.scrollTo` with no `behavior` option, so both silently started animating over several hundred ms instead of jumping instantly — breaking `pinNearTop`'s own documented contract of delivering the trigger at a settled position *before* `overlay.open()` runs. Confirmed by execution: the trigger's `getBoundingClientRect().top` kept changing for 400-600ms *after* `pinNearTop` had already resolved (Combobox: 643→541→93→78, settling ~400ms later); `use_anchor_position_fallback`'s rAF-throttled scroll listener is necessarily one step behind a continuous CSS-driven scroll animation under real scheduling load (this sandbox's concurrent WASM/Chromium contention especially) — a real geometry mismatch between two moments of the *same still-settling* trigger, not a placement-formula defect. One root cause subsumes every Rule 11 failure observed this session, baseline's four plus two more (Combobox(a)/Navbar(a)) caught independently by the lane's own full-suite reproduction — every sub-case calls `pinNearTop` before opening, and the numeric signature (content positioned for an *earlier* trigger reading) was identical across every case inspected. Bisection: `git log -S "scroll-behavior: smooth"` pins the regression to `daebe40` exactly (absent in its parent, present in it); an isolated, non-full-suite Rule 11 rerun did **not** reproduce even pre-fix — this is a timing race whose window opens wide enough only under real CPU contention, so the full `top-layer.spec.ts` suite under 2-worker load is the only reliable regression check for it, not an isolated Rule-11-only run. **Fix (Lane J, commit `55ec9b1`):** qualify both `pinNearTop` scroll calls with `behavior: "instant"` (CSSOM View's standard per-call opt-out of an ancestor's `scroll-behavior: smooth`), restoring the helper's pre-existing contract without touching the legitimate site-wide smooth-scroll feature. Deliberately kept out of `primitives/src/top_layer.rs`: the fallback's tracking is already exactly as good as a rAF-throttled listener can be against a continuous CSS animation it has no visibility into, and scroll-tracking fidelity during a genuine in-flight scroll is Rule 8's own separately-tracked, already-documented limitation (row 21), not Rule 11's. **Repo-wide class note:** any Playwright helper that calls `scrollTo`/`scrollIntoView` without an explicit `behavior` is implicitly coupled to the page's `scroll-behavior` CSS — true everywhere in this suite, not just `pinNearTop`. **Residual, not closed by this fix:** the vertical clamp's own trigger-blind edge-hugging branch remains a plausible independent latent gap — no evidence of it firing today, flagged as a follow-up, not chased further. Verified by execution, own dev server on :8082: full `top-layer.spec.ts` pre-fix 5 failed (2 Rule 11 + 2 Group C + 1 row 21) / post-fix, two consecutive runs, 3 then 2 failed, zero Rule 11 in either. **Live re-run against the shared integration server (not the lane's own isolated :8082) is confirmed: round 2's targeted Playwright run found group D (Rule 11) all green, including all 26 Rule 11 sub-assertions.** `pw-baseline/BASELINE-REPORT.md` Group D is the full evidence trail; distinct from row 21 (Rule 8's own anchor-freeze) and row 70/Group C (a hit-testing interception) — three different mechanisms on the same spec file, not one bug wearing three names
