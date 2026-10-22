---
id: 349
title: "Coarse sizing: global default plus a per-container `data-dx-coarse` attribute"
status: "LANDED 2026-10-21 (lane coarse): data-dx-coarse=auto|on|off, sizes as theme variables…"
size: "M"
area: theme
updated: 2026-10-21
---

# Row 349: Coarse sizing: global default plus a per-container `data-dx-coarse` attribute

## Item

Coarse sizing: global default plus a per-container `data-dx-coarse` attribute.

## Status / trigger

LANDED 2026-10-21 (lane coarse): `data-dx-coarse=auto|on|off`, sizes as theme variables, `check-coarse-sizing.sh`.

## Size

M

## Detail

Today touch sizing is a raw `@media (pointer: coarse)` in 12 component sheets (`date_picker`, `textarea`, `code_block`, `slider`, `input`, `native_select`, `command`, `resizable`, `input_otp`, `calendar`, `combobox`, and `theme.rs`/`main.css`/the theme) and cannot be previewed or forced on a desktop. [Row 291](./291-coarse-pointer-sizing-for-menus-and-option-lists.md) (done) added the coarse menu and option sizes; this row generalises it.

Plan: `data-dx-coarse="auto|on|off"` on `<html>` or any container, like `data-dx-press-feedback` (`dx-components-theme.css` ~1262). `auto` (the default) follows `(pointer: coarse)`; `on`/`off` force it for the subtree. Coarse sizes become theme variables (`--dx-coarse-*`) that the attribute or the media query switches, and the component sheets read the variables. A new `scripts/check-*.sh` bans a raw `pointer: coarse` rule in a component sheet (reasoned allowlist, stale entries fail); add it to `run-gates.sh` and CLAUDE.md. Specs: forced on and off match the media-query result.

## Landed

Theme "Coarse sizing" block: two switch variables (`--dx-pick-coarse`/`--dx-pick-fine`, one is `initial`, the other empty) flipped by `(pointer: coarse)`, `[data-dx-coarse=on|off|auto]`; every size is `var(--dx-pick-coarse, C) var(--dx-pick-fine, F)` declared on `:root, [data-dx-coarse]` (a var()-holding custom property resolves where declared, so it must be redeclared per container). Sizes: menu tokens (row 291), `--dx-coarse-text-entry-font-size/-line-height`, `--dx-coarse-slider-thumb-size/-margin`, `--dx-coarse-resizable-hit-size`, `--dx-coarse-actions-opacity`. `auto` is now `(pointer: coarse)` (was `(any-pointer: coarse) and (hover: none)` for menus). 11 component sheets plus the email-client sheet lost their raw media rules; main.css's raw-control backstop now skips `dx-` controls (it would have beaten a forced `off`). Gate `check-coarse-sizing.sh` (allowlist: theme, main.css). Spec `coarse-sizing.spec.ts` (on/off/auto/nested/invalid, fine and emulated-coarse).
