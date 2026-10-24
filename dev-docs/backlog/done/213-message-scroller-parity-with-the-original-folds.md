---
id: 213
title: "Message Scroller parity with the original (folds row 146)"
status: "LANDED 2026-10-19 (all ten items; markdown landed by its own lane; upstream scroller unchanged…"
size: "M-L"
area: virtual
updated: 2026-10-19
---

# Row 213: Message Scroller parity with the original (folds row 146)

## Item

Message Scroller parity with the original (folds row 146)

## Status / trigger

**LANDED 2026-10-19 (all ten items; markdown landed by its own lane; upstream scroller unchanged since 295a1f1)**

Earlier status: **OWNER DECISIONS 2026-10-15 (markdown, the nine questions): the recommendations** -- pulldown-cmark `=0.13.4` (default features off) in a separate optional `markdown` component; exact pin; model-output images off by default; links http/https/mailto/tel/relative only, external in a new tab with noopener; plain code block with a highlighter slot; streaming heal pass in the component; bare-URL autolinks yes (own pass), math no; both gates (parser allowlist, versions match the lock) added. **Round 3: after the 2026-10-15 round (row 217 first).** **OPEN 2026-10-14 (batch 2 did not take the API items; the demo half landed earlier; markdown decision PENDING the owner: nine questions in the published analysis, https://claude.ai/artifact/GpaybnGgtbQTtYkebbUDwp)**

## Size

M-L

## Detail

Parity audit of the MessageScroller against the shadcn original (research:
`dev-docs/archive/research/message-scroller-parity-2026-10-14.md`, `dev-docs/archive/research/chat-kit-demos-2026-10-14.md`).
**Verdict:** the scroll controller is at parity (identical maths: 8 px edge threshold, 64 px peek, tail spacer, last-anchor
fallback, prepend restore, coalesced observers) and better in places (follow re-arms only at the end, typing in editables does
not release follow, the jump button hands focus to the viewport, smooth to auto under reduced motion, `content-visibility`
virtualization). Ten gaps by impact were listed; all ten landed by 2026-10-19:

1. `use_message_scroller_visibility() -> ReadSignal<Visibility { current_anchor_id, visible_message_ids }>`: an
   IntersectionObserver in `message_scroller.js`, ref-counted by construction (observe on the first subscriber, stop on the last).
2. `InputGroupAlign::BlockStart` / `BlockEnd` for the composer, plus an `input_group` `block` demo.
3. The chat-kit demos (shadcn has 44 chat-kit examples against our 25), landed earlier by the chat lane.
4. `ScrollToMessageOptions.scroll_margin` and `ScrollBehavior::Instant`; the bool returns were NOT implemented on purpose: the
   commands are documented as fire-and-forget (a JS-to-Rust reply would be a guess made before the answer exists; the observable
   outcomes are the `scrollable` and `visibility` signals).
5. `dx-enter-spring-bounce`, `dx-enter-blur-fade`, `dx-enter-slide-side` keyframes in `dx-effects.css` (opacity, scale,
   translate, filter only; finite; reduced motion skips them) and three more presets in the `animation` demo.
6. `default_scroll_position` as a `ReadSignal` and part of `Msg::Config`; a change re-applies the opening position.
7. One RTL assertion (jump button centred in LTR and RTL).
8. `message-scroller-timing.spec.ts`, a release-only 1,000-row streaming append spec.
9. Docs sections (live position, per-call margin and Instant, fire-and-forget, reading scroll state, tracking the reader's
   position, animating new messages). Specs: `playwright/message-scroller-api.spec.ts`. The scroll-fade prop landed with row 186.
10. A blobless full-history clone of shadcn-ui/ui (HEAD `e319626`, 98 commits past `295a1f1`) shows zero changes to any
    message-scroller path, so nothing was ported; the JS provenance header pins both commits. The unrelated upstream additions
    (code-block, trace, checkpoint, the ai-elements docs) became row 265
    (`dev-docs/archive/research/message-scroller-upstream-2026-10-19.md`).

**Markdown (owner decisions 2026-10-15, built 2026-10-19 by its own lane).** The `markdown` registry component
(`preview/src/components/markdown/`) follows the decisions: `pulldown-cmark =0.13.4`, default features off, only
tables, strikethrough and task lists. The event stream is folded into a closed `Node`/`Kind` tree with no raw-HTML variant (raw
HTML, inline HTML and math become text), and a link or image target exists in the tree only as a `SafeHref`, whose single
constructor `safe_href` allows http, https, mailto, tel and relative targets and rejects any target containing whitespace, a
control character or a backslash. External links get `target="_blank" rel="noopener noreferrer"`; images are off by default
(`allow_images` opts in, lazily, `no-referrer`); code blocks are a plain `<pre><code class="language-x">` with a
`highlight: Callback<CodeBlock, Element>` slot and no shipped highlighter; `heal_markdown` heals only the last paragraph while
streaming and is total and idempotent on every prefix of a nasty document; bare-URL autolinks are the component's own pass; no
math. Gate `scripts/check-markdown-parser.sh` (parser allowlist, exact pin, no HTML renderer, no `dangerous_inner_html`), next to
the existing `check-cargo-dep-versions.sh`. Docs, six demos and `playwright/markdown.spec.ts` (structure, link policy including a
real popup with no opener, images, XSS inert, highlight slot, heal on/off, axe light and dark). Remaining at the time: the demo's
monospace face comes from a host-set `--dx-font-mono` (the theme defines none), and `scripts/registry-install-smoke.sh` was not
run by that lane.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/213-message-scroller-parity-with-the-original-folds.md`.*
