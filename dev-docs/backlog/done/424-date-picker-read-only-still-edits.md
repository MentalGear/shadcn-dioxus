---
id: 424
title: "DatePicker `read_only` still lets keys change the value and the popover pick a date"
status: "DONE 2026-10-25. Found by row 393's docs lane. The prop docs (primitives and styled, DatePicker and…"
size: "S"
area: components
updated: 2026-10-25
---

# Row 424: DatePicker `read_only` still edits

## Item

`read_only` only sets `contenteditable="false"` on the segments (`primitives/src/date_picker/segment.rs:484`). Digit and arrow keys still change the value, and the popover still opens and picks. A read-only field should keep focus and selection and refuse every edit, as a native `readonly` input does.

## Status / trigger

DONE 2026-10-25. Found by row 393's docs lane. The prop docs (primitives and styled, `DatePicker` and `DateRangePicker`) are back to the plain contract.

## Size

S

## Detail

- Gate every value write (segment keys, popover pick, clear) on one `read_only` read, so a new edit path cannot skip it. Keep the trigger focusable, set `aria-readonly`, and keep the popover closed or view-only.
- A spec covers keys, popover and clear on a read-only picker, and the prop doc goes back to the plain contract.

## Resolution

- One `read_only` read below the picker root: `BaseDatePickerContext::is_read_only` (`primitives/src/date_picker/context.rs`). The segments (edit keys, the out-of-range clamp), the popover (it refuses to open, and closes if the prop turns on while open) and both calendars (a pick is dropped) ask it; the value callbacks are wrapped once per picker by `use_unless_read_only`, which hands `use_staging` and the date or range context the wrapper, so a path that forgets the check still cannot call `on_value_change` / `on_range_change` (Apply, the calendar's toggle-off, a segment's Backspace clear).
- A read-only segment keeps focus, selection and the ArrowLeft/ArrowRight/Enter focus keys; digits, Up/Down, Home/End, Backspace and Delete do nothing (`read_only_key_effects`, a pure fn with unit tests). Segments carry `aria-readonly="true"`, the picker root `data-read-only="true"`, and the styled trigger stays focusable with `aria-disabled="true"`.
- Spec: `playwright/date-picker-read-only.spec.ts` on the new `read_only` demo, with an editable-picker control. Unit tests: `date_picker::tests` (key effects, SSR attributes).
