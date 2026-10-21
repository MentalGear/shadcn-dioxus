---
id: 321
title: "One merge-queue owner for main (batch, one full gate and one suite per batch, deploy)"
status: "OPEN, documented (owner approved 2026-10-21; the QUEUE format is in lane-rules.md \"Merge queue\"…"
size: "S"
area: process
updated: 2026-10-21
---

# Row 321: One merge-queue owner for main (batch, one full gate and one suite per batch, deploy)

## Item

One merge-queue owner for main (batch, one full gate and one suite per batch, deploy).

## Status / trigger

**OPEN, documented (owner approved 2026-10-21; the QUEUE format is in lane-rules.md "Merge queue"; closes when batch queue-1 is merged and deployed through it)**

## Size

S

## Detail

On 2026-10-20 four sessions each merged main into their own batch, so each re-merged and re-gated whenever main moved (three extra full gates for batch A) and two deploys collided. Agreed 2026-10-21: lanes and threads push fast-gated branches and add them to a QUEUE line on the `claims` branch; the owner merges the queue in batches, runs one full gate and one suite run per batch, merges to main and deploys; nobody else merges to main or deploys. Batch A (PR after #122) is the last uncoordinated merge. Close when `dev-docs/lane-rules.md` documents the QUEUE format (the owner writes it).
