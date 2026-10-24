---
id: 186
title: "shadcn utilities: scroll-fade and shimmer are built but not shipped as documented, public utilities"
status: "LANDED 2026-10-19 (PR #107, round 4: e3ffda0c /docs Utilities section and utilities.spec.ts…"
size: "M"
area: docs
updated: 2026-10-20
---

# Row 186: shadcn utilities: scroll-fade and shimmer are built but not shipped as documented, public utilities

## Item

**shadcn utilities: `scroll-fade` and `shimmer` are built but not shipped as documented, public utilities**

## Status / trigger

**LANDED 2026-10-19** (PR #107, round 4: e3ffda0c `/docs` Utilities section and `utilities.spec.ts`; 3ba8d59f the scroll-fade `fade` prop 2026-10-14). Not built, no owner request: a components-free install of `dx-effects.css` (`shadcn-dioxus add scroll_area` or a hand copy is the documented path). Closed in the 2026-10-20 backlog sync.

## Size

M

## Detail

shadcn documents two utilities (`scroll-fade`, https://ui.shadcn.com/docs/utils/scroll-fade, and `shimmer`). We had both built as
`dx-effects.css` (`.dx-scroll-fade[-x|-y|-t|-b|-l|-r|-s|-e|-none]` with `--dx-scroll-fade-size`, `-{t,b,s,e}-size` and `-reveal`;
`.dx-shimmer[-once|-reverse|-none]` with `--dx-shimmer-color|-duration|-spread|-angle`), but not shipped as documented, public
utilities. The fade is a `mask-image` on the scroller itself driven by CSS scroll-driven animations (`animation-timeline: scroll()`),
no JavaScript, with a static fallback in `@supports not`. Motion gating was already by construction (`use_motion` where a
component applies the shimmer, `check-motion-gating.sh`, the `dx-shimmer-sweep` allowlist entry in
`check-infinite-animations.sh`, its own reduced-motion block).

**The gap, as found:** no public documentation or standalone demo; the file shipped only as a `globalAssets` entry of `attachment`,
`marker` and `message_scroller`; a bare `.dx-shimmer` on another element has nothing to pause it (`use_motion` is applied by the
host, not by the class); and tests reached the utilities only through Attachment and Marker.

**Landed in two steps.**

- *Scroll-fade part (2026-10-15, lane `scrollfade`, `3ba8d59`).* A `ScrollFade` enum (`primitives/src/scroll_area.rs`: `None`
  default, `Y`, `X`, `Top`, `Bottom`, `Start`, `End`) as a `fade` prop on ScrollArea, VirtualList and MessageScrollerViewport
  (default `Bottom` there, preserving its look), always applied on the scrolling element; `dx-effects.css` joined `globalAssets`
  of `scroll_area` and `virtual_list`; documented in `scroll_area/docs.md` ("Edge fade"). It also fixed a utility bug: a scroller
  that stopped overflowing after being scrolled kept its start edge faded (`animation-fill-mode: both` on an inactive timeline).
  `scroll-fade.spec` 14/14. Owner Q&A: no general content scroller and no wrapper component (the mask must sit on the scrolling
  element; a wrapper would mask chrome and need `timeline-scope`).
- *Utilities docs (2026-10-19, lane `utilities`, PR #107, `e3ffda0c`).* `preview/src/utilities_docs.rs` is the new "Utilities"
  section of `/docs` (after Motion): Install, Scroll fade (class and variable tables, a long list, a horizontal strip), Shimmer
  (tables, the gating recipe, a streaming status line gated by `use_motion`). The recipe: spread `use_motion().attributes()` on
  the element with `dx-shimmer`, or use a `data-dx-motion` ancestor, with `dx-shimmer-none` to finish; `dev-docs/motion.md`
  states it too. `playwright/utilities.spec.ts` covers bare scroll-fade (overflow-only, start/middle/end, no overflow after
  scroll, `-none` in either class order, one-edge `-t`/`-b`, the size variable, horizontal and RTL) and shimmer (running, paused
  off screen and resuming, paused by an ancestor, `-none`, reduced motion).

**Not built.** A components-free install of `dx-effects.css`: the CLI's `add` copies a component's `globalAssets` and `init` copies
only the theme, so a new entry would need a component directory. The documented path is `shadcn-dioxus add scroll_area` (or any
component listing the file), else copy the file by hand and link it. No Rust hook: scroll-fade needs no scroll listener.
Related: `dev-docs/archive/research/shadcn-catalog-2026-10-04.md`.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/186-shadcn-utilities-scroll-fade-and-shimmer-are.md`.*
