---
id: 241
title: "Specs still read block-demo rows from a mirror in components/mod.rs"
status: "BACKLOG (lane registry 2026-10-18)"
size: "S"
area: tests
updated: 2026-10-18
---

# Row 241: Specs still read block-demo rows from a mirror in components/mod.rs

## Item

Specs still read block-demo rows from a mirror in `components/mod.rs`

## Status / trigger

**BACKLOG (lane registry 2026-10-18)**

## Size

S

## Detail

`all-routes`, `demo-forms-no-navigation` and `popup-style-isolation` scrape the block rows from `mod.rs` text; row 219 kept a cfg'd-out mirror there and `build.rs`'s `check_block_mirror` fails the build on drift. Let those specs read `variants/` (or a generated JSON the build writes) and delete the mirror.
