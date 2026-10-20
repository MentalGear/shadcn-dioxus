---
id: 253
title: "SSG renders `initial_checked` as a literal `initial_checked=true` attribute, so a prerendered checkbox, switch or radio loads unchecked"
status: "FIXED 2026-10-20 (lane/initial-checked). playwright/oracle/tier2-html/form-attribute.spec.ts 5/5 on…"
size: "S-M"
area: components
updated: 2026-10-20
---

# Row 253: SSG renders `initial_checked` as a literal `initial_checked=true` attribute, so a prerendered checkbox, switch or radio loads unchecked

## Item

On a directly loaded, prerendered page the server HTML carries `initial_checked=true` as a plain attribute instead of `checked`, and hydration never sets `.defaultChecked`, so a default-checked native input loads unchecked and `<form reset>` restores the wrong default.

## Status / trigger

**FIXED 2026-10-20 (lane/initial-checked).** `playwright/oracle/tier2-html/form-attribute.spec.ts` 5/5 on the prerendered `/component/form/` (it was 3/5: `:39` and `:63`), `form-participation` 28/28, checkbox, switch and radio-group specs green; the debug SSG's `component/form/index.html` has `checked=true` on `chk-native` and `chk-outside-native` and no `initial_checked` on any page (it had 81 across the template's pages).

## Size

S-M

## Detail

Found when round 1 moved `playwright/oracle/tier2-html/form-attribute.spec.ts` from the legacy `/component/?name=form&` shell (which redirects client-side, so the client render applies `defaultChecked`) to the prerendered `/component/form/`. `:39` (native checkbox calibration: `terms-outside-native` missing from the submitted entries) and `:63` (after `<form reset>`, `terms-outside-lib=accepted` missing) fail; the old route passes 5/5 against the same build. Evidence: `component/form/index.html` of a debug SSG has `<input id="chk-outside-native" type="checkbox" ... initial_checked=true>`, and the Checkbox primitive's hidden mirror input has the same attribute.

Sites (one class: every `initial_checked` the SSR emits): `preview/src/components/form/component.rs` (the native demo inputs), `primitives/src/checkbox.rs` (hidden mirror input), `primitives/src/switch.rs`, `primitives/src/radio_group.rs`. Likely the same for `initial_value`/`initial_selected` if used. Fix by construction: one hydration-safe default-checked helper (SSR emits `checked`, the client sets `defaultChecked`), used by every site, plus a gate or SSR unit test that no rendered HTML contains `initial_`. The two form-attribute tests stay red until then (not skipped).

## Root cause and fix (2026-10-20)

**Cause, two upstream defects in dioxus 0.7.9 that meet at `initial_checked`:**
1. `dioxus-ssr-0.7.9/src/renderer.rs` `write_attribute` (line 482) writes every attribute that is not in its `BOOL_ATTRS` list (line 425; `checked` and `selected` are there, `initial_*` are not) as `name=value`, so the server HTML carried a literal `initial_checked=true` and, for the native demo inputs and the Checkbox mirror input (no `checked` at all), no content attribute.
2. `dioxus-web-0.7.9/src/hydration/hydrate.rs` `rehydrate_template_node` (line 299-320) only mounts ids and registers listeners for a hydrated node; it applies no attribute. `initial_checked` is honoured only as a *mutation* (`dioxus-interpreter-js` `set_attribute.ts` `case "initial_checked": node.defaultChecked = ...`), i.e. for a node the client creates (client-side navigation, the legacy shell), never for a prerendered one.

So a prerendered default-checked input loaded unchecked and `<form reset>` restored the wrong state.

**Construction:** `primitives/src/default_checked.rs` `use_default_checked(id, default)`. A control renders the plain `checked` attribute (SSR writes it as a content attribute: right for the parser's `.defaultChecked`, for a no-script post, and as the live state; a client-created node takes the live state from it) and calls the hook, whose effect runs on a client only, after a fresh render and after hydration alike, and sets `.defaultChecked` through `eval`. `initial_checked` is never rendered. Used by Checkbox (`BubbleInput`, which gained the `checked` attribute), Switch, RadioItem and the three native default-checked inputs of `FormFixture`. Setting `.defaultChecked` never changes a dirty control, so it does not fight the controlled `checked`.

**Guards:** SSR unit tests in `checkbox.rs`, `switch.rs` and `radio_group.rs` (no `initial_` in the server HTML, `checked` on exactly the default input) and a source scan in `default_checked.rs` (no primitive renders `initial_checked`/`initial_value`/`initial_selected`, reasoned allowlist, stale entries fail). No `scripts/check-*.sh`: the rule lives in the crate that ships the primitives and runs in `cargo test`, and a new gate script also means a `run-gates.sh` entry and a CLAUDE.md sentence for one grep.

**Not covered (same class, left open):** `primitives/src/select/components/select.rs` renders `initial_selected` on the hidden `<select>`'s options. Its server HTML is right (`selected` is a `BOOL_ATTR` and is emitted too), so the literal `initial_selected=true|false` is inert junk, and the client-created case still works through the interpreter; a `use_default_selected` twin would remove it (on the guard's allowlist). `initial_value` is used by preview demo `Input`s inside dialogs, sheets and popovers only, none of them in prerendered HTML today (a latent same-class defect if one is ever rendered at SSR).
