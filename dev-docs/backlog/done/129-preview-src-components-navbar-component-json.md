---
id: 129
title: "preview/src/components/navbar/component.json lacks the theme globalAssets entry every other component has"
status: "LANDED 2026-10-05 (round 3) -- found 2026-10-04 by the border-box lane…"
size: "S"
area: theme
updated: 2026-10-05
---

# Row 129: preview/src/components/navbar/component.json lacks the theme globalAssets entry every other component has

## Item

`preview/src/components/navbar/component.json` lacks the theme `globalAssets` entry every other component has

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- found 2026-10-04 by the border-box lane (`dev-docs/research/theme-2026-10-03/border-box.md` section 2)

## Size

S

## Detail

A navbar-only `dx components add navbar` ships `.dx-navbar` without `dx-components-theme.css`: no role tokens, no `box-sizing` rule, no radius. `button/component.json` carries `"globalAssets": ["../../../assets/dx-components-theme.css"]`; navbar has none. `navbar/style.css:3` therefore keeps its own `box-sizing: border-box` (the only kept redundancy for this reason). **Fix:** add the entry, drop the kept declaration, and add a check (script or `cargo test`) that every `preview/src/components/*/component.json` with a `style.css` lists the theme, so the class (a component that cannot stand alone) cannot recur. **Landed 2026-10-05 (round 3):** `navbar/component.json` now lists `../../../assets/dx-components-theme.css` in `globalAssets`, and navbar's kept `box-sizing: border-box` declaration (and its stale allowlist entry) is removed. **The class check is `scripts/check-component-catalog.sh`** (in `scripts/run-gates.sh`): every component directory with a `style.css` must list the theme in its `component.json`. The same gate also (a) fails when a component's installed files import a sibling component its `componentDependencies` do not declare (it found `form` missing four and `top_layer` one: fixed, so `dx components add` installs compile) and (b) enforces the shadcn-or-extra classification: `CATALOG: &[(&str, Origin)]` in `preview/src/components/mod.rs` has one row per component, and an `Extra` badge shows on the page header, the sidebar entry and the home card for the eight extras (`color_picker`, `drag_and_drop_list`, `navbar`, `tag_group`, `toolbar`, `virtual_list`, `form`, `top_layer`). A `"shadcn": false` manifest field was NOT added: the `dioxus-component-manifest` 0.7.9 manifest rejects unknown fields (`deny_unknown_fields`).
