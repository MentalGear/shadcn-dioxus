---
id: 253
title: "SSG renders `initial_checked` as a literal `initial_checked=true` attribute, so a prerendered checkbox, switch or radio loads unchecked"
status: "BACKLOG (found 2026-10-20 by lane merge-fix)"
size: "S-M"
area: components
updated: 2026-10-20
---

# Row 253: SSG renders `initial_checked` as a literal `initial_checked=true` attribute, so a prerendered checkbox, switch or radio loads unchecked

## Item

On a directly loaded, prerendered page the server HTML carries `initial_checked=true` as a plain attribute instead of `checked`, and hydration never sets `.defaultChecked`, so a default-checked native input loads unchecked and `<form reset>` restores the wrong default.

## Status / trigger

**BACKLOG (found 2026-10-20 by lane merge-fix)**

## Size

S-M

## Detail

Found when round 1 moved `playwright/oracle/tier2-html/form-attribute.spec.ts` from the legacy `/component/?name=form&` shell (which redirects client-side, so the client render applies `defaultChecked`) to the prerendered `/component/form/`. `:39` (native checkbox calibration: `terms-outside-native` missing from the submitted entries) and `:63` (after `<form reset>`, `terms-outside-lib=accepted` missing) fail; the old route passes 5/5 against the same build. Evidence: `component/form/index.html` of a debug SSG has `<input id="chk-outside-native" type="checkbox" ... initial_checked=true>`, and the Checkbox primitive's hidden mirror input has the same attribute.

Sites (one class: every `initial_checked` the SSR emits): `preview/src/components/form/component.rs` (the native demo inputs), `primitives/src/checkbox.rs` (hidden mirror input), `primitives/src/switch.rs`, `primitives/src/radio_group.rs`. Likely the same for `initial_value`/`initial_selected` if used. Fix by construction: one hydration-safe default-checked helper (SSR emits `checked`, the client sets `defaultChecked`), used by every site, plus a gate or SSR unit test that no rendered HTML contains `initial_`. The two form-attribute tests stay red until then (not skipped).
