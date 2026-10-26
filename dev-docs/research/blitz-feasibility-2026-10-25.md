# Blitz / dioxus-native as a renderer for shadcn-dioxus: feasibility (2026-10-25)

Question (owner): "What about using Blitz as a native renderer? I don't think it has any JS."

**Short answer.** Right on the premise: Blitz runs no JavaScript, and `document::eval` on dioxus-native is a defined no-op that returns `Err(EvalError::Unsupported)`; it neither panics nor runs anything. But "no JS" is the problem, not the solution. This library's overlays, positioning, focus handling and measurement all go through `eval`. Blitz has none of the Rust-side replacements for the top layer (`popover`, `showModal`), `:focus-visible`, `:has()`, pointer capture, resize or animation events, or `contenteditable`. **Not a realistic supported target now.** It is a realistic "later" target for a reduced, non-overlay subset, once three upstream facts change (see section 4).

## Method and evidence labels

- **[S]** = read in published source: the crate tarballs from crates.io, downloaded 2026-10-25 and grepped locally (not built, not run). These are the strongest claims here.
- **[W]** = read on a web page (fetched 2026-10-25). Pages mostly carry no date of their own; the fetch date is given.
- **[R]** = read in this repo (`/home/claude/shadcn-dioxus`, read-only grep, 2026-10-25).
- **[I]** = inferred by me, not verified by running anything. Nothing in this document was executed on Blitz.
- Limits: `github.com/DioxusLabs/blitz` could not be cloned or called through `gh` from this sandbox (403). `blitz.is` is reachable only through WebFetch. WebSearch returned nothing useful for Blitz-specific queries. Open-issue state is therefore thin; the one issue page I could read returned only its title and body (the state/date metadata was not extracted).

## Source index (each cited by id below)

| id | URL | date read / published |
|---|---|---|
| C1 | https://crates.io/api/v1/crates/dioxus-native | versions list read 2026-10-25; 0.8.0-alpha.3 published 2026-10-16 |
| C2 | https://crates.io/api/v1/crates/dioxus-native/0.8.0-alpha.3/download (source tarball) | published 2026-10-16 |
| C3 | https://crates.io/api/v1/crates/dioxus-native-dom/0.8.0-alpha.3/download | published 2026-10-16 |
| C4 | https://crates.io/api/v1/crates/blitz-dom/0.3.0-beta.4/download (also `blitz-shell`, `blitz-paint`, `blitz-traits` at the same version) | published 2026-10-16 |
| C5 | https://crates.io/api/v1/crates/stylo/0.22.0/download and `.../stylo_static_prefs/0.22.0/download` | stylo 0.22.0 published 2026-09-30 (C9) |
| C6 | https://crates.io/api/v1/crates/dioxus-native/0.7.10/download and `.../dioxus-native-dom/0.7.10/download`, `.../blitz-dom/0.2.4/download` | 0.7.10 published 2026-07-30; blitz-dom 0.2.4 published 2025-10-22 |
| C7 | https://crates.io/api/v1/crates/blitz-dom (version list) | read 2026-10-25 |
| C8 | https://crates.io/api/v1/crates/dioxus-native/0.7.10/dependencies | read 2026-10-25 |
| C9 | https://crates.io/api/v1/crates/stylo, `.../anyrender_vello`, `.../blitz-dom` crate summaries | read 2026-10-25 |
| G1 | https://github.com/DioxusLabs/blitz (README) | fetched 2026-10-25 |
| G2 | https://github.com/DioxusLabs/blitz/issues/119 (roadmap thread) | fetched 2026-10-25; no date shown |
| G3 | https://github.com/DioxusLabs/blitz/issues/196 ("Implement the popover API") | fetched 2026-10-25; opened March 2025 per the earlier in-repo note (R1) |
| G4 | https://github.com/DioxusLabs/dioxus/releases | fetched 2026-10-25 |
| G5 | https://newreleases.io/project/github/DioxusLabs/dioxus/release/v0.8.0-alpha.3 | fetched 2026-10-25; "7 days ago", so about 2026-10-16 |
| B1 | https://dioxuslabs.com/blog/release-070 | post dated 2025-09-08 |
| B2 | https://blitz.is and https://blitz.is/status (CSS support matrix) | fetched 2026-10-25; no date on page |
| D1 | https://raw.githubusercontent.com/DioxusLabs/dioxus/main/packages/document/src/document.rs (`NoOpDocument`) | fetched 2026-10-25 (main branch) |
| R1 | `/home/claude/shadcn-dioxus/dev-docs/recommended-implementations.md` Caveat 2 | repo, read 2026-10-25 |
| R2 | `/home/claude/shadcn-dioxus/primitives/src/top_layer.rs` ("Renderer-support note", about line 800) | repo |
| R3 | `/home/claude/shadcn-dioxus/primitives/src/message_scroller.rs` ("Renderer arms", line 117) | repo |
| R4 | `/home/claude/shadcn-dioxus/dev-docs/backlog/023-*.md`, `dev-docs/fork-network-research.md` | repo |

