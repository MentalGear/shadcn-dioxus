---
id: 213
title: "Message Scroller parity with the original (folds row 146)"
status: "API ITEMS LANDED 2026-10-19 (items 1, 2, 4-9; item 10 upstream diff remains; markdown is another…"
size: "M-L"
area: virtual
updated: 2026-10-19
---

# Row 213: Message Scroller parity with the original (folds row 146)

## Item

Message Scroller parity with the original (folds row 146)

## Status / trigger

**API ITEMS LANDED 2026-10-19 (items 1, 2, 4-9; item 10 upstream diff remains; markdown is another lane)**

Earlier status: **OWNER DECISIONS 2026-10-15 (markdown, the nine questions): the recommendations** -- pulldown-cmark `=0.13.4` (default features off) in a separate optional `markdown` component; exact pin; model-output images off by default; links http/https/mailto/tel/relative only, external in a new tab with noopener; plain code block with a highlighter slot; streaming heal pass in the component; bare-URL autolinks yes (own pass), math no; both gates (parser allowlist, versions match the lock) added. **Round 3: after the 2026-10-15 round (row 217 first).** **OPEN 2026-10-14 (batch 2 did not take the API items; the demo half landed earlier; markdown decision PENDING the owner: nine questions in the published analysis, https://claude.ai/artifact/GpaybnGgtbQTtYkebbUDwp)**

## Size

M-L

## Detail

**Verdict:** the scroll controller is at parity (identical maths: 8px edge threshold, 64px peek, tail spacer, last-anchor fallback, prepend restore, coalesced observers) and better in places (follow re-arms only at the end, typing in editables does not release follow, `takeFreshAnchor`, open waits until scrollable + a 4s CSS fallback, the jump button hands focus to the viewport not body, smooth -> auto under reduced motion, `content-visibility` virtualization). **Gaps by impact:** (1) `use_message_scroller_visibility` (M); (2) `InputGroupAlign::BlockEnd`/`BlockStart` for the composer (S); (3) missing demos (in progress: chat lane; shadcn has 44 chat-kit examples, 40 docs previews + 4 examples-only, vs our 25); (4) per-call `scroll_margin` + Instant behaviour + bool returns (S-M; the bool needs a JS->Rust reply channel, or document fire-and-forget); (5) animation presets spring-bounce/blur-fade/slide-side as theme keyframes (S-M, gates apply); (6) live `default_scroll_position` (S); (7) one RTL assertion (S); (8) release timing spec (S); (9) docs sections (S); (10) unshallow the shadcn clone and diff upstream since 295a1f1 (S). **Markdown: OWNER DECISION PENDING.** Recommendation: a demo-only helper on `pulldown-cmark` (already a preview build-dependency) with a heal-while-streaming pass and no code highlighting; alternatives: a registry component (every consumer compiles the crate) or plain text. Research: `dev-docs/research/message-scroller-parity-2026-10-14.md` and `dev-docs/research/chat-kit-demos-2026-10-14.md`. **Update 2026-10-15:** the chat-kit demo half landed earlier (item 3). The scroll-fade prop landed (row 186): `fade` on `MessageScrollerViewport`, default `Bottom`. **Markdown:** the dependency and markdown analysis is published as https://claude.ai/artifact/GpaybnGgtbQTtYkebbUDwp (source `dev-docs/research/dependency-audit-2026-10-14.md`, part B); its recommendation is `pulldown-cmark` pinned `=0.13.4` in a separate optional `markdown` component (this refines the demo-only-helper recommendation above), and the nine owner decisions (parser, pin style, images, link policy, code blocks, streaming heal pass, autolinks and math, the new dependency-version gate, which part-A follow-ups to schedule) are still pending. Part-A follow-ups are row 217. Still open here: items 1, 2, 4-10 (the API items).

**Update 2026-10-19 (lane scroller, Round 2):** the remaining API items landed. (1) `use_message_scroller_visibility() -> ReadSignal<Visibility { current_anchor_id, visible_message_ids }>`: an IntersectionObserver in `message_scroller.js` (root margin = margin + peek, thresholds as upstream, `currentAnchorId` by binary search over anchors so skipped chunks are not laid out), ref-counted by construction (`Msg::Observe` is sent on the first subscriber mounting and the last unmounting; no hook, no observer), change-only `["visibility", anchor, ids]` out. (2) `InputGroupAlign::BlockStart` / `BlockEnd` (full-width rows via `order`, group becomes a column) plus an `input_group` `block` demo. (4) `ScrollToMessageOptions.scroll_margin: Option<f64>` and `ScrollBehavior::Instant`; the bool returns are NOT implemented on purpose: the commands are now documented as honestly fire-and-forget (a JS-to-Rust reply would be a guess made before the answer exists, and the observable outcomes are the `scrollable` and `visibility` signals). (5) `dx-enter-spring-bounce` (CSS `linear()` spring), `dx-enter-blur-fade`, `dx-enter-slide-side` keyframes in `dx-effects.css` (opacity/scale/translate/filter only, finite, reduced-motion skips them; no `use_motion` needed because nothing loops) and three more presets in the `animation` demo. (6) `default_scroll_position` is now a `ReadSignal` and part of `Msg::Config`; a change re-applies the opening position on the mounted transcript; the `opening_position` demo no longer remounts through a key. (7) one RTL assertion (button centred in LTR and RTL). (8) `message-scroller-timing.spec.ts`: release-only 1,000-row streaming append spec, skipped unless `buildProfile() === "release"`. (9) docs sections: live position, per-call margin and Instant, fire-and-forget, reading scroll state, tracking the reader's position, animating new messages. Specs: `playwright/message-scroller-api.spec.ts`. **Remaining:** item 10 (unshallow the shadcn clone and diff `packages/react/src/message-scroller` since 295a1f1, pin the commit in the JS provenance header), still unknown drift; item 3 (demos) was already done earlier and needs nothing here. Markdown is the markdown lane's.
