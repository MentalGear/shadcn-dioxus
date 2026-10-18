---
id: 84
title: "Date picker — a11y/UX gaps found but not fixed this round"
status: "Landed 2026-09-20 (round 4, c03f021) — all six findings closed in one pass"
size: "S–M"
area: inputs
updated: 2026-09-20
---

# Row 84: Date picker — a11y/UX gaps found but not fixed this round

## Item

Date picker — a11y/UX gaps found but not fixed this round

## Status / trigger

**Landed 2026-09-20 (round 4, `c03f021`)** — all six findings closed in one pass

## Size

S–M

## Detail

Six findings from the same investigation that landed row 20's closure, recorded rather than fixed (out of that round's own scope): (1) `Calendar`'s unavailable days carry no `aria-disabled` (`calendar.rs`); (2) the month segment has no `aria-valuetext`, so a screen reader announces a bare number instead of a month name; (3) `aria-valuenow` on an untouched segment shows today's date before the person has chosen anything, rather than being absent/indeterminate; (4) Up/Down arrow wraps a segment's value (including the year segment, where wrapping is almost never wanted) instead of clamping; (5) the demo's own i18n language switcher is dead code — `main.rs`'s `i18n().set_language(id)` call is commented out; (6) the first Up/Down keypress on a segment that has never been touched commits that segment's default value with no visible change on screen, which reads as a missed keypress. None of these block row 20's own closure; filed together since one future pass on this primitive's remaining a11y surface can address all six at once. **Closed 2026-09-20 (round 4, `c03f021`):** findings 2, 3 and 6 turned out to be one construction, not three independent fixes — `aria-valuenow`/`aria-valuetext` now mirror `props.value`'s own `Option` faithfully (present once the segment actually holds a value, omitted while it doesn't) instead of substituting `props.default` the way the old `unwrap_or(props.default)` did; that alone fixes finding 3 (no more leaked default) and finding 6 as its direct corollary — an untouched segment's `aria-valuenow` was already reporting the default before any keypress, so the first Up/Down press committing that same default looked like a dropped keystroke, and once "no value" stops being reported as "= default" that same first press becomes a genuine attribute-absent-to-present transition. Finding 2's locale-aware month name rides the same "has a value" gate, via a new `on_format_month: Callback<Month, String>` threaded down to `DatePickerMonthSegment` (and the DateRangePicker's mirror path), defaulting to `Month::to_string()` — the same default `DatePickerCalendarProps` already uses for the grid. Finding 4 became an explicit per-segment wrap/clamp policy (`SegmentValueBounds.wrap`) rather than a blanket fix, calibrated against Chromium's own native `<input type="date">` and the APG spinbutton pattern: month/day stay cyclical (`roll_value`), the year segment now clamps at its bound (`clamp_value`) instead of jumping to the opposite end, since a year is not cyclical and Chromium's own year sub-field does not wrap either. Finding 1 is a direct fix, `aria-disabled` added to `calendar_day_attributes`, mirroring `data-disabled` exactly (same source value, same unconditional presence). Finding 5's i18n switcher is now wired rather than deleted — `main.rs`'s `i18n().set_language(id)` call, previously commented out, is live (a contained two-line fix: import `i18n`, make the call), per this task's own stated preference for a working control over a deleted one. Evidence: every finding got a Playwright test confirmed RED on the unmodified tree (via a git-stash round-trip, so RED is the real base commit, not reasoning) and GREEN after, plus new Rust SSR/unit tests in both `calendar.rs` and `date_picker.rs` (the wrap-vs-clamp unit test additionally verified red by temporarily reverting just that branch); one pre-existing test (the non-digit-keystroke guard) needed updating for the now-legitimate `aria-valuenow === null` case, no other collateral hits found. Left deliberately alone, unrelated: the two pre-existing row-21 popover-anchoring failures (Chromium's own anchor-freeze)
