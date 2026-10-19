---
id: 132
title: ".dx-tabs-content keeps padding: var(--dx-space-8) while Nova's TabsContent has none; consumers not audited"
status: "LANDED 2026-10-14 (phase C, owner decision 4) -- .dx-tabs-content has no padding (Nova); the docs…"
size: "S"
area: theme
updated: 2026-10-14
---

# Row 132: .dx-tabs-content keeps padding: var(--dx-space-8) while Nova's TabsContent has none; consumers not audited

## Item

`.dx-tabs-content` keeps `padding: var(--dx-space-8)` while Nova's TabsContent has none; consumers not audited

## Status / trigger

**LANDED 2026-10-14 (phase C, owner decision 4)** -- `.dx-tabs-content` has no padding (Nova); the docs preview frame, Block preview/code panels and `BlockTabs` declare their own 2rem, so the docs chrome is computed-identical (S1->S2 zero frame-padding diffs); four tabs demos pad their own content

## Size

S

## Detail

`.dx-tabs-content` (`tabs/style.css`, 2rem padding on every side) and the themed variant (`.dx-tabs-content-themed`: 1px border, `--dx-card` fill) pre-date phase B; shadcn's `TabsContent` is `flex-1 text-sm outline-none` with the padding left to its child (usually a Card). A demo that nests a Card in a themed tab pads twice. **Next:** list every consumer (`tabs/component.rs:134`, the demos, `charts_gallery.rs`, docs pages), measure the double padding on a nested-Card tab, then either drop the default padding (and give the two non-Card consumers their own) or document it as a deliberate difference. Phase C material. **2026-10-05:** the shadcn parity wave's Tabs `line` variant and vertical orientation (row 155, group 5) and the `Ghost` fix (row 160) edit the same stylesheet; take them in the same pass.
