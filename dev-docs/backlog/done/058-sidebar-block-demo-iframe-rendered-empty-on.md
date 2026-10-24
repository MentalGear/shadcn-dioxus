---
id: 58
title: "Sidebar block-demo iframe rendered empty on live production; homepage Sidebar card now exposes live config…"
status: "Landed 2026-09-15"
size: "M"
area: build
updated: 2026-09-15
---

# Row 58: Sidebar block-demo iframe rendered empty on live production; homepage Sidebar card now exposes live config…

## Item

**`Sidebar` block-demo iframe rendered empty on live production; homepage Sidebar card now exposes live config for the real site nav -- LANDED 2026-09-15, user-reported and user-requested.** `/component/?name=sidebar&` showed the demo's settings panel but never the `Sidebar` preview (no `[data-slot="sidebar"]` node on any collapse/side combination).

## Status / trigger

Landed 2026-09-15

## Size

M

## Detail

Both reports came from direct testing of the live site; delegated to a Sonnet subagent across three rounds (bug 1 plus feature 2,
the SSG-race investigation, the card-overflow fix), each re-verified against a real browser and a real `deploy-preview.sh` run.

- **Root cause of the empty preview.** `Sidebar`'s responsive check (`MOBILE_BREAKPOINT = 768`, `use_is_mobile()`) reads
  `window.innerWidth` of the document it runs in, which for an `<iframe>` is the iframe's own window. Once row 56's site-nav
  sidebar narrowed the content column, the block demo iframe (`BlockComponentVariantHighlight`, `width: 100%`) measured ~684 px,
  under the breakpoint, so `Sidebar` took its mobile branch (a closed `Sheet`, nothing painted). Proven on the live site by
  widening the iframe 684 to 820 px through the `resize` path (sidebar count 0 to 1). Fixed with `min-width: 820px` on the shared
  iframe plus `overflow-x: auto` on its wrapper (`html,body{overflow-x:clip}` would otherwise clip silently): one construction
  point covering every `ComponentType::Block` demo.
- **Feature: a live homepage Sidebar card.** The card used to link out. `DocsLayout` takes optional `side`/`collapsible`
  signals (other call sites keep their local defaults); `Home()` provides them through a `HomeSidebarControls` context;
  `ComponentGalleryPreview`'s `Block` branch, only for `sidebar` (`try_consume_context`, degrading to the old link elsewhere), shows
  `SidebarGalleryCardControls`, which writes straight to those signals so the real site sidebar reconfigures live. No state leaks
  across navigation.
- **A second layout bug exposed while verifying.** At 1280 px the card's controls were clipped off-screen and unclickable. Cause: a
  flexbox pitfall, `.dx-sidebar-inset` is `flex: 1 1 0%` with no `min-width` override, so its automatic minimum is its content's
  min-content width (the pre-existing `WidgetMasonry` widget, found by DOM bisection); `overflow-x: clip` hid the overflow. Fixed at
  the one flex item (`min-width: 0` on `.dx-sidebar-inset`), not at the content that exposed it.
- **A pre-existing `dx build --ssg` race, guarded.** A 9-build A/B (alternating branch and baseline, full `rm -rf target/dx/preview`)
  showed 100% correlation between build speed and correctness and 0% with source content: builds that recompiled `preview` were
  right, warm-cache builds (~10 s) left every non-home SSG route without its `<script type="module">` hydration bootstrap tag, on
  branch and baseline alike (a dx-cli / dioxus-fullstack race, not fixable from this app). `scripts/deploy-preview.sh` now greps
  every built route's `index.html` for the bootstrap tag and exits 1 with the route list before touching `/docs` (both halves
  verified for real). This is the first of the stale-or-incomplete-SSG class that rows 98, 100 and 116 later met; the
  construction is `scripts/build-ssg.sh` plus `ssg-verify.sh`.
- **Verified by execution** (re-checked by the orchestrating session after two rounds of mis-attributed subagent reports): cargo
  check, fmt and the composition, class-prefix and css-literals guards clean; a clean `deploy-preview.sh` run passed its new check
  (6/6 routes); iframe `innerWidth` 820-822 with sidebar count 1 for all three Sidebar variants; the card's buttons clickable
  without `force`, `data-side` flipping live, `/docs` still defaulting `left` afterwards; `scrollWidth <= clientWidth` at 1280 px.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/058-sidebar-block-demo-iframe-rendered-empty-on.md`.*
