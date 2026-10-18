---
id: 209
title: "Date picker: Apply/Cancel confirm as the default; verify the natural-language demo against shadcn"
status: "LANDED 2026-10-14 (date lane 54ebc1d)"
size: "M"
area: inputs
updated: 2026-10-14
---

# Row 209: Date picker: Apply/Cancel confirm as the default; verify the natural-language demo against shadcn

## Item

Date picker: Apply/Cancel `confirm` as the default; verify the natural-language demo against shadcn

## Status / trigger

**LANDED 2026-10-14 (date lane 54ebc1d)**

## Size

M

## Detail

Owner decision 2026-10-14: `confirm` DEFAULT (stage in panel; Apply commits+closes; Cancel/Escape/outside restores); `confirm: false` keeps close-on-select. First verify shadcn's Natural Language Picker (chrono-node implies a text input; owner believes there is none) and per-demo open/close behaviour. Item 6. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Research (verified against shadcn source) corrected two premises:** shadcn's Natural Language picker DOES have a text input (chrono-node), and NO shadcn date demo has Apply/Cancel -- the confirm default is the owner's own design (D2), not parity. **Landed:** primitive `confirm` prop (primitive default false, styled default true), one `use_staging` hook (the staged selection is dropped whenever `open` changes, so Escape/outside/trigger/Cancel need no special code), a `DatePickerConfirm` handle via `use_date_picker_confirm()` (enabled/can_apply/apply/cancel); Apply is disabled until staged != committed (a range reports only when its end is picked); input and `on_value_change` keep the committed value while staging; with confirm, `close_on_select`/`close_delay` are ignored. Styled: `apply_label`/`cancel_label`, `DatePickerFooter` with a real Button (`component.json` adds button). Demos: main + range = the library default (confirm); keep_open/multi_month/internationalized = confirm off and stay open (shadcn twins); time + unavailable_dates close on select. `natural_language` rewritten to shadcn's (Field "Schedule Date", InputGroup, placeholder "Tomorrow or next week", initial "In 2 days", parse on input, an unparseable string keeps the previous date, ArrowDown opens, "Select date" icon button inside the group, end-aligned popover, select writes the date and closes, "Your post will be published on <date>."); the parser gained in N days/weeks/months, N days ago, next/last week/month, weekdays, ISO, "October 20, 2026", "20 Oct". 35 new tests, `date-picker.spec` 78 passed. **Not ported / open:** `captionLayout="dropdown"` (our Calendar lacks it; row 155 group 8), `sideOffset`; the internationalized demo does not pass `apply_label`/`cancel_label`, so its footer is not localized.
