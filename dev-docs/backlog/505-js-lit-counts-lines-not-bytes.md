---
id: 505
title: "The JS volume ratchets measure text, so a reformat or a shorter name passes for less JavaScript"
status: "BACKLOG. Found 2026-11-07 merging row 436's overlay lane: the IME Escape condition (e.isComposing…"
size: "S"
area: tooling
updated: 2026-11-07
---

# Row 505: the JS volume ratchets measure text, not code

## Item

`scripts/check-js-volume.sh` measures JavaScript held in Rust strings (`js_lit`) in lines and hand-written `.js`/`.ts`
files (`js_file_bytes`, row 444 A2) and the bridge budgets (`check-bridge-behaviours.budgets.tsv`) in bytes. Both units
measure text, not code: joining two lines keeps a line count while the script grows, and shortening a local name lowers
a byte count with no less behaviour. The ratchets then reward formatting and terse names, and they push readable code
toward minified code.

Construction: count tokens. One JavaScript tokenizer (the pinned TypeScript scanner of `check-ts-scan`, `scripts/ts-tools/`,
already in the repo) counts every column in tokens: an identifier is one token whatever its length, whitespace and
comments are none, an added condition adds its tokens. `js_lit` (the code view of each Rust string), `js_file` and the
bridge budgets switch to that one unit in one commit that re-baselines them, and the totals line drops its bytes-per-line
exchange rate. Once it lands, the readable names shortened to fit a byte budget come back at no cost.

## Status / trigger

BACKLOG. Found 2026-11-07 merging row 436's overlay lane: the IME Escape condition (`e.isComposing ||
e.keyCode === 229`) grew `DISMISS_JS` in `primitives/src/lib.rs` by about 40 bytes, and the lane joined the `focusin`
block's closing `}, true);` onto the line before it, so `js_lit` for `lib.rs` stayed at 182 and the gate passed. The
condition itself is needed (an Escape that ends an IME composition must not dismiss the layer); the growth is real and
small, and it is the owner's call whether it stands under a grant once the unit is tokens. The same day row 436's class
lane brought `bridge.js` under its byte budget by renaming locals in `primitives/src/ts/bridge.ts` (`pageObserver` to
`pageMo`, `reg` to `g`): a second instance of the class, so the row covers every volume column, not only `js_lit`. Land before row 445 (the
legacy freeze), which relies on the same counts.

## Size

S

## Detail

- Where the units live: `scripts/check-js-volume.sh` (the `js_lit` and `js_file_bytes` columns and the totals line
  `js_lit*120 + js_file_bytes`, whose 120 is a bytes-per-line exchange rate), `scripts/check-bridge-behaviours.sh` and its
  budgets TSV, and `scripts/lib/ratchet.py`.
- Proof when done: in the self-test, a planted joined line keeps the token count of the added condition (red), a pure
  reformat or a rename of an existing script changes nothing (green); the real tree re-baselines with the per-file
  numbers in the commit message, and `pageMo`/`g` in `bridge.ts` are restored to readable names with the count unchanged.
