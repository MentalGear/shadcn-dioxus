---
id: 233
title: "Docs pages: short, coherent; long sections in disclosures"
status: "LANDED 2026-10-18 (lane docs-page 73c34734), merged on the session branch; deploy pending -- was…"
size: "M"
area: docs
updated: 2026-10-18
---

# Row 233: Docs pages: short, coherent; long sections in disclosures

## Item

Docs pages: short, coherent; long sections in disclosures

## Status / trigger

**LANDED 2026-10-18 (lane docs-page 73c34734), merged on the session branch; deploy pending** -- was: **DONE 2026-10-18 (lane docs-page)**: `build.rs` `wrap_sections_in_disclosures` closes every docs.md `##` (and the intro as "Usage notes") in native `<details>` by construction; text-only Examples became `variants/<v>/notes.md`, shown as a closed "About this example" under each demo (generated `variant_notes()` lookup). Visible words over 77 pages 52409 -> 14789 (median 438 -> 88) -- was: **BACKLOG (owner 2026-10-17)**

## Size

M

## Detail

Owner: Installation, Usage notes, Component Structure, Accessibility and text-only Examples collapsed in an accordion/disclosure; long passages under a header in a disclosure; consider moving the Examples text into the on-page demos in a `<details>`. Audit §4: one template (`ComponentHighlight`, `main.rs:1849-1933`), docs.md -> `docs.html` in `build.rs`, median page 331 words; "Component Structure" on 65 pages, "Accessibility" on 12 (up to 756 words); text-only "Examples" in attachment, bubble, carousel, marker, message, message_scroller. Fix A: `build.rs` wraps each `##` section of docs.md in `<details>` (no per-file edits) and the template wraps Installation; fix C: optional per-variant notes rendered in a `<details>` under each demo, replacing the text-only Examples.
