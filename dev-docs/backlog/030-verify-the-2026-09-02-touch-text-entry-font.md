---
id: 30
title: "Verify the 2026-09-02 touch text-entry font-size floor…"
status: "User re-test after deploy — the sandbox has no WebKit, and while Playwright's iPhone 13 emulation…"
size: "S to verify"
area: tests
updated: 2026-09-02
---

# Row 30: Verify the 2026-09-02 touch text-entry font-size floor…

## Item

Verify the 2026-09-02 touch text-entry `font-size` floor (`playwright/oracle/tier2-html/touch-focus-zoom.spec.ts`; `Combobox`, `Calendar` month/year selects, the navbar language `<select>`, `top_layer`/`form`'s raw reference controls, and the dashboard reply-textarea override) on a real iOS Safari device

## Status / trigger

User re-test after deploy — the sandbox has no WebKit, and while Playwright's iPhone 13 emulation correctly reproduces `(pointer: coarse)`/`(hover: none)` and lets Chromium's DevTools Protocol confirm the *computed* `font-size` values this fix depends on, only real Mobile Safari actually implements the auto-zoom-on-focus behaviour those values are floored against — no engine in this sandbox (Chromium or otherwise) reproduces it, so the oracle can prove the CSS but not the zoom itself

## Size

S to verify

## Detail

Distinct from row 28/29 (those are the iOS-keyboard anchor-tracking fix in `top_layer.rs`; this is the unrelated `font-size`/touch-zoom fix) — kept as its own row rather than appended to row 28, which is about a different bug. Separately, live GREEN re-verification of every fix in this row is itself PENDING as of this writing, for an environment reason rather than a construction doubt: `Combobox`'s fix *was* confirmed RED→GREEN by execution (revert the CSS, see the pre-fix 14px/13.3333px values recur; restore it, see 16px return), but the shared `dx run --web` dev server in this sandbox stopped visibly hot-reloading any further CSS edit afterward -- 10+ minutes, several files, zero rebuild log lines -- coinciding with a concurrent session's own rebuild activity on `top_layer.rs`/`top-layer.spec.ts`. Per this task's own instructions the dev server was not restarted to force the issue; the remaining fixes are correct by construction (identical media-query convention to the already-GREEN-confirmed one, and confirmed present in the on-disk built CSS output) but not re-confirmed live in this session
