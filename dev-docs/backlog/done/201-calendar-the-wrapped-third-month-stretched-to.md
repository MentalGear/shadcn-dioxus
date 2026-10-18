---
id: 201
title: "Calendar: the wrapped third month stretched to the card width; calendar and date picker on Nova"
status: "LANDED 2026-10-14"
size: "S (done)"
area: theme
updated: 2026-10-14
---

# Row 201: Calendar: the wrapped third month stretched to the card width; calendar and date picker on Nova

## Item

Calendar: the wrapped third month stretched to the card width; calendar and date picker on Nova

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

Owner screenshot. Months were `flex: 1 1 auto` with flexible day columns, so a month alone on a row grew to 640px. Months are now equal fixed width (grid day columns `repeat(7, var(--dx-calendar-cell-size))`), wrapped months centre. Nova: cell 28px, `p-2`, ghost nav, bordered 24px caption selects, today/range muted, selected primary; month width `max(7c, 2c + 11rem)` = 232px because the full month name plus year select needs it (Nova's 196px prints "Sep"). The plain-day hover had never painted (a (0,1,0) reset beat the `:where` hover). Date picker group = the Input recipe.
