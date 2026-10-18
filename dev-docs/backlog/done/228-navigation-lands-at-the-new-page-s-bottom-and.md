---
id: 228
title: "Navigation lands at the new page's bottom and glides up (home gallery card links, Back)"
status: "LANDED 2026-10-18 (lane baseline 9eb2e91d), merged on the session branch; deploy pending -- was…"
size: "S"
area: menus
updated: 2026-10-18
---

# Row 228: Navigation lands at the new page's bottom and glides up (home gallery card links, Back)

## Item

Navigation lands at the new page's bottom and glides up (home gallery card links, Back)

## Status / trigger

**LANDED 2026-10-18 (lane baseline 9eb2e91d), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane baseline)** -- was: **PROPOSED for the 2026-10-15 round (new lane `baseline`); waiting for the owner's go**

## Size

S

## Detail

Owner 2026-10-17: a gallery card header link opens the component page at its bottom (e.g. the date range demo). Evidence `research/nav-scroll-smooth-2026-10-17.md`: dioxus-web 0.7.9 does `window.scrollTo(0,0)` on push, but `preview/assets/main.css:90-95` sets ambient `html { scroll-behavior: smooth }`, so the jump animates; the router swaps in the shorter page mid-animation, the browser clamps to its bottom, and it glides to 0 over ~1-1.5 s (Back has the mirror defect; on iOS the glide can stall). Same class already worked around in `charts_gallery.rs:307-328` (`behavior:'instant'`) and unhandled in `scroll_lock.rs:731`. **Construction:** smooth scrolling opt-in, never ambient: `html[data-anchor-scroll] { scroll-behavior: smooth }` (reduced-motion media kept), set by the existing `a[href^="#"]` capture handler in `preview/index.html` and cleared on settle; gate `check-scroll-behavior.sh` (root `scroll-behavior: smooth` only under `[data-anchor-scroll]`); spec `nav-scroll-instant.spec.ts` (push: every sample 0; Back: one step; anchor: still smooth). Row 225 needs its own per-view map, not this rule.
