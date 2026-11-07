---
id: 505
title: "check-js-volume counts JavaScript held in Rust strings by lines, so a reformat hides growth"
status: "BACKLOG. Found 2026-11-07 merging row 436's overlay lane: the IME Escape condition (e.isComposing…"
size: "S"
area: tooling
updated: 2026-11-07
---

# Row 505: `js_lit` counts lines, not bytes

## Item

`scripts/check-js-volume.sh` measures JavaScript held in Rust strings (`js_lit`) in lines, while hand-written `.js`/`.ts`
files are measured in bytes (`js_file_bytes`, row 444 A2). A line count is not a volume: joining two lines keeps the
count while the script grows, and splitting one raises it with no new code. The ratchet then rewards formatting, not
shrinking.

Construction: count `js_lit` the way `js_file_bytes` is counted, in bytes of the string's code view with runs of
whitespace collapsed to one, so a reformat changes nothing and every added token counts. Re-baseline once in the commit
that changes the unit (the totals line and `RATCHET_REQUIRE_BASE` already handle a unit change, as A2 did), and keep
the self-test scenarios: a joined line must no longer hide an added condition.

## Status / trigger

BACKLOG. Found 2026-11-07 merging row 436's overlay lane: the IME Escape condition (`e.isComposing ||
e.keyCode === 229`) grew `DISMISS_JS` in `primitives/src/lib.rs` by about 40 bytes, and the lane joined the `focusin`
block's closing `}, true);` onto the line before it, so `js_lit` for `lib.rs` stayed at 182 and the gate passed. The
condition itself is needed (an Escape that ends an IME composition must not dismiss the layer); the growth is real and
small, and it is the owner's call whether it stands under a grant once the unit is bytes. Land before row 445 (the
legacy freeze), which relies on the same counts.

## Size

S

## Detail

- Where the unit lives: `scripts/check-js-volume.sh` (the `js_lit` column and the totals line `js_lit*120 +
  js_file_bytes`, whose 120 is the bytes-per-line exchange rate this row removes) and `scripts/lib/ratchet.py`.
- Proof when done: a planted joined line in a fixture keeps the byte count of the added condition (red), a pure
  reformat of an existing string changes nothing (green), and the real tree re-baselines with the per-file numbers in
  the commit message.
