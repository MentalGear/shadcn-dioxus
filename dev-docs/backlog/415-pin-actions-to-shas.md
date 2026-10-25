---
id: 415
title: "Pin GitHub Actions to commit SHAs"
status: "OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of…"
size: "XS"
area: build
updated: 2026-10-24
---

# Row 415: Pin GitHub Actions to commit SHAs

## Item

Replace tag references (`actions/checkout@v4`, `Swatinem/rust-cache@v2`, `actions/upload-artifact@v4` in `mutants.yml`) with full commit SHAs and a version comment, and add a gate that rejects tag-only `uses:` lines.

## Status / trigger

OPTIONAL: needs owner approval before it is adopted or scheduled (owner 2026-10-24). Not part of the adopted order. A lane must not pick it up until the owner says go. Trigger: CI reactivation (rows 4 and 33). While CI is frozen (C15, `workflow_dispatch` only), the exposure is small. Add it to the C-table in `dev-docs/supply-chain/README.md` when it lands.

## Size

XS

## Detail

See the item; no further detail in the draft.

Source: `/mnt/project-files/backlog-drafts/2026-10-24-optional-safety-rows.md` (draft D), background in `/mnt/project-files/goose-eval/2026-10-24-goose-vs-wasm.md`.
