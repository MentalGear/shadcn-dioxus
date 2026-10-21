---
id: 352
title: "Link the 'Plus the global attributes.' sentence in props tables to MDN"
status: "Open (owner approved 2026-10-20)."
size: "S"
area: docs
updated: 2026-10-21
---

# Row 352: Link the "Plus the global attributes." sentence in props tables to MDN

## Item

Link the "Plus the global attributes." sentence in props tables to MDN.

## Status / trigger

Open (owner approved 2026-10-20).

## Size

S

## Detail

`preview/build.rs` (~line 363) turns a `#[props(extends = GlobalAttributes)]` spread into the plain text "the global attributes" in each generated Props reference. Render it as a link to `https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes` (external, `rel="noopener"`, matching how the page links other external docs; `check-internal-hrefs.sh` must stay green). The per-element `<code>&lt;tag&gt;</code>` phrase can link to its MDN element page the same way if cheap.
