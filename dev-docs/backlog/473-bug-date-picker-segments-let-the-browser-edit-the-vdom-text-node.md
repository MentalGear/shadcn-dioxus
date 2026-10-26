---
id: 473
title: "Bug: date picker segments let the browser edit the text node the vdom also writes"
status: "APPROVED 2026-10-26 by the owner (\"Ok for the rest\", backlog thread); a batch 30 lane. BACKLOG.…"
size: "M"
area: components
updated: 2026-10-26
---

# Row 473: Bug: date picker segments let the browser edit the text node the vdom also writes

## Item

Date picker segments are `contenteditable` spans whose text is a vdom child (`date_picker/segment.rs:519-538`, `{display_value}`, `contenteditable: !ctx.is_read_only()` at `:525`). Digits cancel the default action (`:271-281`), but Backspace and Delete do not (`:283-299`), and neither does any key outside the match, including Android's `Unidentified` (keyCode 229), which falls to `_ => ()` (`:358`). So the browser's editing default action edits the same text node that Dioxus rewrites with `SetText`: it deletes characters, inserts IME text, and can replace the node or insert a `<br>`. If the node is replaced, Dioxus keeps writing to a detached node. Inferred from the sources, not yet reproduced.

**Where:** Backspace on web and desktop; every digit on Android (web and the mobile app), where the keydown carries no usable key.

**Reproduce first:** type digits into a segment, press Backspace and Delete, and compare the segment's DOM text node and its `display_value` signal; on Android, type a digit through the IME and check the same pair; then check whether the node was replaced (a detached node still receiving `SetText`).

**Source:** [`board-2026-10-26-r6/order-slots.md`](../research/board-2026-10-26-r6/order-slots.md) finding 7 (carried by [`board-2026-10-26-r6/SYNTHESIS.md`](../research/board-2026-10-26-r6/SYNTHESIS.md), "Live defects found in today's code").

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's contenteditable slot (root doc section 2.2) subsumes this defect later, and the fix does not wait for the plan.

## Status / trigger

APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); a batch 30 lane.

BACKLOG. Found by board round 6; reproduce first. Independent of plan approval.

## Size

M

## Detail

- **Files owned:** To be found by the reproduction; expect `primitives/src/date_picker/segment.rs`.
- **Depends on:** None. The plan's "contenteditable subtree" slot (a Rust-rendered contenteditable cancels every editing default action: keys by rsx `prevent_default`, `beforeinput` through an `otpFilter`-like behaviour, since Dioxus 0.7 has no `onbeforeinput`; a gate rule against an rsx text child inside a contenteditable without it) subsumes it; the alternative the board named is a real input with `inputmode` instead of `contenteditable`. Roving focus ([row 462](./462-port-roving-focus-78-set-focus-sites.md), conditional on the phase-0 focus-write witness) covers the segments' auto-advance if that witness is red; otherwise the auto-advance keeps today's `set_focus`.
- **Proof:** A Playwright spec that fails before the fix and passes after (Backspace, Delete and a composed input on a segment); run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess M).
