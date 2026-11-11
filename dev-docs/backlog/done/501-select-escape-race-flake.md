---
id: 501
title: "select-escape-race.spec.ts:32 is flaky under load (also on the deployed base)"
status: "DONE 2026-11-07, landed in PR #168. Root cause: the press's deferred open (OpenGap::defer, one task…"
size: "S"
area: playwright
updated: 2026-11-07
---

# Row 501: select-escape-race.spec.ts:32 is flaky under load

## Item

`playwright/select-escape-race.spec.ts:32` failed in both full changed-spec runs of the 2026-11-07 batch, passes alone
three times, and fails 2 of 48 runs at 4 workers with `--repeat-each=12` on the batch head and 4 of 48 on the deployed
base (`d040554a`), so the batch did not cause it.

## Status / trigger

DONE 2026-11-07, landed in PR #168. Root cause: the press's deferred open (`OpenGap::defer`, one task after `pointerdown`) was only
cancelled by an Escape that reached the trigger. In the failing interleaving the press's task had not run yet when
ArrowDown (a keydown on the trigger) opened the list and moved focus onto an option; the dismiss layer registered, Escape
was taken by it (focus was on the option, so the trigger's gap arm never saw the key) and closed the list
(`aria-expanded` false, focus back on the trigger); then the still-pending press task ran `open_for_pointer` and
reopened the list (`aria-expanded` true, focus on the listbox), which stayed open. Recorded with capture-phase
listeners and a MutationObserver on a failing run: `keydown:ArrowDown`, focusin option, `keydown:Escape`, focusin
trigger, `aria-expanded=false`, focusin listbox, `aria-expanded=true`. Under 4 workers on the debug SSG build the spec
failed 17 of 20 runs on the base (`--repeat-each=20`). Fix: a deferred open is void once anything else has written the
open state since the press (`SelectableContext::open_writes`, bumped by every `set_open` call that changes the state, read by
`OpenGap::current`; a call that repeats the current value does not count, because the popover's late close echo
would void the next press's open), so a late timer cannot repeat or undo what a key, a selection or a dismissal did; `pending()` is
false for a voided wait, so the trigger's Escape arm no longer swallows a key for it. Subsumes every close source
(dismiss layer, option blur, list blur, selection, popover close echo). After: 200 of 200 passed at
`--repeat-each=50 --workers=4` (all four tests of the spec).

## Size

S

## Detail

"Flake" is not a root cause: reproduce under load (4 workers, repeat), record the event order on a failing run (the
Escape relative to the Select's pointer open and its dismiss layer registration, rows 487 and the Escape-gap rule), and
decide whether the spec observes a real race in the Select (fix the component) or races its own setup (fix the spec).
