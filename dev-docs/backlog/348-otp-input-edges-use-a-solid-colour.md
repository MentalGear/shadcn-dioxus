---
id: 348
title: "OTP Input edges use a solid colour, not the translucent dark input token"
status: "Open (owner approved 2026-10-20)."
size: "S"
area: components
updated: 2026-10-21
---

# Row 348: OTP Input edges use a solid colour, not the translucent dark input token

## Item

OTP Input edges use a solid colour, not the translucent dark input token.

## Status / trigger

Open (owner approved 2026-10-20).

## Size

S

## Detail

In dark mode the OTP slot edges are `inset` box-shadows in `--dx-input-otp-edge: var(--dx-input)` (`preview/src/components/input_otp/style.css` lines 49-74), and dark `--dx-input` is `oklch(100% 0 0deg / 15%)` (`preview/assets/dx-components-theme.css:304`). Neighbouring slots overlap by a pixel (`--dx-input-otp-end/-start`), so the translucent edge stacks and darkens where slots touch (screenshot `otp-input-border-overlap.png` in the project uploads).

Plan: add a theme token for an opaque input edge (the input colour mixed over `--dx-background`, in `--dx-hover-mix-space`) and point `--dx-input-otp-edge` and every other edge that can overlap at it. Keep `--dx-input` for fills and single borders. Check `check-hover-mix-space.sh` and the OTP specs after.
