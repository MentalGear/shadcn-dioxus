---
id: 335
title: "Message Scroller demos: animation demo shows LLM output, both fade demos look default, previous-context demo is unclear"
status: "BACKLOG (owner 2026-10-21)"
size: "S"
area: components
updated: 2026-10-21
---

# Row 335: Message Scroller demos: animation demo shows LLM output, both fade demos look default, previous-context demo is unclear

## Item

Message Scroller demos: animation demo shows LLM output, both fade demos look default, previous-context demo is unclear.

## Status / trigger

**BACKLOG (owner 2026-10-21)**

## Size

S

## Detail

Owner review 2026-10-21 of https://mentalgear.github.io/shadcn-dioxus/component/message_scroller/: (1) the 'animation' demo is about the chat bubble's entrance, yet it shows LLM-style streamed output below the bubbles -- show only bubbles, or move the demo to the Message component if the animation belongs to Message; (2) the 'fade' demo's two variants both look like the default -- make the variants visibly differ (e.g. edge fade on vs off, top vs bottom) or merge them; (3) the 'previous context' demo is unclear and may lack actual context -- give it real earlier history to load (and a caption saying what it demonstrates). Check shadcn's message-scroller / ai-elements conversation demos for the intended shape.
