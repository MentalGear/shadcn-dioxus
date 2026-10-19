---
id: 261
title: "A --dx-font-mono theme token"
status: "LANDED 2026-10-19 (lane fonts)"
size: "S"
area: theme
updated: 2026-10-19
---

# Row 261: A --dx-font-mono theme token

## Item

The theme defines no monospace font token; `markdown` code reads `var(--dx-font-mono, inherit)` and its demos set the token inline.

## Status / trigger

**LANDED 2026-10-19 (lane fonts)**

## Size

S

## Detail

`check-component-fonts.sh` only allows `inherit` or a `var(--dx-font-…)` token in a component sheet, so a code face needs a
theme token. Add `--dx-font-mono` (Geist Mono, which the docs site already loads; take it with row 162's self-hosted fonts)
to `dx-components-theme.css`, drop the inline demo overrides in `preview/src/components/markdown/variants/`, and audit
`kbd`, `input_otp` and code samples on the docs pages for the same need.

**Update 2026-10-19 (lane fonts): LANDED with row 162.** `--dx-font-mono` (and `--dx-font-sans`) are theme tokens in `dx-components-theme.css` (zero-specificity `:where(:root)`, system monospace default); the docs site sets Geist Mono. The four inline `--dx-font-mono` overrides in `markdown/variants/*` are removed. Audit: `kbd` stays `inherit` (shadcn uses `font-sans`), `input_otp` inherits, the only mono consumers are markdown code (reads the token) and the docs-site code samples (main.css, now the token). markdown `style.css` keeps `var(--dx-font-mono, inherit)` (harmless fallback; not in this lane's set).
