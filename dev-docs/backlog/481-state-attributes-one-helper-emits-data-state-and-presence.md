---
id: 481
title: "State attributes: one helper emits data-state and Base UI presence attributes"
status: "IN PROGRESS (2026-10-29), lane state-attributes-481. Landed: primitives/src/state_attrs.rs (the…"
size: "M"
area: primitives
updated: 2026-10-29
---

# Row 481: State attributes, one helper for `data-state` and presence attributes

## Item

Components emit the Radix vocabulary (`data-state="checked"`), while shadcn's `base-nova` classes style on Base UI presence attributes (`data-checked`, `data-unchecked`, `data-open`, `data-closed`, `data-pressed`, `data-active`). One helper emits both, presence attributes are omitted when false, and a gate makes the helper the only way to write a state attribute. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix B (cross-cutting 1 and 2).

## Status / trigger

IN PROGRESS (2026-10-29), lane `state-attributes-481`. Landed: `primitives/src/state_attrs.rs` (the helper), every component under `primitives/src` (chart excepted) and the preview wrappers that emit state attributes themselves go through it, `scripts/check-state-attributes.sh` (the gate, with a shrink-only baseline of the legacy valued calls), and [`dev-docs/state-attributes.md`](../state-attributes.md) (the vocabulary and the per-attribute table). `data-state` is emitted unchanged beside the new presence attributes; no CSS or spec moved.

Open: the valued booleans (`data-disabled` and the others the gate counts in `scripts/check-state-attributes.baseline.tsv`) stay `true|false` until the stylesheets and specs that select `="true"` / assert `data-disabled="false"` move to presence; then `data-state` last. `SwitchThumb` has no context to read yet, so it renders no state attribute.

## Size

M

## Detail

- **The class.** Boolean `data-*` attributes are emitted valued (`data-x=true|false`, dioxus-ssr renders a `Bool` as `name=value`), so a presence selector (`[data-x]`, Tailwind `data-open:`) would match the false case, and pasting base-nova class strings is unsafe. The scan reported this as a live bug; the main loop's check says it is not: the only presence-selector user (the questionnaire) already omits false values (`then_some`). It is a convention to enforce, not a defect to fix.
- **Construction.** One helper beside `merge_attributes` in `primitives/src/lib.rs`: given a state (checked, open, pressed, active, indeterminate, ...), it emits `data-state` and the matching Base UI presence attribute (`data-checked` or `data-unchecked`, `data-open` or `data-closed`, `data-pressed`, ...), and emits the presence attribute only when true. A gate (`check-*.sh`, registered in `run-gates.sh`, then `scripts/gates-doc.py --write`) fails any component that writes a `data-state` or a presence attribute outside the helper, and any valued boolean read by a presence selector.
- **Migration.** Dual-emit first (no CSS change). Then move CSS and specs to the presence attributes gradually, `data-state` last: 94 `data-state` selectors in 29 preview CSS files and 88 Playwright files depend on it today, and the `tier3-radix` oracle compares against it.
- **Files owned:** `primitives/src/lib.rs` (helper), every primitive's `attributes!` site for state, the gate script, then preview CSS per component.
- **Proof:** a unit test of the helper (both attributes, presence omitted when false); the gate with its self-test; the existing specs green on dual-emit.
- **Related:** [row 482](./482-control-gaps-slider-form-participation-read-only-and-small-attributes.md) adds `data-pressed` on Toggle and `read_only`, which use this helper.
