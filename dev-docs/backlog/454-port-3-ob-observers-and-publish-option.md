---
id: 454
title: "Port 3.OB: observers and the publish option (L27, S16, S14)"
status: "LANDED in PR #180 (2026-11-13): L27, S16, S14 are Rust over the bridge. The main-thread proof (0…"
size: "S"
area: primitives
js-grant: js_file_bytes=2400, bytes=4900 (measured on lane/port-ob-454: js/bridge.js +2,358 bytes; ts/bridge.ts +4,340 and types/bridge.d.ts +518 bytes, for the publish option and the rows L27, S16 and S14 first need; the JavaScript total falls by about 13,000; the row that first needs a verb adds it, root doc section 5.2)
adds: []
updated: 2026-11-12
---

# Row 454: Port 3.OB: observers and the publish option (L27, S16, S14)

## Item

Port L27 (`keyboard_inset.rs`), S16 (`chart/components/chart.rs`) and S14 (`scroll_area.rs`, as an `observe(..., publish: var)` option of the bridge; the depth stays CSS where scroll-driven animations exist) to bridge observers.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

LANDED in PR #180 (2026-11-13): L27, S16, S14 are Rust over the bridge. The `main-thread` proof (0 long tasks) is a `@timing` case, deactivated by the owner's decision of 2026-11-02, so it did not run in the batch's release suite; [row 494](./494-re-enable-the-timing-cases-after-the-rust-port.md) re-measures it with the other timing cases. Open here: the `dev-docs/js-interop.md` rows listed under "Open". Approved 2026-10-26 (scope 2).

## Size

S

## Detail

- **Files owned:** `keyboard_inset.rs`, `chart/components/chart.rs`, `scroll_area.rs`. Must not touch `activity.rs`; `motion` gate semantics.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (grant rule) and [row 445](./done/445-phase-0-2-the-legacy-freeze.md) (legacy freeze); a candidate row also needs its own grant row, and an `adds:` row where it adds a behaviour.
- **Proof:** Chart, scroll-area and keyboard-inset specs on a release build; `main-thread` stays at 0 long tasks.
- **Hours (inferred):** 7 (inferred); removes about 144 lines (21 per hour).
- **Decision in Rust:** the logic this row moves is a pure Rust function over typed inputs; only reading and writing the page goes through the bridge ([`decisions.md`](../decisions.md), "Behaviour is decided in Rust").

## Built (lane/port-ob-454, 2026-11-12)

**What the three scripts were.**
- L27 (`keyboard_inset.rs`, 98 lines): listened to `visualViewport` resize and scroll, window resize and document `focusin`; decided keyboard-open (`innerHeight - vv.height > 60`, `scale == 1`); wrote `--dx-keyboard-inset`, `--dx-keyboard-top`, `data-dx-keyboard` on the panel; on a field focus or a height change it (next frame) added scroll room (`padding-bottom`, `overflow-anchor`) to the nearest vertical scroller and `scrollBy` to centre the field. Deadline: none (a late inset is harmless).
- S16 (`chart.rs`, 13 lines): one read at mount of tab visibility, the chart's rect, `checkVisibility` and the window size. Deadline: none, one read.
- S14 (`scroll_area.rs`, 31 lines): a `ResizeObserver` (scroller and first child) measuring the scrollbar gutters into `--dx-scrollbar-inline/block-size`; and, without scroll-driven animations only, a per-frame scroll listener writing the four edge depths.

