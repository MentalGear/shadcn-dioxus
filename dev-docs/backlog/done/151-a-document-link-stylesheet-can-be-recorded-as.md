---
id: 151
title: "A document::Link stylesheet can be recorded as present and never inserted: a subtree torn down in the turn it…"
status: "FIXED 2026-10-04, merged (66b19d3, PR #84 / 001a5f0; preview/src/eager_head.rs)"
size: "M"
area: theme
updated: 2026-10-04
---

# Row 151: A document::Link stylesheet can be recorded as present and never inserted: a subtree torn down in the turn it…

## Item

A `document::Link` stylesheet can be recorded as present and never inserted: a subtree torn down in the turn it first renders drops its head elements, then every later `Link` for the same href is skipped as a duplicate

## Status / trigger

**FIXED 2026-10-04, merged** (`66b19d3`, PR #84 / `001a5f0`; `preview/src/eager_head.rs`)

## Size

M

## Detail

**A class, third occurrence:** row 46 (`main.css` after `ComponentBlockDemo`'s redirect; its fix moved `GlobalHead` into `AppLayout`, sound for those sheets only), an earlier page with no `Button` stylesheet, and now `/component/X/` rendering without the header `LanguageSelect` and the theme picker's `Popover` stylesheets (the picker's panel computed `border 3px / radius 0` instead of `1px / 10px`). **Mechanism, read from dioxus 0.7.9 source:** `Link` records `href\|rel` in a root-context de-duplication set at render (`dioxus-document elements/link.rs:123-137`) but the web document appends the `<link>` from `queue_effect` on the Link's own scope (`dioxus-web document.rs:160-164`); `Runtime::remove_scope` drops a removed scope's queued effects (`dioxus-core runtime.rs:190-193`), and effects only run once no scope is dirty. The trigger is an ancestor's first-run `use_effect` that dirties a scope unmounting the subtree: the legacy `/component/?name=X` shell renders `Navbar`, then `nav.replace(..)`s from its own effect. **Construction:** `EagerHeadDocument` wraps the platform `Document` and leaves `create_link/meta/script/style` at the trait defaults (`create_element_in_head` through `eval`, which `WebDocument` runs synchronously), so the `<link>` is in `<head>` before `Link` returns, in the same breath the set records it; `create_head_component` (the hydration handshake) is delegated untouched; installed once in `App` via `use_eager_head_document()` on every client build and never on the server, where `ServerDocument` must keep collecting the SSR head. One install covers every component, including the wrappers `dx components add` copies out. **Guards:** `scripts/check-eager-head-document.sh` (static: the wrapper exists, never uses `queue_effect`, `App` installs it) and `playwright/oracle/tier2-html/stylesheets-present.spec.ts` (behavioural: every rendered component's stylesheet is applied on fresh loads of every route type). The script's header cites this finding by name rather than by row; this row is its anchor.
