---
id: 97
title: "A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one"
status: "CLOSED 2026-10-23: the multi-root claim was disproved; the real class (dynamic children inside…"
size: "DONE by construction…"
area: misc
updated: 2026-10-23
---

# Row 97: A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one

## Item

A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one

## Status / trigger

**CLOSED 2026-10-23: the multi-root claim was disproved; the real class (dynamic children inside raw-text elements) is now a zero-tolerance gate.** Found 2026-09-20 (round 5) adding a scoped `<style>` tag beside `CarouselContent`'s track **2026-10-20 sync: checked, no guard script for un-fragmented rsx siblings exists; a second instance would make it a class. No work in flight.**

## Size

DONE by construction (`raw-text-dynamic-child` in `scripts/check-attr-spread-collision.sh`, 0 sites at introduction)

## Detail

The `<style>` tag rendered correctly in SSR markup and then vanished on the second render, which in a real mount happens almost immediately via this component's own effects — so the served HTML looked right while `getComputedStyle` reported `scroll-snap-stop: normal` in the live browser. Reading the SSR output alone would never have caught it; only a live computed-style check did. Fixed by wrapping the siblings in an explicit `Fragment`, mirroring the pattern `drag_and_drop_list.rs` already uses, with two unit tests pinning it (present on first render, still present after `render_immediate` flushes effects). **Not swept:** no audit was done for other multi-sibling rsx returns in `primitives/src/**` that could have the same shape. Worth a guard script if a second instance turns up — per CLAUDE.md, two occurrences would make it a class.  **Update, same day: this instance no longer exists** — the `<style>` tag it was keeping alive turned out to be unparseable anyway (row 96) and was deleted along with the `Fragment` that wrapped it. The underlying Dioxus behaviour the row describes is still real and still worth knowing, so the row stays: a second, un-fragmented rsx sibling renders on the first pass and is dropped by the next. Note the sequence though — the `Fragment` fix was correct *and* insufficient, because it made the element survive while the CSS inside it was still being discarded by the parser. Two different failures with the same visible symptom (`scroll-snap-stop` computing `normal`), and fixing the first one made the second one look fixed too. A live `getComputedStyle` read distinguishes them; the served HTML does not.

**Resolution 2026-10-23: there is no dropped-sibling bug; the class is dynamic text inside a raw-text element.** Measured, write-up and table in [`dev-docs/issues/raw-text-hydration-markers.md`](../../issues/raw-text-hydration-markers.md). (1) At `67f05223` with the `Fragment` removed, that commit's own `no_skip_style_tag_survives_the_post_effect_re_render` passes: the regression test never failed on the shape it pinned. (2) Seven multi-root shapes in a VirtualDom, the exact carousel shape included, all keep the second root through the effect re-renders. (3) In a hydrated fullstack build, bare roots and `Fragment` behave identically: the `<style>` is present in both and broken in both, because `dioxus-ssr`'s `<!--node-idN-->`/`<!--#-->` markers around a dynamic text node are literal text inside `style`/`script`/`textarea`/`title` (the CSS parser turns them into one unmatched rule; a `textarea` shows them in its value). Static text and `dangerous_inner_html` are clean. So the 2026-09-20 symptom had one cause, row 96's markers, and the `Fragment` fixed nothing. **Construction:** the rsx analyzer behind `check-attr-spread-collision.sh` reports `raw-text-dynamic-child` (any non-static child of a raw-text element outside `svg`/`math`) with no baseline; the sweep of `primitives/src`, `preview/src` and `templates` found 0 sites. Pinned by six analyzer unit tests, five of them red before the rule existed.
