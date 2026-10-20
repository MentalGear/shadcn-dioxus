---
id: 303
title: "Charts and Demos pages have no docs sidebar"
status: "LANDED 2026-10-20 (lane charts-page, branch lane/charts-page). ChartsPage and Demos are DocsLayout…"
size: "S–M"
area: preview
updated: 2026-10-20
---

# Row 303: Charts and Demos pages have no docs sidebar

## Item

Why is the docs menu different on the charts page (and the demos page)?

## Status / trigger

**LANDED 2026-10-20 (lane `charts-page`, branch `lane/charts-page`).** `ChartsPage` and `Demos` are `DocsLayout` children (`DocsNavActive::Charts` / `::Demos`); the sidebar's "Start" group gained "Demos" and "Charts" items (Charts is current on every chart type via `NavLink`'s `active`), the shell's `SidebarInset` is the one `<main>`, and the "No sidebar on this page" comments are gone. The chart grid's columns now follow the page's own width (`@container charts-page`: 2 from 640px of content, 3 from 960px) because the sidebar takes 272px at 1280px. Specs: `charts_gallery.spec.ts` ("the charts and demos pages have the docs sidebar", the column test), `nav_link.spec.ts`. Screenshots: `/home/user/tgt/shots-charts-page/{before,after}-{charts,demos}-{1280,390}.png`. Originally: OPEN (owner question 2026-10-19).

## Size

S–M

## Detail

- **Why:** routes that render through `DocsLayout` (Home, `/docs`, every component page) get its `SidebarProvider`, so they show the sidebar and the navbar's sidebar trigger.
- `ChartsPage` (`charts_gallery.rs:~358`) and `Demos` (`main.rs:1632-1637`) render a bare `Navbar {}` with no `DocsLayout`. The navbar's `try_consume_context::<SidebarCtx>()` finds no sidebar, so the trigger and the menu silently disappear.
- It was never decided against; those pages were simply built as standalone galleries. shadcn's own `/charts` is also sidebar-less, but the inconsistency here is ours.
- **Target:** wrap both pages in `DocsLayout`.
  - Charts gets a new `DocsNavActive::Charts`, highlighting the Charts sidebar group (row 117).
  - Demos gets its own active item.
  - Re-check the chart grid's width at 1280 and 390 px with the sidebar taking space.
- Remove the "No sidebar on this page" comments.
