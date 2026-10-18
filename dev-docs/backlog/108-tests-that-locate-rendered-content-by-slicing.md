---
id: 108
title: "Tests that locate rendered content by slicing an HTML string from a specific attribute-string match break…"
status: "Found 2026-09-25 (round 8) while landing row 93; two occurrences in one round makes it a class per…"
size: "BUILT 2026-09-26 (dab751c); was…"
area: tests
updated: 2026-09-25
---

# Row 108: Tests that locate rendered content by slicing an HTML string from a specific attribute-string match break…

## Item

Tests that locate rendered content by slicing an HTML string from a specific attribute-string match break when attribute order changes — the same class showed up twice this round (`calendar.rs`'s unit test, chart's `tooltip.rs::tooltip_fragment()`), both caused by row 93's `merge_attributes` construction sorting its output by attribute name

## Status / trigger

Found 2026-09-25 (round 8) while landing row 93; two occurrences in one round makes it a class per `CLAUDE.md`'s own rule, not two coincidences

## Size

**BUILT 2026-09-26 (`dab751c`)**; was: proposed, not built (that lane owned `dev-docs/**` only)

## Detail

**2026-09-26 (audit batch): BUILT** in `dab751c` -- `primitives/src/test_support.rs` (`#[cfg(test)]` start-tag parser, `find_elements`/`inner_html`); migrated both named instances plus a third latent one in `tooltip.rs`'s hydration-parity test. Proven with `merge_attributes` temporarily reversed: migrated tests still pass. **Root cause:** a test that does something like `html[html.find("aria-label=\"...\"")..]` is implicitly assuming a stable attribute order that nothing in Dioxus's own contract actually guarantees (`dioxus-core`'s own VNode data model has no ordering invariant on its dynamic-attribute slot to begin with — see row 93's root-cause account) — `merge_attributes` sorting by name was simply the first thing to make that latent assumption visible, and any future change to attribute construction order could do it again. **Proposed fix by construction, not per-instance:** a shared test helper (e.g. in a `playwright`/Rust-test-support module already used across specs) that parses a rendered start tag into an attribute name→value map — a real, if minimal, tag-attribute parser — rather than any test slicing a string match out of raw HTML. Every call site that currently slices from an attribute match would switch to `attrs.get("aria-label")`-style lookups, which are order-independent by construction and can never break this way again, regardless of what future attribute-ordering change lands. Two instances subsumed by this construction: `calendar.rs`'s unit test (was slicing from `aria-label="…"`) and chart's `tooltip.rs::tooltip_fragment()` (was slicing from a literal match instead of the tag's own `<`) — both were re-anchored on the tag start as an interim fix, not rewritten onto the proposed helper, so they remain individually correct but do not yet demonstrate the construction. Not built this round: this lane owns `dev-docs/**` and the four playwright config files only, not test-support code [BUILT 2026-09-26, see the first sentence]
