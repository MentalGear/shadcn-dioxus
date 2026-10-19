---
id: 206
title: "Modal initial focus by opening modality (no mobile keyboard pop on tap)"
status: "LANDED 2026-10-14 (focus lane c1b2505, then 3fa14b1)"
size: "S-M"
area: a11y
updated: 2026-10-14
---

# Row 206: Modal initial focus by opening modality (no mobile keyboard pop on tap)

## Item

Modal initial focus by opening modality (no mobile keyboard pop on tap)

## Status / trigger

**LANDED 2026-10-14 (focus lane c1b2505, then 3fa14b1)**

## Size

S-M

## Detail

Owner request. Keyboard-opened: first field; pointer/touch-opened: the panel itself (`tabindex=-1`); explicit `autofocus` wins. One place in the dialog/focus-trap primitive covers Dialog, AlertDialog, Sheet, Drawer, Command, modal Popover. Item 3. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** every native-`<dialog>` modal opens through `use_dialog_open_driver` (`primitives/src/lib.rs`); at open time the opener (`document.activeElement`) decides: keyboard iff it is not `<body>` and matches `:focus-visible` -> browser default (first control). Otherwise (mouse, tap, Safari's unfocused clicked button, `default_open`, a shortcut with focus on body) -> the panel itself: `tabindex=-1` plus inert children around `showModal()` and `focus({preventScroll})`. An `[autofocus]` descendant always wins (the Command palette input keeps focus on tap, as in shadcn, so the phone keyboard still pops for the palette by design). Covers Dialog, AlertDialog, Sheet, Drawer, CommandDialog and modal Popover, on every page. The first construction (a page-level modality tracker installed by Dialog/AlertDialog roots) was rejected in review: install-order dependency (the modal Popover behaved differently depending on whether a dialog was on the page); the `:focus-visible` read at open time replaced it (3fa14b1). `modal-initial-focus.spec` 35 tests (including a page with no Dialog); combined run 213 passed. Known limit, documented in `dialog.rs`: a keyboard open, Escape, then a mouse click with no other pointer interaction keeps first-control focus (today's behaviour).
