---
id: 468
title: "Bug: form reset leaves a controlled text input showing empty while Rust keeps the old value"
status: "APPROVED 2026-10-26 by the owner (\"Ok for the rest\", backlog thread); for the next session (owner…"
size: "S"
area: inputs
updated: 2026-10-26
---

# Row 468: Bug: form reset leaves a controlled text input showing empty while Rust keeps the old value

## Item

On `<form>` reset the controlled text input has no reset path: the DOM shows an empty field while Rust still holds "abc", and the next render of its template writes "abc" back at an arbitrary moment.

**Where:** both renderers (a checkbox's state converges through the legacy reset listener, on desktop with a frame of skew; the text input does not).

**Reproduce first:** a controlled `Input` in a `<form>`; type "abc"; press a reset button; the field is empty; trigger any re-render of the template (change an unrelated signal that the input's component reads) and check the field.

**Source:** [`board-2026-10-26-r5/order-runtime.md`](../research/board-2026-10-26-r5/order-runtime.md) scenario D (finding 3, no owner for `value`).

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's construct subsumes this defect later, and the fix does not wait for the plan. Source: the last section of [`board-2026-10-26-r5/SYNTHESIS.md`](../research/board-2026-10-26-r5/SYNTHESIS.md) (live defects the board found in today's code).

## Status / trigger

APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); for the next session (owner 2026-10-26: "We will do it all in a fresh session").

FIXED on branch `lane/form-reset-468` (2026-11-06), awaiting merge. Found by board round 5; independent of plan approval.

**Repro** (`playwright/form-reset-controlled.spec.ts`, demo `input/variants/reset`): a controlled `Input`, a controlled `Textarea` and an uncontrolled `Input value="preset"` in a form; type, press the reset button. Red on the base build: the DOM empties, the state line stays `name=abc bio=some bio`.

**Root cause.** Every other form control (`Checkbox`, `Switch`, `RadioGroup`, `Select`, `Slider`, `InputOtp`, `Combobox`) already calls `use_form_reset_listener` and converges. Only the themed `Input` and `Textarea` wrappers had no reset path: the browser restores the DOM value and tells no Rust state, and the owner's next render writes its old signal back.

**Fix by construction** (`primitives/src/form_reset.rs`): `use_form_reset_value` is the one reset path for a text control the owner's signal holds. It ensures an id, listens through the existing legacy `use_form_reset_listener` (no new JS), reads the DOM value after the browser restored it through `bridge::get(Prop::Value)`, and the wrappers call `oninput` then `onchange` with a Rust-built `FormEvent` carrying it. Two constructions were needed beyond the hook, both found by running it:
- The `reset` event fires before the browser restores the values, and the microtasks carrying the message run in between, so the read waits one task (`sleep(0)`).
- Reading per field raced: the first field's report re-rendered the owner, which wrote the old text of the not-yet-read fields back (measured: textarea read `""`, then the controlled input read `"abc"`). The fields that heard the reset now queue, one task reads all of them in one bridge transaction, and the handlers run back to back before the owner renders once.
- `bridge::get` awaited from a task (no render of its own) panicked in the flush effect (`runtime.rs:223 unwrap on None`): queued effects run with an empty scope stack, and `flush` reaches `document::eval` and context lookups. `after_render` now runs its closure inside the root scope (`primitives/src/bridge/transport.rs`). The bridge's own callers all ask from render effects, which is why no earlier caller hit it.

**Proof.** The spec is red on the base build (`reset468base`, `name=abc bio=some bio` after reset) and green on the fixed build (`reset468`, 1 passed, 2.2 s on a debug SSG).

**Second round (2026-11-06, `lane/form-value-468b`): the class, not the instance.** The first round fixed `Input` and `Textarea` on reset. The owner asked for the best-by-construction fix, so the cause was deconstructed (below) and every hole of the class was routed through the one path in `form_reset.rs`. Specs added to `playwright/form-reset-controlled.spec.ts` (demo `input/variants/reset` extended with a controlled `NativeSelect` and `Checkbox`): the reset of a controlled `NativeSelect` and `Checkbox`, and the form-state restore after `goBack()`. Both were red on the base build (`val468base`: the select stayed `banana` after the reset; after `goBack()` the state line read `name: bio: fruit:apple` while the DOM held the restored `abc`) and green on the fixed build.

