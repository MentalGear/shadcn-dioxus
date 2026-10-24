# Platform rationale: the caveats and rules behind the primitives' platform choices

Why the primitives prefer platform features (`<dialog>`, the Popover API, CSS anchor positioning) and fall back to
JavaScript only where no platform feature exists, plus the rules that came out of each incident. About 25 code files cite
the sections below by number or name (`Caveat 1`, `Caveat 2`, `§1`, `§5`, `§8`, `§9`, `§11`, "keyboard open contract"),
so the headings stay. The per-gap build plans that used to sit beside them (what to port from which fork, the order to
build in, the dated landing addenda) are archived in
[`../archive/research/recommended-implementations.md`](../archive/research/recommended-implementations.md); what landed is
in [`../plan.md`](../plan.md) and the backlog rows.

Sources compared when this was written (2026-08/09): this repo, `radix-ui/primitives`, `dignifiedquire/dx-components`,
`sarendipitee/dioxus-components`, `jcgruenhage/dioxus-components` and the WHATWG HTML spec.

---

## The finding that changes the plan

`dignifiedquire`'s `dialog.rs` renders a modal dialog with the native `<dialog>` element opened by `showModal()`, which
provides a focus trap, ESC-to-close and inert siblings as native browser behaviour. One native element closes four gaps
at once for modal dialogs: focus trap, focus restore, background inertness, and top-layer rendering that escapes every
ancestor `overflow`, `transform` and stacking context. `inert` is also strictly stronger than `aria-hidden`: it removes
background content from the accessibility tree and blocks focus and pointer interaction. **Prefer platform features;
fall back to JavaScript only where no platform feature exists.** Two caveats gated this.

### Caveat 1 — why upstream left `<dialog>`: no recorded reason

| Date | Commit | |
|---|---|---|
| 2025-04-11 | `b3f6de53` "progress: dialog" | **Added** `<dialog>` + `showModal()`, one-way: `showModal()`/`close()` driven from the `open` signal with no `onclose`/`oncancel` sync |
| 2025-06-12 | `797b343e` "Fix keyboard navigation for the dialog and toast components (#47)" | **Removed** `showModal()` and the `dialog {` element for a focus trap plus a document-level Escape listener (upstream `main`) |
| 2026-05-13 | `7b25d863` "Port Dialog to native `<dialog>` + show_modal" | Went back the other way, in `dignifiedquire`'s fork only |

