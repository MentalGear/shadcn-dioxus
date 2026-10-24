# Message Scroller parity: shadcn original vs ours (read-only research, 2026-10-14)

> Provenance: produced by a read-only research agent on 2026-10-14 from shadcn-ui/ui main @30a58d4. Copied here from the agent's working notes; summarised in backlog row 213.

No repo files edited; no cargo/dx run; the site was not started. Evidence is code read from both trees.

## 0. Sources and caveats

- shadcn clone `/home/user/shadcn-ui/ui` is **sparse** (only `apps/v4/{examples,registry}` checked out) and **shallow** (1 commit, `30a58d4`, 2026-10-12). `packages/` and `apps/v4/content` exist in the git tree (blob:none promisor) and were read with `git show HEAD:<path>` into the scratchpad (`.../scratchpad/ms/`).
- The styled UI file (`apps/v4/registry/bases/{base,radix}/ui/message-scroller.tsx`, 139 lines; `new-york-v4/ui/...`, 130) is only a **styling wrapper**. All behaviour lives in `@shadcn/react`, `packages/react/src/message-scroller/` (the files below). The prompt's grep of `packages` found nothing only because it is not checked out.
- Docs: `apps/v4/content/docs/react/message-scroller.mdx` (API tables) and `.../components/base/message-scroller.mdx` (behaviour guide, 15 rules).
- **Change since 295a1f1: not determinable.** Commit `295a1f1` is not in the clone (`git cat-file` fatal) and history is 1 commit. Only evidence: `message-scroller.tsx` is the same at `30a58d4` as the catalog doc (2026-10-04) describes (139 lines, same viewport classes), and the catalog's description of the controller matches the code read now (mode machine, 8px, 64px, 180ms). Treat "unchanged" as likely, unverified. Cheap way to settle: `git fetch --unshallow` the clone, then `git log -- packages/react/src/message-scroller`.
- The catalog doc (section 4.1) cited `packages/react/src/message-scroller/` for these behaviours, so its source was the upstream repo at the time; everything quoted below was re-read now.
- Behavioural claims about "ours is better" come from reading code, not from running either. Marked **(read, not run)** where it matters.

shadcn files (all under `packages/react/src/message-scroller/`): `components.tsx` 446, `use-message-scroller-controller.ts` 774, `use-message-scroller-commands.ts` 328, `use-message-scroller-refs.ts` 200, `geometry.ts` 387, `types.ts` 240, `stores.ts` 97. Tests: 49 jsdom, 17 geometry, 17 real-browser, 6 perf.
Ours: `primitives/src/message_scroller.rs` 1183, `message_scroller.js` 921, `preview/src/components/message_scroller/{component.rs 181, style.css 180, docs.md 123, variants/{main,last_anchor,load_history,long}}`, `playwright/message_scroller.spec.ts` 505 (26 tests).

## 1. API table

### Provider
| shadcn (name, type, default) | Ours | Status |
|---|---|---|
| `autoScroll: boolean = false` | `auto_scroll: ReadSignal<bool>` default false (rs:352) | present |
| `defaultScrollPosition: "start"\|"end"\|"last-anchor" = "end"` | `default_scroll_position: DefaultScrollPosition` default `End` (rs:358) | present, **but mount-only**: not in `Msg::Config` (rs:275-283). shadcn re-applies on a live prop change (controller.ts:106-113). |
| `scrollEdgeThreshold = 8` | `scroll_edge_threshold: ReadSignal<f64>` = 8.0 (rs:362) | present |
| `scrollPreviousItemPeek = 64` | `scroll_previous_item_peek` = 64.0 (rs:365) | present |
| `scrollMargin = 0` | `scroll_margin` = 0.0 (rs:369) | present |
| - | `virtualize: Virtualization::{None, ContentVisibility}` default `ContentVisibility` (rs:373) | **extra in ours** |

