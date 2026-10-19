---
id: 148
title: "A \"Chat\" sidebar category for the chat kit and message_scroller"
status: "LANDED 2026-10-19 -- recorded 2026-10-04"
size: "S"
area: virtual
updated: 2026-10-19
---

# Row 148: A "Chat" sidebar category for the chat kit and message_scroller

## Item

A "Chat" sidebar category for the chat kit and `message_scroller`

## Status / trigger

**LANDED 2026-10-19** -- recorded 2026-10-04

## Size

S

## Detail

`message`, `bubble`, `marker`, `attachment` and `message_scroller` map to `ComponentCategory::DataDisplay` (`preview/src/components/mod.rs`, `category_of`), so five chat parts sit in a group that also holds Table, Avatar and Badge. Add a `Chat` variant (its `ComponentCategory::ALL` entry, the sidebar arm in `preview/src/main.rs` near the `DataDisplay` arm, an icon); the sidebar loop is `for cat in ComponentCategory::ALL`, so those five are the only mapping change. Check `check-preview-composition.sh`, the docs-sidebar spec and `check-internal-hrefs.sh` for hard-coded category lists or links.

**Update 2026-10-19:** landed with the `markdown` component (row 213). `ComponentCategory::Chat` (label "Chat", `ALL` entry between Data display and Charts) in `preview/src/components/mod.rs`; `category_of` maps `message`, `bubble`, `marker`, `attachment`, `message_scroller` and `markdown` to it (they left Data display); the sidebar icon arm (`MessageSquare`) is in `preview/src/docs_nav.rs`, not `main.rs`, which is where the sidebar loop `for cat in ComponentCategory::ALL` lives. Checked for hard-coded category lists: `check-preview-composition.sh` and `check-internal-hrefs.sh` have none; `playwright/docs-nav-sort.spec.ts` asserted seven groups and their labels, now eight with Chat. New `playwright/chat-category.spec.ts`: Chat sits between Data display and Charts and holds exactly the six chat components, Data display lost them and keeps Avatar/Table/Card/Data table, a Chat-group link navigates.
