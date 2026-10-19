---
id: 156
title: "Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal…"
status: "2026-10-15: batch 2 did NOT take the rest; still open: snapPoints, the non-modal mode and the…"
size: "L"
area: overlays
updated: 2026-10-15
---

# Row 156: Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal…

## Item

Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal mode); ours keeps the Vaul shape

## Status / trigger

**2026-10-15: batch 2 did NOT take the rest; still open: `snapPoints`, the non-modal mode and the Responsive Dialog-to-Drawer demo.** **BACKLOG (Add later)** -- recorded 2026-10-05 from the shadcn comparison ("where our port has fallen behind"); not in the next-session order, no go-ahead yet

## Size

L

## Detail

shadcn's `drawer` was rebuilt on Base UI (it left Vaul) and gained three things ours lacks: (1) **`snapPoints`**, stopping at partial heights (L), (2) **nested drawers**, the parent receding behind the child (M), (3) a **non-modal mode**. Ours (`primitives/src/drawer.rs`, `preview/src/components/drawer/`) composes `Dialog` plus a pointer-drag gesture in the Vaul shape and releases on 25% of the panel or 0.5 px/ms (`component-backlog.md`, Drawer row). The comparison also lists a **Responsive Dialog-to-Drawer** demo (S, Add later; `use_is_mobile` already exists in Sidebar): take it with this row. **Sequencing:** row 141 (this round) changes the Dialog/Drawer exit path, and row 73's self-subscribing-effect hazard is in this drag code, so go after 141 and re-read shadcn's current `registry/bases/base/ui/drawer.tsx` first: the drift here is from the comparison's read, not yet checked against a running build. **Update 2026-10-14:** row 208 (edge lane 0579af5) covered nested drawers (`DrawerNestCtx`: the parent recedes, only the top-most drags, Escape closes the top-most; demo `drawer/variants/nested`) and the `Auto` side (bottom below 768px, right at/above); the edge-panel recipe is now shared with Sheet (row 207). **Still open on this row:** `snapPoints` (L), the non-modal mode, and the Responsive Dialog-to-Drawer demo; batch 2 takes them together with row 155 groups 2-9. **Update 2026-10-15:** batch 2 landed row 155 groups 2-9 and rows 158(a), 166 and the scroll fade, but not this row's remaining items. What remains: (1) `snapPoints` (L), stopping at partial heights; (2) the non-modal mode; (3) the Responsive Dialog-to-Drawer demo (S, `use_is_mobile` exists in Sidebar). Nested drawers and the `Auto` side stay as landed with row 208. Still re-read shadcn's current `registry/bases/base/ui/drawer.tsx` first.
