---
id: 232
title: "Docs: one place for installation; the CLI everywhere"
status: "LANDED 2026-10-18 (lane docs-page 73c34734), merged on the session branch; deploy pending -- was…"
size: "S-M"
area: build
updated: 2026-10-18
---

# Row 232: Docs: one place for installation; the CLI everywhere

## Item

Docs: one place for installation; the CLI everywhere

## Status / trigger

**LANDED 2026-10-18 (lane docs-page 73c34734), merged on the session branch; deploy pending** -- was: **DONE 2026-10-18 (lane docs-page)**: Installation is one closed `<details>` per component page holding the CLI command and the closed manual files; the header keeps an "Install" link to `#installation`; the dx alternative is documented once, on /docs; hero, starter, README, three docs.md and ~20 `component.rs` comments say `shadcn-dioxus`. Guarded by `playwright/docs-structure.spec.ts` (one Installation, one command, none outside it) -- was: **BACKLOG (owner 2026-10-17)**

## Size

S-M

## Detail

Audit §3: CLI primary in README, Docs page, per-page header, starter README; still dx-only in the home hero (`main.rs:2641`), `templates/starter/src/main.rs:5,7,21`, `Dioxus.toml:8`, `README.md:124`, `message_scroller/docs.md:141`, `scroll_area/docs.md:58`, `top_layer/docs.md:4`, ~20 `component.rs` comments shown in the Code tab. A component page explains installation up to five times (header commands, a one-sentence "Installation" section, "Manual installation files", the Code tab, docs.md "Component Structure"). Fix: the commands live only in the Installation section; the rest link to it.
