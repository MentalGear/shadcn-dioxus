---
id: 49
title: "pinNearTop scroll-helper bug in top-layer.spec.ts, found on first execution. Rule 11(c)'s Popover case — the…"
status: "Landed 2026-09-04"
size: "S"
area: tests
updated: 2026-09-04
---

# Row 49: pinNearTop scroll-helper bug in top-layer.spec.ts, found on first execution. Rule 11(c)'s Popover case — the…

## Item

**LANDED 2026-09-04: `pinNearTop` scroll-helper bug in `top-layer.spec.ts`, found on first execution.** Rule 11(c)'s Popover case — the section's only executable assertion — had never once actually opened its popover; it burned the full 5-minute per-test timeout every run. `pinNearTop` assumes its trigger is in normal document flow and scrolls the page by `rect.top`, but `#edge-bottom-popover-trigger`'s row is `position: fixed; bottom: 4px`, so it computed a large arbitrary scroll (+659px measured) that dragged the fixture's unrelated `#stack-sibling` (`position: absolute; z-index: 9999`) into the trigger's rectangle, where it intercepted the click. Fixed in the helper itself — it now walks ancestors for `position: fixed` and returns early — rather than in the one case, so a future fixed-position trigger cannot hit the same bug again. The case now passes in ~15s. **Not a regression from this session** — this code dates to 2026-09-02 and was authored without a browser

## Status / trigger

Landed 2026-09-04

## Size

S

## Detail

First execution of this exact case since it was written (2026-09-02) — this session is the first time this repo has had a working Playwright container, so this was the first chance any session had to notice the helper never worked. Harness-only fix, no product code touched
