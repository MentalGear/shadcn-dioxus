---
id: 247
title: "avatar-loading spec has a 450 ms floor too tight for this machine"
status: "LANDED 2026-10-21 (lane test-tools): the floor was the gap between component mount and the…"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 247: avatar-loading spec has a 450 ms floor too tight for this machine

## Item

avatar-loading spec has a 450 ms floor too tight for this machine

## Status / trigger

**LANDED 2026-10-21 (lane test-tools): the floor was the gap between component mount and the data-hydrated flag; the spec now anchors on the wasm resource's responseEnd and asserts the full 600 ms delay (>=595)**

## Size

S

## Detail

Validation 2026-10-19: avatar-loading.spec:115 waited 445.5 ms against a 450 ms floor in the full run and both reruns; the floor, not the code. Widen it or measure relative to the configured fallback delay.
