---
id: 3
title: "Phase 4.2 — native <dialog>+showModal() for Dialog and AlertDialog"
status: "Landed (modal Dialog/AlertDialog web arm → real <dialog> + showModal(); native/Blitz arm and…"
size: "M–L"
area: overlays
updated: 2026-09-01
---

# Row 3: Phase 4.2 — native <dialog>+showModal() for Dialog and AlertDialog

## Item

**Phase 4.2** — native `<dialog>`+`showModal()` for Dialog and AlertDialog

## Status / trigger

**Landed** (modal `Dialog`/`AlertDialog` web arm → real `<dialog>` + `showModal()`; native/Blitz arm and non-modal `Dialog` unchanged)

## Size

M–L

## Detail

Narrow-leaf `#[cfg(feature = "web")]` split per Construction B (`dev-docs/phase4-spike-findings.md`) — originally built on `#[cfg(target_family = "wasm")]`, corrected to the `web` feature axis in the 2026-09-01 cfg-axis incident fix (`dev-docs/recommended-implementations.md` Caveat 1) — reusing 4.4's axis and axis-swap caveat; close/cancel sync + `dialog.open` guard via new `use_dialog_close_sync`/`use_dialog_open_driver` (`lib.rs`); real child-component split (`DialogContentModal`/`DialogContentNonModal`, mirroring `popover.rs`) rather than a gated callback, confirming 4.4's warning that `use_global_keydown_listener`'s unconditional `preventDefault()` would otherwise fight the native `cancel` event; scroll lock kept as-is (proven sufficient in spike round 2). New backdrop-click dismiss (`use_dialog_backdrop_dismiss`, rect-based, since a `showModal()` backdrop click's `event.target` is the dialog itself, defeating `use_outside_dismiss`'s containment check). No `focus_scope.rs`/`aria_hidden.rs` existed in this repo to delete (recommended-implementations.md's more general framing referred to forks that never got ported here); the vendored `focus-trap.ts`/`.js` pair stays live — still used by the native/Blitz arm and by `Popover`'s (untouched) modal arm on both targets. New cosmetic gap, same treatment as row 19: `Sheet`'s slide-out animation (defined on the content element, now a `<dialog>`) is cut short by the browser's immediate `display:none` on close, ahead of `use_animated_open`'s unmount-hold
