---
id: 401
title: "check-select-css names a mutation that died instead of printing a blank verdict"
status: "LANDED. Each job now leaves <name>.verdict, <name>.err and <name>.status; the status is written by…"
size: "S."
area: tooling
updated: 2026-10-24
---

# Row 401: check-select-css names a mutation that died

## Item

`scripts/check-select-css.sh` runs four mutation self-tests as background subshells that each write a verdict file. Under load (a full `run-gates.sh --fast`) one subshell died and left an EMPTY verdict: the gate printed blank `check-select-css:` lines and went red with no reason (seen once; three runs alone were green).

## Status / trigger

LANDED. Each job now leaves `<name>.verdict`, `<name>.err` and `<name>.status`; the status is written by a wrapper around the job, so a job killed from outside still has one. `settle` prints the verdict, or `FAIL -- mutation 'X' produced no verdict (exit N): <stderr tail>` (137 is named as SIGKILL, typically the OOM killer). Parallelism is bounded by `MUTATION_JOBS` (default 2, was all four at once): each job is a copy of `playwright/` plus a node process, and memory pressure beside a full gate run is the suspected cause (the actual death was not captured, which is why the cause is named in the failure line rather than assumed). A job that dies without a verdict is retried once, alone, with a printed note, since losing a child to the machine says nothing about the promise under test; a second death is the FAIL. The mutation semantics are unchanged.

Evidence: five green runs in a row (16-23 s each); a `node` shim that SIGKILLs the job of `token-blind` once gives the note, the retry and a green gate, and always gives `FAIL -- mutation 'token-blind' produced no verdict (exit 137 = SIGKILL, typically the OOM killer)` and exit 1; the shim also showed at most two self-tests alive at once; three concurrent runs were all green.

## Size

S.

## Detail

Not done: lowering the cost of a mutation (copying `playwright/` four times) by mutating in place; the copies are what keep the mutation proof from touching the working tree.
