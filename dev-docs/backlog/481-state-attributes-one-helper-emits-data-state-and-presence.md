---
id: 481
title: "State attributes: one helper emits data-state and Base UI presence attributes"
status: "BACKLOG. From the scan of 2026-10-28."
size: "M"
area: primitives
updated: 2026-10-28
---

# Row 481: State attributes, one helper for `data-state` and presence attributes

## Item

Components emit the Radix vocabulary (`data-state="checked"`), while shadcn's `base-nova` classes style on Base UI presence attributes (`data-checked`, `data-unchecked`, `data-open`, `data-closed`, `data-pressed`, `data-active`). One helper emits both, presence attributes are omitted when false, and a gate makes the helper the only way to write a state attribute. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix B (cross-cutting 1 and 2).

## Status / trigger

BACKLOG. From the scan of 2026-10-28.

## Size

M

## Detail

- **The class.** Boolean `data-*` attributes are emitted valued (`data-x=true|false`, dioxus-ssr renders a `Bool` as `name=value`), so a presence selector (`[data-x]`, Tailwind `data-open:`) would match the false case, and pasting base-nova class strings is unsafe. The scan reported this as a live bug; the main loop's check says it is not: the only presence-selector user (the questionnaire) already omits false values (`then_some`). It is a convention to enforce, not a defect to fix.
- **Construction.** One helper beside `merge_attributes` in `primitives/src/lib.rs`: given a state (checked, open, pressed, active, indeterminate, ...), it emits `data-state` and the matching Base UI presence attribute (`data-checked` or `data-unchecked`, `data-open` or `data-closed`, `data-pressed`, ...), and emits the presence attribute only when true. A gate (`check-*.sh`, registered in `run-gates.sh`, then `scripts/gates-doc.py --write`) fails any component that writes a `data-state` or a presence attribute outside the helper, and any valued boolean read by a presence selector.
- **Migration.** Dual-emit first (no CSS change). Then move CSS and specs to the presence attributes gradually, `data-state` last: 94 `data-state` selectors in 29 preview CSS files and 88 Playwright files depend on it today, and the `tier3-radix` oracle compares against it.
- **Files owned:** `primitives/src/lib.rs` (helper), every primitive's `attributes!` site for state, the gate script, then preview CSS per component.
- **Proof:** a unit test of the helper (both attributes, presence omitted when false); the gate with its self-test; the existing specs green on dual-emit.
- **Related:** [row 482](./482-control-gaps-slider-form-participation-read-only-and-small-attributes.md) adds `data-pressed` on Toggle and `read_only`, which use this helper.
