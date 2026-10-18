---
id: 56
title: "Docs site now dogfoods the real Sidebar component as its own navigation — LANDED 2026-09-15, user-requested.…"
status: "Landed 2026-09-15"
size: "M"
area: build
updated: 2026-09-15
---

# Row 56: Docs site now dogfoods the real Sidebar component as its own navigation — LANDED 2026-09-15, user-requested.…

## Item

**Docs site now dogfoods the real `Sidebar` component as its own navigation — LANDED 2026-09-15, user-requested.** The docs site's own nav was a hand-rolled `DocsSidebar` in `main.rs` (raw `<aside>`/`<nav>`/`<button>` elements, its own `dx-docs-sidebar-*` classes, its own toggle/backdrop) — a parallel implementation duplicating what the shipped `Sidebar`/`SidebarProvider`/`SidebarInset` already do, never dogfooded even on the site showcasing them. Replaced with a new `DocsLayout` composing `SidebarProvider`/`Sidebar`/`SidebarContent`/`SidebarInset`, mirroring the composition `sidebar/variants/main/mod.rs`'s own demo and the email-client dashboard already use (`SidebarMenuButton`'s `as` prop swaps in the real router `Link` for SPA navigation; `SidebarInset` supplies the page's one `<main>` landmark). Applied to all three site pages — `/docs`, each component page, and (newly) the **homepage** `/` itself, whose hero + component gallery previously had no sidebar nav at all and are now the `SidebarInset` content of a real sidebar layout. `DocsNavActive` replaces the old `Option<&'static str>` active-param, which conflated "on Docs" with "on Home" (both were `None`) — each page now gets its own distinct active state, verified via direct DOM query (not just visual) to confirm zero false-positive active links on the homepage. **A real cross-cutting CSS conflict found and fixed along the way:** `sidebar/style.css`'s desktop rendering assumes its wrapper IS the whole page (`position: fixed` column, `100svh` tall) — true for the two other call sites (`ComponentBlockDemo`, `EmailClientDashboard`), both rendered outside the site's own Navbar/Footer chrome; `DocsLayout` shares the page with those, so a fixed column would float over the footer past the fold. Re-scoped under `.dx-sidebar-wrapper.dx-docs-shell` to a `position: sticky` in-flow column with a width-collapse instead of the fixed-position slide-off, keeping the same collapse transition without fighting page chrome the component doesn't own. **Verified by execution:** `cargo check`/`cargo fmt --check` clean; `check-preview-composition.sh`/`check-dx-class-prefix.sh`/`check-css-literals.sh` all green; `preview.spec.ts` (axe on home + a component page), `sidebar.spec.ts`, `navbar.spec.ts` — 13/13 passed, independently re-run; active-nav-link state and footer/sidebar non-overlap after scrolling confirmed via direct DOM query and screenshot, not visual inspection alone

## Status / trigger

Landed 2026-09-15

## Size

M

## Detail

User-requested after two rejected placement options (a homepage-grid card, a small thumbnail) clarified the actual ask was site-wide nav dogfooding, not a demo-card change; delegated to a Sonnet subagent, independently re-verified before landing. `oracle/hydration-parity.spec.ts` not run — needs a separate SSG build lane orthogonal to this composition-only change