PR #47 gives no architectural rationale. The concrete defect behind it: a native Escape stranded the `open` signal at
`true`, so the dialog could never reopen (issue #45, "cannot be used with just the keyboard"). Dioxus has no typed dialog
event binding (`dioxus#2962` onclose, `dioxus#4319` dialog methods), so the removal was a rational retreat from a one-way
binding forced by a framework gap, compounded by uniformity (AlertDialog and Toast never used `<dialog>`). Blitz
compatibility is unlikely to have been the motive: the replacement is more JavaScript-dependent, not less, and native is
not in CI.

**Verified by execution** (Chromium; spike notes in
[`../archive/research/phase4-spike-findings.md`](../archive/research/phase4-spike-findings.md)): the stranded-signal repro,
the close-sync fix, top-layer/inertness/focus-restore subsumption, free nested dialogs and `popover="auto"`. Two hazards
reading did not predict:

1. **The silent no-`showModal` state.** Dioxus commits the declarative `open` attribute during render, before the effect
   that calls `showModal()` runs, so a runtime guard reads stale state. Solved by construction with a cfg-split component
   body whose web arm never binds `open` and whose native arm never calls eval.
2. **A scrollbar-gap layout shift** in the scroll lock (15 px under a classic scrollbar). Fixed by construction with a
   permanent `scrollbar-gutter: stable` baseline plus an `overflow-y` toggle (§5). A claimed scroll-lock bypass behind a
   native `<dialog>` was falsified: `locator.click()` had auto-scrolled the trigger into view before the lock engaged.

#### The cfg axis: markup splits on the `web` feature, never on `target_family = "wasm"`

A 2026-09-01 production incident: the SSG prerender runs the **server binary**, a host (non-wasm) build, and every
overlay component split its rendered markup on `#[cfg(target_family = "wasm")]`. That is false on the SSG server, so the
server rendered the native arm (plain `div`s, no `popover` attribute, no `<dialog>`) while the wasm client hydrated
against a structurally different tree; event delegation is wired by walking the hydrated tree, so listeners were silently
left unattached page-wide.

**The rule** (`scripts/check-cfg-axis.sh` enforces it): rendered markup, component structure and attribute choice split on
the **`web` Cargo feature** (`#[cfg(feature = "web")]` / `#[cfg(not(feature = "web"))]`), a *renderer* question that is
true for the wasm browser client and for a host build with that feature on (the fullstack SSG server, `dioxus-desktop`'s
webview). `target_family = "wasm"` is an *execution-target* question and is legitimate only for genuinely wasm-only
internals nested inside a feature-gated hook, never at module or component granularity. `document::eval` is a
cross-renderer Dioxus API that compiles and runs (inertly, without a real document) on every target, so leaf hooks need no
inner target guard. The wiring half: `preview/Cargo.toml`'s `server` feature enables `dioxus-primitives/web`.
`playwright/oracle/hydration-parity.spec.ts` is the black-box oracle for the deployed symptom.

#### Second divergence class: attribute override order

A component that emits an explicit attribute and then spreads caller `attributes` after it
(`div { aria_label: "default", ..attributes }`) diverges between SSR and the hydrated DOM whenever a caller overrides
that name: SSR serializes both into one start tag and WHATWG HTML's duplicate-attribute parse error keeps the **first**
(the component's default), while the CSR path applies attributes sequentially and keeps the **last** (the caller's).
Server and client agree on tree shape and disagree on a computed value, which fails axe's `landmark-unique` where two
landmarks are told apart by a caller-supplied name. Fixed by construction with `merge_attributes` (dedupe, caller wins);
`hydration-parity.spec.ts` Rule 4 is the oracle and `scripts/check-attr-spread-collision.sh` the static guard
([`../issues/duplicate-attribute-root-cause.md`](../issues/duplicate-attribute-root-cause.md)). The one sub-case it cannot
fix is a literal `style: "..."` string colliding with the caller's CSS-shorthand style properties: see
`primitives/src/lib.rs`'s `fold_style_attributes`.

### Caveat 2 — non-browser targets: mostly dissolves

Dioxus *desktop* runs a real webview, so `<dialog>` and `popover=` work there. The non-browser renderer is
`dioxus-native`, built on **Blitz**, checked against `DioxusLabs/blitz@main`:

| Capability | Blitz |
|---|---|
| `showModal` / `show_modal` | **0 occurrences** |
| `popover` attribute support | **0** |
| top-layer API | **0** |
| `inert` attribute | **0** (its 3 hits are about `<template>` contents) |
| `document::eval` | `dioxus-native` implements it as `NoOpDocument.eval` (`packages/dioxus-native/src/contexts.rs`) |
| UA stylesheet | Full Firefox-derived rules: `dialog`, `dialog:not([open]) { display: none }`, `dialog:modal`, `dialog::backdrop`, `[popover]:not(:popover-open)` |

The decisive line is `eval`: it is a no-op on native, so every `document::eval` site in `primitives/src` already does
nothing there, and the library's interactive behaviour is effectively web-and-webview-only. A native-first Dialog is
therefore not worse on Blitz than the alternative, with one design detail: Blitz styles `dialog:not([open])` as
`display: none`, so **bind `open` declaratively as the floor** (a visible, in-flow, non-modal dialog on Blitz) and call
`showModal()` where `eval` works. Never do both blindly: `showModal()` on a dialog already opened via the attribute throws
`InvalidStateError`. Blitz's popover support is an open issue (`DioxusLabs/blitz#196`, March 2025) with no linked work: a
native-first library is aligned with where Blitz is heading, but plan no schedule around it.

---

## Rules per gap

### 1. Form participation — `RadioGroup`, `Select`, `Switch`

Build on this repo's own `Checkbox` pattern with Radix's attribute completeness.

| Decision | Take from | Why |
|---|---|---|
| A real submittable element | Both agree | HTML admits only `button`/`input`/`select`/`textarea` to the entry list; ARIA cannot change this, and `ElementInternals` is unavailable because Dioxus renders plain DOM |
| Hiding technique | This repo (`checkbox.rs`): `aria_hidden`, `tabindex="-1"`, `position:absolute; opacity:0; pointer-events:none` | In-tree, reviewed, consistent |
| Render unconditionally | This repo, over Radix's `closest('form')` gate | The gate needs a DOM query; in Dioxus that is a `document::eval` round-trip and a hydration hazard. Radix itself defaults the gate to `true` when it cannot measure |
| Attribute set: `required`, `disabled`, `form` | Radix | `form` allows a control outside the `<form>` |
| `Select`: hidden native `<select>` with mirrored `<option>`s | Radix's BubbleSelect | A real `<select required>` gets native validation UI, which a hidden `<input>` cannot provide |

**Why upstream diverged from Radix here:** it is a documented framework limitation. Radix's gate is a synchronous
ancestor query (`control.closest('form')`) available the moment a ref attaches; Dioxus's `MountedData` exposes focus,
scroll and rect operations, not ancestor traversal. Reproducing the gate would cost an async, first-paint-racy
`document::eval` per control and is a hydration hazard under SSG. Follow upstream: render unconditionally, which is also
Radix's own behaviour when it cannot measure. The cost is cosmetic (autofill heuristics may notice a stray named input).
Revisit only if Dioxus gains ancestor awareness
([`../issues/drafts/dioxus-issue-nudges.md`](../issues/drafts/dioxus-issue-nudges.md) tracks the framework gap).

### 2. Modal dialogs

Native `<dialog>` + `showModal()`. That subsumes a `focus_scope.rs` and `aria_hidden.rs` port and the vendored
`focus-trap` pair; nested modals become a browser concern.

### 3. Non-modal overlays — menus, popovers, tooltips

`popover="auto"` / `"manual"` for top-layer rendering and light dismiss, with the declarative floor of Caveat 2. It fixes
clipping inside `overflow:hidden` and transformed ancestors, which no CSS workaround can. Two durable rules:

- **An anchored overlay's tracking is unconditional and visual-viewport-aware.** `use_anchor_position_fallback`
  (`primitives/src/top_layer.rs`) installs its `scroll`/`resize` tracking for the entire open lifetime of every anchored
  overlay (not gated on whether the first measurement needed the fallback), and reads every viewport size through
  `window.visualViewport` (`window.inner*` does not shrink for an iOS on-screen keyboard). A snapshot taken at open cannot
  see a keyboard summoned by the very focus that opened the overlay. Oracle: `top-layer.spec.ts` Rule 11.
- **Light dismiss must not see the click that opened the overlay.** `use_dialog_backdrop_dismiss` defers its listener one
  `requestAnimationFrame`, so the opening gesture's click (which iOS can retarget at the freshly shown dialog) never
  reaches it. Oracle: `native-dialog.spec.ts` Rule 8.

### 4. Focus restore for the menu family

Radix's semantics (return focus to the trigger unless the close came from interacting outside) on `use_refocus_on_close_unless`'s
shape, plus a correction from execution: the menus left focus on the item of the closed menu, so focus must also move off
the item.

### 4b. Keyboard open contract (menu-family triggers)

`keyboard-matrix.spec.ts` found the same bug shape independently in `DropdownMenu`, `Menubar` and `Select`: each trigger's
open keys (Enter, Space, ArrowDown, ArrowUp) were separate `match` arms and only some also requested initial focus.
**One "open with focus" path per component**, so opening and requesting focus cannot drift apart:

- `DropdownMenu`: `DropdownMenuContext::open_with_focus(CollectionPlacement)` sets `initial_focus` then `open`, resolved by
  `use_deferred_collection_focus`.
- `Menubar`: `initial_focus` covers Enter and Space as well as the arrows; Space is wired at `MenubarMenu`'s `onkeydown`
  because `MenubarTrigger` wires only `onpointerup`.
- `Select`: `SelectContext::open_with_selected_or_first_focus()` serves Enter and Space (selected option, else first).
  Alt+ArrowDown is the one open key that must not move focus (APG optional): `keep_trigger_focus` suppresses the list's
  own "focus the listbox container" fallback for that path. Plain pointer-click open is untouched on all three.

Each rule is quoted from the pattern's own APG "Keyboard Interaction" section in the matrix oracle, not from memory.

### 4c. Pattern-class role contract (menu-family roles)

`DropdownMenu` once rendered the APG listbox roles while `ContextMenu` and `Menubar` (the same menu-button pattern class)
used the menu roles, because each hand-wrote its own role literals. `primitives/src/menu_semantics.rs` now holds the three
literals (`MENU_ROLE`, `MENU_ITEM_ROLE`, `MENU_TRIGGER_HASPOPUP`) with their APG citations, and all three components read
from it; `oracle/tier1-apg/menu-roles.spec.ts` is calibrated against the vendored APG reference. **When sibling components
already implement one conformance rule correctly, extract what they agree on into one shared definition** and have the
oracle check it is used everywhere.

### 5. Body scroll lock

`scroll_lock.rs` refcounts for nested modals and restores the original `overflow` value, with a guard against the unlock
flash when a second modal opens while the first tears down. Native `<dialog>` does **not** prevent background scrolling, so
the lock is needed regardless of item 2. Construction: a permanent `scrollbar-gutter: stable` baseline plus an
`overflow-y` toggle; the textbook `padding-right` recipe is wrong for `position:fixed` elements and a transient gutter
toggle shifts overlay-scrollbar platforms. No iOS momentum-scroll handling.

### 6. Collision detection

CSS Anchor Positioning with `position-try-fallbacks: flip-block, flip-inline` covers FLIP natively (the three `@supports`
blocks in `top_layer.rs`), with `use_anchor_position_fallback` as the JS fallback where the browser lacks it (iOS 18
Safari at the time). The JS fallback also clamps `target.left` into the viewport for a `side="top"`/`"bottom"` placement
(`top-layer.spec.ts` Rule 12) and publishes the trigger's live width as `--dx-anchor-width` (a themed stylesheet opts in
with `min-width: var(--dx-anchor-width, 100%)`). `ContextMenu` is positioned at click coordinates, so it needs its own
viewport clamp. Open remainder: [`../backlog/010-phase-5-collision-detection.md`](../backlog/010-phase-5-collision-detection.md).

