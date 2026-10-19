---
id: 261
title: "A --dx-font-mono theme token"
status: "BACKLOG (found 2026-10-19 by lane markdown)"
size: "S"
area: theme
updated: 2026-10-19
---

# Row 261: A --dx-font-mono theme token

## Item

The theme defines no monospace font token; `markdown` code reads `var(--dx-font-mono, inherit)` and its demos set the token inline.

## Status / trigger

**BACKLOG (found 2026-10-19 by lane markdown)**

## Size

S

## Detail

`check-component-fonts.sh` only allows `inherit` or a `var(--dx-font-…)` token in a component sheet, so a code face needs a
theme token. Add `--dx-font-mono` (Geist Mono, which the docs site already loads; take it with row 162's self-hosted fonts)
to `dx-components-theme.css`, drop the inline demo overrides in `preview/src/components/markdown/variants/`, and audit
`kbd`, `input_otp` and code samples on the docs pages for the same need.
