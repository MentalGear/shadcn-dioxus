---
id: 153
title: "Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden=\"until-found\"…"
status: "PROTOTYPED 2026-10-19 (lane virtual-grid, round 4) -- works in Chromium as an opt-in text_value…"
size: "M"
area: virtual
updated: 2026-10-19
---

# Row 153: Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as hidden="until-found"…

## Item

Native find-in-page for WINDOWED virtual lists: keep per-item text in the DOM as `hidden="until-found"` placeholders, then sync the virtualizer to where the browser found the match

## Status / trigger

**PROTOTYPED 2026-10-19 (lane `virtual-grid`, round 4)** -- works in Chromium as an opt-in `text_value` prop; the Ctrl+F find-next path still needs a manual check (owner idea 2026-10-05)

## Size

M

## Detail

**Problem:** a windowed `VirtualList` (above the content-visibility threshold of row 143) keeps only ~viewport + 2 x buffer rows in the DOM, so Ctrl+F sees about 30 of 2,000 rows (survey section 1.1) and SSR/SSG renders none. **Owner idea:** keep the off-screen items' text in the DOM before and after the rendered window so the browser can find it, then move the virtualizer to where the browser scrolled. **Refined design:** per-item lightweight text placeholders carrying only a proposed per-item `text_value(i)` (a callback returning the searchable text, no markup; not an existing prop) inside an element with `hidden="until-found"`. Such an element costs no layout until revealed and is searchable in Chromium 102+, Firefox 139+ (events; 148 full) and Safari 26.2 (partial, reveals without scrolling, WebKit 304174); older engines treat it as plain `hidden`, so they fall back to not-findable, with no regression. When a match is found the browser reveals the placeholder and fires `beforematch` on that exact element (survey section 6), which names the **item index directly**. That is better than reading the scroll position after the find, which is ambiguous because unrevealed placeholders have no height, so the scroll offset cannot be mapped back to an index. The virtualizer then scrolls its window to that index, renders the real row, and re-hides or re-uses the placeholder. **Open risks to prototype:** (1) "find next" continuity: the browser's active match lives in a DOM node that is swapped when the real row replaces the placeholder, so the next Enter may restart from the top or skip; measure per engine. (2) DOM and memory cost: one text node per item (about 1 node) against about 9 for a full row, but at 100k items that is still ~100k nodes (survey probe: +445 MB at 100k with content-visibility rows, +12 MB windowed); measure, and consider chunked placeholders (one element per N items, an IntersectionObserver or a second `beforematch` step picking the item after the reveal). (3) Accessibility: `until-found` content is hidden from the accessibility tree, so screen-reader browse history still stops at the rendered window; no worse than today, but not a fix for it. (4) Unsupported browsers (and Safari 26.2's no-scroll reveal) fall back to not-findable. (5) Headless `window.find` did not fire `beforematch` in the survey's probe: the Ctrl+F path has to be verified by hand, and by the keyboard in a real browser. **Applies to:** `VirtualList` windowed mode now and, if windowing is ever approved for it (row 146 defers it), `message_scroller`. **Related:** row 144 (the same attribute on Collapsible, Accordion and Tabs, owner-approved), row 143 (the cv mode this complements), row 147 (the Safari 18-25 gap on `content-visibility: auto`, which this design does not have: `until-found` is not skipped content). Lives in the shared engine per row 154. **2026-10-05:** stays a dependent row of 154. In 154's outline this is the Windowed + find-in-page strategy (layer 2): `beforematch` names the item index, and the core's `scroll_to(index, align)` (layer 1) moves the window to it. It waits for 143 and for 154's engine, and is not in the next-session order.

## Prototype findings (2026-10-19, lane `virtual-grid`)

**Built** (opt-in, nothing changes without the prop): `VirtualList { text_value: |idx| String, .. }` in windowed mode mounts, after hydration, a transparent `pointer-events: none`, `aria-hidden` layer over the canvas holding ONE `hidden="until-found"` placeholder per row (the row's text, `height` = its estimate), grouped in chunks of 100. One capturing `beforematch` listener on the container (through `js_listener.rs` `listen`) names the row; Rust scrolls the engine's real position of that row to the middle of the viewport (`Engine::scroll_target`, the `scroll_to(index)` of row 154's layer 1), which mounts it. Demo: `virtual_list` variant `find_in_page` (10,000 rows, a 100,000-row button). Spec: `playwright/virtual-list-find.spec.ts`, 4 tests, 8/8 over two repeats on a debug lane-ssg build.

**Measured (Chromium 141 headless, debug build):**

| Question | Result |
|---|---|
| Is unmounted text findable? | Yes: `window.find("Row 7777 of 10000")` is true and scrolls the container to the placeholder, so the window follows and mounts the real row (the plain windowed list's same check is false). |
| Does a real reveal fire `beforematch` with the index? | Yes, via a `#:~:text=Row%207777%20of%2010000` text-fragment link: `beforematch` fired on placeholder 7776 even though the placeholders mount after hydration (Chromium keeps retrying the fragment search), and row 7777 ended in view. `window.find` neither reveals nor fires it (as the survey found). |
| Risk 1, find-next continuity | **Not measurable here:** `window.find` on until-found content returns true but leaves a collapsed selection at the match start, so repeated calls restart at the first match (also on a static page), and Playwright cannot open the browser's find bar. What the prototype does instead is remove the cause by construction: the first version rendered placeholders only for UNMOUNTED rows, so the layer changed on every window move and a find walk skipped rows (26 -> 56 -> 86 for "7 of 10000"); the layer now depends only on `(count, estimates, text)` and never re-renders while scrolling, so the DOM the find walks is stable. Price: a mounted row's text is in the DOM twice (its match counts twice; ~30-40 rows). **Still to do by hand:** Ctrl+F, "7 of 10000", Enter x10 in Chromium, Firefox 139+, Safari 26.2. |
| Risk 2, DOM and memory | 10,000 rows: 10,180 elements, 7.2 MB JS heap. 100,000 rows: 101,098 elements (+ one text node each), 17.2 MB JS heap; mounting the 90,000 new placeholders took ~2.9 s (debug). Scrolling: 20 frames of 400 px took 313 ms at 100k with the layer against 315 ms for the plain windowed list, **but only because each chunk is a `content-visibility: auto` box of its summed estimates**: without it every window move re-laid out all placeholders (2.6 s for the same 20 frames). Find still searches the skipped chunks. |
| Risk 3, accessibility | The placeholders are not in the accessibility tree (`ariaSnapshot` of the list does not contain an unmounted row's text); the layer is also `aria-hidden`, so a revealed placeholder adds nothing. No worse than plain windowing, not a fix for screen-reader browsing. |
| Risk 4, unsupported engines | Not measured here (Chromium only); engines without `until-found` treat it as `hidden`, so they fall back to today's behaviour. |
| Risk 5, headless | Confirmed: `window.find` does not fire `beforematch`. The text-fragment link is the automatable reveal path. |

**Found on the way (both are classes, not instances):** (a) Dioxus 0.7.9's web interpreter treats an attribute NAMED `hidden` as boolean and drops `"until-found"`; the repo's construction for it already exists (`collapsible::hidden_attribute` writes the upper-case `HIDDEN` slot, which `setAttribute` lower-cases), and the prototype uses the same slot. (b) Dioxus 0.7.9 appends new children with a 16-bit count: one parent receiving 65,536 or more new children in one edit keeps only the last `n mod 65536` (switching to 100,000 rows lost rows 10,000..75,535 when chunks were fragments). Any component that can mount 65k+ siblings at once is exposed; the layer nests rows in chunk elements. Worth an upstream issue.

**Verdict:** workable as an opt-in in Chromium (find, text-fragment links, `beforematch` -> index -> scroll all verified; scroll cost unchanged with the cv chunks). Before it is promoted from "experimental": the manual Ctrl+F find-next check per engine (risk 1), a Firefox/Safari run, and a release-build mount timing for 100k rows. Placeholder positions use the estimates, so with poor estimates the browser's own scroll lands near, not on, the row; the `beforematch` scroll then corrects it, but `window.find` (no `beforematch`) does not.