---

## 1. Does it run JavaScript? What does `document::eval` do?

- **No JavaScript engine.** The README says Blitz "can already render many popular no-JS websites" and "We don't yet have Blitz bindings for other languages (JavaScript, Python, etc)" [G1, 2026-10-25]. The roadmap thread says JS is "not a 2025 priority", that it "might be on the cards", and that the architecture leaves room for an engine to slot in [G2, 2026-10-25]. The 0.7 blog post says "Pages that depend on JavaScript ... may render poorly" [B1, 2025-09-08]. I grepped `blitz-dom`, `blitz-shell`, `dioxus-native` and `dioxus-native-dom` for quickjs, boa, v8 and deno: nothing [S, C2-C4].
- **`document::eval` on dioxus-native is a no-op returning an error.**
  - `DioxusNativeDocument::eval` is `NoOpDocument.eval(_js)` [S, C2 `src/contexts.rs:19-20`; the same line exists in 0.7.10, C6].
  - `NoOpDocument`'s evaluator returns `Poll::Ready(Err(EvalError::Unsupported))` from both `poll_join` and `poll_recv`, and `send()` returns `Err(EvalError::Unsupported)` [W, D1, 2026-10-25, main branch].
  - So it is **unsupported-with-error, not panic, not silent success.**
  - Consequence [I]: any `eval(...).await.unwrap()`, or an `.recv()` loop that waits for a reply, either fails immediately or spins on an error. Each of our 99 `eval(` call sites (54 files, [R]) would need an audit for how it treats `Err`. Our current "native arm" avoids this by compiling the `eval` away under `not(feature = "web")` (46 `cfg(not(feature = "web"))` sites in 18 primitives files [R]).
- **Dioxus handlers that can panic on dioxus-native.** `dioxus-native-dom`'s event converter has `unimplemented!()` bodies for: cancel, animation, before-input, clipboard, composition, drag, image, media, selection, toggle, transition, resize, visible [S, C3 `src/events.rs:41-141`].
  - In 0.8.0-alpha.3 `convert_mounted_data` is implemented. In stable 0.7.10 it is `unimplemented!("... requires support in blitz")` [S, C6 `dioxus-native-dom/src/events.rs:68`].
  - These can only panic if the event is dispatched. Blitz's event enum has no variant for toggle, animation, transition, resize or visible [S, C4 `blitz-traits/src/events.rs:215-260`], so I infer they simply never fire. That is quieter but worse for us: `onresize` is used in `slider.rs`, `virtual_grid.rs` and `table_rows.rs` [R], and on native it would never deliver [I].

## 2. Feature support relevant to this library

