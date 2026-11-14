---
id: 520
title: "A carousel drag release makes its destination current when the scroll settles, so the arrows and the indicators lag behind the pointer"
status: "APPROVED 2026-11-14 (owner: \"do the cheap one\", of the two ways to register the slide earlier).…"
size: "S."
area: primitives
js-grant: js_lit=22, js_file_bytes=2640 (the owner asked for the change on 2026-11-14; js_file_bytes stands for the total, which counts a js_lit unit as 120 bytes; measured: carousel/drag.rs js_lit 537 to 546, carousel/scroll.rs 530 to 543)
js-fix: primitives/src/carousel/drag.rs, primitives/src/carousel/scroll.rs
updated: 2026-11-14
---

# Row 520: a carousel drag release registers its destination at the release

## Item

The owner reports that the carousel's arrows and indicators "take a bit" to follow the slide. An arrow click, a key,
an indicator pick, the API's `scroll_to` and an autoplay step set the selected slide at the intent, before the scroll.
A drag release does not: its destination is computed at the release (`primitives/src/carousel/drag.rs`, "Release
settle") and reaches the selection only through the settle after `scrollend` (`scroll.rs`).

## Status / trigger

APPROVED 2026-11-14 (owner: "do the cheap one", of the two ways to register the slide earlier). Built on
`lane/carousel-arrows`.

## Size

S.

## Detail

- **The construction.** The release announces its destination before it scrolls; the tracking script reports it at
  once and the settle at the end of that scroll finds the same slide and reports nothing. A slide the track passes is
  never reported, because no settle runs while a release scroll is in flight.
- **Not in this row.** Following the nearest slide during a free wheel or touch scroll (the costlier way). The two
  scripts are legacy scripts of the carousel's port row ([464](./464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md) owns their move to
  Rust); this row changes them under a `js-fix`.
- **Proof.** `playwright/carousel-nav-opacity.spec.ts`: after a drag release the dragged-to indicator is current
  before the release scroll arrives; the cases for an arrow, two quick clicks, an indicator pick and RTL guard the
  paths that register at the intent.
