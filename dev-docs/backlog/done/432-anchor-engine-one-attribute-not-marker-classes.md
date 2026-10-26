---
id: 432
title: "Anchor engine selects on one data attribute, not nine hand-kept marker classes"
status: "DONE 2026-10-26 (lane anchor-attr-432). Owner approved 2026-10-25 (row 365 sweep, P7)."
size: "M"
area: primitives
updated: 2026-10-26
---

# Row 432: The anchor engine selects on one attribute, not a list of marker classes

## Item

A floating panel that asks to be anchored is anchored, but today the engine's stylesheet names ten `.dx-anchor-<kind>[popover]` classes in dozens of selector lines, so a new anchored overlay is a second edit in a list far from it. A missing entry renders un-anchored, a visual failure found late. Construction: one attribute, `data-dx-anchored="<kind>"`, written by `anchored_content_attributes` (already called by 25 sites, the one writer, as `data-dx-modality` is). The engine selects `[data-dx-anchored][popover]`. A new anchored overlay is anchored by calling the function it must already call.

## Status / trigger

DONE 2026-10-26 (lane anchor-attr-432). Owner approved 2026-10-25 (row 365 sweep, P7).

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

## Outcome

Done 2026-10-26, lane `lane/anchor-attr-432`.

- **One writer.** `anchored_content_attributes` now takes an `AnchorKind` (`primitives/src/top_layer/attributes.rs`) and writes `data-dx-anchored="<kind>"` last, so a caller's own attribute cannot unscope it. The twelve call sites pass their kind (the dropdown and context-menu submenus pass `DropdownMenu`, the item-aligned Select passes `SelectItemAligned`, which writes nothing: its own script places it); the ten `class: "dx-anchor-<kind>"` literals and the Popover's two `format!("{} dx-anchor-popover")` classes are gone.
- **The engine enumerates nothing.** `ANCHOR_POSITIONING_CSS_JS_LITERAL` went from 686 lines of selector groups to rules scoped by `[data-dx-anchored]:is([popover], :modal)`; the only kind it names is `hover-card` (wider gap), excluded from the generic rules through a zero-specificity `:where(:not(...))`. Specificity is unchanged by construction: `(0,2,0)` for the base, `+1` per `data-side`/`data-align`, as the class plus `[popover]` selectors were. The dead `.dx-anchor-popover:modal[popover]...:dir(rtl)` selectors (a modal dialog never has `popover`) were dropped, and the RTL rules stay `[popover]`-only so the modal arm is not newly matched.
- **Gate.** `scripts/check-anchor-marker.sh` (in `run-gates.sh`, described in `dev-docs/gates.md`): a file that calls `use_anchor_position_fallback` calls `anchored_content_attributes` at least as often; no `dx-anchor-<name>` class literal or `data-dx-anchored` write outside `top_layer/`; every rule of the engine stylesheet is scoped by the attribute and names no kind but `hover-card`. Comments are not read, so component sheets' prose was left alone. A 15-case self-test (mutations red, the committed engine green) runs with the gate; the allowlist is empty and keeps the escape hatch.
- **Check first (inferred item), resolved.** Per-kind CSS differs only in the hover card: every one of the 26 rules of the old `@supports` block was classified by a script as covering all nine kinds, the eight non-hover-card kinds (plus the modal Popover), or the hover card alone, with no other grouping.
- **Evidence of no positional change:** a Playwright probe opened every anchored overlay of the `top_layer` fixture and of the tooltip, hover card, popover, dropdown menu, context menu, menubar, select, combobox, navigation menu, navbar, date picker and color picker pages, in LTR and RTL, and recorded each open panel's rect and the computed position, inset, margin, transform, min-width, `--dx-anchor-width` and try-fallbacks; identical before and after (counts in the lane report).
- **Does not subsume:** the hover-card gap, and the point-anchored `ContextMenuContent` (it never calls `anchored_content_attributes`).
