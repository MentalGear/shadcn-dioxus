---
id: 350
title: "Coarse demo: Auto/On/Off in the theme palette and a side-by-side docs section"
status: "Open, needs row 349 (owner approved 2026-10-20)."
size: "S"
area: docs
updated: 2026-10-21
---

# Row 350: Coarse demo: Auto/On/Off in the theme palette and a side-by-side docs section

## Item

Coarse demo: Auto/On/Off in the theme palette and a side-by-side docs section.

## Status / trigger

Open, needs row 349 (owner approved 2026-10-20).

## Size

S

## Detail

Add an Auto/On/Off control to the theme palette beside press feedback (`preview/src/theme.rs`) that writes `data-dx-coarse` on `<html>`, and a docs section that renders a fine and a coarse container side by side so the sizes can be compared on a desktop. Blocked by [row 349](./349-coarse-sizing-global-default-plus-a-per-container-attribute.md) for the attribute.
