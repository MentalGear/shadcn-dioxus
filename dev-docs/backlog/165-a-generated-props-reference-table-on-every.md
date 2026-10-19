---
id: 165
title: "A generated props reference table on every component's docs page, produced in preview/build.rs (~100 public…"
status: "NARROWED 2026-10-19 (backlog review): primitives Props are fully documented (missing_docs on)…"
size: "M (the generator is S-M; writing ~100…"
area: build
updated: 2026-10-19
---

# Row 165: A generated props reference table on every component's docs page, produced in preview/build.rs (~100 public…

## Item

A generated props reference table on every component's docs page, produced in `preview/build.rs` (~100 public props are undocumented today: select 31, calendar 18, date_picker 14)

## Status / trigger

**NARROWED 2026-10-19 (backlog review): primitives Props are fully documented (missing_docs on); about 63 of 334 preview wrapper props lack docs; the generated table is still to build**

## Size

M (the generator is S-M; writing ~100 doc comments is the bulk)

## Detail

**The gap.** A component's docs page (`preview/src/components/<name>/docs.md`) describes usage, but nothing lists the props a component takes: name, type, default, what it does. The audit counted ~100 public props with no doc comment, concentrated in `select` (31), `calendar` (18) and `date_picker` (14); a reader has to open the source. **Construction (so a prop cannot ship undocumented):** `preview/build.rs`, which already turns each component's `docs.md` and `component.json` into `docs.html` and `description.txt` in `OUT_DIR`, also reads the `#[derive(Props)]` structs of the component's primitive (`primitives/src/<name>.rs`) and its styled wrapper (`preview/src/components/<name>/component.rs`) and emits a props table (name, type, `#[props(default)]` value, the field's doc comment), appended after the examples. Fail the build, or a `scripts/check-*.sh` gate (added to `scripts/run-gates.sh` and CLAUDE.md's gate sentence), when a public prop field has no doc comment, so the ~100 get written once and the class stops. **Cautions:** `build.rs`'s only parser dependency today is `pulldown-cmark` (no `syn`), so decide in the lane between a small scan of the derive structs and `syn` as a new build-dependency; register `cargo:rerun-if-changed` per source file (row 92's stale `OUT_DIR` class); `#[props(extends = ...)]` spreads and `ReadSignal<T>` need display rules (show the inner type, say "plus the global attributes"); and `build.rs` must be REWRITTEN, not touched, for the `dx serve` watcher to notice (CLAUDE.md). **Sequence:** write the doc comments first for the three big components (one docs lane per component, disjoint files), the generator after, or the gate fails on day one. **Does not cover:** events and parts (the table lists props; a component's parts stay in `docs.md`).
