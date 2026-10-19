---
id: 138
title: "Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and…"
status: "LANDED 2026-10-19 (lane toast: update/close by id, Loading, promise, action, stacking, swipe)…"
size: "M-L (was M; the folded-in items add an…"
area: overlays
updated: 2026-10-19
---

# Row 138: Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and…

## Item

Toast depth parity: loading type, action button, promise helper, update/close by id, stacking and swipe-dismissal are missing (shadcn rebuilt its toast on Base UI Toast)

## Status / trigger

**LANDED 2026-10-19 (lane toast: update/close by id, Loading, promise, action, stacking, swipe)** -- see "Update 2026-10-19" below; was: **BACKLOG** -- recorded 2026-10-04 from the catalog audit (§6.4); depth, not a catalog gap (`toast` is present); **scheduled in the next-session order (2026-10-05)**; the Base UI drift from the shadcn comparison is folded in below

## Size

M-L (was M; the folded-in items add an S and an M)

## Detail

**2026-10-05 fold-in (shadcn comparison, "where our port has fallen behind"):** shadcn rebuilt `toast` on Base UI Toast on 2026-07-23 and ours has four types and none of the new behaviour. The comparison's list, all "Add later": (0) **close or update a toast by id** (S; `show` returns nothing to hold on to; the prerequisite of the promise step), (1) a **`loading` type** (S; `ToastType` stops at `Info`), (2) the **action button** (`actionProps`, e.g. Undo; M), (3) **`promise`** (one toast moves from loading to success or error; M, needs (0)), (4) **swipe to dismiss** (M), plus the **stacking** below. A sensible order: (0), (1), (3), (2), stacking, (4). The four numbered steps below are the original 2026-10-04 plan and still hold. shadcn's base `toast` (Base UI Toast, release note 2026-07-23) documents types, an **action button**, **promise** (loading -> success/error), **stacking** and **swipe dismissal**. `primitives/src/toast.rs` has the four types and `ToastOptions { description, duration, permanent }` but no action, promise helper, swipe or stack (grep of `toast.rs` and `toast/component.rs` finds none). Take them as four small steps, each with its own spec: (1) an `action: Option<(label, callback)>` on `ToastOptions` rendered as a `Button` inside the toast, keyboard reachable, which dismisses on activation; (2) `use_toast().promise(future, {loading, success, error})` that updates one toast in place and announces the outcome (`role="status"`, not a second toast); (3) stacking: collapsed pile with hover/focus expand, `prefers-reduced-motion` honoured; (4) swipe dismissal with pointer events (threshold + velocity, RTL-aware direction). Base has no `sonner` page, so this is the toast's own parity target.

## Update 2026-10-19

Landed in lane `toast` (branch `lane/toast`), all six steps in the planned order:

- (0) `Toasts::show`/`success`/`error`/`warning`/`info` return a `ToastId` (opaque newtype over a process-wide monotonic `AtomicU64`: no public constructor or arithmetic, never reused). `update(id, ToastUpdate)` edits in place (same element, a `revision` bump restarts the countdown) and returns whether it applied; `close(id)`. A closed or trimmed id is a no-op and, being unique, can never hit another toast. Existing callers compile unchanged (`ToastId` implements `SpawnIfAsync<()>` so an event handler may still end in `toast.info(..)` without a semicolon).
- (1) `ToastType::Loading` (spinner via the library `Spinner`, aria-hidden under the toast's own live region). Never auto-dismissed is derived from the type in `ToastRecord::effective_duration`, so loading -> success starts the countdown by itself. The countdown moved out of a conditional hook into a keyed `ToastTimer` child.
- (3) `Toasts::promise(future, PromiseMessages::new(loading, |v| .., |e| ..))`: one toast, loading -> success/error in place. `ToastContent`'s live region is `status` (polite) and `alert` for an error, keyed on its role so the error is a freshly inserted alert region. The future runs via `spawn_forever` (outlives the starter; settling a closed toast is a no-op).
- (2) `ToastOptions::action(label, cb)`, `ToastAction`, `use_toast_action()` / `ToastActionHandle::activate()` and a primitive `ToastActionButton`; the preview skin renders the library `Button` (outline, sm). Activation runs the callback then dismisses.
- (stacking) the collapsed pile with hover/focus expand already existed in CSS; now specified, and `prefers-reduced-motion: reduce` turns the entrance animation and restack transitions off.
- (4) swipe to dismiss: `use_toast_swipe` (pointer events through `js_listener` `listen`): 45 px threshold or a flick (>= 10 px at >= 0.11 px/ms over 100 ms); dismiss direction is the viewport half the toast sits in, so it follows the toaster position and RTL with no extra prop; writes `--toast-swipe-x` and `data-swiping`, mapped to `translate` in the stylesheet (marked `rtl-physical`).
- Specs: `playwright/toast-{update,loading,promise,action,stacking,swipe}.spec.ts` (19 tests; with `toast.spec.ts` 25/25 green on a lane-ssg build containing the change). Demos: `loading`, `action`, `promise`, `update`, `stacking`, `swipe` next to `main` and `types`; docs.md has a section per feature.

Left open: F6 focuses the LAST mounted provider's region when a page mounts several (the docs page has eight toast demos), a pre-existing property of one global F6 listener per provider; the legacy `dx-slide-up-*` / `dx-toast-slide-in` keyframes still use `transform` (they restate the stacking `scale`, and are on the `check-anchored-keyframes.sh` ALLOW list, which is outside this lane); converting them to `translate`/`scale` needs those two ALLOW entries removed in the same change. Swipe is horizontal only (Base UI also allows vertical for top/bottom toasters). Auto-dismiss does not pause on hover/focus (Base UI does).
