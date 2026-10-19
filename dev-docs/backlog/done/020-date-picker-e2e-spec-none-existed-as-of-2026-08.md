---
id: 20
title: "date_picker e2e spec — none existed as of 2026-08-31 (calendar.spec.ts does not cover DatePicker)"
status: "Landed 2026-09-19 (batch 3) — the rest of this row's original scope (segment…"
size: "— (closed)"
area: tests
updated: 2026-09-19
---

# Row 20: date_picker e2e spec — none existed as of 2026-08-31 (calendar.spec.ts does not cover DatePicker)

## Item

date_picker e2e spec — none existed as of 2026-08-31 (calendar.spec.ts does not cover DatePicker)

## Status / trigger

**Landed 2026-09-19 (batch 3)** — the rest of this row's original scope (segment typing/arrows/backspace, range picker) is now covered

## Size

— (closed)

## Detail

22 new tests landed (`779bc9f`): segment typing ×4, arrows ×3 (incl. Home/End), Left/Right/Backspace ×3, tab order, aria ×3, range ×4, internationalized ×2, unavailable ×1, one known-defect regression test. Two real fixes alongside (`11a98ff`): Home/End now work on a date segment, and a dangling `aria-labelledby` on date segments (pointing at an id that does not exist) is removed — verified live, segment focus 9→1→12 and dangling ids 24→0. **Two page hangs were found and fixed** during this same pass, in `DateRangePicker` only (the single `DatePicker` never hung, since its own context already guarded equal writes) — see the new row 83 for the shared mechanism and construction. **Other findings, not fixed:** see row 84. Row 21's own anchoring residue was re-measured with more precision, not re-investigated further — see row 21's own 2026-09-19 addendum
