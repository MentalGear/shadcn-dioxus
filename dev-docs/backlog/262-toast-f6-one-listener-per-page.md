---
id: 262
title: "Toast: one F6 listener per page, not one per ToastProvider"
status: "BACKLOG (found 2026-10-19 by lane toast)"
size: "S"
area: primitives
updated: 2026-10-19
---

# Row 262: Toast: one F6 listener per page, not one per ToastProvider

## Item

Every `ToastProvider` registers its own global F6 listener, so on a page with several providers F6 focuses the last-mounted region.

## Status / trigger

**BACKLOG (found 2026-10-19 by lane toast)**

## Size

S

## Detail

The toast docs page now mounts eight toast demos, each with its own provider; F6 ("jump to notifications") lands on whichever
region mounted last, not the one with toasts. Construction: one page-lifetime F6 handler (installed once, like the modality
writer) that cycles through the regions that currently hold toasts, most recent first; providers register their region with
it instead of listening themselves. Spec: two providers, toasts in the first, F6 focuses the first.
