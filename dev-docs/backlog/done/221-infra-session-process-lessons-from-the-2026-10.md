---
id: 221
title: "Infra: session process lessons from the 2026-10-14/15 batch (memory, lock fairness, gate worktree…"
status: "DONE 2026-10-19 (lane lock 08b2966d): FIFO ticket queue scripts/lane-lock.sh (gate before lane…"
size: "S-M"
area: process
updated: 2026-10-19
---

# Row 221: Infra: session process lessons from the 2026-10-14/15 batch (memory, lock fairness, gate worktree…

## Item

Infra: session process lessons from the 2026-10-14/15 batch (memory, lock fairness, gate worktree, late-landing gates, account identifiers)

## Status / trigger

**DONE 2026-10-19 (lane lock 08b2966d): FIFO ticket queue scripts/lane-lock.sh (gate before lane, LANE_SLOTS slots, MemAvailable guard, legacy ssg.lock interop on slot 0, killed holders and waiters never block), wired into lane-ssg, gate-at and lane-cargo, gate check-lane-lock; proven with real contention (two concurrent 96-route builds, marker in both). Lessons: dx's SSG pre-render port 9999 is a hidden single-holder resource, now isolated per build in its own network namespace (scripts/lane-netns.sh); the SIGKILL-of-wrapper gap is documented; old-style flock callers can still starve queued slot-0 waiters; Playwright beside 2 slots is risky (LANE_SLOTS=1 while it runs)**

## Size

S-M

## Detail

**(1) Memory.** About ten concurrent agents were OOM-killed at ~17:30 UTC (the kernel killed clippy and Chrome; the machine has 15 GB, a `preview` rustc or clippy driver is ~2.5 GB and clippy runs the bin and test drivers in parallel). Rule now: every cargo or dx command takes ONE global heavy-job lock and runs with `CARGO_BUILD_JOBS=2`, Playwright at most `--workers=2`, and a crash or "Target closed" is checked against `dmesg` before it is believed. **(2) Lock fairness.** `flock` is not FIFO: the main loop's gate and deploy jobs waited 1-1.5 h behind about eight lane jobs. Candidate: a ticket (FIFO) lock or a priority lock for gate and deploy jobs. **(3) Gates in a pinned worktree.** A merge into the main checkout while `run-gates.sh` was reading it left conflict markers under the run (it aborted in seconds; the test step had passed on the clean tree). Construction used: gates run in a pinned detached worktree (`gate-at.sh <commit> <log>`); candidate for `scripts/run-gates.sh --at <commit>`. **(4) Gates that land after an agent branches.** Several lanes went red on merge because they never ran gates that landed after they branched; the brief was changed: before handback, merge the session branch into the lane branch and run the full `scripts/run-gates.sh`, and the main loop still runs it on the merged tree. **(5) Account identifier leak.** A research agent sent the owner's account email in the `User-Agent` of its first crates.io request batch (the agent recorded it in its own report; its later requests used a generic agent string). Rule added to the lane brief: never send the user's email, name or any account identifier to an external service (User-Agent, headers, query strings, payloads); use a generic agent such as `shadcn-dioxus-research`. Candidate: also state it in CLAUDE.md. Row 216 (shared target mtime hazard) is the sibling infra row. See `dev-docs/agent-model-fit.md`. **2026-10-18 lock-throughput analysis and owner decision:** after a container restart, ten relaunched lanes each needed at least one lane build and one full `run-gates.sh` (15-21 min each: baseline 922 s, masonry 976 s, keyboard 1113 s, a merge gate 1236 s) on ONE exclusive lock, about 4 h of serial lock time; measured waits: keyboard 36 min wall for 18 min of gates, masonry and charts over an hour queued; first-run reds (keyboard 5 nits) cost a whole extra slot; bare `flock` picks a random waiter; the shared target plus the mtime touch (disk allows no per-lane target) makes every run a full workspace recompile at `CARGO_BUILD_JOBS=2`. Owner adopted three procedures (`lane-rules.md` "Gate procedure", CLAUDE.md lane bullet): (1) lanes run only the fast gates, the main loop runs the full gates once per merge batch; (2) first-come, first-served lock with merge gates first; (3) 4-5 lanes per batch. Still to build: a ticket-queue wrapper replacing bare `flock` in `lane-ssg.sh`/`gate-at.sh` (FIFO with a priority lane for merge gates); freeing ~6 GB so a second lock slot (two concurrent builds fit in memory) becomes possible.
