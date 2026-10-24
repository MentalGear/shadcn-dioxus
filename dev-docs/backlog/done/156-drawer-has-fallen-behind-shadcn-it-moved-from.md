---
id: 156
title: "Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal…"
status: "LANDED 2026-10-19 (lane drawer: snap points, non-modal, responsive demo; see Update)"
size: "L"
area: overlays
updated: 2026-10-19
---

# Row 156: Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal…

## Item

Drawer has fallen behind shadcn: it moved from Vaul to Base UI Drawer (snap points, nested drawers, non-modal mode); ours keeps the Vaul shape

## Status / trigger

**LANDED 2026-10-19 (lane drawer: snap points, non-modal, responsive demo; see Update)**

## Size

L

## Detail

shadcn's `drawer` was rebuilt on Base UI (it left Vaul) and gained three things ours lacks: (1) **`snapPoints`**, stopping at partial heights (L), (2) **nested drawers**, the parent receding behind the child (M), (3) a **non-modal mode**. Ours (`primitives/src/drawer.rs`, `preview/src/components/drawer/`) composes `Dialog` plus a pointer-drag gesture in the Vaul shape and releases on 25% of the panel or 0.5 px/ms (`dev-docs/archive/rounds/component-backlog.md`, Drawer row). The comparison also lists a **Responsive Dialog-to-Drawer** demo (S, Add later; `use_is_mobile` already exists in Sidebar): take it with this row. **Sequencing:** row 141 (this round) changes the Dialog/Drawer exit path, and row 73's self-subscribing-effect hazard is in this drag code, so go after 141 and re-read shadcn's current `registry/bases/base/ui/drawer.tsx` first: the drift here is from the comparison's read, not yet checked against a running build. **Update 2026-10-14:** row 208 (edge lane 0579af5) covered nested drawers (`DrawerNestCtx`: the parent recedes, only the top-most drags, Escape closes the top-most; demo `drawer/variants/nested`) and the `Auto` side (bottom below 768px, right at/above); the edge-panel recipe is now shared with Sheet (row 207). **Still open on this row:** `snapPoints` (L), the non-modal mode, and the Responsive Dialog-to-Drawer demo; batch 2 takes them together with row 155 groups 2-9. **Update 2026-10-15:** batch 2 landed row 155 groups 2-9 and rows 158(a), 166 and the scroll fade, but not this row's remaining items. What remains: (1) `snapPoints` (L), stopping at partial heights; (2) the non-modal mode; (3) the Responsive Dialog-to-Drawer demo (S, `use_is_mobile` exists in Sidebar). Nested drawers and the `Auto` side stay as landed with row 208. Still re-read shadcn's current `registry/bases/base/ui/drawer.tsx` first.


## Update 2026-10-19 (lane drawer)

Landed the three remaining items, composing the Dialog primitive (no change to `dialog.rs`/`modality.rs`):

- **`snap_points`** (`primitives/src/drawer.rs`, `SnapPoint::{Fraction, Px}`; props `snap_points`, `active_snap_point`, `default_active_snap_point`, `on_active_snap_point_change`). One source of truth for the resting offset: the active point writes `--dx-drawer-snap` / `--dx-drawer-snap-max` on the panel, the stylesheet derives `--dx-drawer-rest` and the panel's `transform` (and the drag's inline `transform`, and the nested step-back) read it; the panel is as large as the largest point and nothing is measured to place it. Release = the ONE old closing rule applied below the lowest point (`should_close_on_release`), else the snap point nearest the release position projected by the velocity (`settle_on_release`); the old no-snap drawer is the same code on rests `[0]`. Drag code uses no new effect (handlers only; `check-self-subscribing-effects` stays green); the drag offset is displaced only while dragging or held after a drag-close.
- **Non-modal** (`is_modal: false`, existing prop name kept for Dialog/Popover consistency): no scrim (`scrim = overlay && is_modal`, one place), page interactive and scrollable, no focus trap/scroll lock; the root wrapper (`data-dx-modal="false"`) lets pointer events through. Outside press/focus still dismiss (the dialog primitive's non-modal arm = Base UI's default); a Base UI `disablePointerDismissal` equivalent and a correct `aria-modal` on that arm need a change in `dialog.rs` (not this lane's file): open.
- **Responsive Dialog-to-Drawer** demo (`variants/responsive_dialog`) reusing `sidebar::use_is_mobile` in place (no move).
- Also fixed: the drawer's `prefers-reduced-motion` transition rule sat before the base `.dx-drawer` transition and lost to it (now after it).
- Demos `snap_points`, `non_modal`, `responsive_dialog`; docs.md; Rust unit tests for rests, settle, damping and style; specs `drawer-snap.spec.ts`, `drawer-non-modal.spec.ts`, `drawer-responsive.spec.ts`.
- No keyboard stepping between snap points (Base UI has none either); a control that sets the controlled point is the route. On touch a top/bottom drawer at its largest point lets content pan, so its drag starts from the handle.
