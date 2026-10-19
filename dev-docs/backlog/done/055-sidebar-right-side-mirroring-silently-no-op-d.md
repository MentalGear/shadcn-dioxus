---
id: 55
title: "Sidebar right-side mirroring silently no-op'd when collapsible: SidebarCollapsible::None — LANDED 2026-09-15…"
status: "Landed 2026-09-15"
size: "S"
area: tests
updated: 2026-09-15
---

# Row 55: Sidebar right-side mirroring silently no-op'd when collapsible: SidebarCollapsible::None — LANDED 2026-09-15…

## Item

**`Sidebar` right-side mirroring silently no-op'd when `collapsible: SidebarCollapsible::None` — LANDED 2026-09-15, user-reported.** `Sidebar()`'s `None` branch renders a parallel `.dx-sidebar-static` element (distinct from the main desktop branch's `.dx-sidebar-container`), and unlike that branch it never set `data-side`. The only CSS rule mirroring the sidebar to the right (`style.css`: `.dx-sidebar-wrapper:has([data-side="right"]) { flex-direction: row-reverse; }`, previously scoped to `.dx-sidebar-desktop[data-side="right"]`) requires that attribute, so with "Collapse: None" selected in the demo's own settings panel, toggling "Side: Right" had zero visual effect — the sidebar stayed pinned left. The default Offcanvas/Icon collapse modes already mirrored correctly; the bug was scoped specifically to `None`, which is why it only appeared "when choosing other configs." A second, same-root-cause defect found alongside it: `.dx-sidebar-static` also never set any `position`, so its child `SidebarRail` (`position: absolute`) had no positioned ancestor and rendered pinned to a page corner instead of the sidebar's own edge, for both sides. **Fixed by construction, not per-instance:** (1) `component.rs` — added `data-side` to the `None` branch's attributes; (2) `style.css` — broadened the mirroring selector to key on `[data-slot="sidebar"][data-side="right"]` (a slot all three Sidebar render paths already share) instead of the desktop-only class, so one selector covers every current and future branch; added `position: relative` to `.dx-sidebar-static`, giving `SidebarRail` the same containing block the main branch already gets for free from `.dx-sidebar-container`'s `position: fixed`. **Verified by execution:** `getBoundingClientRect()`/`getComputedStyle()` before/after at 1280x800 — sidebar rect identical for Left/Right before the fix (bug), correctly mirrored after (`x=0–256` vs `x=1009–1265`); `SidebarRail` moved from a page-corner rect (`x=1257–1273` / `x=-8–8`) to hugging the sidebar edge (`x=248–264` / `x=1001–1017`) on both sides. New geometry-based regression test `playwright/sidebar.spec.ts`, "desktop: side switch mirrors sidebar with collapsible=none" (mirrors `select.spec.ts`'s convention — asserts rects actually swap, not just that `data-side` changes, since the pre-existing "side switch updates data-side" test only checked the attribute and is exactly why it missed this). Full `sidebar.spec.ts` suite: **7/7 passed** (6 pre-existing + 1 new), no regressions. The test's own width assertion was initially written against `document.documentElement.clientWidth` and failed by exactly 15px — root-caused to row 1's own `scrollbar-gutter: stable` baseline (not a new bug), fixed by comparing against the wrapper element's own `boundingClientRect` instead

## Status / trigger

Landed 2026-09-15

## Size

S

## Detail

Found via user report ("alignment bugs" when "choosing other configs, like the right side"); delegated to a Sonnet subagent per this repo's debugging convention, independently re-verified (including the scrollbar-gutter false start) before landing
