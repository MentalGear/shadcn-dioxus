---
id: 505
title: "The JS volume ratchets measure text, so a reformat or a shorter name passes for less JavaScript"
status: "PARTLY ADDRESSED, the construction itself still BACKLOG. Of the two instances found 2026-11-07, one…"
size: "S"
area: tooling
updated: 2026-11-11
js-grant: "bytes=56 js_file_bytes=56 (owner 2026-11-11: revert the two name-shortening instances in primitives/src/ts/bridge.ts this row found -- pageObserver/reg/batch restored from pageMo/g/b -- so readable names are not lost from code the Rust port will read; measured raise bridge.ts 42676->42732, bridge.js 19900->19956)"
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

PARTLY ADDRESSED, the construction itself still BACKLOG. Of the two instances found 2026-11-07, one is reverted
under a grant (not fixed by construction, which still needs building); the other still stands, waiting on the same
construction.

- **Instance 1, reverted (owner 2026-11-11, PR #169's grant, row 436's class lane):** the lane had renamed
  `primitives/src/ts/bridge.ts` locals (`pageObserver` to `pageMo`, `reg` to `g`, a `sweep` parameter to `b`) only to
  bring `bridge.js` under its byte budget. The owner's instruction was to revert it -- readable names should not be
  lost from code the Rust port will read -- so `bridge.ts` reads `pageObserver`/`reg`/`batch` again, raised against
  a grant row (`js-grant: bytes=56 js_file_bytes=56`, this row's own front matter) rather than against the
  token-count construction below, since that construction is not built yet.
- **Instance 2, still standing:** the IME Escape condition (`e.isComposing || e.keyCode === 229`) grew `DISMISS_JS`
  in `primitives/src/lib.rs` by about 40 bytes, and row 436's overlay lane joined the `focusin` block's closing
  `}, true);` onto the line before it, so `js_lit` for `lib.rs` stayed at 182 and the gate passed. The condition
  itself is needed (an Escape that ends an IME composition must not dismiss the layer); the line join is still in
  `lib.rs` today, not reverted -- unlike instance 1, joining two lines loses no name, so there was nothing the
  owner's 2026-11-11 instruction asked to revert here. It stays as the row's second example of the class and the
  construction's second thing to fix.

Land before row 445 (the legacy freeze), which relies on the same counts.

## Size

S

## Detail

- Where the units live: `scripts/check-js-volume.sh` (the `js_lit` and `js_file_bytes` columns and the totals line
  `js_lit*120 + js_file_bytes`, whose 120 is a bytes-per-line exchange rate), `scripts/check-bridge-behaviours.sh` and its
  budgets TSV, and `scripts/lib/ratchet.py`.
- Proof when done: in the self-test, a planted joined line keeps the token count of the added condition (red), a pure
  reformat or a rename of an existing script changes nothing (green); the real tree re-baselines with the per-file
  numbers in the commit message. `lib.rs`'s joined `focusin` line (instance 2) can then be unjoined at no cost; it
  stays joined only until this lands. `bridge.ts`'s names (instance 1) are already readable, under the grant above,
  so there is nothing left to restore there -- this row's remaining work is the construction itself.
