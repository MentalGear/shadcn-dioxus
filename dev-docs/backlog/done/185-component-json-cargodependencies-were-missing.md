---
id: 185
title: "component.json cargoDependencies were missing for 14 components; the gate and the smoke app both hid it"
status: "LANDED 2026-10-11"
size: "S (done)"
area: build
updated: 2026-10-11
---

# Row 185: component.json cargoDependencies were missing for 14 components; the gate and the smoke app both hid it

## Item

`component.json` `cargoDependencies` were missing for 14 components; the gate and the smoke app both hid it

## Status / trigger

**LANDED 2026-10-11**

## Size

S (done)

## Detail

Found by the post-merge test on GitHub: `kbd`, `area_chart`, `bubble`, `empty` and `message` use `dioxus_primitives` but declared no `cargoDependencies`, so in an app that did not already depend on it (an `init`'d existing app, a bare `cargo new`) `dx components add kbd` / `shadcn-dioxus add kbd` left `Cargo.toml` unchanged and `cargo check` failed with E0433. Invisible because `templates/starter`, the preview and `registry-install-smoke.sh`'s app all pre-declared `dioxus-primitives`. The inventory found the same hole in `alert`, `breadcrumb`, `native_select`, `spinner` (primitives) and `bar_chart`, `context_menu`, `dropdown_menu`, `menubar` (`dioxus-icons`); `area_chart` and `radar_chart` only worked through `chart` and now declare it too. By construction: (1) `scripts/check-installed-paths.sh` now requires every external crate a component's installed `.rs` files use (`use <crate>` roots and `<crate>::` paths for any crate named in the workspace or any manifest, plus any `dioxus_*`; `#[cfg(test)]` modules and the `dioxus_attributes` re-export inside a `use dioxus_primitives::{..}` tree are not counted) to be named in `cargoDependencies` of the component or of its `componentDependencies` closure (both installers `cargo add` the whole closure: dx 0.7.9 `run` "collect all required rust dependencies", `cli/src/add.rs` `to_install`); red proof: restoring kbd to no entry fails the gate naming `dioxus_primitives`; the gate also rejects a repeated manifest key (the first smoke run died on a doubled `cargoDependencies` in `radar_chart`, which `json.load` had silently accepted). (2) `scripts/registry-install-smoke.sh` strips `dioxus-primitives` (and its `/web` feature references) from the rendered starter, so the install must add it. Not covered: a crate used only in a component's tests (`dioxus_ssr` in `calendar`, `form`, `toast`, ...) is not added; a user's `cargo test` of an installed component needs it as a dev-dependency (the manifest has no dev field).
