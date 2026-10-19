---
id: 9
title: "Select listbox scroll lock"
status: "LANDED 2026-09-18, same commit as row 8 (1e81ad2, shared plumbing). SelectList's listbox now locks…"
size: "S"
area: overlays
updated: 2026-09-18
---

# Row 9: Select listbox scroll lock

## Item

Select listbox scroll lock

## Status / trigger

**LANDED 2026-09-18**, same commit as row 8 (`1e81ad2`, shared plumbing). `SelectList`'s listbox now locks page scroll while open, matching Radix Select's default, via the crate's existing `scroll_lock.rs` mechanism — `ScrollLockGuard` mounted inside `SelectListRendered` on both the web and native arms, exactly like `DropdownMenuContentRendered`/`ContextMenuContent` already do for their `modal` prop, and `ensure_scrollbar_gutter_baseline` called from `Select`/`SelectMulti`'s root (matching `DropdownMenu`'s root). Opt-out via a new `scroll_lock: ReadSignal<bool>` prop (default `true`) on `Select` and `SelectMulti` (which shares `SelectList`), threaded through the themed preview wrapper (`preview/src/components/select/component.rs`) for API parity with the other forwarded props. New case in `playwright/oracle/tier3-radix/scroll-lock.spec.ts` ("Select locks and releases page scroll"), and that file's stale "Select is not wired to any scroll lock" header note updated to point at it.

## Size

S

## Detail

No Playwright fixture exercises `scroll_lock=false` — documented gap, not built this round (lane E's own hand-back). Checks: `cargo test -p dioxus-primitives` 119 unit + 120 doc passed; stylelint not run (no CSS touched)
