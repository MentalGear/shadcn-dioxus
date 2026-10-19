---
id: 153
title: "Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden=\"until-found\"…"
status: "OPEN (research) -- owner idea 2026-10-05; nothing prototyped"
size: "M"
area: virtual
updated: 2026-10-05
---

# Row 153: Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden="until-found"…

## Item

Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as `hidden="until-found"` placeholders, then sync the virtualizer to where the browser found the match

## Status / trigger

**OPEN (research)** -- owner idea 2026-10-05; nothing prototyped

## Size

M

## Detail

**Problem:** a windowed `VirtualList` (above the content-visibility threshold of row 143) keeps only ~viewport + 2 x buffer rows in the DOM, so Ctrl+F sees about 30 of 2,000 rows (survey section 1.1) and SSR/SSG renders none. **Owner idea:** keep the off-screen items' text in the DOM before and after the rendered window so the browser can find it, then move the virtualizer to where the browser scrolled. **Refined design:** per-item lightweight text placeholders carrying only a proposed per-item `text_value(i)` (a callback returning the searchable text, no markup; not an existing prop) inside an element with `hidden="until-found"`. Such an element costs no layout until revealed and is searchable in Chromium 102+, Firefox 139+ (events; 148 full) and Safari 26.2 (partial, reveals without scrolling, WebKit 304174); older engines treat it as plain `hidden`, so they fall back to not-findable, with no regression. When a match is found the browser reveals the placeholder and fires `beforematch` on that exact element (survey section 6), which names the **item index directly**. That is better than reading the scroll position after the find, which is ambiguous because unrevealed placeholders have no height, so the scroll offset cannot be mapped back to an index. The virtualizer then scrolls its window to that index, renders the real row, and re-hides or re-uses the placeholder. **Open risks to prototype:** (1) "find next" continuity: the browser's active match lives in a DOM node that is swapped when the real row replaces the placeholder, so the next Enter may restart from the top or skip; measure per engine. (2) DOM and memory cost: one text node per item (about 1 node) against about 9 for a full row, but at 100k items that is still ~100k nodes (survey probe: +445 MB at 100k with content-visibility rows, +12 MB windowed); measure, and consider chunked placeholders (one element per N items, an IntersectionObserver or a second `beforematch` step picking the item after the reveal). (3) Accessibility: `until-found` content is hidden from the accessibility tree, so screen-reader browse history still stops at the rendered window; no worse than today, but not a fix for it. (4) Unsupported browsers (and Safari 26.2's no-scroll reveal) fall back to not-findable. (5) Headless `window.find` did not fire `beforematch` in the survey's probe: the Ctrl+F path has to be verified by hand, and by the keyboard in a real browser. **Applies to:** `VirtualList` windowed mode now and, if windowing is ever approved for it (row 146 defers it), `message_scroller`. **Related:** row 144 (the same attribute on Collapsible, Accordion and Tabs, owner-approved), row 143 (the cv mode this complements), row 147 (the Safari 18-25 gap on `content-visibility: auto`, which this design does not have: `until-found` is not skipped content). Lives in the shared engine per row 154. **2026-10-05:** stays a dependent row of 154. In 154's outline this is the Windowed + find-in-page strategy (layer 2): `beforematch` names the item index, and the core's `scroll_to(index, align)` (layer 1) moves the window to it. It waits for 143 and for 154's engine, and is not in the next-session order.
