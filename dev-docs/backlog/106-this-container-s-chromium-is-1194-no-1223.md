---
id: 106
title: "This container's Chromium is 1194 (no 1223, unlike some earlier sandboxes this repo's docs mention) — 15…"
status: "Found 2026-09-25 (round 8) running the full gate suite"
size: "Environment/engine class, not a product…"
area: tests
updated: 2026-09-25
---

# Row 106: This container's Chromium is 1194 (no 1223, unlike some earlier sandboxes this repo's docs mention) — 15…

## Item

This container's Chromium is `1194` (no `1223`, unlike some earlier sandboxes this repo's docs mention) — 15 `oracle/tier2-html/top-layer.spec.ts` rows failed against it (2026-09-25; Rule 8 ColorPicker is no longer one of them, see the 2026-10-05 note)

## Status / trigger

Found 2026-09-25 (round 8) running the full gate suite

## Size

Environment/engine class, not a product defect — no fix scheduled here; relates to row 21

## Detail

Rules 1 (DropdownMenu/Menubar), 4/5/6 (including the CALIBRATION rows, run against a bare native `<div popover>` with no library code involved at all), 8 (ColorPicker), and 11 (Popover with no anchor) — 15 rows total. All 15 reproduce byte-for-byte on unmodified `f103bbd`, so none are a regression from this round. The CALIBRATION rows failing against plain native markup is the decisive evidence: those rows exist specifically to establish what the browser itself does before comparing the library against it, so a CALIBRATION failure means the engine/environment doesn't behave the way the rule assumes, not that the library is wrong — this is an engine/environment class, per row 21's own already-established finding, not a new one **2026-10-05 (round 3, final verification) note:** Rule 8's ColorPicker case is NOT an engine/environment failure: it measured the wrong trigger (the header theme picker's `anchor-name` came first) in a viewport too short for the 386 px panel, so `flip-block` correctly opened it above and a scroll flipped it back. It is fixed in the spec and PASSES (row 172 has the account; row 21's engine diagnosis is withdrawn for it). The other rows of this list were not re-counted one by one, but the final full release run (2759 tests, 16 failed, all 16 resolved) shows no top-layer case red other than the by-design Rule 4c `selftest:unfixed:id`, so read the "15 rows fail on Chromium 1194" figure as historical until a deliberate per-rule re-count says otherwise.
