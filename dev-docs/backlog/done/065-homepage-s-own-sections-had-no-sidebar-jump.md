---
id: 65
title: "Homepage's own sections had no sidebar jump-links — LANDED 2026-09-18, user-requested (same feature as row…"
status: "Landed 2026-09-18"
size: "S–M"
area: build
updated: 2026-09-18
---

# Row 65: Homepage's own sections had no sidebar jump-links — LANDED 2026-09-18, user-requested (same feature as row…

## Item

**Homepage's own sections had no sidebar jump-links — LANDED 2026-09-18, user-requested (same feature as row 62, second page).** Row 62 built a heading-derived sidebar TOC for `/docs`, nested under its "Overview" nav item; the user then asked for the exact same treatment on the homepage ("opening the sidebar ... should list the items on the current overview page and a click should jump to them (anchor) on page"), clarifying an earlier ambiguous report that this session initially (and, after extensive live testing across dev/production builds and desktop/mobile, unsuccessfully) tried to root-cause as a navigation bug in row 62's own feature — it wasn't a bug there at all; the homepage simply never had the feature. **Generalized rather than duplicated, per this repo's standing "two occurrences = a class" rule:** `DocsLayout`'s `docs_sections` prop is renamed `page_sections` (still `Option<&'static [&'static str]>`) — page-agnostic, since which top-level "Start" nav item it nests under is now decided purely from the existing `active: DocsNavActive` value rather than a second prop. A new shared `page_sections_submenu(sections)` fn holds the `SidebarMenuSub`/`SidebarMenuSubItem`/`SidebarMenuSubButton` markup exactly once and is called from two guarded sites (`active == DocsNavActive::Home` nests under a new "Home" item, `active == DocsNavActive::Overview` under "Overview" as before) — one rendering block, not two copies of the same markup for two pages. A new "Home" `SidebarMenuItem` (`House` icon, `Route::home()`, `is_active` on `DocsNavActive::Home`) was added to the "Start" group above "Overview", since nesting the homepage's own section links under "Overview" (a link that navigates *away* to `/docs`) would have been confusing — a same-page jump-list belongs under a nav item that represents the page you're actually on. `Home()` gets its own `HOME_SECTIONS: &[&str] = &["Sample interfaces", "All components"]`, reusing row 62's `docs_section_slug` as-is (already page-agnostic despite living near `DOCS_SECTIONS`) — `"sample-interfaces"`'s `id` lives on `WidgetMasonry()`'s own `h2`, threaded in via a new `heading_id: String` prop since that heading is rendered by a separate `#[component]` fn (the heading *text* itself still reads from `HOME_SECTIONS[0]` directly, same module, so text and id can't drift from each other); `"all-components"`'s `id` is computed inline exactly like `Docs()` does for its own headings. `ComponentHighlight()` (component pages) still passes nothing and shows both "Home" and "Overview" as plain, sub-item-less links, unchanged. **Verified by execution, independently re-checked by the orchestrating session:** `cargo check --workspace`/`cargo fmt --check` clean; all three composition/class-prefix/css-literal lint scripts clean; `sidebar.spec.ts`/`navbar.spec.ts`/`preview.spec.ts` re-run, 13/13 passed (including the homepage axe scan, clean with the new "Home" item + icons present); independently confirmed against a live `dx serve` instance: `/` shows exactly "Sample interfaces"/"All components" under "Home", clicking "Sample interfaces" moves that `h2` from `y: 791` to `y: 60` (a real scroll, URL stays on `/` and gains `#sample-interfaces`, no route change); `/docs` unchanged (still its original 3 sub-items under "Overview", "Home" now present with none); a component page shows both "Home" and "Overview" as plain links with zero sub-items between them

## Status / trigger

Landed 2026-09-18

## Size

S–M

## Detail

User-requested, clarifying an initially-ambiguous report this session spent significant effort trying (and failing) to reproduce as a bug in row 62's own feature before the user's follow-up message pinned down the actual page in question; delegated to a single Sonnet subagent (file: `main.rs` only), independently re-verified — cargo/lint, the existing suite, and fresh from-scratch scroll-behavior/structure measurements on all three page types — before landing
