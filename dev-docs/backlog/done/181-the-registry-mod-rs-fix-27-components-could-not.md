---
id: 181
title: "The registry mod.rs fix: 27 components could not be installed (E0583 in the user's tree), and no gate noticed"
status: "LANDED 2026-10-10"
size: "S (done)"
area: build
updated: 2026-10-10
---

# Row 181: The registry mod.rs fix: 27 components could not be installed (E0583 in the user's tree), and no gate noticed

## Item

The registry `mod.rs` fix: 27 components could not be installed (E0583 in the user's tree), and no gate noticed

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

What `dx components add` copies is the component's whole directory minus the manifest's `exclude`; 27 components had no `mod.rs` (the preview's `examples!` declares the module inline and never reads one), so a user's `cargo check` failed with E0583 while every gate was green. Fixed by adding `mod.rs` (exactly `mod component; pub use component::*;`) to each, and by `scripts/check-installed-paths.sh` (in `run-gates.sh`), which requires `mod.rs`, `component.rs` and `style.css` in every `component.json` directory, root `component.json` membership, existing `exclude` and `globalAssets` entries, and every `crate::components::<other>` use listed in `componentDependencies`. The slow end-to-end proof is row 182.
