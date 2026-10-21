---
id: 349
title: "Coarse sizing: global default plus a per-container `data-dx-coarse` attribute"
status: "Open (owner approved 2026-10-20)."
size: "M"
area: theme
updated: 2026-10-21
---

# Row 349: Coarse sizing: global default plus a per-container `data-dx-coarse` attribute

## Item

Coarse sizing: global default plus a per-container `data-dx-coarse` attribute.

## Status / trigger

Open (owner approved 2026-10-20).

## Size

M

## Detail

Today touch sizing is a raw `@media (pointer: coarse)` in 12 component sheets (`date_picker`, `textarea`, `code_block`, `slider`, `input`, `native_select`, `command`, `resizable`, `input_otp`, `calendar`, `combobox`, and `theme.rs`/`main.css`/the theme) and cannot be previewed or forced on a desktop. [Row 291](./done/291-coarse-pointer-sizing-for-menus-and-option-lists.md) (done) added the coarse menu and option sizes; this row generalises it.

Plan: `data-dx-coarse="auto|on|off"` on `<html>` or any container, like `data-dx-press-feedback` (`dx-components-theme.css` ~1262). `auto` (the default) follows `(pointer: coarse)`; `on`/`off` force it for the subtree. Coarse sizes become theme variables (`--dx-coarse-*`) that the attribute or the media query switches, and the component sheets read the variables. A new `scripts/check-*.sh` bans a raw `pointer: coarse` rule in a component sheet (reasoned allowlist, stale entries fail); add it to `run-gates.sh` and CLAUDE.md. Specs: forced on and off match the media-query result.
