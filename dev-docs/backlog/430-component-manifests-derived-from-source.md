---
id: 430
title: "Component manifests derived from source, not hand-kept"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P5)."
size: "M"
area: tooling
updated: 2026-10-25
---

# Row 430: Component manifests are derived from source, not hand-kept

## Item

Each of the 83 `preview/src/components/*/component.json` manifests and each `mod.rs` is hand-kept, and gates check them against the Rust files after the fact. Agreement between two authored files is a computation, not a decision. Construction: a generator writes the computed fields (`componentDependencies`, `cargoDependencies`, `globalAssets`, `exclude`, `mod.rs`) from the sources, and `run-gates.sh` runs `--check` for byte-identity, the `build-js-bundle` / `check-js-bundle` model. A manifest cannot disagree with the files it ships, because it is their output.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P5).

## Size

M

## Detail

Source: [final sweep note, P5](../research/final-sweep-2026-10-25.md). Scripts and 83 JSON files, no product code.

- **Measured:** 76 manifests carry the identical `exclude: ["variants","docs.md","component.json"]` and `globalAssets: [theme]`; seven add `dx-effects.css`. 26 list `componentDependencies`; all list `cargoDependencies` with the same `dioxus-primitives` git block. `cat */mod.rs | sort | uniq -c`: 47 on two lines, 36 on one.
- **Already computed by gates:** `check-installed-paths.sh:21-53` (componentDependencies from `crate::components::<other>`, external crates of the closure, `mod.rs` shape, `exclude`/`globalAssets` existing); `check-cargo-dep-versions.sh`, `check-registry-url.sh`, and `check-component-catalog.sh` checks 2, 2b and 3.
- **Failures this caused** (CLAUDE.md): 27 missing `mod.rs` found only when a user's `cargo check` hit E0583; `kbd` and four more without `cargoDependencies` (E0433). That is why `registry-install-smoke.sh` (~6 GB, minutes) exists.
- **Construction:** `scripts/gen-component-manifests.sh`, or a `props_scan`-style Rust scanner (the precedent is `preview/props_scan.rs`). Hand-authored fields stay: `name`, `description`, `authors`. External-crate detection uses the same `cfg` rule the gate has.
- **Subsumes:** the "manifest says X, source says Y" halves 2 and 3 of `check-installed-paths`; the `cargoDependencies` floor check of `check-cargo-dep-versions`; `check-component-catalog` checks 2, 2b and 3 for the derived fields. **Does not subsume:** `registry-install-smoke.sh` (it tests dx itself), the `CATALOG` classification (`Origin::Shadcn|Extra` is a decision, not a computation), and the `document::Link` line in each component.
- **Gate:** `gen-component-manifests.sh --check` in `run-gates.sh`. The four existing gates stay as cross-checks until the generator has run clean for a batch, then lose their duplicated halves.
