---
id: 200
title: "ButtonGroupSeparator was a horizontal separator in a horizontal group"
status: "LANDED 2026-10-14"
size: "S (done)"
area: theme
updated: 2026-10-14
---

# Row 200: ButtonGroupSeparator was a horizontal separator in a horizontal group

## Item

`ButtonGroupSeparator` was a horizontal separator in a horizontal group

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

It was 0x1px (invisible) until phase C's `flex-shrink: 0` made it a 172px bar that pushed buttons off-screen at 390px. The separator now takes the group's cross orientation from a context (row -> vertical 1px line, as Nova); the `orientation_vertical` prop is removed. Vertical `.dx-separator` `height: 100% -> auto` (a percentage height disables `align-self: stretch` and resolved to 0 in a content-height row).
