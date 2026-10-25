---
id: 432
title: "Anchor engine selects on one data attribute, not nine hand-kept marker classes"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P7)."
size: "M"
area: primitives
updated: 2026-10-25
---

# Row 432: The anchor engine selects on one attribute, not a list of marker classes

## Item

A floating panel that asks to be anchored is anchored, but today the engine's stylesheet names ten `.dx-anchor-<kind>[popover]` classes in dozens of selector lines, so a new anchored overlay is a second edit in a list far from it. A missing entry renders un-anchored, a visual failure found late. Construction: one attribute, `data-dx-anchored="<kind>"`, written by `anchored_content_attributes` (already called by 25 sites, the one writer, as `data-dx-modality` is). The engine selects `[data-dx-anchored][popover]`. A new anchored overlay is anchored by calling the function it must already call.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P7).

## Size

M

## Detail

Source: [final sweep note, P7](../research/final-sweep-2026-10-25.md). Primitives `top_layer`, nine overlay primitives, one browser spec run.

- **Where (verified):** `primitives/src/top_layer/styles.rs:157-158` and `:164-172` write the group `.dx-anchor-tooltip[popover], .dx-anchor-hover-card[popover], .dx-anchor-popover[popover], .dx-anchor-popover:modal, ... .dx-anchor-combobox[popover]`, then again once per `data-side` (`:208-261` and onward) and per `data-align`. Counts from `grep -o "dx-anchor-[a-z-]*"` over primitives: `dx-anchor-popover` 53, `dropdown-menu` 29, `navbar` 26, `tooltip` 25.
- **Why the list exists:** the doc at `styles.rs:45-66` says a hashed CSS-module class would not match the engine's `@supports` rules (`dev-docs/issues/css-module-supports-scoping.md`), so every consumer carries a plain hand-written class.
- **No static gate checks completeness.** `check-anchored-keyframes.sh` is about `transform` in keyframes; the only checks are browser specs (`oracle/tier2-html/top-layer.spec.ts` and the per-component ones).
- **Root cause:** the marker is both the scope ("anchor me") and the name ("I am a tooltip"), so the engine enumerates names.
- **Subsumes:** the ten-name lists and the omit-one-name failure. **Does not subsume:** per-kind differences (hover card's own rules at `styles.rs:268-301`), and the point-anchored context menu (`dx-anchor-context-menu`, deliberately not in the list).
- **Gate:** `check-anchor-marker.sh` (new): every file that calls `use_anchor_position_fallback` also calls `anchored_content_attributes`, and no `dx-anchor-` class literal appears outside `top_layer/`. Browser half: the existing top-layer spec.
- **Check first (inferred):** that per-kind CSS differs only in hover card. A first pass is a grep of each non-grouped selector in `styles.rs` (686 lines, of which only the first hover-card rules and the grouped selectors were read).
