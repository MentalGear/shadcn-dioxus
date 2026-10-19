---
id: 300
title: "Home hero title \"shadcn-dioxus\" in bold"
status: "OPEN. This is a different element from PR #105: that PR made the navbar brand bold (700…"
size: "S"
area: preview
updated: 2026-10-19
---

# Row 300: Home hero title "shadcn-dioxus" in bold

## Item

The big home-page title should be bold.

## Status / trigger

OPEN. **This is a different element from PR #105:** that PR made the navbar brand bold (700, `main.css:386`). The hero `h1.dx-hero-heading` is still `font-weight: 300` (`hero.css:82`), and `.dx-hero-title` only sets a colour.

## Size

S

## Detail

- Set `font-weight: 700` on `.dx-hero-title` to match the navbar brand. The subtitle stays 300.
