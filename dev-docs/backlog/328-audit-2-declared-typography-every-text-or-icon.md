---
id: 328
title: "Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit"
status: "BACKLOG (owner 2026-10-21: do it if it holds up)"
size: "M"
area: theme
updated: 2026-10-21
---

# Row 328: Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit

## Item

Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit.

## Status / trigger

**BACKLOG (owner 2026-10-21: do it if it holds up)**

## Size

M

## Detail

Audit section 4: of ~200 text/icon-bearing parts, 68 declare no `font-size` and 151 no `line-height`; of 64 component roots 20 declare a size and 5 a line-height; 12 of 17 popup roots declare no size. A part that inherits its size is right only by accident (Select's scroll buttons at 14px in a card footer, row 268). Construction: a gate reading roots from `component.rs` (outermost element per `#[component]`, plus `popover`/`dialog`/`role=menu|listbox|dialog|tooltip` elements) that requires each root's class to declare font-size and line-height through a token or slot, shrink-only baseline, `typography-inherit: <reason>` marker. Complements the popup-root reset (row 218 half). Shrinks popup-style-isolation further and touch-focus-zoom.
