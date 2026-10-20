---
id: 251
title: "Native checkbox in the prerendered /component/form/ ignores initial_checked until the page is client-created"
status: "BACKLOG (lane specs2 2026-10-19)"
size: "S"
area: components
updated: 2026-10-19
---

# Row 251: Native checkbox in the prerendered /component/form/ ignores initial_checked until the page is client-created

## Item

Native checkbox in the prerendered /component/form/ ignores initial_checked until the page is client-created

## Status / trigger

**BACKLOG (lane specs2 2026-10-19)**

## Size

S

## Detail

Found by lane specs2 (row 164) when form-participation.spec.ts moved from the legacy `?name=` route to the path route: the prerendered `/component/form/` emits the native checkbox with `initial_checked=true`, but hydration does not apply it, so on a hard load the box is unchecked; it is checked only when the page is created on the client, which is how the legacy route hid it. form-participation's CALIBRATION fails on the path route, so that spec stays on the legacy route with a `legacy-route-ok:` marker until this is fixed. Root-cause in the checkbox/form demo's SSR (attribute vs property `checked`), then move the spec to the path route.