### Parts
| shadcn | Ours | Status |
|---|---|---|
| `Root` (div props) | `MessageScroller` | present |
| `Viewport`: `preserveScrollOnPrepend = true`; `role="region"`, `aria-label="Messages"`, `tabIndex=0` overridable | `MessageScrollerViewport { preserve_scroll_on_prepend = true }`, same ARIA defaults (rs:105-112), overridable via merged attributes | present |
| `Content`: `role="log"`, `aria-relevant="additions"`, `aria-busy`, `spacerClassName` | `MessageScrollerContent { spacer_class }`, same role/relevant (rs:135-140); `aria-busy` passes through attributes | present |
| `Item`: `messageId?`, `scrollAnchor=false`; emits `data-message-id`, `data-scroll-anchor="true\|"false"` | `MessageScrollerItem { message_id: Option<String>, scroll_anchor=false }` (rs:157-181) | present |
| `Button`: `behavior="smooth"`, `direction="end"`, `render`, children default icon+label; `data-active`, `data-direction`; `inert` + `tabIndex=-1` when inactive | `MessageScrollerButton { direction, behavior=Smooth, onclick, r#as }` (rs:214-274); same attrs; SSR renders inert/inactive | present (`render` is `r#as`, repo idiom) |
| - | `MessageScrollerRows { count, start, render_row }` chunked rows (rs:184-211) | **extra in ours** |

### Hooks and command options
| shadcn | Ours | Status |
|---|---|---|
| `useMessageScroller()` -> `scrollToMessage(id, opts) : boolean`, `scrollToEnd(opts) : boolean`, `scrollToStart(opts) : boolean` | `use_message_scroller()` -> `MessageScrollerApi { scroll_to_end(behavior), scroll_to_start(behavior), scroll_to_message(id, ScrollToMessageOptions) }`, all return `()` (rs:281-303) | present, **no return value** (fire-and-forget over `dioxus.send`) |
| option `align: start\|center\|end\|nearest = "start"` | `ScrollAlign::{Start, Center, End, Nearest}` (rs:205) | present |
| option `behavior: ScrollBehavior ("auto"\|"instant"\|"smooth") = "auto"` | `ScrollBehavior::{Auto, Smooth}` (rs:194) | different: no `Instant` |
| option `scrollMargin: number` (per call, default = provider) | `ScrollToMessageOptions { align, behavior }` only (rs:219-224); `Msg::Message` has no margin (rs:295-299); JS passes `null` (js:570) | **missing** (row 146 stage 2 knows) |
| `useMessageScrollerScrollable()` -> `{start, end}` | `use_message_scroller_scrollable() -> ReadSignal<Scrollable>` (rs:311), change-only messages | present |
| `useMessageScrollerVisibility()` -> `{currentAnchorId, visibleMessageIds}` | none; js:15-16 says "Stage 1 omits the visibility store" | **missing** |

### Data attributes
`data-scrollable`, `data-autoscrolling` (root + viewport), `data-pending-scroll` (root + viewport, only for `end`/`last-anchor`), item `data-message-id`/`data-scroll-anchor`, button `data-active`/`data-direction`: all present in ours (js:367-379, rs:78-81, 105-112). Extras in ours: `data-preserve-scroll-on-prepend` (viewport), `data-virtualize` (content), `data-keep-rendered` (rows), `data-message-scroller-{root,viewport,content,spacer,chunk,button}` (discovery hooks).

## 2. Behaviour semantics

Notation: S = shadcn file:line (from `packages/react/src/message-scroller/`), O = ours.

**2.1 Follow the stream (autoScroll).**
S controller.ts:148-177 `reconcileFollowMode`: arms whenever `autoScroll && !scrollable.end && mode not settling/anchored`; releases when `mode===following-bottom && scrollable.end && scrolledUp && !autoscrolling` (`scrolledUp = scrollTop < last - 0.5`). Release by intent: `userScrollIntent` (controller.ts:632-643) on wheel/touchmove/`USER_SCROLL_KEYS` (components.tsx Viewport handlers; types.ts:`USER_SCROLL_KEYS` ArrowUp/Down, Home/End, PageUp/PageDown, Space). Threshold: `scrollEdgeThreshold` 8 px (geometry.ts:12-35).
O js:393-415 same machine, **arms only when the reader scrolled DOWN** (`scrolledDown`, js:398-411), deviation noted at js:28-30. O `userScrollIntent` js:734-742 same. O `onWheel` js:751-755 ignores wheel-down when `atEnd()`; `onTouchMove` js:761-767 likewise; `onKeyDown` js:769-774 ignores keys typed into editable controls and toward-end keys at the end.
Class (b), ours arguably better: nothing-moved gestures do not release; a release is not undone by a state commit that lands before the smooth-scroll animation moves the viewport (the race documented at js:398-402). Cost: a content shrink that clamps `scrollTop` at the end does not re-arm (shadcn would, since it re-arms on "at end" alone). Neither is covered by shadcn's own claim "selecting text, opening a link or searching should stop the interface" (docs rule 3): the code of both only handles wheel/touch/keys/scrollbar-drag.
"Button does not strobe": S commitScrollState controller.ts:194-197 forces published `end=false` while following; O js:423-426 same (plus `&& cfg.autoScroll`).

