---
id: 348
title: "OTP Input edges use a solid colour, not the translucent dark input token"
status: "LANDED 2026-10-21 (lane coarse): --dx-input-edge opaque token, OTP edges use it. Other overlapping…"
size: "S"
area: components
updated: 2026-10-21
---

# Row 348: OTP Input edges use a solid colour, not the translucent dark input token

## Item

OTP Input edges use a solid colour, not the translucent dark input token.

## Status / trigger

LANDED 2026-10-21 (lane coarse): `--dx-input-edge` opaque token, OTP edges use it. Other overlapping edges (Button Group outline seam) not changed.

## Size

S

## Detail

In dark mode the OTP slot edges are `inset` box-shadows in `--dx-input-otp-edge: var(--dx-input)` (`preview/src/components/input_otp/style.css` lines 49-74), and dark `--dx-input` is `oklch(100% 0 0deg / 15%)` (`preview/assets/dx-components-theme.css:304`). Neighbouring slots overlap by a pixel (`--dx-input-otp-end/-start`), so the translucent edge stacks and darkens where slots touch (screenshot `otp-input-border-overlap.png` in the project uploads).

Plan: add a theme token for an opaque input edge (the input colour mixed over `--dx-background`, in `--dx-hover-mix-space`) and point `--dx-input-otp-edge` and every other edge that can overlap at it. Keep `--dx-input` for fills and single borders. Check `check-hover-mix-space.sh` and the OTP specs after.

## Landed

New theme token `--dx-input-edge` (light: `--dx-input`; dark: white 15% mixed into `--dx-background` in `--dx-hover-mix-space`); `--dx-input-otp-edge` reads it. Spec: `input_otp.spec.ts` "Slot edges are opaque" (light and dark). Left open: the Button Group seam (`button_group/style.css`, `margin-inline-start: -1px` over `--dx-input` inset rings) overlaps the same way in dark and was not in this lane's files.
