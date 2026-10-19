---
id: 248
title: "Select renders its placeholder until mount, so a Select with a default value shifts layout at hydration"
status: "BACKLOG (2026-10-19)"
size: "S-M"
area: components
updated: 2026-10-19
---

# Row 248: Select renders its placeholder until mount, so a Select with a default value shifts layout at hydration

## Item

Select renders its placeholder until mount, so a Select with a default value shifts layout at hydration

## Status / trigger

**BACKLOG (2026-10-19)**

## Size

S-M

## Detail

The Select primitive shows the placeholder until its options register, then the selected label; any Select with a default_value and no placeholder changes width at hydration (the data_table page-size Select caused CLS 0.0131 > 0.01). Fix in the primitive: render the default value's label in SSG and the first client render.