**Construction.**
- L27: one `Session` of four `listen`s (visual viewport `resize` with `initial`, `scroll`, window `resize`, document `focusin` capture), every message carrying `viewport.{innerHeight,height,offsetTop,scale}` (new field rows read the viewport whatever the event target is). Pure Rust: `keyboard_of`, `needs_reveal`, `slack_step`, `scroll_delta` (unit-tested). Writes: `set_var`, the new `write` verb (`data-dx-keyboard`, a slack marker, inline `padding-bottom`, `overflow-anchor`, closed `Slot` table), `scroll_by_top`. The scroller is addressed by the new `Target::Scroller { within }` (nearest vertical scroller of the active element inside the panel); the text-field test is `Prop::Matches(TEXT_FIELD)`.
- S16: one `Transaction` of three `get`s (`VisibilityState`, `Rect` + `CheckVisibility`, `InnerWidth/Height`), the pure `painted_in_view`.
- S14: gutters are Rust (`scrollbar_gutter`, `scroll_range`) over one `get` per resize observation, written only when a gutter changed. The depth where scroll-driven animations are missing is CSS: the `@supports not (animation-timeline: scroll())` block of `dx-effects.css` computes the four depths from `--dx-scroll-top/-left` (copied by the page) and `--dx-scroll-range-y/-x` (written with the gutters); threshold = a `* 1000` ramp over the first pixel. `driven` is one `Prop::ScrollTimelineSupported` read (closed row: `CSS.supports("animation-timeline: scroll()")`); the inline reads are the closed `InlinePaddingBottom` and `InlineOverflowAnchor` rows. The CSS path where the animations exist is untouched.
- **`publish` option** (`listen` option only; the observer form was removed, no user): `options.publish: [{ var: "--dx-...", field, unit? }]`. At the moment the listener flushes (hot events: the frame's last event), and once at install with `initial`, the page copies `field` (an event field row) as a number plus `unit` into `var` on the observed element via `set` (`--dx-` only). A listener that publishes and names no field sends nothing to Rust. No condition, no component state: transport. Rust: `Publish` in `ListenOptions.publish`. S14 uses it (scroll is an event stream; the bridge has no scroll observer). `--dx-scroll-top/-left` have one writer: the listener seeds them at install and publishes every scroll; Rust writes only the ranges (`range_vars`, unit-tested).

**JavaScript.** Removed 142 lines of script text in Rust strings (`js_lit` 128 counted lines, 2 `install` sites, 1 `eval` sink and its `expect`). Added to the bridge: `bridge.ts` +77 net lines (86 added), `bridge.d.ts` +11, `js/bridge.js` +2,358 bytes (19,956 to 22,314). Total volume falls (`js_lit*120 + bytes`: 508,767 to 495,765 by the gate's own formula). The per-file raise of `bridge.js` (+2,358 bytes) and of `check-bridge-behaviours`' `bridge.ts` bytes (42,732 to 47,072) and `bridge.d.ts` (6,111 to 6,629) need a grant row; `js-grant: none` does not cover them.

**Proof.** `cargo test -p dioxus-primitives` (1,106+ lib tests; `state_attrs::ssr_tests::dialog_renders_open_as_presence` fails on the base too), `bun test primitives/ts-test` 75 pass, clippy clean, `keyboard-inset.spec` (35), `scroll-fade-scrollbar`, new `scroll-fade-fallback.spec.ts` (forces the fallback in Chromium: `CSS.supports` false plus the fallback rules copied out of the sheet) green on the built tree. Not run on the base build: the new fallback spec.

**Renderers.** Web: wasm, messages microtask-late. Webview: L27 at most one message per frame per listener while the viewport changes (about 60-180 per second during a keyboard animation or pan, 0 otherwise; Rust writes only on change), plus 3 round trips per reveal; S14 gutters 1 get per resize frame (2 messages) and 0 per scroll frame (publish is in the page); S16 one round trip at mount. Blitz: all of it is a no-op (`get`/`Session` unsupported); a Blitz arm needs `MountedData` rects and a platform keyboard-inset source for L27 (on Blitz there is no soft keyboard through a page), `get_client_rect`/resize events for the S14 gutters (no scroll-driven CSS depth without a page either), and a layout query for S16.

**Resolver reasoning.** `Target::Scroller` (`scrollerOf` in `bridge.ts`) stays a resolver: which element scrolls is a fact about the rendered page that the adapter reads (computed `overflow-y` of the ancestors), not a component decision; Rust takes the element and decides the rest. On Blitz the renderer's own scroll-container lookup takes its place. Vectors: nearest `auto`/`scroll`/`overlay` ancestor inside `within`, fallback to the root, a hidden-overflow ancestor skipped. `Slot::KeyboardOpen` (`data-dx-keyboard`) stays a `write` row: the panel element is rendered by the primitive `DialogContent`, which does not know the keyboard state held by the child `KeyboardInset`, so only the styled components could carry it as an rsx attribute and they are not this row's to edit.

**Findings (behaviour differences).** (1) The field test is a selector (`TEXT_FIELD`), not `isContentEditable`: a contenteditable inherited through `contenteditable=inherit` or an invalid value is not matched; a `contenteditable=false` island inside an editable host is excluded. (2) The reveal reads one frame or more later (messages, then three transactions) where the script ran in the next animation frame. (3) `focusin` sends one message for every focus move in the document while a panel is open (a mount-time cost of one message per move, filtered in Rust). The edge depths and the gutters settle after one resize message, one `get` round trip and the writes (about three hops) where the old script wrote in the same frame: on the web that is a microtask each, so a user sees nothing; on a webview the mask edge of a resize drag may trail by a frame (inferred, not measured). (4) The slack marker `data-dx-keyboard-slack` is on the scroller while room is held. (5) The fallback depth ramps over `1px .. 1px + depth/1000` instead of a step at 1px.

**Open.** `dev-docs/js-interop.md` needs rows for `publish` (listener only), `Target::VisualViewport`/`Scroller`, `Prop::{VisibilityState, CheckVisibility, InlinePaddingBottom, InlineOverflowAnchor, ScrollTimelineSupported}`, `write`/`Slot`, and the viewport field rows. **Settled.** The keyboard variables are declared once in `dx-components-theme.css` (`@property`, universal syntax; all four panel components list it as a global asset) and the scroll variables in `dx-effects.css`, so `check-css-vars-defined` is green. The `main-thread` proof (0 long tasks) is [row 494](./494-re-enable-the-timing-cases-after-the-rust-port.md)'s: its cases are `@timing` and deactivated.
