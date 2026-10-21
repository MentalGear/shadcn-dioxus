---
id: 351
title: "RTL demos always come last in a component's demo list"
status: "Open (owner approved 2026-10-20)."
size: "S"
area: docs
updated: 2026-10-21
---

# Row 351: RTL demos always come last in a component's demo list

## Item

RTL demos always come last in a component's demo list.

## Status / trigger

Open (owner approved 2026-10-20).

## Size

S

## Detail

`preview/build.rs` orders variants by `variants/order.txt` (line 124), so an `rtl` variant sits wherever a curator put it (the Calendar's `variants/` has `rtl` beside `order.txt`). Make the build sort any variant named `rtl` (or ending in `-rtl`/`_rtl`) after all others, whatever `order.txt` says, and drop the now-redundant `rtl` lines from the order files. Check `check-docs-*` gates and the demo-order specs.
