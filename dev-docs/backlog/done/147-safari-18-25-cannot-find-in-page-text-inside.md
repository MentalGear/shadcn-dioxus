---
id: 147
title: "Safari 18-25 cannot find-in-page text inside content-visibility: auto-skipped content (WebKit bug 283846)"
status: "CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code -- a note, no behaviour change"
size: "S"
area: virtual
updated: 2026-10-19
---

# Row 147: Safari 18-25 cannot find-in-page text inside content-visibility: auto-skipped content (WebKit bug 283846)

## Item

Safari 18-25 cannot find-in-page text inside `content-visibility: auto`-skipped content (WebKit bug 283846)

## Status / trigger

**CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code -- a note, no behaviour change**

## Size

S

## Detail

Applies wherever this round used `content-visibility: auto`: the home-page cards (`.dx-component-card`, `.dx-widget-card`), the component-page variants (`.dx-component-variant`) and `message_scroller` rows. Chromium (probed: `window.find` revealed a skipped row; `focus()` on a skipped button worked), Firefox 125+ and Safari 26+ keep find-in-page, focus and `#fragment` working on skipped content; Safari 18-25 does not, so the "find-in-page keeps working" wording in `message_scroller`'s docs and the card comment in `main.css` is true except there. Safari <= 26 also lacks scroll anchoring (`overflow-anchor` ships in 27), so a wrong block-size estimate above the viewport shows as a visible shift. **Related cost of the same mechanism, measured in the scroll-jank lane and not fixed:** a first-visit programmatic jump (`scrollIntoView`, `#fragment`) to a never-rendered card lands -600..+1,200 px off target (`auto` remembers sizes only for rendered cards; the home page has no such jump, wheel and keyboard scrolling measured CLS 0). **Action:** add the Safari caveat to the `message_scroller` docs page and to the `main.css` comment.
