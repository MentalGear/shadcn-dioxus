---
id: 110
title: "Playwright's own actionability pre-scroll (.hover()/.focus()/.click()'s auto-scroll) animates under the…"
status: "Instance 2, 2026-09-25 (round 8b): carousel.spec.ts's \"hovering the carousel stops rotation…\"…"
size: "Diagnosis M (two hypotheses tried and…"
area: tests
updated: 2026-09-25
---

# Row 110: Playwright's own actionability pre-scroll (.hover()/.focus()/.click()'s auto-scroll) animates under the…

## Item

Playwright's own actionability pre-scroll (`.hover()`/`.focus()`/`.click()`'s auto-scroll) animates under the site's global `html { scroll-behavior: smooth }` (`preview/assets/main.css:54`) — a class, confirmed at a second instance

## Status / trigger

Instance 2, 2026-09-25 (round 8b): `carousel.spec.ts`'s "hovering the carousel stops rotation…" failed roughly 3-4 times per 5-10 runs on the SSG build. Instance 1, 2026-09-18 (`daebe40`, confirmed by reading that commit: its own message states verbatim "html gets scroll-behavior: smooth so the jump animates"): `oracle/tier2-html/top-layer.spec.ts`'s `pinNearTop` helper (~line 1212)

## Size

Diagnosis M (two hypotheses tried and refuted before the real one); the by-construction fix is **BUILT 2026-09-26 (`a8849a7`)**; was: proposed, not built

## Detail

**2026-09-26 (audit batch): BUILT** in `a8849a7` -- `playwright/fixtures.ts` overrides the worker `browser` fixture so every context gets `html { scroll-behavior: auto !important }`; every spec imports `test`/`expect` from it, enforced by the new `scripts/check-playwright-fixture-import.sh` (it caught `navbar.spec.ts` the same day). The two ad-hoc patches were simplified onto it. **Two hypotheses refuted, in order, before the real mechanism.** The main loop's own first read — a mid-transition autoplay tick counted as an extra advance — was disproved by a MutationObserver + `performance.now()` probe: `mouseenter` fires, a genuine `mouseleave` follows ~250-300ms later, and rotation resumes on a fresh ~1200ms cadence. The actual cause: the whole page was still smooth-scrolling (~900px) from `.hover()`'s own `scrollIntoViewIfNeeded` pre-check — an unqualified scroll defaults to `behavior: "auto"`, which itself defers to the ancestor's CSS `scroll-behavior: smooth` — dragging the carousel out from under the still-resting cursor. Confirmed two independent ways: forcing `scroll-behavior: auto` site-wide dropped it to 0/15 failures; an instant pre-scroll also gave 0/15. No `carousel.rs` change — component logic was correct throughout. **Fixed for instance 2** with a `scrollIntoViewInstant()` call before `.hover()` (and `.focus()`, as defence-in-depth) in `62183b3`. **Instance 1** (`top-layer.spec.ts`'s `pinNearTop`, per that helper's own inline comment dated 2026-09-18) predates this round and carries the identical fix, arrived at independently: `behavior: "instant"` on its `scrollIntoView`/`scrollTo` calls, opting out of the same CSS property for the same reason. **Per `CLAUDE.md`'s own "two occurrences is a class" rule — proposed here, not built [BUILT 2026-09-26, `a8849a7`, see the first sentence]** (this lane owns `dev-docs/**` only): a shared Playwright fixture, or an `addInitScript`/`page.addStyleTag` in the local + CI configs, setting `html { scroll-behavior: auto !important }` for the test run. This subsumes both instances above and any future actionability pre-scroll, without a per-call-site fix each time. It does **not** touch a test's own *explicit* `scrollIntoView({behavior: 'smooth'})` call — verified by reading `carousel.spec.ts`'s "paged transition animates" test, which asserts on exactly such a call: an explicit `behavior` argument overrides the ancestor CSS default regardless of what that CSS says, so this construction leaves animation tests working. **Weaker alternative:** scope `main.css`'s rule under `@media (prefers-reduced-motion: no-preference)` — subsumes both instances too, but only for a runner that emulates `prefers-reduced-motion: reduce`, which this repo's configs do not do by default (confirmed: no `reducedMotion` project setting in any of the four `*.local.config.ts` files or in `playwright.config.ts`), so it would need that setting added as well to actually take effect
