---
id: 501
title: "select-escape-race.spec.ts:32 is flaky under load (also on the deployed base)"
status: "BACKLOG. Found by the batch verifier, 2026-11-07."
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

BACKLOG. Found by the batch verifier, 2026-11-07.

## Size

S

## Detail

"Flake" is not a root cause: reproduce under load (4 workers, repeat), record the event order on a failing run (the
Escape relative to the Select's pointer open and its dismiss layer registration, rows 487 and the Escape-gap rule), and
decide whether the spec observes a real race in the Select (fix the component) or races its own setup (fix the spec).
