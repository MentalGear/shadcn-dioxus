---
id: 138
title: "Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and…"
status: "BACKLOG -- recorded 2026-10-04 from the catalog audit (§6.4); depth, not a catalog gap (toast is…"
size: "M-L (was M; the folded-in items add an…"
area: overlays
updated: 2026-10-05
---

# Row 138: Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and…

## Item

Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and swipe-dismissal are missing (shadcn rebuilt its toast on Base UI Toast)

## Status / trigger

**BACKLOG** -- recorded 2026-10-04 from the catalog audit (§6.4); depth, not a catalog gap (`toast` is present); **scheduled in the next-session order (2026-10-05)**; the Base UI drift from the shadcn comparison is folded in below

## Size

M-L (was M; the folded-in items add an S and an M)

## Detail

**2026-10-05 fold-in (shadcn comparison, "where our port has fallen behind"):** shadcn rebuilt `toast` on Base UI Toast on 2026-07-23 and ours has four types and none of the new behaviour. The comparison's list, all "Add later": (0) **close or update a toast by id** (S; `show` returns nothing to hold on to; the prerequisite of the promise step), (1) a **`loading` type** (S; `ToastType` stops at `Info`), (2) the **action button** (`actionProps`, e.g. Undo; M), (3) **`promise`** (one toast moves from loading to success or error; M, needs (0)), (4) **swipe to dismiss** (M), plus the **stacking** below. A sensible order: (0), (1), (3), (2), stacking, (4). The four numbered steps below are the original 2026-10-04 plan and still hold. shadcn's base `toast` (Base UI Toast, release note 2026-07-23) documents types, an **action button**, **promise** (loading -> success/error), **stacking** and **swipe dismissal**. `primitives/src/toast.rs` has the four types and `ToastOptions { description, duration, permanent }` but no action, promise helper, swipe or stack (grep of `toast.rs` and `toast/component.rs` finds none). Take them as four small steps, each with its own spec: (1) an `action: Option<(label, callback)>` on `ToastOptions` rendered as a `Button` inside the toast, keyboard reachable, which dismisses on activation; (2) `use_toast().promise(future, {loading, success, error})` that updates one toast in place and announces the outcome (`role="status"`, not a second toast); (3) stacking: collapsed pile with hover/focus expand, `prefers-reduced-motion` honoured; (4) swipe dismissal with pointer events (threshold + velocity, RTL-aware direction). Base has no `sonner` page, so this is the toast's own parity target.
