---
id: 419
title: "Toast close button: a bigger hit target and a hover style"
status: "LANDED (lane/toast-close-target, for merge queue batch 21). Trigger for reopening: a close target…"
size: "S"
area: components
updated: 2026-10-25
---

# Row 419: Toast close button: a bigger hit target and a hover style

## Item

Owner, 2026-10-25: the toast close button must be easier to hit and have a hover style. It was the bare `×` glyph (about 18 px, no padding) and its only hover was a colour change, which the error toast had to undo with an opacity rule.

## Status / trigger

LANDED (lane/toast-close-target, for merge queue batch 21). Trigger for reopening: a close target under 28 px on a fine pointer, or a hover tint that does not read on a typed fill.

## Size

S

## Detail

`.dx-toast-close` is a square target the size of the Sheet's close: `--dx-control-height-sm` (28 px) on a fine pointer and 2.25rem (36 px) on a coarse one, picked through `--dx-pick-coarse`/`--dx-pick-fine`. Negative block and inline-end margins let it reach into the toast's padding, so a one-line toast keeps its height. Hover (inside the fine-pointer media query) and press tint the target with `color-mix` of the toast's own foreground, so one rule reads on the plain toast and every typed fill, and the error toast's opacity override is gone. A focus-visible ring matches the Sheet's close. `toast.spec.ts` checks the size and the hover tint.
