---
id: 83
title: "DateRangePicker page hang — DateElement's sync-down/sync-up effects form an unconditional two-way reactive…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Found and fixed 2026-09-19 (batch 3…"
size: "S"
area: inputs
updated: 2026-10-19
---

# Row 83: DateRangePicker page hang — DateElement's sync-down/sync-up effects form an unconditional two-way reactive…

## Item

DateRangePicker page hang — `DateElement`'s sync-down/sync-up effects form an unconditional two-way reactive cycle

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). Found and fixed 2026-09-19 (batch 3, date-picker + range-hang lanes)

## Size

S

## Detail

**Same family as rows 73/74** (a reactive-cycle/hook-discipline defect that froze the whole tab, found live during a Playwright run, not by any static check). Two page hangs, confirmed to be ONE mechanism: (1) typing a complete year+month+day into a `DateRangePicker`'s own segment inputs (no popover involved) hangs the JS thread forever, moments after the demo logs the correct value; (2) opening a range popover, clicking a start day then an end day (completing the range) hangs identically. Root cause: `DateElement`'s sync-down effect (reads `props.selected_date`, writes the segment's year/month/day) and its sync-up effect (reads the three segments, calls `on_date_change`, which writes `start_date`/`end_date` back) form an unconditional two-way cycle, and `Signal::set` (dioxus-signals 0.7.9, `*self.write() = value`) marks dirty and reschedules subscribers **unconditionally** — so once the cycle completes, each hop re-triggers the next, forever. Evidence: the renderer measured at 97-125% CPU with RSS growing +4MB/s unbounded; CDP `Profiler`/`Debugger` commands never returned (the render thread never yields — a synchronous wasm loop, not a slow one). The single `DatePicker`/`RangeCalendar` alone never hung, because `DatePickerContext::set_date`/`DateRangePickerContext::set_range` already guarded against an equal write. Construction (`b66d00e`): one guarded choke point, `DateRangeInputContext::set_start_date`/`set_end_date` (`if *self.start_date.peek() != date { ... }`), every producer of a start/end write now routed through it, mirroring the equality guard the context-level setters already had one layer up — no change needed to `calendar.rs`/`popover.rs`/`top_layer.rs`. Verified (`c899951`): both of `date-picker.spec.ts`'s own `test.fail()` hang guards now pass; the full targeted 22-file batch finished in 3.9 minutes (a hang would have burned the whole per-test timeout); re-confirmed in batch-3 integration's own runs, every time. **Standing rule, proposed by this finding, not yet automated:** any write that closes a reactive cycle must be conditioned on the value actually changing, because `Signal::set` never is by itself. `scripts/check-self-subscribing-effects.sh` (row 79) does **not** catch this class — verified directly, it reports clean on the pre-fix file, because this is a multi-effect cycle closed through a `Callback`/method call, not one effect's own tracked-read-then-write the existing heuristic looks for. A wider check (whole-file read/write set plus `Callback` resolution plus guarded-vs-unconditional detection) is specified but **not built**: a narrower heuristic risks false-positiving on every legitimate parent-notification effect in the codebase — left as a review-only rule until a real construction is designed
