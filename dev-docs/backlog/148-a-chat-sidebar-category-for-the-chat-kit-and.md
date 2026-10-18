---
id: 148
title: "A \"Chat\" sidebar category for the chat kit and message_scroller"
status: "OPEN -- recorded 2026-10-04"
size: "S"
area: virtual
updated: 2026-10-04
---

# Row 148: A "Chat" sidebar category for the chat kit and message_scroller

## Item

A "Chat" sidebar category for the chat kit and `message_scroller`

## Status / trigger

**OPEN** -- recorded 2026-10-04

## Size

S

## Detail

`message`, `bubble`, `marker`, `attachment` and `message_scroller` map to `ComponentCategory::DataDisplay` (`preview/src/components/mod.rs`, `category_of`), so five chat parts sit in a group that also holds Table, Avatar and Badge. Add a `Chat` variant (its `ComponentCategory::ALL` entry, the sidebar arm in `preview/src/main.rs` near the `DataDisplay` arm, an icon); the sidebar loop is `for cat in ComponentCategory::ALL`, so those five are the only mapping change. Check `check-preview-composition.sh`, the docs-sidebar spec and `check-internal-hrefs.sh` for hard-coded category lists or links.
