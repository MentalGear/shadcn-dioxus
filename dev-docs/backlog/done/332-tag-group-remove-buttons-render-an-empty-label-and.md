---
id: 332
title: "Tag Group remove buttons render an empty label and disabled on the server"
status: "LANDED 2026-10-21"
size: "S"
area: components
updated: 2026-10-21
---

# Row 332: Tag Group remove buttons render an empty label and disabled on the server

## Item

Tag Group remove buttons render an empty label and disabled on the server.

## Status / trigger

**LANDED 2026-10-21**

## Size

S

## Detail

Found 2026-10-21 by the row-329 SSR-vs-first-render parity spike (Round 4 thread): the server HTML of tag_group's remove buttons carries `aria-label="Remove item "` (the item name missing) and `disabled`, and the client's first render fixes both -- the row 306 class (state the server cannot see until an effect runs). A screen reader or a no-JS visitor gets an unlabelled, disabled control until hydration. Fix by rendering the label and enabled state from props on the server; the row-329 parity test then pins it.

## Resolution (lane ssr-parity, 2026-10-21)

`TagOption` now puts its render-time `text_value` and `disabled` memos in `TagOptionCtx`; `TagRemoveButton` builds `aria-label` and `disabled` from them, not from the effect-filled item registry. Removed `TagGroupState::text_value`/`can_remove_item`. The three tag_group demos and `/component/tag_group/` left the row-329 parity baseline (cargo test ssr_parity green).
