---
id: 141
title: "Dialog, AlertDialog, Sheet and Drawer exit animations play only where transition: overlay allow-discrete…"
status: "LANDED 2026-10-05 (round 3; the construction below was built, on a wider root cause) -- recorded…"
size: "M"
area: theme
updated: 2026-10-05
---

# Row 141: Dialog, AlertDialog, Sheet and Drawer exit animations play only where transition: overlay allow-discrete…

## Item

Dialog, AlertDialog, Sheet and Drawer exit animations play only where `transition: overlay allow-discrete` exists (Chromium); a deferred close in `use_dialog_open_driver` would make them cross-browser

## Status / trigger

**LANDED 2026-10-05 (round 3; the construction below was built, on a wider root cause)** -- recorded 2026-10-04 from the overlay-scrim construction

## Size

M

## Detail

The shared scrim block's own comment states the limit (`dialog/style.css`, the identical block in `alert_dialog`, `sheet`, `drawer`): `close()` drops a `<dialog>` from the top layer synchronously and its `::backdrop` goes with it, so the exit only plays through `overlay ... allow-discrete` on the dialog itself, and "engines without `overlay` transitions keep the old snap-out". Firefox and Safari snap today (per the scrim lane; Chromium animates). The modal Popover already has the cross-browser form: `use_popover_modal_driver` (`primitives/src/popover.rs`) keeps the dialog modal until the content and scrim fades have both ended and only then calls `close()`, where `use_dialog_open_driver` (`primitives/src/lib.rs`) calls `close()` at once. **Construction:** give `use_dialog_open_driver` the same deferred close (wait for the panel's and the scrim's `animationend`/`transitionend`, bounded by a timeout, immediate under `prefers-reduced-motion`), so all five modal surfaces ride one driver and `overlay allow-discrete` becomes a refinement instead of the mechanism. Watch: reopen while closing, `use_dialog_close_sync` (Escape and form-method closes), `playwright/assert-backdrop-fade.ts` still green, and the proof needs Firefox and WebKit (row 4). **Landed 2026-10-05 (round 3).** Owner report 2026-10-05: Dialog close "glitches", a side Sheet "changes size" while closing, the Drawer "glitches". **Root cause (measured):** `use_dialog_open_driver` called `dialog.close()` the instant `open` went false, so the panel left the top layer in one frame, and on the HOME page a `position: fixed` panel is then sized and clipped by `.dx-component-card` (`content-visibility: auto`, added in round 2): Sheet 384x800 -> 384x256 and Drawer 1265x346 -> 1105x205 mid-slide. The component pages have no such ancestor, which is how the earlier verification missed it. **Construction:** the shared driver keeps the dialog open until its own and its `::backdrop`'s animations settle (bounded at 1.5 s, `DIALOG_EXIT_TIMEOUT_MS`), then calls `close()`; a generation counter guards a reopen; `transition: overlay allow-discrete` is gone from the stylesheets, so Firefox and Safari get a real exit by construction (executed on Chromium only: this sandbox has no Firefox or WebKit, row 4); `dialog:not([open]) { display: none }`; the modal Popover's web arm now calls the same driver. **Gates:** oracle `playwright/modal-exit.spec.ts` with `modal-consumers.ts` (consumers derived from source: dialog, alert_dialog and popover call the driver, sheet, drawer and command compose `DialogRoot`; a new modal component with no entry fails the first test) and `assert-modal-exit.ts` (a closing modal is never rendered outside the top layer, keeps one layout box on every rendered frame, and `close()` waits for the longest exit animation). **Found by the same pass:** the scroll lock released at unmount, ~250 ms after `close()`, not at the `open` flip as assumed, and now releases on the dialog's `close` event (rows 149 and 159); an upward Drawer drag closed the drawer because a handle press that ended over the backdrop read as a backdrop click, so backdrop dismiss now requires the press to have STARTED on the backdrop (every dialog); Drawer flick velocity now uses the events' `timeStamp`s (100 ms window, at least 10 px, a 25% distance rule, a rubber band, `pointercancel` snaps back). **Landed with it:** an `overlay` prop (default true; Popover default false) on DialogRoot, AlertDialogRoot, Drawer, CommandDialog and the styled wrappers, through a theme rule `:where(dialog[data-dx-overlay="off"])::backdrop { ... !important }`, `overlay` demos and `playwright/overlay-switch.spec.ts`. Measurements and the spec invariants: [round-3 note](../../research/round3-2026-10-05.md), section 1.
