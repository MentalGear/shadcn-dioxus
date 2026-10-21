---
id: 336
title: "Docs: the browser tab title does not change when navigating to another page"
status: "BACKLOG (owner 2026-10-21)"
size: "S"
area: docs
updated: 2026-10-21
---

# Row 336: Docs: the browser tab title does not change when navigating to another page

## Item

Docs: the browser tab title does not change when navigating to another page.

## Status / trigger

**BACKLOG (owner 2026-10-21)**

## Size

S

## Detail

Owner report 2026-10-21: navigating client-side from one docs page to another (e.g. one component page to a different component page) leaves the document title of the first page. Each route should set its own title (`document::Title`, as the not-found page does at `preview/src/main.rs:1805`) and the router swap must replace it; check whether the SSG's static `<title>` is overwritten after hydration, and add a spec that navigates between two component pages and reads `document.title`.
