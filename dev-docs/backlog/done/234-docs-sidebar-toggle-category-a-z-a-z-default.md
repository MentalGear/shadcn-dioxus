---
id: 234
title: "Docs sidebar: toggle category / A-Z, A-Z default"
status: "LANDED 2026-10-18 (lane docs-site 37db3980), merged on the session branch; deploy pending -- was…"
size: "M"
area: docs
updated: 2026-10-18
---

# Row 234: Docs sidebar: toggle category / A-Z, A-Z default

## Item

Docs sidebar: toggle category / A-Z, A-Z default

## Status / trigger

**LANDED 2026-10-18 (lane docs-site 37db3980), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 in `dev-docs/research/next-round-plan-2026-10-15.md` (lane docs-site)** -- was: **BACKLOG (owner 2026-10-17)**

## Size

M

## Detail

Audit §5: groups `main.rs:1577`, items `main.rs:1606` (`demos_in_category`), category order from the enum `components/mod.rs:45-102`; items already A-Z within a group. Needs a segmented control + one flat A-Z group as the default; SSG emits the default, so a remembered choice must apply before paint (prepaint attribute + CSS) to avoid a flash.
