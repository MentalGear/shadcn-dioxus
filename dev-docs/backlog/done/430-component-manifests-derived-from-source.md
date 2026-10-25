---
id: 430
title: "Component manifests derived from source, not hand-kept"
status: "LANDED (lane/manifests-430). Trigger for reopening: a manifest field that is computable from the…"
size: "M"
area: tooling
updated: 2026-10-25
---

# Row 430: Component manifests are derived from source, not hand-kept

## Item

Each of the 83 `preview/src/components/*/component.json` manifests and each `mod.rs` is hand-kept, and gates check them against the Rust files after the fact. Agreement between two authored files is a computation, not a decision. Construction: a generator writes the computed fields (`componentDependencies`, `cargoDependencies`, `globalAssets`, `exclude`, `mod.rs`) from the sources, and `run-gates.sh` runs `--check` for byte-identity, the `build-js-bundle` / `check-js-bundle` model. A manifest cannot disagree with the files it ships, because it is their output.

## Status / trigger

LANDED (lane/manifests-430). Trigger for reopening: a manifest field that is computable from the sources and is still hand-kept.

## Size

M

## Detail

Source: [final sweep note, P5](../../research/final-sweep-2026-10-25.md). Scripts and 83 JSON files, no product code.

- **Measured:** 76 manifests carry the identical `exclude: ["variants","docs.md","component.json"]` and `globalAssets: [theme]`; seven add `dx-effects.css`. 26 list `componentDependencies`; all list `cargoDependencies` with the same `dioxus-primitives` git block. `cat */mod.rs | sort | uniq -c`: 47 on two lines, 36 on one.
- **Already computed by gates:** `check-installed-paths.sh:21-53` (componentDependencies from `crate::components::<other>`, external crates of the closure, `mod.rs` shape, `exclude`/`globalAssets` existing); `check-cargo-dep-versions.sh`, `check-registry-url.sh`, and `check-component-catalog.sh` checks 2, 2b and 3.
- **Failures this caused** (CLAUDE.md): 27 missing `mod.rs` found only when a user's `cargo check` hit E0583; `kbd` and four more without `cargoDependencies` (E0433). That is why `registry-install-smoke.sh` (~6 GB, minutes) exists.
- **Construction:** `scripts/gen-component-manifests.sh`, or a `props_scan`-style Rust scanner (the precedent is `preview/props_scan.rs`). Hand-authored fields stay: `name`, `description`, `authors`. External-crate detection uses the same `cfg` rule the gate has.
- **Subsumes:** the "manifest says X, source says Y" halves 2 and 3 of `check-installed-paths`; the `cargoDependencies` floor check of `check-cargo-dep-versions`; `check-component-catalog` checks 2, 2b and 3 for the derived fields. **Does not subsume:** `registry-install-smoke.sh` (it tests dx itself), the `CATALOG` classification (`Origin::Shadcn|Extra` is a decision, not a computation), and the `document::Link` line in each component.
- **Gate:** `gen-component-manifests.sh --check` in `run-gates.sh`. The four existing gates stay as cross-checks until the generator has run clean for a batch, then lose their duplicated halves.

## Outcome

Landed on lane/manifests-430, as a script (not a `props_scan`-style Rust scanner): the logic the gates already computed was
Python over text, so it moved there, and a cargo build is not needed to run it (no target dir, seconds).

- `scripts/gen-component-manifests.sh [--check]` (a thin wrapper) runs `scripts/gen-component-manifests.py`. It writes, for
  every `preview/src/components/<dir>`: `mod.rs` (`mod component;\npub use component::*;\n`) and `component.json`
  (`exclude`, `globalAssets`, `componentDependencies`, `cargoDependencies`), and the root `component.json` `members`
  (every directory but `top_layer`, sorted). `name`, `description`, `authors` are read from the existing manifest; a new
  component starts as those three fields and the script fills in the rest. The release `rev` that
  `scripts/stamp-release-rev.sh` writes is the one other input and is carried over, never invented.
- Derivation, over the INSTALLED files (the directory minus `variants`, `docs.md`, `component.json`): sibling components
  from `crate::components::<x>`, `super::super::<x>` (only when `<x>` is a component directory) and
  `asset!("/src/components/<x>/..")`; external crates from the gate's `use`/`crate::` scan (cfg(test) modules dropped),
  mapped to Cargo.toml names, `dioxus-primitives` always, version from `Cargo.lock` (`=` for dioxus-icons and
  pulldown-cmark, bare otherwise); primitives `features` from the cargo-gated modules the files name; the theme asset
  when `style.css` exists and `dx-effects.css` when the files, or a primitives module they use (transitively through
  `crate::<module>`: `virtual_list` re-exports `scroll_area::ScrollFade`, whose `class()` writes `dx-scroll-fade-*`),
  emit a `dx-scroll-fade`/`dx-shimmer` class. No `assets.txt` was needed.
- Gate: `scripts/check-component-manifests.sh` (in `run-gates.sh`, group "Registry and installed components"; gates.md
  regenerated) runs `--check` and then a 12-mutation self-test on a scratch copy of the inputs. CLAUDE.md names no gate
  any more, so it needed no edit.
- Regenerated: 61 files differ from the previous tree, all explained: 37 `mod.rs` normalised to one trailing newline (36 had
  none, `card` two); the root members sorted; `cargoDependencies` order now `dioxus-primitives` first then alphabetical
  (breadcrumb, native_select, spinner); `componentDependencies` sorted (bar_chart, checkpoint, data_table, radial_chart and
  the others re-ordered); JSON layout normalised (sidebar-style one-item-per-line arrays now inline, `componentDependencies`
  after `cargoDependencies` in `dialog`). Content changes: `alert` and
  `drawer` no longer list `dioxus-icons` (neither uses it); `componentDependencies` entries nothing imports are gone
  (`chart_tooltip`, `line_chart`, `radar_chart` -> `card`; `color_picker` -> `popover`, it composes `dioxus_primitives::popover`;
  `pie_chart` -> `select`; `direction` -> `button`, `tabs`). `date_picker` keeps `popover`: it reaches it as
  `super::super::popover::*`, a path none of the four gates scanned (they only read `crate::components::`).
- Not done on purpose: the four gates keep their duplicated halves until this generator has run clean for a batch (the
  row's own rule); then drop `check-installed-paths` halves 2-3, the `cargoDependencies` floor check of
  `check-cargo-dep-versions` and `check-component-catalog` checks 2, 2b, 3 for the derived fields. Follow-up for that
  cleanup: `check-component-catalog` check 3 (and `check-installed-paths`) should learn `super::super::<x>` or be dropped.
  `registry-install-smoke.sh` is run by the main loop at merge: the dropped dependencies and crates are the part it proves.
