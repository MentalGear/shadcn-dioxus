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

## Superseded (2026-10-25)

The opaque `--dx-input-edge` token is removed. It was `--dx-input` pre-composited over `--dx-background`, so on any other surface (a card, the preview frame) and over the slot's own dark fill it no longer matched a normal Input edge, and the OTP edges read too faint (owner report 2026-10-25). The OTP slots already draw each divider once (top, bottom and inline-end per slot, inline-start on the first), so they read `--dx-input` again, the same edge as Input. Button Group's seam (the "left open" item) is fixed by construction instead: a follower is clipped by the one pixel column it overlaps, so only the previous child paints the seam, as Nova's `border-l-0` does (`button_group/style.css` "The seam"; spec `button_group.spec.ts` "Seams match the outer edge", which compares screenshot pixels).
