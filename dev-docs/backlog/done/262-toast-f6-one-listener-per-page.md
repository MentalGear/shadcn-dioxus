---
id: 262
title: "Toast: one F6 listener per page, not one per ToastProvider"
status: "LANDED 2026-10-19 (lane toast2: one page-lifetime F6 handler, providers register)"
size: "S"
area: primitives
updated: 2026-10-19
---

# Row 262: Toast: one F6 listener per page, not one per ToastProvider

## Item

Every `ToastProvider` registers its own global F6 listener, so on a page with several providers F6 focuses the last-mounted region.

## Status / trigger

**LANDED 2026-10-19 (lane toast2: one page-lifetime F6 handler, providers register)**

## Size

S

## Detail

The toast docs page now mounts eight toast demos, each with its own provider; F6 ("jump to notifications") lands on whichever
region mounted last, not the one with toasts. Construction: one page-lifetime F6 handler (installed once, like the modality
writer) that cycles through the regions that currently hold toasts, most recent first; providers register their region with
it instead of listening themselves. Spec: two providers, toasts in the first, F6 focuses the first.

## Update 2026-10-19

Landed in lane `toast2`. `ToastProvider` no longer listens for F6 (`use_global_keydown_listener` is gone from `toast.rs`). The ONE handler is `TOAST_F6_JS` (`window.__dxToastF6` plus a single `document` keydown listener): every provider runs the script through `use_js_listeners`, the first creates the registry and the listener, each takes a reference, and the last provider to unmount removes the listener again (reference-counted like `activity.rs`, so `listener-inventory.spec.ts` stays at baseline when the form-fixture route mounts and unmounts a provider; a first, never-removed variant failed that spec and was replaced). Each provider's effect reports `(region id, toast count, newest ToastId)` whenever its list changes. F6 focuses the region holding the most recent toast (`ToastId` is one process-wide monotonic counter, so recency needs no clock), pressing it again from inside a region goes on to the next region with toasts (wrapping), a region that is empty or gone from the document is never a candidate, and with no candidate F6 is left to the browser. Construction: listener count is independent of provider count, and "which region" is decided in one place, so the bug class (N providers, last-mounted wins) cannot recur.

Evidence: `playwright/toast-f6.spec.ts` (4 tests): F6 lands on the first region when only it holds a toast on the eight-provider docs page, is a no-op with no toasts, cycles most-recent-first and wraps across two regions, and a CDP listener inventory finds exactly one `F6` keydown handler on `document` with eight providers mounted. Lane-ssg build containing the change: all 37 `toast[-.]` specs (incl. the 4 new F6 tests) plus the 22 `listener-inventory` tests green (59/59). Nothing remains.