**2.2 New-turn anchoring and tail spacer.**
S handleContentChange controller.ts:437-461: a new `scrollAnchor` row among appended rows -> `scrollToElement(anchor, {align:"start"}, {keepPreviousPeek:true})` (commands.ts:172-231: margin = `scrollMargin + peek`; sets spacer height `scrollTop + clientHeight - contentBottom`; mode `anchored-to-message`; `streamingTurnRef = el`). Several new anchors while following -> `scrollToEnd` instead (controller.ts:445-452). Hand-off: S handleResize controller.ts:515-529 hands anchored -> follow when spacer goes `>0 -> 0`.
O js:685-696 and js:457-465, 499-518 (`setTailSpacerHeight`, `scrollToElement`), js:711-732 (`handleResize`): line-for-line the same maths (`js:503` equals `getTailSpacerHeight`, geometry.ts:280-294). Class: present, identical.
Difference (b, ours better, **read not run**): S's `list.length === previousCount` branch uses `getUnanchoredScrollAnchor(items, handledAnchors)` (controller.ts:464-479, geometry.ts:130-141) which returns the first anchor never scrolled to. An opened saved thread has user rows never "handled", so swapping a typing marker for its reply (same row count) would anchor-scroll to the first user message. O `takeFreshAnchor` (js:312-320, rationale js:31-33, 110-115) only treats a row as a new anchor when it was not one before.

**2.3 Opening position, flash guard.**
S applyDefaultScrollPosition controller.ts:325-381: `start`/`end` via scrollToStart/End("auto"); `last-anchor`: last anchor, falls back to end when `contentBottom - anchorTop <= clientHeight` (controller.ts:357-364), else anchor with peek. `data-pending-scroll` initial true for `end` and `last-anchor` (refs.ts:125-131), cleared when applied (refs.ts:186-193) or the transcript is empty (controller.ts:715-723).
O js:586-607 identical rules; SSR flash guard rs:430 (`holds_pending`), rs:78-81, 105-112; CSS `visibility:hidden` (style.css `.dx-message-scroller-viewport[data-pending-scroll]`). Present.
Differences: (b) O waits until the viewport really is a scroll container (`ready()` js:349-353, `pollOpen` js:611-627, 120-frame cap) because the stylesheet can arrive after the script; S has no such poll. (c) O CSS fallback reveals the viewport after 4 s if the controller never runs (style.css). (a) S re-applies on a live `defaultScrollPosition` change (controller.ts:106-113); O does not (config carries no position).

