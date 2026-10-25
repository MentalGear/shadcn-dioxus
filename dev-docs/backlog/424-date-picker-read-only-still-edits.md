---
id: 424
title: "DatePicker `read_only` still lets keys change the value and the popover pick a date"
status: "BACKLOG. Found by row 393's docs lane (2026-10-25). The prop docs describe the current behaviour…"
size: "S"
area: components
updated: 2026-10-25
---

# Row 424: DatePicker `read_only` still edits

## Item

`read_only` only sets `contenteditable="false"` on the segments (`primitives/src/date_picker/segment.rs:484`). Digit and arrow keys still change the value, and the popover still opens and picks. A read-only field should keep focus and selection and refuse every edit, as a native `readonly` input does.

## Status / trigger

BACKLOG. Found by row 393's docs lane (2026-10-25). The prop docs describe the current behaviour until this lands.

## Size

S

## Detail

- Gate every value write (segment keys, popover pick, clear) on one `read_only` read, so a new edit path cannot skip it. Keep the trigger focusable, set `aria-readonly`, and keep the popover closed or view-only.
- A spec covers keys, popover and clear on a read-only picker, and the prop doc goes back to the plain contract.
