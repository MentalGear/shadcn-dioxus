---
id: 159
title: "JS listeners added through eval (or shipped scripts) and never removed: a class, widening row 149"
status: "LANDED 2026-10-05 (round 3) -- fixed as the widened row 149: one shared scope-owned listener helper…"
size: "M"
area: process
updated: 2026-10-05
---

# Row 159: JS listeners added through eval (or shipped scripts) and never removed: a class, widening row 149

## Item

JS listeners added through `eval` (or shipped scripts) and never removed: a class, widening row 149

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- fixed as the widened row 149: one shared scope-owned listener helper plus a gate

## Size

M

## Detail

Found by the VirtualList lane (2026-10-05) in `form`, `sidebar`, `theme.rs`, `focus-trap.js`, `pointer.rs`, `scroll_lock.rs` and `index.html`: a listener is attached from Rust through `document::eval` or from a bundled or inline script, and nothing ties its removal to the scope or element that owns it, so it lives for the page; for non-passive `wheel`/`touchmove` ones it also taxes scrolling (row 149 is the first instance, found by the scroll-jank round; rows 150 and 151 are siblings of the same "set up outside the scope that owns it" shape). **Construction (not instance patches):** one shared scope-owned listener helper (add on mount or lock, remove on drop, passive by default, one place for the teardown) that every site above goes through, plus a gate (a `check-*.sh` added to `scripts/run-gates.sh` and to CLAUDE.md's gate sentence) that fails on a raw `addEventListener` in an `eval` string or shipped script that neither uses the helper nor carries a reasoned allowlist marker, in the shape of `check-uncleared-intervals.sh` and `check-blocking-scroll-listeners.sh`. **Subsumes:** the seven named sites and row 149. **Does not cover:** timers (row 150's gate) and the carousel's own bridges unless the gate finds them. Row 154's `use_virtualizer` bridge is built on the same helper. The final docs pass of the round fills in the account. **Landed 2026-10-05 (round 3), the account this row promised:** the construction as proposed. **Helper:** `primitives/src/js_listener.rs` (`JsListeners` and `use_js_listeners`): a script registers through `listen(target, type, handler, options)`, removal is DERIVED (undone when the value drops: unmount, effect re-run or lock release), and `wheel`/`touch*` listeners are passive by default. **Inventory (15 true leaks fixed):** `scroll_lock.rs` 4 (installed at the first lock and kept for the page), `pointer.rs` 4 (on `window`, at first use), the `form` fixture 6 (adds with no removes: two on `document`, one more pair per visit), and the focus trap's stacking registrations; `sidebar`, `theme.rs` and `index.html` were audited by the same scan. The Popover modal arm now uses the shared dialog driver. **Gate:** `scripts/check-js-listeners.sh` (in `scripts/run-gates.sh`): a raw `addEventListener(` in `primitives/src` or `preview/src` (`*.rs`, `*.js`, `*.ts`) or `preview/index.html` fails unless it is in `js_listener.rs`, carries a `listener-ok: <reason>` marker, or is a correct hand-paired registration in a file of the `PAIRED_BY_HAND` ratchet (52 sites in 11 files, only ever shrinking: row 167). **Oracle:** `listener-inventory.spec.ts` (row 149's note). **Not covered, said plainly:** other APIs (`web_sys`, `on*` properties, vendored scripts), the three Observers, the generated `focus-trap.js` (its `.ts` source is scanned), and timers (row 150's gate). The inventory table and the oracle's four assertions: [round-3 note](../../research/round3-2026-10-05.md), section 4.
