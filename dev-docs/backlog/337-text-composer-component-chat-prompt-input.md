---
id: 337
title: "Text composer component (chat prompt input)"
status: "BACKLOG (owner 2026-10-21)"
size: "M"
area: components
updated: 2026-10-21
---

# Row 337: Text composer component (chat prompt input)

## Item

Text composer component (chat prompt input).

## Status / trigger

**BACKLOG (owner 2026-10-21)**

## Size

M

## Detail

Owner question 2026-10-21. A chat/prompt composer like ai-elements' PromptInput (row 265 lists `prompt-input` as not yet ported): an auto-growing textarea, Enter to send and Shift+Enter for a newline with an IME-composition guard, a submit button that becomes Stop while a response streams, a toolbar slot (model select, tools), attachment chips with paste and drag-and-drop, disabled and busy states, form participation, a11y labels, RTL and the mobile keyboard inset. The parts exist already (textarea, input_group, attachment, kbd, button, tooltip, select). Estimate M (about one lane-day): the autosize and key handling in a small primitive, the styled component with 3-4 demos (basic, with attachments, with toolbar/model select, inside message_scroller's chat), specs and docs; attachments via paste/drop push it toward M-L. Not started -- owner decision whether to build.
