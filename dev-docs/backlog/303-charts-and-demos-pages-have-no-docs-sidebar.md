---
id: 303
title: "Charts and Demos pages have no docs sidebar"
status: "OPEN (owner question 2026-10-19)."
size: "S–M"
area: preview
updated: 2026-10-19
---

# Row 303: Charts and Demos pages have no docs sidebar

## Item

Why is the docs menu different on the charts page (and the demos page)?

## Status / trigger

OPEN (owner question 2026-10-19).

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
