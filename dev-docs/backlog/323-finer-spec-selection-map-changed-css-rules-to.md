---
id: 323
title: "Finer spec selection: map changed CSS rules to the classes and routes that use them"
status: "BACKLOG (proposed 2026-10-21, instead of splitting main.css)"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 323: Finer spec selection: map changed CSS rules to the classes and routes that use them

## Item

Finer spec selection: map changed CSS rules to the classes and routes that use them.

## Status / trigger

**BACKLOG (proposed 2026-10-21, instead of splitting main.css)**

## Size

M

## Detail

Today any change to `main.css`, `main.rs` or a theme sheet makes `playwright/tools/select.mjs` select ~1,800 tests. Splitting `main.css` per page was rejected: it breaks the one render-blocking bundle (`check-css-delivery.sh`), changes cascade order for equal-specificity rules, and the docs-shell rules are shared anyway. Instead `select.mjs` parses the changed CSS rules' selectors into classes and maps classes to the components and routes that render them (from the SSG HTML), selecting only those specs.
