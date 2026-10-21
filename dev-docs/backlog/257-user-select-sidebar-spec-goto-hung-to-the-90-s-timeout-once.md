---
id: 257
title: "user-select.spec.ts /component/sidebar/ page.goto hung to the 90 s timeout once in the release suite"
status: "OPEN (not reproducible: 1 failure in the batch-A release suite; 17 of 17 green since on release and…"
size: "S"
area: playwright
updated: 2026-10-20
---

# Row 257: one unexplained `goto` hang on /component/sidebar/

## Item

Chunk 7 of the batch-A release suite (3fd7844b, served at :4830): `user-select.spec.ts:55` "every control on /component/sidebar/ computes user-select: none" timed out at 90 s inside `page.goto(..., waitUntil: "networkidle")` (`hydration.ts:47`). The 35 neighbouring `user-select` tests (same worker pool, same second) took ~1 s each.

## Status / trigger

OPEN (not reproducible: 1 failure in the batch-A release suite; 17 of 17 green since on release and debug builds of the batch and of main)

## Size

S

## Detail

**Evidence it is not the batch.**

15 repeats on the release build (2 workers) pass in 1.7-2.1 s; 2 repeats on debug c15d9751 and 2 on debug main 4a8847b0 pass in ~4 s. The spec aborts every non-local request (`open()`), so the avatar demos' deliberate `httpbin.org` hang is not it. A `networkidle` that never arrives means a request stayed open to the static server or a font; `scripts/serve-spa.py` is a `ThreadingHTTPServer` and its log shows `ConnectionResetError`s from clients that left.

**Next step (only if it recurs).**

Run the failing chunk with `DEBUG=pw:api` and keep the request log (`page.on("requestfailed"/"request")`) so the open request is named. Until then a single 90 s timeout in a 575-test chunk is load noise and is not worth a change.