### 7. `use_animated_open`

Catch at the end of the chain and decline to send (so a stale task cannot overwrite newer state), plus a **per-cycle
generation counter** so the rejection path can resolve when it is still the current cycle. Swallowing per-animation
rejections lets a stale close task hide an element after a reopen.

### 8. Typeahead

Prefix matching for the *menu* family only; **leave `select/` alone**, whose Levenshtein, keyboard-layout-aware matcher
with a configurable timeout is better than Radix's.

### 9. RTL

Direction context/provider as in `direction.rs`, plus the concept of `direction_aware_key()` in the existing
`collection.rs` handlers; do not replace the collection system with a roving-focus port. The one mirroring rule is
`resolve_horizontal` in `direction.rs` (gate: `scripts/check-direction-keys.sh`).

### 10. Text-entry font-size floor (iOS Safari touch-zoom)

Every focusable text-entry element (text-like `input`, `textarea`, `select`, `[contenteditable]`) computes
`font-size >= 16px` at a coarse-pointer viewport: WebKit zooms the page when a focused field is smaller (platform
behaviour, not a W3C rule; `playwright/oracle/tier2-html/touch-focus-zoom.spec.ts`). `maximum-scale=1` /
`user-scalable=no` is not the fix: it fails WCAG 1.4.4. Two layers, because a themed component ships without the app's
CSS: each themed component with a text-entry element carries its own `@media (pointer: coarse)` 16px rule in its
`style.css` (found by execution, not by grepping for `input`: Combobox's input, Calendar's invisible native
`<select>`s), and `preview/assets/main.css` has one catch-all for raw elements. Exclusion: Select's hidden native
`<select>` (`tabindex="-1"`, unreachable by tab or tap).

