---
id: 40
title: "Auto-kind popovers' exposure to the opening-gesture light-dismiss false positive — audited 2026-09-03; Navbar…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Navbar half FIXED 2026-10-04 (d9bcfda…"
size: "M (done)"
area: tests
updated: 2026-10-19
---

# Row 40: Auto-kind popovers' exposure to the opening-gesture light-dismiss false positive — audited 2026-09-03; Navbar…

## Item

**`Auto`-kind popovers' exposure to the opening-gesture light-dismiss false positive -- audited 2026-09-03; Navbar half FIXED
2026-10-04 (`d9bcfda`), the Menubar/DropdownMenu/Select/Popover half closed as refuted 2026-10-03.** `DropdownMenu`, `Menubar`,
`Select` and non-modal `Popover` (`PopoverKind::Auto`) rely on the browser's native light-dismiss; the worry was an engine whose
touch-to-click synthesis retargets a click at a DOM mutation under it (the class of quirk `native-dialog.spec.ts` Rule 8 fixes for
the modal arm), which could read the tap that opened one as "outside". Not reproducible in this sandbox's Chromium.

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). **Navbar half FIXED 2026-10-04 (`d9bcfda`, verified on the release build of `b9e9580`); the rest CLOSED as refuted 2026-10-03, so no device is needed any more.** Original trigger: A real device confirming this actually happens (unlike the modal-arm case, no construction fix was applied here this round — see below for why), or a report against one of these four specifically

## Size

M (done)

## Detail

**Audit correction 2026-09-26.** `popovertarget` (the WHATWG declared invoker) was evaluated and rejected: every trigger is a
toggle button and `popovertarget`'s default action is toggle too, so layering a browser-native toggle onto the crate's own risks
a double-toggle race. `Manual` plus `use_outside_dismiss` (Combobox's precedent) was deferred: it would lose `auto`'s free
cross-instance exclusivity for four components with green suites.

**Re-audit 2026-10-03 (main at `2241da8`).**

- *Refuted for the click-open family.* The WebKit quirk retargets a synthesized `click`, but native light dismiss never reads
  `click`: it uses trusted `pointerdown` plus `pointerup`, and ignores a popover that is not yet open at `pointerdown`/`pointerup`.
  `DropdownMenu`, `Select` and non-modal `Popover` open from `click` (after `pointerup`), `Menubar` from `onpointerup`, so none can
  be dismissed by its own opening gesture on any engine. No `popovertarget` is needed, which also dissolves the double-toggle worry.
- *The real exposure the audit missed.* `NavbarTrigger` opened on `onpointerdown` and its content is `popover="auto"`: if
  `showPopover()` lands between `pointerdown` and `pointerup` (any real finger hold), `pointerup` sees an open auto popover with
  `sameTarget` null equal to null and the browser hides it. `navbar.spec.ts` cannot see it (Playwright `tap()` has zero hold).
  Probe: `playwright/oracle/tier2-html/auto-popover-opening-gesture.spec.ts` (CDP touchStart, 250 ms hold, touchEnd; asserts no
  `toggle` to closed on any `[popover]`; Navbar the suspected red, DropdownMenu the green control).

**Resolution 2026-10-04 (Navbar lane, `d9bcfda`).** The probe was RED on release `9f8399d` before the fix
(`toggles=[{open},{closed}]`) and the control green, so the exposure was real. `NavbarTrigger` now toggles on `onclick` (after
`pointerup`); `onpointerdown` keeps only `prevent_default()` (the blur guard, so tapping a child menuitem does not blur the
trigger) plus a snapshot `open_at_down = Some(is_open())` that `onclick` consumes, needed because on touch `NavbarNav`'s compat
`onmouseenter` hover-open fires between `pointerup` and `click` and would otherwise close the menu the tap just opened. A click
with no snapshot (Space, AT) falls back to live state; any `keydown` clears a stale snapshot. A grep of every `popover="auto"`
consumer found none other opening from `pointerdown`, and the class is now a written contract on `top_layer::PopoverKind::Auto`
("auto popovers are shown only from click, pointerup, keyboard, hover-compat or timer, never `onpointerdown`"), pinned by the
probe. Verified on the release build of `b9e9580`: probe 2/2 green, `navbar.spec.ts` 5/5, `keyboard-matrix.spec.ts` 77 passed /
1 skipped.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/040-auto-kind-popovers-exposure-to-the-opening.md`.*