Our CSS/JS reality [R, counts from grep on 2026-10-25]: `:has(` x100 in 30 component sheets; `:focus-visible` x110 in 50 sheets; `color-mix` in 33 of 83 sheets; `oklch` throughout the theme; `@starting-style` in the theme and 3 sheets; `hover: hover` in 46 sheets; `@keyframes` x17 in the theme plus 15 sheets; `scrollbar-gutter: stable`; `:popover-open`/`:modal` x5; real `@container` rules: none (only comments); no `light-dark()`.

| Feature | Blitz status | Evidence | Effect on us |
|---|---|---|---|
| `popover` attribute / top layer | **Not implemented.** `:popover-open` always false; the UA sheet hides `[popover]:not(:popover-open)`, so an element with `popover` is permanently invisible. Issue #196 "Implement the popover API" is open ("an excellent fit for Blitz"). | [S, C4 `blitz-dom/src/stylo.rs:499` and `assets/default.css:38`]; [W, G3]; [R, R2] | Overlays need a different arm. Already handled today by never emitting the attribute on native [R2]. |
| `<dialog>` + `showModal()` | **No API.** UA CSS has `dialog:modal`, `-moz-top-layer`, `::backdrop`, but `:modal` is hard-coded false and there is no `show_modal` in any crate. `dialog[open]` renders as in-flow absolute. | [S, C4 `stylo.rs` (`Modal => false`), `default.css:956-985`; grep for `show_modal`/`showModal`: 0 hits in 4 crates] | Modal arm = `open` attribute plus own backdrop and focus handling. Matches the "declarative `open` floor" in R1. |
| `position: fixed` | Supported in layout. | [W, G1 "absolute/fixed positioning"]; [S, C4 `node/node.rs:1296`] | Enough to build an in-flow "portal at root" overlay. |
| `position: sticky` | **In progress** (blitz#887). | [W, B2 /status] | Two component sheets use it. |
| Overflow scrolling, scroll APIs from Rust | Wheel and drag scrolling; `MountedData::scroll`, `scroll_to` (scroll-into-view), `get_scroll_offset`, `get_scroll_size` are implemented in 0.8.0-alpha.3. `Scroll` DOM event exists. No `scrollend`. | [S, C3 `events.rs:200-300`; C4 `scrolling.rs`; `blitz-traits/events.rs` has `Scroll`, no scroll-end] | `scrollend`: no equivalent. |
| Focus management | `MountedData::set_focus(true/false)` implemented (note in source: "TODO: queue focus events"); `focus_next_node`/`focus_prev_node`; Tab handling in `events/keyboard.rs`. No `inert`. | [S, C3 `events.rs:299-312`; C4 `document.rs:1671-1720`] | A Rust focus trap is possible with `onkeydown` plus `set_focus` [I]. Background cannot be made inert. |
| `:hover`, `:focus` | Supported via element state. | [S, C4 `stylo.rs`: `Hover`, `Focus` arms] | Works. |
| **`:focus-visible`, `:focus-within`** | **Hard-coded `false`.** | [S, C4 `blitz-dom/src/stylo.rs`: `FocusWithin => false`, `FocusVisible => false`; also false in blitz-dom 0.2.4, C6] | **110 `:focus-visible` uses in 50 sheets would never match. Our focus-ring tokens (box-shadow on `:focus-visible`) would not paint on native** [I: the rule shape is from the project docs, not re-read for every sheet]. |
| **`:has()`** | **Disabled.** Stylo's `layout.css.has-selector.enabled` defaults to `false` and Blitz's pref list does not turn it on. | [S, C5 `stylo_static_prefs/preferences.toml`; C4 `document.rs:428-439` lists the prefs Blitz sets: grid, flexbox.balance, unimplemented, columns, basic-shape-shape, tree-counting, progress-function, variable_fonts, writing-mode (feature-gated), threads] | 100 uses in 30 sheets. Rules containing `:has()` would fail to parse and be dropped [I], e.g. Alert's icon layout, Alert Dialog, Attachment. |
| `@container` | Pref `layout.container-queries.enabled` false; blitz.is lists `container-type`/`container` as unsupported. | [S, C5]; [W, B2] | We have no live `@container` rules, so no impact. |
| `@starting-style` | Pref `layout.css.starting-style-at-rules.enabled` false. | [S, C5] | Entrance fades would not play [I]. Cosmetic, since the elements would still appear. |
| Anchor positioning | Not supported (all `anchor-name`, `position-anchor`, `position-area`, `position-try` rows are unsupported). Pref also false. | [W, B2]; [S, C5] | We position by JS today, so this is moot. We would need Rust positioning from `get_client_rect`. |
| `color-mix()`, oklch | Stylo parses both; I found no pref gating them (only `color-mix-multi-color`, which is true). blitz.is does not list them. Whether the painted colour matches Chrome is **not verified**. | [S, C5 prefs]; [W, B2 silent] | Probably works [I]. Needs a render check. |
| `@property` | Pref true. | [S, C5] | Probably works [I]. |
| CSS transitions and `@keyframes` animations | "Animations" and "Transitions" rows are supported; the animation set is wired into the style resolve loop. No `transitionend`/`animationend` DOM events. | [W, B2]; [S, C4 `stylo.rs:99-175`, `resolve.rs`; no such events in `blitz-traits`] | CSS animation works. Our `getAnimations()`-based "wait for exit animation" has no equivalent; the exit state machine must be Rust timers [I]. |
| `backdrop-filter` | Partial, Skia backend only. | [W, B2] | Cosmetic, a handful of sheets. |
| `scrollbar-gutter` | Not found in layout code beyond an overflow-based gutter; not verified. | [S, C4 `layout/inline.rs:274`] | Unverified. |
| `@media (hover: hover)`, `(pointer: fine)` | **Evaluate as true on desktop.** Blitz's device passes `PointerCapabilities::default()`, which in Stylo is FINE+HOVER on non-mobile and COARSE only on iOS, Android and OHOS. | [S, C4 `stylo_device.rs:84-85`; C5 `servo/media_features.rs`, `impl Default for PointerCapabilities`] | Our `hover-media` wrapping works as intended on desktop (I first expected the opposite; corrected by reading the source). |
| `prefers-color-scheme` | Supported; the shell passes a Light/Dark `ColorScheme` into the Stylo device. | [S, C4 `stylo_device.rs:76-83`] | Works. Our theme `apply_mode` JS (`<html data-theme>`, `.dark` class) cannot run, so the mode must be set from Rust [I]. |
| `contenteditable` | **No hits** in any of the four Blitz crates. Text input exists only for `<input>`/`<textarea>` (`TextInputData`). IME events for those exist; the roadmap lists composition events and the EditContext API as not done. | [S, grep; C4 `node/text.rs`, `events/ime.rs`]; [W, G2] | DatePicker's segments (`contenteditable`) cannot work. They would need to become `<input>`s or buttons [I]. |
| Pointer capture | **No API.** No capture code in Blitz pointer handling; no `setPointerCapture`. | [S, C4 `events/pointer.rs` grep: only an unrelated scrollbar "capture" comment] | Slider, Resizable, color-picker drags would need an alternative, e.g. window-level move listeners via `use_window_event` [I]. |
| Resize/visibility observation | No resize, intersection or mutation events. | [S, C4 `blitz-traits/events.rs`, C3 `events.rs:136-141`] | `onresize`, ResizeObserver, IntersectionObserver users (virtualization, slider, scroll-area) lose their input [I]. Rect polling through `get_client_rect` is the only substitute. |
| SVG (charts) | Behind the default `svg` feature. An inline `<svg>` is serialised to a string and rasterised by usvg as an image. | [S, C4 `layout/construct.rs:460-476`, `Cargo.toml` `svg` feature] | Static charts render; per-element hover/focus/events inside the SVG and page CSS variables will not work [I]. Our pure-RSX chart engine would draw but not be interactive. |
| Accessibility | AccessKit integration, a default feature. Role mapping, `aria-labelledby/-describedby/-expanded/-hidden` handled in `accessibility.rs`. | [S, C4 `accessibility.rs`; C2 features `accessibility` in `default`] | Present, but partial. Coverage of our ARIA (e.g. `aria-activedescendant`, `aria-modal`, live regions) is not verified [I]. |
| `MountedData` (rect, scroll, focus) | In **0.8.0-alpha.3**: `get_client_rect`, `get_scroll_offset`, `get_scroll_size`, `scroll`, `scroll_to`, `set_focus` implemented. In **stable 0.7.10 / blitz-dom 0.2.4**: `convert_mounted_data` is `unimplemented!()`. The 0.8 alpha.3 changelog fixes RefCell double-borrow panics when these are called during render. | [S, C3 vs C6]; [W, G5] | Rust-first measurement exists only on the Dioxus 0.8 line. |
| Clipboard, file dialogs | Cargo features `clipboard`, `file-dialog` exist (default). No Dioxus-level API found. | [S, C2 `Cargo.toml`] | Our `navigator.clipboard` uses (3 files) have no Dioxus path [I]. |
| Local storage, web APIs | Explicitly out of scope: "we *don't* provide features like webrtc, websockets, bluetooth, localstorage, etc." | [W, G1] | Anything using `localStorage` (3 files) needs a Rust store. |

## 3. Maturity

- **Version and status.** `blitz-dom`, `blitz-shell`, `blitz-paint`, `blitz-html` and `blitz` are at **0.3.0-beta.4** (published 2026-10-16); the previous stable line is 0.2.4 (2025-10-22) [S/W, C7, C9]. The README says "Blitz is currently in a **beta** state ... still many bugs and missing features ... actively working on bringing it up to production quality" [W, G1, 2026-10-25]. The 0.7 blog called Blitz "a work in progress" and "very young" [W, B1, 2025-09-08].
- **dioxus-native.** Latest is **0.8.0-alpha.3** (2026-10-16, the same day as blitz beta.4), whose notes say "Blitz moved to 0.3.0-beta.4" [W, G5; C1]. Latest *stable* is 0.7.10 (2026-07-30), built on blitz 0.2.x (`blitz-dom ^0.2.4`) [C1, C8]. About 45k total downloads for dioxus-native versus about 300k for blitz-dom [C1, C9]; download counts are weak evidence of use.
- **Cadence.** Blitz 0.3 alphas/betas ran from 2026-04-23 through 2026-10-16: eleven pre-releases in six months [C7]. Dioxus 0.8 alphas: 2026-05-19 (alpha.0), 2026-07-31 (alpha.1), 2026-10-14 (alpha.2), 2026-10-16 (alpha.3) [C1]. 0.8 stable is not out.
- **Pairing.** dioxus-native 0.7.x pairs with Dioxus 0.7.x and blitz 0.2.x. Dioxus 0.8.0-alpha.x pairs with blitz 0.3.0-beta. **This repo is on Dioxus 0.7.8** [R, `Cargo.toml:21`].
- **Stack.** Stylo 0.22.0 (2026-09-30), Taffy, Parley, anyrender_vello 0.15.0 (2026-10-03), winit, AccessKit [C9, G1]. Backends: Vello GPU/hybrid/CPU, optional Skia [S, C2 features].
- **Production users.** I found none documented. The 0.7 blog shows sample pages, a Wikipedia search form and dioxus-motion demos only [B1]. A third-party editor project in this repo's research notes (`leaf-dioxus`) says it uses Dioxus Native, which is an unverified claim from a 2-star project [R, `dev-docs/research/text-editor-options-2026-10-21.md` row 8]. Treat production use as unproven.

## 4. Verdict for this library

### 4.1 Realistic target now or later?

- **Now: no.** Three independent blockers, each upstream and outside our control:
  1. **No top layer.** There is no popover, no `showModal`, no `inert`, and no anchor positioning. 28 of our 54 eval files touch `showPopover`/`showModal` [R]. Issue #196 is open (opened March 2025; I could not read its latest state) [G3, R1].
  2. **No `:focus-visible`, no `:has()`.** Both are hard-coded or pref-disabled. Our focus ring and about a third of component layouts would degrade silently [S, C4, C5]. Fixing these means upstream Blitz or Stylo-pref changes (not assessed for cost here).
  3. **MountedData and the Dioxus pairing.** The measurement API we would use in place of `getBoundingClientRect` exists only in Dioxus 0.8 alpha. We are on 0.7.8, and 0.8 is not stable [C3, C6, C1].
- **Later: plausible, in tiers.** A non-overlay subset is achievable once Dioxus 0.8 is stable and a library baseline passes its own gates on it. Overlays should wait for upstream, or for a decision to accept a Rust-only portal arm.

### 4.2 What supporting it would require, and which eval uses have no Blitz equivalent

Rust-first means every `eval` is replaced by Dioxus/Blitz Rust APIs or a CSS-only construction.

**Replaceable in Rust on Dioxus 0.8 + Blitz 0.3** (mapping inferred from the API list in section 2 [I]):
- Layout reads (`getBoundingClientRect`): `MountedData::get_client_rect`.
- Focus and scroll: `set_focus`, `scroll`, `scroll_to`.
- Window events and the back button: `use_window_event`, `use_back_button` [S, C2 `src/hooks.rs`].
- Colour-scheme mode: set from Rust.
- One-frame measure-and-place positioning: only as read-after-mount with a second render, so one frame of flicker unless layout can be hidden until placed.
- Escape/outside-click dismissal: `onkeydown` plus a full-screen backdrop `div`.
- Focus trap: Tab handling in Rust (Blitz has `focus_next_node` but no Dioxus-facing hook, so it is hand-rolled).

**No Blitz equivalent at all** (largest eval-using areas first; counts are files in `primitives/src` + `preview/src` on 2026-10-25 [R]; the ResizeObserver/IntersectionObserver/MutationObserver/visualViewport/clipboard/localStorage/elementFromPoint counts come from a plain substring grep and are approximate):
1. **Top layer** (`showPopover`/`showModal`/light-dismiss), 28 files. The substitute is a root-level `position: fixed` portal with manual z-order, losing page `inert`, native `Esc`, `::backdrop` and stacking over `overflow` clips.
2. **Pointer capture** (`setPointerCapture`), 3 files: slider, resizable, drag surfaces [I].
3. **`ResizeObserver`/`onresize`** (14 files), **`IntersectionObserver`** (8), **`MutationObserver`** (8). There are no events at all; polling rects each frame is the only route [I].
4. **`getAnimations()`/animation-end waits**, 6 files: no Web Animations API and no animation or transition DOM events.
5. **`scrollend`**, 1 file: no scroll-end event.
6. **`matchMedia`**, 8 files: no API. CSS `@media` itself works, and the shell has a colour-scheme value, so some uses can become CSS and the rest need a Rust signal fed from `use_window_event`.
7. **`contenteditable`** (DatePicker segments): unsupported, and the EditContext API is unstarted [G2].
8. **`visualViewport`** (6 files, mobile keyboard inset): no equivalent on desktop; unknown on Android/iOS Blitz.
9. **Clipboard (3), `localStorage` (3)**: no Dioxus-level API found; need Rust crates.
10. **`elementFromPoint`** (5 files): hit-testing is internal to Blitz and I found no public API [I].
11. **Anything CSS-gated:** `:has()`, `:focus-visible`, `@starting-style` (see section 2).

### 4.3 Effort sizes (all inferred [I])

| Tier | Scope | Size |
|---|---|---|
| T0 | Compile the primitives for dioxus-native on the Dioxus 0.8 line (we are on 0.7.8: a migration that is itself a row), run the existing "native arm", add a visual smoke harness. | M, mostly the 0.8 migration, which would be needed anyway |
| T1 | Static/in-flow components (Button, Card, Badge, Tabs, Accordion, Toggle, Separator, Progress, Alert, Table) with the CSS fixes: a `:focus` fallback for the ring, `:has()` rules rewritten into slot variables or data attributes. | M-L: roughly 30 sheets for `:has`, 50 for focus, and a second visual baseline |
| T2 | Interactive non-overlay (Slider, Switch, Checkbox, Radio, Select-in-flow, Tabs keyboard, VirtualList). Pointer capture, resize and `matchMedia` replacements. | L |
| T3 | Overlays (Dialog, Popover, Dropdown, Tooltip, Select, Combobox, Command, Toast, Navigation Menu, Date Picker), Rust portal + positioning + focus trap + dismissal. | XL, with a ceiling set by upstream #196 and `inert` |
| T4 | Charts with interaction, DatePicker segments. | Blocked (contenteditable, SVG events) |

**Testing cost is the hidden size.** The entire Playwright oracle and the 30-odd gates are browser-only; there is no test harness for Blitz [R]. A native lane needs a new one: headless Vello-CPU screenshots at best, with no DOM-query oracle. I would count that as at least a third of T1-T3.

### 4.4 Should it be a separate backlog row, and with what gating criteria?

**Yes, one row, status "parked / research-gated", not scheduled.** Suggested title:

> **dioxus-native/Blitz as a supported renderer** (after row 434; depends on a Dioxus 0.8 migration row)

Gate to *start*, all of:
1. Dioxus 0.8 is **stable** and our primitives build on it (Blitz's `MountedData` rect, scroll and focus APIs only exist there).
2. Blitz (stable or beta, stated by upstream) implements or accepts: `:focus-visible` (not hard-coded false), `:has()` (the pref on by default), and either the popover attribute or a documented replacement for the top layer. Re-check #196, blitz.is/status and `blitz-dom/src/stylo.rs`'s `match_non_ts_pseudo_class`.
3. A written decision on test strategy for a renderer with no DOM oracle (screenshot comparison on Vello CPU, or accept manual QA).

Gate to *claim support*, per tier:
- **T1:** every shipped component's static render matches the web reference within a stated tolerance, focus is visible by keyboard, and a CI job builds the starter on `dioxus-native`.
- **T2/T3:** each component's interaction passes a native-specific spec, or is marked "web only" in the component's docs and the registry manifest, with a build-time check that the page does not use a web-only component under the native feature.

Cheap actions that need no new row:
- Keep the existing `feature = "web"` arm as the declared native behaviour (inert, no overlays).
- Add a note to the docs stating that dioxus-native is unsupported today and why (the three blockers above).
- Re-run this check when Dioxus 0.8 reaches stable.

## 5. Caveats and open questions

- Nothing was run on Blitz. The CSS and event findings come from reading published source at blitz-dom 0.3.0-beta.4 and 0.2.4, and from blitz.is, which could not be checked for freshness.
- I could not read the current state of issues #196 and #887 beyond their titles and bodies, because the GitHub metadata was not exposed to my fetcher. They may have moved.
- `color-mix`, oklch, `scrollbar-gutter`, inline-SVG fidelity and AccessKit coverage of our ARIA are unverified; each needs a render or AT test, not a source read.
- The size estimates are mine and unmeasured. Treat them as ordering, not budget.
- **My own early assumption was wrong on one point:** I expected `@media (hover: hover)` to be false on Blitz. The source shows it is true on desktop, so that gate is fine.
- Row 434 itself was not found in `dev-docs/backlog/` or `STATUS.md` by grep, so "after row 434" is the owner's wording, not something I verified.
