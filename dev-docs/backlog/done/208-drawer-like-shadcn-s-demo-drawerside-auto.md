---
id: 208
title: "Drawer like shadcn's demo: DrawerSide::Auto default, rounded, nested drawers"
status: "LANDED 2026-10-14 (edge lane 0579af5)"
size: "M"
area: overlays
updated: 2026-10-14
---

# Row 208: Drawer like shadcn's demo: DrawerSide::Auto default, rounded, nested drawers

## Item

Drawer like shadcn's demo: `DrawerSide::Auto` default, rounded, nested drawers

## Status / trigger

**LANDED 2026-10-14 (edge lane 0579af5)**

## Size

M

## Detail

Owner decision 2026-10-14: Auto is the DEFAULT (bottom below md, side at/above). Bottom rounds its top edge; side floats inset, fully rounded; nested drawers via a stack context (parent scales/offsets, Escape/drag close the top only). Folds in row 155's Sheet `side` prop. Item 5. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** `use_resolved_edge_side`: one `matchMedia` at `--dx-breakpoint-md` 768px (bottom below, right at/above); a `default_open` drawer resolves Auto after one eval round-trip. Nested drawers: `DrawerNestCtx` (the parent gets `data-nested-open` and scales ~0.96 + 16px like Vaul `NestedRoot`), only the top-most drags, Escape closes the top-most, one scroll lock; new demo `drawer/variants/nested`. Floating rounded side drawers and Auto are the owner's brief, not shadcn source (shadcn's drawer is Vaul with an explicit direction, default bottom). Covered row 156's nested-drawer item and the Auto side; snap points and non-modal mode remain on row 156. Sheet `side` as a typed prop is row 155 group 7.
