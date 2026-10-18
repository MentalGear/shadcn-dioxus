---
id: 62
title: "/docs sidebar's \"Start\" group had a single \"Overview\" link and no icons anywhere in the site nav — LANDED…"
status: "Landed 2026-09-17"
size: "M"
area: build
updated: 2026-09-17
---

# Row 62: /docs sidebar's "Start" group had a single "Overview" link and no icons anywhere in the site nav — LANDED…

## Item

**`/docs` sidebar's "Start" group had a single "Overview" link and no icons anywhere in the site nav — LANDED 2026-09-17, user-requested.** The user asked for the Overview page's sidebar to expose its own real content (jump-to-section links, generated from its actual headings, not hand-typed) plus icons across the sidebar. **Generated, not duplicated:** a single `const DOCS_SECTIONS: &[&str]` (`main.rs`) and one `docs_section_slug` helper (lowercases, collapses non-alphanumeric runs to one hyphen) are the one source of truth for the `/docs` page's three `h2` headings' visible text, their `id`s, and the sidebar sub-items' labels/`href`s — all four render sites read from the same array/fn, so they cannot drift out of sync with each other the way two hand-typed copies of the same string eventually would. **Wired through `DocsLayout`** via a new `#[props(default)] docs_sections: Option<&'static [&'static str]>`, following the exact `side`/`collapsible`/`default_open` pattern already established there — only `Docs()` passes `Some(DOCS_SECTIONS)`; `Home()`/`ComponentHighlight()` pass nothing and render the "Start" group exactly as before (verified structurally unchanged on both). **Nesting** reuses the existing `SidebarMenuSub`/`SidebarMenuSubItem`/`SidebarMenuSubButton` primitives (`sidebar/component.rs`) — the same ones `sidebar/variants/*/mod.rs`'s own demos already use for nested sub-lists — rather than inventing new markup. **"Jump there" is a plain `a { href: "#slug" }`** via the `as` escape hatch, not the router's `Link`, since this is a same-page anchor scroll, not a route change; `html { scroll-behavior: smooth }` was added (`main.css`, alongside the pre-existing `scroll-padding-top` rule for the sticky navbar) so the jump animates instead of cutting. **Icons** (all `dioxus_icons::lucide`, already imported elsewhere in this file): `BookOpen` on "Start", `FileText` on "Overview", `Hash` (the conventional anchor/TOC marker) on each heading-derived sub-item, and one distinct icon per `ComponentCategory` group (Forms → `SquareCheck`, Navigation → `Compass`, Overlays → `Layers`, Feedback → `Bell`, Disclosure → `ChevronsUpDown`, Data display → `LayoutGrid`) — every icon carries `aria-hidden="true"` (mirroring the dashboard's own `LucideIcon` convention) since each sits beside a visible text label, so axe has no accessible-name gap to flag. Deliberately out of scope: per-individual-component icons (30+ distinct components, a much larger and more subjective task) — only category-level and the new section links got one. `SidebarGroupLabel` gained a `gap` in its CSS (`sidebar/style.css`) to support this icon+label pairing — it had none, being text-only until now; `SidebarMenuButton`/`SidebarMenuSubButton` already had the right icon+label spacing from existing collapsed-icon-mode CSS, so nothing else was needed. **Verified by execution, independently re-checked by the orchestrating session (concurrently with row 63's unrelated Input OTP fix, disjoint files, both re-verified separately before landing):** `cargo check --workspace`/`cargo fmt --check` clean; `check-preview-composition.sh`/`check-dx-class-prefix.sh`/`check-css-literals.sh` all clean; `sidebar.spec.ts`/`navbar.spec.ts`/`preview.spec.ts` re-run, 13/13 passed; a fresh manual axe scan against `/docs` itself (not covered by any pre-existing spec) came back with **0 violations**; independently confirmed against a live `dx serve` instance: exactly 3 sub-items under "Start" with the right labels and one icon each, clicking "Add a component" moves that `h2` from `y: 998` to `y: 144` (a real scroll, not just a URL change, and the URL stays on `/docs`), `/` and a component page both show 0 sub-items under "Overview" (unaffected)

## Status / trigger

Landed 2026-09-17

## Size

M

## Detail

User-requested; delegated to a single Sonnet subagent (files: `main.rs`, `sidebar/style.css`, `main.css` only), independently re-verified — cargo/lint, the existing suite, and a fresh manual axe scan plus live scroll-behavior measurement not covered by any pre-existing test — before landing
