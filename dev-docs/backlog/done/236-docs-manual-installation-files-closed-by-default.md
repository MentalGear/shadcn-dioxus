---
id: 236
title: "Docs: \"Manual installation files\" closed by default"
status: "LANDED 2026-10-18 (lane docs-page 73c34734), merged on the session branch; deploy pending -- was…"
size: "-"
area: docs
updated: 2026-10-18
---

# Row 236: Docs: "Manual installation files" closed by default

## Item

Docs: "Manual installation files" closed by default

## Status / trigger

**LANDED 2026-10-18 (lane docs-page 73c34734), merged on the session branch; deploy pending** -- was: **DONE 2026-10-18 (lane docs-page)**: asserted on every component page by `docs-structure.spec.ts` (closed on load and after Installation is opened) and in `preview.spec.ts` -- was: **CONFIRMED already true (2026-10-17)**

## Size

-

## Detail

Owner: never open by default. Audit §7: a plain `<details>` with no `open` (`main.rs:1905`) on every page; `preview.spec.ts:54` opens it by clicking. Keep; fold into row 232/233's restructure (stays collapsed).