**The rule (owner decision 2026-11-06, "use the best-by-construction solution, by deconstructing until the root cause").** A form control's value has two writers, the DOM and the owner's Rust signal. Any DOM-side value change the owner is not told about desyncs them, and the next render writes the stale value back. The class is every case where the browser changes a value without the usual input event: (1) a form reset (`form.reset()`, `<button type=reset>`); (2) the browser's form-state restore on history navigation, which puts the previous page's values into the server-rendered controls before the wasm hydrates (a back/forward-cache restore keeps the whole page alive, DOM and signals, so it cannot disagree; Playwright's Chromium launches with the cache off, so the restore path is what its `goBack()` exercises, and the cache itself is not testable there); (3) typing into a server-rendered field before hydration (same read at mount). Autofill was checked, not assumed: it dispatches `input` and `change` in current engines, so it already reaches the owner. Every DOM-side value change reaches the owner through the control's ONE existing change callback, controlled and uncontrolled alike, through one shared primitive path (`primitives/src/form_reset.rs`): text controls report as an edit (`oninput`, then `onchange`, with the DOM value), every other control calls its value-change callback with the default. A separate `onreset` is rejected: an owner who forgot to wire it would get the desync back. React differs: it reports no reset to a controlled input (long-standing React issue), so a controlled React input keeps its state and the DOM is rewritten from it; this library reports the reset as an edit, which is also what the plan's value owner (rows 447 and 458) does.

**Audit: on reset, does a CONTROLLED instance report through its change callback?**

| Control | Reset path | Controlled instance told? | Restore (history navigation) |
|---|---|---|---|
| `Checkbox` | `use_form_reset_listener` on the mirror, `set_checked(default)` | yes, `on_checked_change` | mirror was restorable and disagreed with the visible state; the mirror is `autocomplete=off` now |
| `Switch` | same, `set_checked(default_checked)` | yes, `on_checked_change` | same fix |
| `RadioGroup` | listener on the item mirror, `ctx.set_value(default)` | yes, `on_value_change` | same fix |
| `Select`, `SelectMulti` | listener on the mirror `<select>`, `reset_to_default` / `use_controlled` setter | yes, `on_value_change` / `on_values_change` | mirror `autocomplete=off` |
| `Slider`, `RangeSlider` | listener on the first hidden input, `set_value(default)` | yes, `on_value_change` | hidden inputs are `type=hidden`, never restored |
| `InputOtp` | listener on the real input, `set_value(default)` | yes, `on_value_change` | HOLE: the real input is restorable; now `use_dom_value_restore` reports it |
| `Combobox`, `ComboboxMulti` | listener on a hidden anchor, `reset_to_default` / `set_values(default)` | yes, `on_value_change` / `on_values_change`; the visible input follows the selection label and the query is already empty while the list is closed (a click on a reset button closes it first) | the input is `autocomplete=off`, never restored |
| `CommandInput` | none | HOLE: the DOM went back to its `value` attribute, the query kept the typed text | `autocomplete=off`; reset now reports through `on_query_change` |
| `Input`, `Textarea` | `use_form_value_report` | yes, `oninput` then `onchange` (the previous round) | HOLE: now reported at mount |
| `NativeSelect` | none | HOLE: no reset path; now `use_form_value_report` | HOLE; reported at mount when the owner passes `value` |
| `InputGroupInput`, `InputGroupTextarea` | compose `Input` and `Textarea` | yes | yes |
| `ColorPicker` area and channel inputs, calendar month and year `<select>` | none | not value-owning form data: they are view state of a widget and carry no `name`; a form reset restores their DOM and the next render of the widget writes its own state back, which is the correct end state | n/a |
| `Questionnaire` choices | model-owned `checked` | the questionnaire is its own state machine with its own `on_reset` button callback; not a `<form>` reset participant | n/a |

**Fix by construction** (second round, `lane/form-value-468b`). `use_form_reset_value` is now `use_form_value_report` (attribute-driven wrappers) over `use_dom_value_report` (by id) over `use_dom_value_restore` (the mount half), all in `form_reset.rs`, sharing the one batched read below. The wrappers no longer build `FormEvent`s: they hand over their `oninput`/`onchange` and the primitive calls them. Mirror inputs carry `MIRROR_AUTOCOMPLETE`. The reset listener is the legacy `use_form_reset_listener`, reused and not grown; no JS was added. The mount read compares the DOM value with the `value` the owner rendered, so an untouched control never reports.

Known limit: a `NativeSelect` whose owner passes no `value` attribute has nothing to compare at mount (the DOM value of a select is its first option's), so a restored selection is not reported there; a reset still is.

**Desktop** (reasoned from code, not run): both the listener and the read go through `document::eval`; the read is one IPC round trip later than on web, and the batching above holds the handlers until all values arrived, so it converges with a frame of lag, the same as the checkbox.

Known limit: a field unmounted between the reset and the read has its handler called on a dropped signal; the window is one task.
- Proof on the final tree (ab88b8e4, after the reset demo's `aria_label` additions), debug lane-ssg build: `form-reset-controlled.spec.ts` 3/3 green and `all-routes` `input` axe green (both red on base earlier); specs of every touched control (checkbox, switch, radio-group, select, native-select, input_otp, command, input, textarea, input_group, plus select-escape-race, select-scroll-buttons, user-select) 156/156; 54 form-adjacent spec files (native/check/radio/switch/input/textarea/select/command/combobox/field/date/slider/toggle/otp/dialog/sheet/popover) 677 passed, 1 skipped; `all-routes` axe for the 16 form routes 16/16. Two `all-routes` axe timeouts (carousel, calendar) in a wider run under box load (load average 7-8 from other builds) passed alone. The full `test:changed` selection (3,504 tests, 221 files) was stopped at ~200 tests (no red) as impractical on a loaded box. `clippy-server` PASS; `run-gates.sh --fast`: 95 of 103 gates, 0 red.

## Size

S

## Detail

- **Files owned:** To be found by the reproduction.
- **Depends on:** None. The plan's value owner (rows 447 and 458) subsumes it by construction: on `reset` it re-dispatches a real `input` on the control with the predicted post-reset level, so the owner's own rsx `oninput` sees it in order (Dioxus 0.7.9 also delivers `onreset`); a local fix (an `onreset` that sets the signal) is allowed meanwhile.
- **Proof:** A Playwright spec that fails before the fix and passes after; run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess S).

## First paint of a controlled Textarea (2026-11-07, `lane/firstpaint-fix`)

The deploy's release suite failed `first-paint-styles.spec.ts` on `/component/textarea/` (12 differences, first `div.dx-component-preview-frame.dx-tabs-content > div` width 221.328px -> 215px). Root cause, measured by comparing the JS-off and hydrated DOM: a `<textarea>` has no `value` attribute and Dioxus SSR does not render a textarea's value as content, so the server rendered it empty. The mount report (`use_dom_value_restore`) read the DOM value `""`, found it different from the owner's `"sample text"`, and reported it as an edit, which emptied the demo's signal: the hydrated `<p>` read `Description here: ` where the static page read `Description here: sample text`, and the shrink-to-fit preview frame was 17px narrower.

The defect is that the mount report took "the server rendered less than the owner holds" for a browser restore. The rule of this row is to report a DOM-side change the owner was not told about; a restore or an edit changes a control's `value` but not its `defaultValue` (what the server rendered). Fix by construction, in the one shared read of `primitives/src/form_reset.rs`: the batched bridge read now asks for `Prop::DefaultValue` too (a new row of the bridge's closed `PROPS` table, `defaultValue`), and the mount report fires only when `value !== defaultValue`. The reset path is unchanged (it reports the DOM value after the browser restored the default). The `goBack()` restore case of `form-reset-controlled.spec.ts` still reports (restored text differs from the default), the textarea no longer clobbers the owner. An attempt to render the value as server content with `dangerous_inner_html` fixed the width but needed a new JS sink the JS-volume rule does not allow, and is dropped. Class: only text controls read through this path (`Input`, `Textarea`, `InputOtp`'s mirror; `NativeSelect` has no owner value to compare), so checkbox and select mirrors need no `defaultChecked` read. Left open, row 503: the JS-off textarea is still empty.