### 11. Main-thread responsiveness (interaction latency)

**Reference:** kciter, *"The Browser's Main Thread Is Expensive"* (<https://kciter.so/posts/the-expensive-main-thread/en/>).
Everything except compositing shares one thread; a 60 Hz frame is ~16.6 ms of which ~10 ms is realistically yours; a task
over 50 ms is a long task and shows in INP and TBT. A library never owns the page's budget, so a component must be cheap
per interaction and never hold the thread across a gesture.

**Rules for new work:**

1. **Rule 1:** a handler must not do synchronous layout reads *and* writes (layout thrash): measure once, then write, or
   defer the write to the next `requestAnimationFrame`.
2. Any listener on `scroll`, `wheel`, `touchstart` or `touchmove` is `passive: true` unless it genuinely calls
   `preventDefault()`, and then the comment says why (the body scroll lock and `ContextMenu`'s scroll suppression are the
   deliberate exceptions).
3. Animations in a themed `style.css` use `opacity`/`transform`; a layout-property animation (Accordion's
   `grid-template-rows`) is bounded to 300 ms and gets a frame sampler (`accordion-animation.spec.ts`).
4. Nothing in a primitive busy-waits: use the settle-loop shape (re-measure across successive `requestAnimationFrame`s).
5. Work that cannot be split (a large parse) is not the library's business: expose it as an async hook.

**Where the code applies them:** `use_anchor_position_fallback` coalesces passive `scroll`/`resize`/`visualViewport`
listeners into one `requestAnimationFrame` re-measure per frame (`top-layer.spec.ts` Rules 11-12);
`use_dialog_backdrop_dismiss` attaches one frame after open (`native-dialog.spec.ts` Rule 8); anchor positioning goes
through the `@supports (anchor-name: --a)` CSS block where the browser has it and runs the JS fallback only where needed;
`touch-action: manipulation` removes the double-tap wait (`touch-double-tap.spec.ts`). The enforcing oracle is
`oracle/tier2-html/main-thread.spec.ts` (`PerformanceObserver` long tasks and event timing; backlog row 45).

### Global stylesheet rule: never `@import` a remote stylesheet from the app stylesheet

A stylesheet applies only once every `@import` in it has loaded, so a stalled font request silently disabled all of
`preview/assets/main.css`. Web fonts load from `<link rel="preconnect">` + `<link rel="stylesheet">` in one shared head
component (`GlobalHead`), the app stylesheet has no `@import`, and `oracle/tier2-html/global-stylesheet.spec.ts` asserts on
every route that `main.css` is in the applied set.
