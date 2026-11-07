---
id: 482
title: "Control gaps: Slider form participation, read_only, Toggle data-pressed, Tabs activate_on_focus"
status: "MERGED in PR #156 (7d50e068), DEPLOYED in 1ea333fa; the optional items landed in PR #166. Items 1…"
size: "S-M"
area: components
updated: 2026-10-28
---

# Row 482: Control gaps against both upstreams

## Item

Four behaviours that Radix and Base UI both have (or that Base UI has and base-nova uses) and this library lacks. Source: [`radix-vs-base-ui-2026-10-28.md`](../../research/radix-vs-base-ui-2026-10-28.md), appendix B.

## Status / trigger

MERGED in PR #156 (`7d50e068`), DEPLOYED in `1ea333fa`; the optional items landed in PR #166. Items 1 to 4 are built: Slider and RangeSlider `name`/`form`, one hidden `type=hidden` input per thumb (the first carries the id the shared form-reset listener uses, so a reset restores `default_value`), `on_value_commit` (pointer release after a press that changed a value, each keyboard step that changes one; not a reset) and `RangeSlider::min_steps_between_thumbs` (one `bounds_for` serves clamping and `aria-valuemin/max`); `read_only` on Checkbox, Switch and RadioGroup (`aria-readonly` plus a `data-readonly` presence attribute; the group carries `aria-readonly`, its items only `data-readonly`, because `aria-readonly` is not allowed on `role=radio`; a radio group's arrow keys still move focus); Toggle `data-pressed` (ToggleGroup items inherit it); Tabs `activate_on_focus`. The optional rest is settled in "Remaining optional items" below: Tabs `data-active`/`data-hidden` were already built, Slider thumb push and `edge` alignment are built as opt-ins, the `Control` part is not built (reason below). Row 481's helper (`state_attrs.rs`) has landed since (PR #156), and the `read_only` sites use its `readonly` function.

## Remaining optional items (2026-11-06)

Compared against `@base-ui/react` 1.9.0 (npm tarball, `slider/utils/resolveThumbCollision.js`, `getPushedThumbValues.js`, `slider/thumb/SliderThumb.js` lines 160-240, `tabs/panel/TabsPanelDataAttributes.d.ts`, `tabs/tab/TabsTabDataAttributes.d.ts`):

- **Tabs `data-active` / `data-hidden`: already built.** `TabTrigger` spreads `state_attrs::active` and `TabContent` `state_attrs::panel_hidden` (the row 481 helper), so the selected tab carries `data-active` and every other panel `data-hidden`, next to Radix's `data-state`. The Status above was stale; a spec (`tabs.spec.ts`, "the selected tab carries data-active ...") now pins it.
- **Slider thumb push collision: built as opt-in.** Base UI's default is `push`; ours stays `Clamp` (Radix's behaviour, and what shipped), so nothing changes for existing users. `RangeSlider { thumb_collision: ThumbCollision::Push }` moves the neighbour to keep `min_steps_between_thumbs` steps (`place_thumb`, the shared pure placement; `bounds_for` bounds a pushing thumb by the room its neighbour needs). Divergence from Base UI: its `getPushedThumbValues` also restores a pushed thumb toward its press-start value when the pusher turns back; ours leaves a pushed thumb where it was pushed. Base UI's `swap` behaviour is not built.
- **Edge alignment: built as opt-in** (`thumb_alignment: ThumbAlignment::Edge` on Slider and RangeSlider). Thumb and range are placed with `calc(P% + F * var(--dx-slider-thumb-size, 0px))` (F is `0.5 - P/100`, a pure function of the value), the theme sets the custom property to the thumb's width, and the pointer and drag deltas map onto track minus thumb using the thumb's size measured once at mount (`get_client_rect`; the measurement moves no pixels). Not built: re-measuring when the thumb or track resizes later. Base UI measures with a ResizeObserver and a prehydration script.
- **First paint (2026-11-07, `lane/firstpaint-fix`).** The first version placed the edge-aligned thumb from the mount measurement, so the server drew it centred and the hydrated page edge-aligned: the deploy's release suite failed `first-paint-styles.spec.ts` on `/component/slider/` (`div.dx-slider-range` width 320px -> 308px, the `edge` demo). Red on a base debug build, green after the `calc()` construction; any other control that places itself from a measurement would be this class (a layout the server cannot know must come from a style token, not from a mount measurement).
- **Slider `Control` part: not built, on purpose.** In Base UI the Control is the pointer surface that wraps the track and thumbs; here the Slider root already is that surface (it owns the pointer handler and the rect the drag math reads), and the track is a styling part. A separate Control would only be a second wrapper element with nothing to own, and would move the rect measurement for no behaviour gain. Revisit if a consumer needs the wrapper for layout.
- Public API: `primitives/public-api.txt` gains `ThumbCollision`, `ThumbAlignment` and the props `RangeSliderProps.thumb_collision`, `RangeSliderProps.thumb_alignment`, `SliderProps.thumb_alignment` (all defaulted, so existing callers are unchanged).
- Specs: `playwright/slider-collision-edge.spec.ts` (new), `playwright/tabs.spec.ts`. Demos: `slider/variants/range_push`, `slider/variants/edge`. The expected-red cases on a base build are all of `slider-collision-edge.spec.ts` (the `range_push` and `edge` demos and their props do not exist there, so the labelled sliders are absent); the `tabs.spec.ts` presence test is green on base (the attributes already existed). The `range_push` demo uses a 10-step gap on a 20..60 range so its thumbs do not overlap at rest (`coarse-glyph-controls.spec.ts` checks hit-target overlap per slider demo).

## Size

S-M

## Detail

1. **Slider form participation (M).** No `name`, no hidden input (verified by grep in `primitives/src/slider.rs`). Add a `name` and a hidden input per thumb (Radix: a bubble input per thumb; Base UI: a visually hidden `<input type=range>` in each thumb, `SliderThumb.mjs`), `on_value_commit` (Radix `onValueCommit`, Base UI `onValueCommitted`) and `min_steps_between` thumbs. Optional and not part of this row: push collision, `edge` thumb alignment, a Control part. The form-reset listener in `lib.rs` and `default_checked.rs` are shared files.
2. **`read_only` on checkbox, switch and radio group (S).** Base UI `readOnly` sets `aria-readonly` and `data-readonly` and blocks changes; absent everywhere here. Uses the row 481 helper for the attribute.
3. **`data-pressed` on Toggle and ToggleGroup items (S).** `aria-pressed` and `data-state` exist; base-nova styles through `aria-pressed:`, so this is vocabulary, not a fix.
4. **`activate_on_focus` on Tabs (S).** Ours activates manually (Base UI's default); add the opt-in for automatic activation (Radix's default). Optional with it: `data-active` on the tab, `data-hidden` on the panel.

- **Files owned:** `primitives/src/{slider,checkbox,switch,radio_group,toggle,toggle_group,tabs}.rs`, their themed components and specs. Do not touch `lib.rs` beyond the form-reset listener.
- **Proof:** a form-submission spec for the slider (single and range, with `name`); a spec per `read_only` control; a Tabs spec for focus-activation.
