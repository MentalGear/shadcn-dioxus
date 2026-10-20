---
id: 97
title: "A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one"
status: "Found 2026-09-20 (round 5) adding a scoped <style> tag beside CarouselContent's track 2026-10-20…"
size: "DONE for this instance (explicit…"
area: misc
updated: 2026-10-20
---

# Row 97: A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one

## Item

A second, un-fragmented rsx sibling renders on the first pass and is silently dropped by the next one

## Status / trigger

Found 2026-09-20 (round 5) adding a scoped `<style>` tag beside `CarouselContent`'s track **2026-10-20 sync: checked, no guard script for un-fragmented rsx siblings exists; a second instance would make it a class. No work in flight.**

## Size

DONE for this instance (explicit `Fragment`); the class is NOT swept

## Detail

The `<style>` tag rendered correctly in SSR markup and then vanished on the second render, which in a real mount happens almost immediately via this component's own effects — so the served HTML looked right while `getComputedStyle` reported `scroll-snap-stop: normal` in the live browser. Reading the SSR output alone would never have caught it; only a live computed-style check did. Fixed by wrapping the siblings in an explicit `Fragment`, mirroring the pattern `drag_and_drop_list.rs` already uses, with two unit tests pinning it (present on first render, still present after `render_immediate` flushes effects). **Not swept:** no audit was done for other multi-sibling rsx returns in `primitives/src/**` that could have the same shape. Worth a guard script if a second instance turns up — per CLAUDE.md, two occurrences would make it a class.  **Update, same day: this instance no longer exists** — the `<style>` tag it was keeping alive turned out to be unparseable anyway (row 96) and was deleted along with the `Fragment` that wrapped it. The underlying Dioxus behaviour the row describes is still real and still worth knowing, so the row stays: a second, un-fragmented rsx sibling renders on the first pass and is dropped by the next. Note the sequence though — the `Fragment` fix was correct *and* insufficient, because it made the element survive while the CSS inside it was still being discarded by the parser. Two different failures with the same visible symptom (`scroll-snap-stop` computing `normal`), and fixing the first one made the second one look fixed too. A live `getComputedStyle` read distinguishes them; the served HTML does not.