**2.4 Previous-item peek.** S `scrollPreviousItemPeek` default 64 added to `scrollMargin` only for `keepPreviousPeek` moves (commands.ts:193-198). O js:501 same. Present. No demo adjusts the peek (shadcn's `message-scroller-previous-context`).

**2.5 Prepend preservation.**
S capturePrependAnchor controller.ts:288-309 (first visible row with `data-message-id` + viewport-relative top), restore controller.ts:260-286 corrects by delta only when the browser did not already; `didPrepend = preserve && previousFirstItemIndex > 0` (controller.ts:427-428).
O js:631-649 and js:678-684: identical. Present. O extension: also looks inside chunks (js:322-342), and `MessageScrollerRows { start }` keeps chunk grouping stable across prepends.

**2.6 Jump button.**
S Button components.tsx: visible iff `state[direction]`; `inert` + `tabIndex=-1` when inactive; click -> `event.currentTarget.blur()` then `scrollToStart/End({behavior})` (default `"smooth"`). Visibility threshold = `scrollEdgeThreshold`. Direction `start` rotates the icon (styled wrapper).
O js:357-365 `updateButtons`, rs:234-274. Differences: (b, ours better) focus: S blurs the button, so keyboard focus falls to `<body>` when it turns inert; O hands focus to the viewport first (`releaseButtonFocus` js:846-851, spec test "is operable from the keyboard (PageUp, Tab, Enter) and hands focus back to the viewport" spec:251). (b) O honours reduced motion: `smooth` becomes `auto` when `prefers-reduced-motion` (js:126-127, 475); S's primitive does not, and its styled button's transition classes are not gated (message-scroller.tsx class string has no `motion-reduce`) - **read, not run**. Reduced-motion CSS on our button: style.css `@media (prefers-reduced-motion: reduce)`.

**2.7 `scrollToMessage`.**
S commands.ts:258-291: missing id with `itemCount===0` is queued and returns true; missing id after mount returns false; also marks default position applied (so it does not override). Flush in `registerMessage` or next content change (commands.ts:293-316, controller.ts:612-614). Options `align` (geometry.ts:204-265 all four), `behavior`, `scrollMargin`. Mode `settling-jump`.
O js:560-582 same queue semantics, js:266-286 same align maths. Missing: per-call `scrollMargin`, return value, `instant`. Extra (c): `reaimJump` js:522-536 re-aims a far jump for 1 s while `content-visibility` rows render at real size (needed because of our virtualize mode; S has no equivalent because its rows are always estimated by `contain-intrinsic-size` only, **its far-jump accuracy is unknown**).

**2.8 Visibility tracking.**
S controller.ts:542-603 `observeVisibility`: IntersectionObserver `root: viewport`, `rootMargin: -(scrollMargin + peek)px 0 0 0`, `threshold [0,.01,.5,1]`, ref-counted (only while a subscriber exists); `getMessageScrollerVisibilityState` geometry.ts:37-106: `visibleMessageIds` in document order; `currentAnchorId` = last anchor whose top reached the reading line `viewportTop + scrollMargin + peek` (+0.5), stays set after it scrolls off.
O: none. Class (a). The state is computable in JS from our existing geometry; the Rust side needs a second change-only channel (`["visibility", anchorId, [ids]]`), and the ref-count (start observing on first subscriber) needs `Msg::Observe(bool)` driven by a hook that mounts/unmounts.

**2.9 Resize handling.** S: ResizeObserver on viewport (components.tsx Viewport effect) and on content (Content effect), both coalesced into one rAF to avoid "ResizeObserver loop" errors; MutationObserver `childList` on content for row changes. O js:776-783 and js:796-819: same, observers on viewport and content, rAF coalesced; plus MutationObserver on each chunk (js:785-794). Present.

**2.10 Reduced motion.** See 2.6. Only difference is ours (c).

**2.11 Keyboard and focus, ARIA, live regions.** S Viewport `role=region`, `aria-label=Messages`, `tabIndex=0` (components.tsx); Content `role=log`, `aria-relevant=additions`, `aria-busy` passthrough; Button `inert`. O identical defaults (rs:105-140) with a rustdoc and a test (`viewport_and_content_carry_shadcn_aria_defaults`) and axe at rest and with the button visible (spec:490-500). Present. O extra (c): focused row and selection ends are kept out of `content-visibility` skipping (js:159-190).

**2.12 RTL.** S button uses `inset-s-1/2 -translate-x-1/2 rtl:translate-x-1/2`. O `inset-inline:0; margin-inline:auto` (style.css) needs no flip. The scroll maths is block-axis only in both. The row-146 "RTL pass over the button placement" is therefore **probably already satisfied by construction (not verified in a browser; the spec has no RTL test, grep `rtl` returns nothing)**; what remains is one RTL assertion, not a change.

**2.13 Virtualization / content-visibility.** S: not virtualized; styled Item has `[content-visibility:auto] [contain-intrinsic-size:auto_10rem]` on every row always (message-scroller.tsx Item class), no live-edge protection. O: `Virtualization::{None, ContentVisibility}`, chunking (`MessageScrollerRows`, groups of 20), keep-rendered live edge (8 rows), focus row and selection ends (js:156-190). Class (c), a deliberate superset (row 146: windowing deferred by owner).

**2.14 Provider live config.** S: `autoScroll` false->true after mount: layout effect scrolls to end if mode is following (controller.ts:755-766). O `config` handler js:855-873 arms following when at end, then scrolls. Present. `defaultScrollPosition` live change: see 2.3.

**2.15 Native/Blitz arm.** O: inert shell without `document::eval` (rs module doc "Renderer arms"). S: n/a. Class (c).

## 3. Differences classified

(a) We are missing it
1. `use_message_scroller_visibility` (`currentAnchorId`, `visibleMessageIds`).
2. Per-call `scroll_margin` on `scroll_to_message` (and on end/start, which ignore it in shadcn too).
3. Return value from `scroll_to_message`/`scroll_to_end`/`scroll_to_start` (`bool`: false = id unknown after mount / viewport not mounted).
4. `ScrollBehavior::Instant`.
5. Live `default_scroll_position` change re-applies the opening position.
6. Demo coverage: shadcn ships 12 message-scroller examples (demo, anchoring, previous-context, streaming, opening-position, load-history, animation, commands, visibility, scrollable, state, group-chat) plus two AI-helper demos; ours has 4 (main, last_anchor, load_history, long). Missing demos: anchoring toggle, previous-context peek control, opening-position toggle, commands/jump menu, visibility/TOC, group chat (marker as anchor), animation presets, scrollable/state readout (ours folds a status readout into `main`), composer. See `research-chat-demos.md` in the same folder for the cross-component table.
7. Docs sections absent in ours: "Animating new messages", "Tracking the reader's position", "Reading scroll state" as separate sections, a "Virtualization with TanStack" equivalent (n/a here).

(b) We behave differently
| Area | shadcn | Ours | Better |
|---|---|---|---|
| Follow arming | at end | only after scrolling down to end (js:398-411) | ours (avoids release race); cost: no re-arm on a clamp |
| Release gestures | any wheel/touch/key | skips gestures that cannot move the reader (wheel-down at end) and keys typed in editable controls (js:751-774) | ours |
| Anchoring a swapped row | `getUnanchoredScrollAnchor` may re-anchor an old user row | `takeFreshAnchor` | ours (read, not run) |
| Opening before CSS ready | applies immediately | polls `ready()` | ours |
| Button focus on inert | `blur()` -> `<body>` | focus to viewport | ours |
| Smooth + reduced motion | smooth always | `auto` under reduce | ours |
| Content bottom | max over all rows | last row only (js:235-245) | ours for perf (assumes in-flow order; breaks with reordering/absolute rows) |
| Row skipping | every row `content-visibility:auto` | configurable, live edge/focus/selection kept rendered | ours |

(c) Ours has, shadcn lacks: `Virtualization` + `data-keep-rendered`, `MessageScrollerRows` chunking, `reaimJump`, 4 s CSS reveal fallback, native/Blitz inert arm, `pollOpen`, `data-preserve-scroll-on-prepend`, Rust-side change-only `scrollable` signal, release-only timing spec, SSR/hydration tests.

## 4. Recommended implementation list (by user-visible impact)

Size S/M/L. Files named per item. Row 146 stage-2 items marked [146].

| # | Item | Size | Files | Notes |
|---|---|---|---|---|
| 1 | **`use_message_scroller_visibility`** [146] | M | `primitives/src/message_scroller.js` (IntersectionObserver as S controller.ts:542-603, state fn from geometry.ts:37-106, `observe`/`unobserve` messages), `primitives/src/message_scroller.rs` (hook + `Signal<VisibilityState>` + `Msg::Observe`, change-only `["visibility", anchor, ids]`), `preview/.../component.rs` (re-export), new variant, `playwright/message_scroller.spec.ts`, docs.md | Unlocks the jump-menu/TOC demo and active-turn UIs. Must stay ref-counted so it costs nothing without a subscriber. |
| 2 | **Faithful composer: `InputGroupAlign::BlockEnd` (and `BlockStart`)** | S | `preview/src/components/input_group/{component.rs, style.css, docs.md}` (enum at component.rs:9-19; CSS at style.css:71-75), a demo variant | shadcn names are `inline-start/inline-end/block-start/block-end`; ours `Start/End` stay as inline. Needs: group flips to a column/wrap when a block addon is present (shadcn uses `has-[>[data-align=block-end]]:h-auto flex-col`; **exact classes unverified, read `input-group.tsx` lines 25-40 and the group base class**), addon `order`, `width:100%`. Also a bare text span as the "control" (shadcn's composer is a read-only span), so no textarea needed. `check-cross-component-overrides.sh` applies: compose through slot variables. |
| 3 | Demo set to match shadcn's 12: anchoring toggle, previous-context (peek control), opening-position, commands (jump menu), visibility (needs #1), scrollable/state, group chat (Marker anchor) | M (S each) | `preview/src/components/message_scroller/variants/<new>/mod.rs`, `docs.md`, `playwright/message_scroller.spec.ts` | Mostly composition of what exists. `DemoForm` rule applies to any `<form>`. |
| 4 | `scroll_to_message` per-call `scroll_margin` [146], return `bool`, `Instant` | S-M | `message_scroller.rs` (`ScrollToMessageOptions`, `Msg::Message`, `ScrollBehavior`), `message_scroller.js` (`jumpToMessage` js:560-572 pass margin; `handle` case "message"), spec | `bool` needs a reply channel (JS -> Rust `["jump", id, ok]`) and an `async fn`/`Signal`; if that is judged too heavy, document "fire and forget" and skip. Check `check-preview-composition.sh` allowed_qualified for any new hook name. |
| 5 | Animation demo (7 presets) | S (3 presets), M (all 7) | demo-only CSS for fade/slide-up/pop/scale-fade/ using existing `dx-fade-in`/`dx-pop-in`/`dx-drop-in`/`dx-edge-in-*` (theme: `preview/assets/dx-components-theme.css:1056-1117, 1412-1482`); new theme keyframes for `spring-bounce`, `blur-fade`, `slide-side` | See below. |
| 6 | Live `default_scroll_position` | S | `message_scroller.rs` (`Msg::Config` gets the field, rs:275-283), `message_scroller.js` (`handle` case "config": reset `defaultApplied`, call `applyDefaultScrollPosition`, no re-hide) | Low impact; shadcn's opening-position demo toggles it. Ours can remount instead. |
| 7 | RTL assertion [146] | S | `playwright/message_scroller.spec.ts` | One test with `dir=rtl`: button is centred, `inset-inline` placement; expected to pass with no code change. |
| 8 | Release-build timing spec [146] | S | spec exists (spec:441, "release only"); extend to streaming 1,000-row append cost | Only on release SSG (`PW_BUILD_PROFILE`). |
| 9 | Docs: reading-scroll-state, tracking position, animation sections; confirm Safari 18-25 note (row 147) | S | `preview/src/components/message_scroller/docs.md` | `docs.md` already carries the find-in-page note; whether row 147 is closed is not verified. |
| 10 | Re-sync check against upstream | S | none (research) | Unshallow the clone and diff `packages/react/src/message-scroller` since 295a1f1 before declaring parity; pin the commit in `message_scroller.js` provenance header. |

### 4.1 Theme entrance keyframes (item 5)
shadcn's presets (`apps/v4/lib/message-animations.ts`, all on `motion/react`, the user row only, `initial:false` for others, `useReducedMotion` skips entrance):
- `fade` 0.22s ease-out opacity; `slide-up` y 10 -> 0, 0.26s cubic-bezier(.16,1,.3,1); `slide-side` x 18 -> 0, 0.28s same curve; `pop` scale .94, y 6, origin bottom-right, spring 500/34/0.7; `spring-bounce` y 12, scale .96, spring 520/30/0.7; `blur-fade` blur(4px) y 6, 0.28s ease-out; `scale-fade` scale .98, 0.24s.
- We have `dx-fade-in`, `dx-pop-in`, `dx-drop-in`, `dx-edge-in-{top,bottom,left,right}` (translate-based). Missing: spring (needs CSS `linear()` easing to emulate a spring; no JS), blur (`filter` in keyframes: the `infinite-animations` gate only restricts infinite animations, but `anchored-keyframes` forbids `transform` in keyframes; use `translate`/`scale`), slide-side (new `translate` keyframe, or reuse `dx-edge-in-right`'s shape with a smaller distance via a custom property).
- Gate friction: `check-duplicate-keyframes.sh` (no byte-identical bodies), `check-anchored-keyframes.sh` (no `transform`), `check-motion-gating.sh`. Put them in the theme (once), selected through a slot variable, not in the component sheet. Reduced motion must skip the entrance (`--dx-motion-duration-reduced`).
- One structural point from shadcn's docs: animating the **Item** (not an inner wrapper) with transform/opacity is safe for anchoring; animating height/margin/padding is not. Our row is the `MessageScrollerItem` div, so the same rule holds.

### 4.2 Markdown rendering (dependency decision for the owner)
shadcn uses `streamdown` ^2.5 (+ `@streamdown/code`, shiki) via `apps/v4/components/markdown.tsx`; it is **not a registry component**, only an example helper. Used by `message-markdown`, `bubble-markdown`, and streamed assistant text in the animated demos. Streamdown specifically handles *incomplete* markdown while streaming (unclosed `**`, fences, links).
Already in this workspace: `pulldown-cmark = "0.13.0"` as a **build-dependency** of `preview` (`preview/Cargo.toml:24`; registry copy 0.13.3, ~13.7k lines of source, pure Rust, `memchr`/`bitflags`/`unicase`/`pulldown-cmark-escape` deps). It parses to events (so a Dioxus renderer can map events to `rsx!` without `dangerous_inner_html`), is wasm32-clean (no I/O, no threads; default features `getopts`+`html` are native conveniences, turn off `getopts`). Compiled wasm size not measured here (**unmeasured; expect tens of KB gzip, to be measured with `twiggy`/`wasm-opt` before deciding**).
Candidates (none has been built here; sizes are not measured; "wasm32" below is from the crate's design, not tested in this repo):
| Crate | Fit | Notes |
|---|---|---|
| `pulldown-cmark` | Best: already vendored, event stream -> `rsx!`, CommonMark + GFM-ish options (tables, strikethrough, tasklists) | Unclosed constructs render literally while streaming: needs a small "heal" pass (close open `**`, `` ` ``, fences, `[`) before parsing; Streamdown's `remend` is the reference behaviour. Syntax highlighting: this repo already has `dioxus-code` in `preview` build-deps (`lang-css` only), a code-block path exists. |
| `markdown` (markdown-rs) | Pure Rust, no_std-friendly, spec-compliant, AST (`mdast`) | Larger than pulldown-cmark in my recollection (**unverified**); AST is convenient for rendering; no streaming healing either. |
| `comrak` | Full GFM, AST; heavy defaults (clap, syntect) | Only with `default-features=false`; arena AST is awkward in Dioxus; clearly the heaviest. |
| A Dioxus-specific markdown crate (e.g. `dioxus-markdown`) | Hides the event loop | Existence/version/maintenance **unverified**; adds a dependency on a third-party Dioxus pin. |
| Avoid: JS renderer via eval (marked/markdown-it) | no new Rust dep | Loses SSR/hydration parity; contradicts the repo's rule that first render is identical on both sides. |
Decision points for the owner: (1) ship it as a **component** (`markdown`, in the registry, with `cargoDependencies`) or only as a demo helper like shadcn; (2) heal-while-streaming pass in scope or not; (3) code-block highlighting in scope or plain `<pre>`; (4) gate cost: `check-installed-paths.sh` requires external crates in `cargoDependencies`, so a new crate means every consumer of the component compiles it. A demo-only plain-text `Bubble` is the zero-dependency alternative (the current state).

## 5. Verdict

Behaviour of the controller is at parity or better for everything in shadcn's stage-1 list: follow, release, anchoring with peek and tail spacer, opening position with flash guard, prepend preservation, jump button, commands, resize, ARIA. Gaps are API-level, not scroll-level: visibility hook, per-call margin, command return values and `instant`, live default-position change, and the demo set (4 of 12). The "RTL pass" in row 146 is likely a test only. Upstream drift since 295a1f1 is unknown (shallow clone) and should be checked once before calling parity final.
