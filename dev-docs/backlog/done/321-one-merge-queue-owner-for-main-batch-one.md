---
id: 321
title: "One merge-queue owner for main (batch, one full gate and one suite per batch, deploy)"
status: "LANDED. The QUEUE format is documented (dev-docs/lane-rules.md \"Merge queue\", line 254) and in use…"
size: "S"
area: process
updated: 2026-10-24
---

# Row 321: One merge-queue owner for main (batch, one full gate and one suite per batch, deploy)

## Item

One merge-queue owner for main (batch, one full gate and one suite per batch, deploy).

## Status / trigger

**LANDED.** The QUEUE format is documented (`dev-docs/lane-rules.md` "Merge queue", line 254) and in use: merge queue batches 1-13 are on main (PRs #121-#140, batch 13 as `b8357ca8`), each with one full gate and one suite run, and batches 14 and 15 follow the same path (`dev-docs/STATUS.md`, "Merge queue").

## Size

S

## Detail

On 2026-10-20 four sessions each merged main into their own batch, so each re-merged and re-gated whenever main moved (three extra full gates for batch A) and two deploys collided. Agreed 2026-10-21: lanes and threads push fast-gated branches and add them to a QUEUE line on the `claims` branch; the owner merges the queue in batches, runs one full gate and one suite run per batch, merges to main and deploys; nobody else merges to main or deploys. Batch A (PR after #122) is the last uncoordinated merge. Close when `dev-docs/lane-rules.md` documents the QUEUE format (the owner writes it).
