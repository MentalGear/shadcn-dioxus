---
id: 226
title: "Touch opens: a blue focus ring on overlay panels, and the keyboard rises because a field is focused on open"
status: "LANDED 2026-10-18 (lane focus + follow-up lane mousefocus 8d1bec7c (mouse opens focus the first…"
size: "M"
area: overlays
updated: 2026-10-18
---

# Row 226: Touch opens: a blue focus ring on overlay panels, and the keyboard rises because a field is focused on open

## Item

Touch opens: a blue focus ring on overlay panels, and the keyboard rises because a field is focused on open

## Status / trigger

**LANDED 2026-10-18 (lane focus + follow-up lane mousefocus 8d1bec7c (mouse opens focus the first control, owner 2026-10-18)), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane focus)** -- was: **PROPOSED for the 2026-10-15 round (focus lane, with 223); waiting for the owner's go**

## Size

M

## Detail

Owner 2026-10-17 (iOS, deployed 83758d5): rings around panels/inputs after a touch open; "do not focus input elements as soon as a modal/overlay/popover opens" on touch. Evidence `research/touch-open-focus-2026-10-17.md` (Chromium iPhone emulation plus an iOS-like mode where a tapped button takes no focus). Row 206 IS deployed. Causes: (1) the ring is the UA `outline: auto` on the focused `<dialog>`/popover panel -- no panel sheet resets outline and `check-focus-outline-reset.sh` only sees rules that paint a ring; (2) on iOS a tap leaves focus on `<body>`, so the script-focused panel matches `:focus-visible`; (3) row 206's opener check reads `:focus-visible`, and iOS keeps focus on a previously tapped text input, which always matches it -> treated as a keyboard open -> first field focused (Sheet measured); (4) non-modal Popover has no modality logic (browser `showPopover()` focuses the first field: the keyboard rises); (5) Command autofocuses its input by design (row 206); (6) DropdownMenu/Menubar triggers, Checkbox and DatePicker nav force `focus()` after a pointer and ring. **Construction:** ONE modality writer (`primitives/src/modality.rs`; page-lifetime capture pointerdown/keydown -> `<html data-dx-modality>`, installed from `PREPAINT_JS` and as an eval prefix, coarse-pointer fallback; never reads `:focus-visible`/`activeElement`); ONE open-focus policy shared by `use_dialog_open_driver`, `use_popover_sync` and Command (keyboard: browser default; mouse: panel, Command keeps its input; touch/pen: panel, never an editable element, `[autofocus]` only if not editable); ONE ring rule (theme `outline: none` on programmatic focus containers; `--dx-ring-focus*` transparent under mouse/touch modality, `--dx-ring-field` kept so a tapped field still rings). Gate `check-open-focus.sh` (no `autofocus`/`set_focus(true)`/`.focus(` outside the helper without `focus-ok:`); spec `touch-open-focus.spec.ts` (every overlay, tap open -> active element not editable, no outline, no ring; stale-focused-input case; keyboard counterpart unchanged). Also `autocapitalize="off"` on the Command and Combobox inputs (row 227 research). Not covered: Combobox (its input is the trigger).
