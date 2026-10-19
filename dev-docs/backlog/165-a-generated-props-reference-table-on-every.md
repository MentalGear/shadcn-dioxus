---
id: 165
title: "A generated props reference table on every component's docs page, produced in preview/build.rs (~100 public…"
status: "LANDED 2026-10-19 (generator + gate + spec; props structs only, events and parts stay in docs.md)"
size: "M (the generator is S-M; writing ~100…"
area: build
updated: 2026-10-05
---

# Row 165: A generated props reference table on every component's docs page, produced in preview/build.rs (~100 public…

## Item

A generated props reference table on every component's docs page, produced in `preview/build.rs` (~100 public props are undocumented today: select 31, calendar 18, date_picker 14)

## Status / trigger

**LANDED 2026-10-19** (generator + gate + spec; props structs only, events and parts stay in docs.md)

## Size

M (the generator is S-M; writing ~100 doc comments is the bulk)

## Detail

**The gap.** A component's docs page (`preview/src/components/<name>/docs.md`) describes usage, but nothing lists the props a component takes: name, type, default, what it does. The audit counted ~100 public props with no doc comment, concentrated in `select` (31), `calendar` (18) and `date_picker` (14); a reader has to open the source. **Construction (so a prop cannot ship undocumented):** `preview/build.rs`, which already turns each component's `docs.md` and `component.json` into `docs.html` and `description.txt` in `OUT_DIR`, also reads the `#[derive(Props)]` structs of the component's primitive (`primitives/src/<name>.rs`) and its styled wrapper (`preview/src/components/<name>/component.rs`) and emits a props table (name, type, `#[props(default)]` value, the field's doc comment), appended after the examples. Fail the build, or a `scripts/check-*.sh` gate (added to `scripts/run-gates.sh` and CLAUDE.md's gate sentence), when a public prop field has no doc comment, so the ~100 get written once and the class stops. **Cautions:** `build.rs`'s only parser dependency today is `pulldown-cmark` (no `syn`), so decide in the lane between a small scan of the derive structs and `syn` as a new build-dependency; register `cargo:rerun-if-changed` per source file (row 92's stale `OUT_DIR` class); `#[props(extends = ...)]` spreads and `ReadSignal<T>` need display rules (show the inner type, say "plus the global attributes"); and `build.rs` must be REWRITTEN, not touched, for the `dx serve` watcher to notice (CLAUDE.md). **Sequence:** write the doc comments first for the three big components (one docs lane per component, disjoint files), the generator after, or the gate fails on day one. **Does not cover:** events and parts (the table lists props; a component's parts stay in `docs.md`).

**Update 2026-10-19 (LANDED).** Re-count first: the audit's ~100 were already mostly written (select, calendar and date_picker had none missing); the scan found **55** undocumented fields, all in styled wrappers (combobox 16, command 15, avatar 8, tag_group 6, pagination 7 incl. three private `onclick`/`onmousedown`/`onmouseup`, table, badge, color_picker 1 each), now documented (`///` lines only). **Construction:** one std-only scanner, `preview/props_scan.rs` (a small scan, not `syn`: only column-0 `#[derive(.. Props ..)] pub struct` items are read, a field is what sits between top-level commas, so no new build-dependency and the gate can build it with bare `rustc`), is `#[path]`-included by both `preview/build.rs` and the gate helper `scripts/props-scan/main.rs`, so the page and the gate cannot disagree. `build.rs` appends a closed "Props reference" disclosure (`id="docs-props-reference"`, a `docs.md` declaring the same `##` fails the build) to every component's `docs.html`, one table per props struct of `src/components/<name>/component.rs` and `primitives/src/<name>.rs` / `<name>/**`: name, type, default, doc. Display rules: `ReadSignal<T>`/`Option<T>` show `T`; optional by type shows `None` (dioxus defaults `Option<T>` and `ReadSignal<Option<T>>`); bare `#[props(default)]` shows the type's default (`false`, `""`, `no-op`, ...); `default = ReadSignal::new(Signal::new(x))` shows `x`; no default shows `required` (`children: Element` shows `empty`, dioxus defaults it); `extends = ..` is the sentence "Plus the global attributes." Per-file `cargo:rerun-if-changed` on every scanned source. **Gate:** `scripts/check-props-docs.sh` (in `run-gates.sh`) fails any prop field, pub or not (the builder setter is public either way), without a doc comment; seeded violation (deleted BadgeProps.variant doc) exits 1 naming `BadgeProps.variant`, restored exits 0. Table styling: `.dx-docs-props*` in `preview/assets/main.css`. Spec: `playwright/docs-props-table.spec.ts`. **Not covered:** `#[component] fn` arguments (props declared as function parameters rather than a derive), events and parts. CLAUDE.md's gate sentence still needs `check-props-docs` added (main loop).
