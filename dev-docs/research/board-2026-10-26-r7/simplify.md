# Board round 7, lens `simplify` (2026-10-26)

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 1, DETAIL findings: 8**

Summary:
- I tested every named mechanism against one question: which live instance (an audit id or a code path in b28 or l30) needs it today? About 25 of the roughly 90 mechanisms either have no live instance, duplicate a mechanism the plan already has, or exist only for a port that runs only if the owner schedules it. None of the removals touches the class guarantee: JS only where Rust cannot, and no new JS where Rust could.
- **The one DESIGN finding is a simplification of carrier 1** (finding 1). The transition port (an owner-rendered hidden `input` that receives a synthetic `input` event) duplicates synchronous events that Dioxus 0.7.9 already delivers on the vdom's ordered path, on the element each owner already renders: `onbeforetoggle` (a non-bubbling element listener), `oncancel`, `onfocus`/`onfocusin` and `oninput`. Every focus instance the slot table cites (`tabs.rs:389`, `date_picker/segment.rs:536`, `virtual_list.rs:487`) is already one of these rsx handlers. The epoch can be counted from the same events on both sides, so it needs no payload. Stream boundaries are settled by dropping the gesture's subscription, through the plan's own Drop-registry rule.
- The DETAILs:
  - gate the roving-focus row and the `MountedData` write ban on the phase-0 focus-write witness (about 30 h, conditional);
  - fold `focus_prevent_scroll` into the focus owner;
  - take the document scroller out of the slot table by bracketing `showModal()` in the executor's open step;
  - defer the `scrollOwner` intent kinds that only the owner-scheduled ports use;
  - move dead-surface registry and vocabulary entries to the exits;
  - drop three of six inventory layers;
  - defer 3.HV (11 h, 0 lines removed);
  - treat the floors as the boot probe's case list rather than a second mechanism.
- Firm savings are about 22-26 agent-hours, plus about 30 h conditional on a witness. A contributor learns about 25 fewer named things, including the whole port rule set (hidden host, root `stopPropagation`, the no-form rule, the hydration-parity line, the liveness handshake, the `dispatchPort` effect and its witness).
- Kept, because a live instance needs them (attacks that failed, at the end): the inbound pump, G's watchdog next to the CSS fail-open, the lease, `stop`, the control arm, the executor's tree-ordered batches, the category (a) and (c) platform rows, and the witness harness's library probes.

Evidence labels: **V** = read in the repo or in the pinned crate source, **W** = a web or spec source, **I** = inference. "root" is `l434d4/dev-docs/research/sync-js-root-2026-10-26.md` at 640cad14. "b28" is `/home/user/wt/b28`, "l30" is `/home/user/wt/l30`. Dioxus paths are relative to `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

---

## The inventory: every named mechanism against its live instance

"Live" means an audit id or a code path that needs the mechanism on the default scope 2. "Owner-scheduled" means it is used only by 3.CA, 3.MS or 3.WB, which run only if the owner schedules them (root 5.0).

| family | mechanism | live instance today | verdict |
|---|---|---|---|
| root, placement | root rule, 5 deadline rows, 2 kinds, one-shot call, 4 conditions | all 103 ids (root 3.3) | keep |
| bridge | one eval sink, `Program` enum, closed verb tables, verb-row cap, `transaction`, listen/observe/sample, `REDUCERS` | V11, W1, C3, every R row | keep |
| enforcement | clippy sink ban, per-site `#[expect]`, grant rows, ratchet, legacy freeze, file inventory, tsc-API scans, budgets TSV | the 107 eval lines and the legacy literals | keep |
| enforcement | `check-hot-events` baseline plus payload table | about 20 hot rsx sites | keep the gate, defer its drain lane 3.HV (finding 8) |
| enforcement | the `MountedData` write ban (82 `set_focus` and 9 `scroll_to` sites under `#[expect]`) | a desktop-only race, not reproduced | gate on a witness (finding 2) |
| registry | `otpFilter`, `scrollLock`, `dropAnywhere`, `pan`, `settle`, `openFocus`, `stickToEnd`, `labelClick`, `hotkey` | S11; L10/L11; L25; O5/C4/V17; W3/W5/W7; L16/L18; W8; L23; O1/V22 | keep |
| registry | `scrollOwner` with three intent kinds | one-shot: W2, S4, V6. Level: W8 (owner-scheduled). Snap: carousel (owner-scheduled) | build the one-shot kind only (finding 5) |
| registry | `scrollStep`, `roving`, `wheelBand`; `pan`'s `css-var`, `scroll-by-with-refusal`, `scroll-absolute` and reserved `stream` variants | none: each exists only if a witness is red or a verdict says so | move to the exits (finding 6) |
| vocabulary | `release` | C4 `drag.rs:448`, `toast.rs:1413`, which the plan itself lands as deletion | drop from the vocabulary (finding 6) |
| vocabulary | `stop` | C4 `carousel/drag.rs:634` (capture-phase click suppression) | keep |
| admission | witness per (site, effect), 4 library probes, per-element projection, control arm, `WitnessViolation` ban, init-script witness mode, completed-paint delays, frame-counted inputs, RO dispatcher | S11 first, then every behaviour | keep |
| admission | platform rule rows: (a) UA-reserved shortcut, (c) engine default action | O1 F6, V22 Ctrl/Cmd+B; `dropAnywhere`'s `dataTransfer` | keep |
| admission | platform rule row (b): activation-gated API | none: V14 is a preview allowlist row, `copyText` does not exist (V: `clipboard` appears only under `preview/src`) | remove until a `copyText` row (finding 6) |
| admission | `verdict.json` lag/order ground | C5 (owner-scheduled) | keep, as the plan states it |
| inventory | `site!` ids, `site!` scan, per-install effect refusal, `Decision<E>`, DOM-less compile | every behaviour | keep |
| inventory | per-kind typestate constructors, the debug `track_caller` cross-check, the `__dxBehaviourCoverage` hook | duplicates of the scan and the witness suite | drop (finding 7) |
| order | slot, epoch, `based_on`, the Rust-caused rule, the one inbound pump | Accordion `beforematch` across panel sessions (`collapsible.rs:64-86`), T12, the executor's reports | keep |
| order | transition port: hidden host, synthetic `input`, root `stopPropagation`, no-form rule, hydration-parity line, liveness handshake, `dispatchPort` effect and witness | every instance it serves has a Dioxus event already (finding 1) | replace (finding 1) |
| order | focus owner | L17 refocus through `TriggerHandle`, L18, L20, `settle`'s focus, `openFocus` | keep, small |
| order | `focus_prevent_scroll` verb, its caller TSV, the `FORBIDDEN_VERBS` exception, admission's "green focus witness adds a caller-listed verb row" | L20 (2 sites), which the focus owner already serves | merge into the focus owner (finding 3) |
| order | the document scroller slot row; L13 through `scrollOwner`; `use_early_scroll_capture` | `scroll_lock.rs:267-298`: it exists only because `showModal()` scrolls the window | the executor's open step brackets it (finding 4) |
| order | native slot owners, mount-frozen helper, rsx native-slot sink scan, value-as-owner-write | rows 468, 469; `combobox/components/input.rs:152` | keep |
| order | contenteditable slot row plus its gate rule | row 473 (date segments) | keep (an `otpFilter` site, not a new kind; finding 6) |
| command, G | one executor, `pageObserver`, exit wait (`AbortError` counts as done), tree-ordered batch, layer-tree invariant | T4-T7, L5, S8/S9, the menubar | keep |
| command, G | G pending CSS, fail-open animation, watchdog, two paint tiers, `engine: always/fallback`, per-surface list | S1 (always), T8/T11 fallback, O3, V19, W8's opening position | keep |
| command, G | per-feature floors table plus boot probe | T8 | one mechanism: the probe, with the floors as its case list (finding 9) |
| behaviour runtime | `decide` plus adapter, vectors, lease, ref-counting, declared target/phase/passive, composition order | every behaviour; the lease: `scrollLock`, `hotkey` | keep |
| lanes | 3.HV (11 h, 0 lines removed); the roving-focus row (about 30 h, 0 lines removed) | an inferred desktop cost; a desktop-only race | defer behind their measurement or witness (findings 8, 2) |

---

## Findings

### 1. DESIGN (simplification), major: carrier 1 is Dioxus's own synchronous events on the element the owner already renders; the transition port and its rule set go

**Claim attacked.** Root 2.2, order contract, carrier 1: "a page transition that any rsx-handled decision reads reaches Rust synchronously on the vdom's own ordered path: the transition port, a hidden `input` that each owner renders in its own component ... and that receives a synthetic `input` event carrying the transition". Its rules are: a synchronous hook; root `stopPropagation`; never inside a `<form>`; installed before `data-hydrated` with a liveness handshake; a hydration-parity line; the `dispatchPort` vocabulary effect with its own witness (admission points 2 and 6). Each slot in the slot table names the port as its carrier. Root :329 rejects rsx because "`ToggleData` is empty and the HTML spec coalesces `toggle` tasks".

**Evidence: each instance the port serves already has a synchronous, vdom-ordered Dioxus event.**

- **Popover open state** (the motivating "dismiss then the trigger's `onclick`", V b28 `popover_sync.rs:200-225`):
  - V: Dioxus 0.7.9 delivers `onbeforetoggle` (`dioxus-html-0.7.9/src/events/generated.rs:300`).
  - V: it is non-bubbling (`dioxus-core-types-0.7.9/src/bubbles.rs:100`), so the interpreter attaches the listener to the element itself (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:109-130`).
  - V: the event runs synchronously on web (`dioxus-web-0.7.9/src/dom.rs:88-116`) and by blocking XHR on desktop (root 1.2).
  - W: `beforetoggle` fires synchronously inside the show and hide popover algorithms. Only `toggle` is queued and coalesced (<https://html.spec.whatwg.org/multipage/popover.html#hide-popover-algorithm>; root :294 and round 6 whole-core finding 2 rely on exactly this to choose `beforetoggle` as the port's hook).
  - So the plan's hook is already a Dioxus event. The port only re-dispatches it as a synthetic `input` on a second element to obtain a payload.
- **The payload is not needed.**
  - I: `beforetoggle` always flips the state. A call on an already-showing popover returns before firing; a show can be cancelled only by a listener calling `preventDefault()`, and the library has none.
  - I: so Rust tracks the DOM level as a flip per event. Every event reaches Rust, because the listener is in the element's creation template and is attached before the executor first shows the element: the panel mounts with its spread in the opening render (root :326), and the executor shows it at the next checkpoint.
  - I: the epoch is the count of `beforetoggle` events on the slot. The executor counts them on the page and Rust counts its handler calls: one event stream, delivered synchronously, so the counts are equal.
  - I: the page records `last_page_transition` on a page-caused event. Rust tells its own from a page-caused one with no payload: after the flip, the believed level either equals the level Rust published or it does not.
  - I: `based_on` is Rust's count, and the fence is unchanged: `based_on >= last_page_transition`.
  - I: the executor's divergence memory keyed by scope (root :327) also becomes unnecessary for a re-created panel, because Rust holds the dismissed state itself.
  - The hooks already return a spread (item 0.4), and a `Vec<Attribute>` spread carries listeners as well as attributes.
- **Dialog open state** ("Escape then Enter"):
  - V b28 `lib.rs:848-852`: every `cancel` is prevented, and L2 becomes an rsx `oncancel` (root 3.1), so a library dialog closes only by Rust's command (a Rust-caused transition).
  - The residual is a close without `cancel`: Chrome skips `cancel` without intervening user activation, and a consumer's `<form method="dialog">` closes natively. Where `<dialog>` has `beforetoggle` (Chrome 132, Firefox 133, Safari 26; root :294) the same rsx `onbeforetoggle` covers it.
  - On older engines it is a level-stamped fact on the pump (carrier 2), which the plan already uses for post-paint closes of bridge-only slots.
- **Focus.**
  - V: all three slot-table citations for "rsx handlers decide on it" are rsx focus handlers already: `tabs.rs:389` (`onfocus`), `date_picker/segment.rs:536-543` (`onfocus`), `virtual_list.rs:487-493` (`onfocusin`).
  - Their transitions already arrive in page order on the vdom path. The port's document `focusin` hook adds one blocking request per focus change, app-wide, on desktop (root :293 prices it).
- **Controlled value.**
  - A keystroke fires the input's own rsx `oninput`, which carries `value` (V `serialize.ts` input path).
  - For the transitions that fire no `input` event (form reset, form-state restoration, the install-time level that row 469 needs), the value owner dispatches a real `input` on the input element itself after the change. That element carries a `data-dioxus-id` and already has the handler.
  - I: this is also the user-visible fix for row 468, because a controlled input should hear that its value changed.
- **Stream boundaries** (Slider and Resizable's stale `up`, V b28 `pointer.rs:33-48`, `move_interaction.rs:119-146`; the drawer's drag end against Escape).
  - V root :298: the pump dispatches "through a registry entry that `Drop` removes". V l30 `session.rs:434-447`: a `Subscription` has an explicit `Drop`.
  - I: so a per-gesture subscription, dropped by the next rsx `pointerdown` (which `start_pointer` already handles inline, `move_interaction.rs:119-146`) or by the drawer's close, makes the stale `up` undeliverable by construction.
  - The start is rsx already, so no port is needed for the start either.

**What breaks, concretely (I).**
1. **Desktop id reuse.** An event on an old element whose id was reclaimed can reach the new element's handler (root 1.4, event-routing row), and one misrouted `beforetoggle` inflates the count. The port's payload carried the slot id. This is the Dioxus-wide hazard the plan already lists for every rsx handler, and the window is a light dismiss of a panel that is being removed in the in-flight render: removal itself hides with `fireEvents` false, so no event fires.
2. **Cost of Rust's own toggles on desktop.** Rust-caused `beforetoggle`s now also reach Rust: one blocking request per open or close, issued from the edit application, the same class as `onmounted` (root 1.4). This is human-rate, and `bridge-cost` prices it.
3. **The dialog skipped-cancel case on pre-`beforetoggle` engines** falls to carrier 2, as described above.

**Concrete change.**
- Carrier 1 is "the owner's own element's Dioxus event: `onbeforetoggle`, `oncancel`, `onfocus`/`onfocusin`, `oninput`, with the epoch counted from that event on both sides".
- The value owner re-dispatches `input` on the input itself for reset, restoration and the install level.
- Stream boundaries go by subscription identity.
- Delete:
  - the hidden port element, its spread from the hooks, the root `stopPropagation`, the no-form rule, the hydration-parity line and the port liveness handshake;
  - the `dispatchPort` effect and its witness;
  - the port rows of the slot table.
- Keep the before-`data-hydrated` install ordering only for behaviours (`gotoHydrated` still needs `hotkey` to exist).
- Item 0.7 gets a one-hour spike: the counts stay equal across dismiss, Escape, open-then-close inside one task, and re-key.

**Saves.** About 5-7 h:
- item 0.7's port and handshake, about 1-2 h;
- item 0.4's port element and hooks, about 1 h;
- 3.GS's +1 h for start, end and cancel on the port;
- 3.FC's port share;
- the `dispatchPort` witness pair;
- the "port reads the `toggle` report" specs.

It also removes about 8 named rules that every lane touching an owner must learn.

**Subsumes.** Carrier 1's instances: dismiss then `onclick`, Escape then Enter, Slider's drag twice, the drawer's drag end against Escape, the focus rows and the value rows. It does not subsume a close with no `cancel` on Chrome before 132, Firefox before 133 or Safari 18.x (carrier 2 covers it).

**Why DESIGN.** It replaces the mechanism of one of the order contract's two carriers. The carrier itself, the vdom's ordered path, stays. If the owner reads the port as a parameter of the carrier, this is the one borderline item in the report.

### 2. DETAIL, major: the roving-focus row and the `MountedData` write ban are justified only by a desktop race nothing has reproduced; gate them on the phase-0 focus-write witness

**Claim attacked.** Root 5.4: the roving-focus row (about 30 h, 0 lines removed) and item 0.1's ban on `MountedData::{set_focus, scroll, scroll_to, scroll_to_with_options}`, with 82 plus 9 per-site `#[expect]`s.

**Evidence.**
- V `dioxus-web-0.7.9/src/events/mounted.rs:115-141`: on web, `set_focus` is a direct `focus()`, so the race exists on desktop only (root :319 says the same: "web is unaffected").
- V root 5.4: the roving row removes no JavaScript, so it does not serve the class guarantee.
- V root :300 and item 0.6: the plan already builds the counterfactual that would show the race, "a late roving write after a click into another field". No row says what happens if it is green.
- V js-interop.md: the "Rust first" rule still lists `MountedData` writes as the API, while the ban forbids them. The two contradict each other until the ban's precondition is shown.

**Change.**
- The ban and the roving row open only when item 0.6's focus-write witness is red, or when row 442's desktop-cadence build reproduces the key-repeat loss.
- Until then `set_focus` stays legal Rust, the ban is not added (no 91 `#[expect]` lines in item 0.1), and the roving row stays in the backlog.
- New composites still follow condition 3 (`aria-activedescendant` first), which costs nothing.

**Saves.** About 30 h, conditional, plus about 1 h of `#[expect]` churn in item 0.1.

**Subsumes.** The 78-site port's hours, until evidence exists.

### 3. DETAIL, minor: `focus_prevent_scroll` is a focus-owner operation, not a verb with its own caller table

**Claim attacked.** Root 3.2 (L20), 5.2 and admission point 5: a closed verb row `focus_prevent_scroll`, legal only from a reasoned caller TSV, with an exception in `FORBIDDEN_VERBS`. Separately, "a green `focus` or `capture` witness adds a caller-listed verb row keyed to the witness id".

**Evidence.**
- V root :301: the focus owner "is what `openFocus`, `roving`, `settle`'s focus, the refocus on close and `focus_prevent_scroll` all call", and it already decides `preventScroll`.
- V root :550: the verb is "called through the focus owner".
- V: L20 has two sites (`carousel/root.rs:470`, `context.rs:181`).

**Change.**
- Every focus write is a focus-owner operation, with `preventScroll` as an argument.
- Delete the verb row, its caller TSV, the `FORBIDDEN_VERBS` exception and the green-`focus` landing rule (a green `focus` witness lands in the focus owner).
- A green `capture` witness lands as deletion, since touch capture is implicit and a mouse capture is expected red.

**Saves.** About 1 h, one TSV and two rules.

### 4. DETAIL, minor: the document scroller leaves the slot table if the executor's open step brackets `showModal()`

**Claim attacked.** The slot table's row for the document scroller and its `scrollOwner`; L13 "written through `scrollOwner`"; the Rust-caused instance "the scroll-lock restore after `showModal`'s focus scroll".

**Evidence.**
- V b28 `scroll_lock.rs:267-298`: the capture and restore exist because `showModal()` moves `window.scrollY` by hundreds of pixels. `use_early_scroll_capture` (9 references) exists only to win a race between two effects against that jump.
- V root :325 and :328: the executor's open step is one JavaScript turn that already calls `showModal()` and runs `openFocus`.

**Change.**
- The open step saves `scrollX`/`scrollY`, calls `showModal()` and restores in the same turn, before paint. The close step's focus hand-back goes through the focus owner with `preventScroll`.
- No Rust write to the window's scroll remains.
- `scrollLock` keeps blocking input, and L13 becomes part of `scrollLock` (its registry row already lists L13).
- Delete the document scroller row, `use_early_scroll_capture` and that Rust-caused spec instance.

**Saves.** About 1-2 h. It also removes the order contract's only page-global scroll slot.

### 5. DETAIL, minor: build only the one-shot intent kind of `scrollOwner`; the level and snap kinds belong to owner-scheduled ports

**Claim attacked.** Root :301 and :414: three intent kinds (one-shot, level, snap). Item 0.7's classifier spike includes a snap re-snap.

**Evidence.**
- V root 5.0 and 5.4: the level kind serves W8 (3.MS), and `stickToEnd`'s detach is that kind. The snap kind serves the carousel (3.CA and row 470).
- Both ports run only if the owner schedules them (scope 2, the default).
- The live one-shot consumers are W2, S4 and V6.

**Change.**
- 3.VS builds the one-shot kind.
- 3.MS adds the level kind, through `stickToEnd`, and 3.CA adds the snap kind.
- Item 0.7's spike keeps wheel, touch, key and a scrollbar `pointerdown` against anchoring, and drops the snap case.

**Saves.** About 1 h in phase 0, and two concepts for any lane that does not touch W8 or the carousel.

### 6. DETAIL, minor: registry, vocabulary and platform-table entries justified only by a hypothetical are exits, not entries

**Claim attacked.**
- The registry rows `scrollStep`, `roving` and `wheelBand` ("exists only if its witness is red" or "only if a verdict").
- `pan`'s enumerated output variants (`css-var`, `scroll-by-with-refusal`, `scroll-absolute`, reserved `stream`).
- The `release` effect.
- Platform category (b).

**Evidence.**
- V root :419: round 6 already moved `dialogKeyForward` and `place` out on the rule "a row nothing uses is dead surface". These rows meet the same rule.
- V root :346: `release` "lands as deletion".
- V b28: the only `releasePointerCapture` calls are the legacy `carousel/drag.rs:448` and `toast.rs:1413`.
- V b28: `clipboard` occurs only under `preview/src`, so category (b) has no library instance.
- V b28 `date_picker/segment.rs`, row 473: the contenteditable cancel needs only a reject-all `beforeinput` cancel, which is `otpFilter`'s `prevent` half at a second site, not "an `otpFilter`-like behaviour".

**Change.**
- Move them into one "exits, not entries" list. Each gets a variant only with its `adds:` row, witness or verdict.
- Drop `release` from the vocabulary, since the port deletes the explicit calls.
- Category (b) is added with the first `copyText` row. The table is owner-edited, so it costs nothing to add later.
- The date segments become an `otpFilter` site.

**Saves.** About 1 h of gate and vocabulary code, and about 9 named entries.

### 7. DETAIL, minor: three of the six inventory layers are redundant

**Claim attacked.** Admission point 1 and 5.3 item 7. Per-kind typestate constructors (`Hotkey::<Prevent>`), the `site!` scan "cross-checked against `track_caller` in debug builds", and the `__dxBehaviourCoverage` hook "as a cross-check".

**Evidence.**
- V root :335: the scan already requires each `site!` literal to be unique and at exactly one location, with a witness per site name and effect.
- V root :335 and :426: the adapter refuses an undeclared effect per install, and `Decision<E>` refuses it at `tsc` time.
- I: an empty declared effect set is a scan failure ("a site declares no effect: use `listen`"), so typestate generics per behaviour add nothing a gate does not already catch, and they cost an API per kind.
- I: each site's scenario file mounts it under its witness, so the coverage hook re-proves what the witness suite proves.

**Change.**
- Keep `site!`, the scan, per-install refusal, `Decision<E>` and the DOM-less compile.
- Drop the typestate, the `track_caller` cross-check and the coverage hook, which root 5.6 also uses as "a post-suite check that every variant ran".

**Saves.** About 2 h across item 0.1, item 0.3 and each behaviour row.

### 8. DETAIL, major (hours): defer 3.HV until the latency session or `bridge-cost` shows the cost

**Claim attacked.** Root 5.4: candidate row 3.HV, 11 h, 0 lines removed.

**Evidence.** V root 5.0: "3.HV (11 hours, 0 lines removed) reverses Rust handlers that lane 2C landed, on a desktop cost that is inferred and unmeasured; `check-hot-events` with a shrink-only baseline already stops growth." It moves Rust rsx handlers to Rust over the bridge, so no JavaScript is added or removed, and the class guarantee is untouched either way.

**Change.**
- Keep the gate and its baseline (item 0.1).
- Open 3.HV only on item 0.5's distribution or a `bridge-cost` red for one of its sites.
- The shared-file serialisations that exist only for 3.HV go with it: `select/components/list.rs`, `chart.rs`, `drag_and_drop_list.rs`, the reorder demo.

**Saves.** 11 h, and three serial file hand-offs.

### 9. DETAIL, minor: the floors table is the boot probe's case list, not a second mechanism

**Claim attacked.** Root 2.1: "A is used behind a floor per feature, and a probe decides at run time".

**Evidence.**
- V root :254: on a WKWebView "a 'Safari 26.2 floor' cannot be read there and only the probe classifies".
- Nothing in the plan sniffs a version at run time.
- The probe's cases (edge shift, reopen three times, a nested submenu, a scaled parent) are exactly the floor bugs.

**Change.** State one mechanism: the boot probe classifies the engine tier, and the floors table documents why each probe case exists and which engines are expected to fail it. This is wording only.

**Saves.** One concept.

---

## The smallest construct that holds the class guarantee

Kept, each with the instance that needs it:
- the root rule and the deadline table with its two kinds (all 103 ids);
- the closed bridge with one eval sink and `Program` (the 107 eval lines);
- sinks plus the ratchet, grant rows and the legacy freeze (they stop recurrence);
- the registry of nine behaviours (each with a live audit id);
- admission per (site, effect) by library probes, with the control arm, plus category (a)/(c) platform rows and `verdict.json` for next-frame rows;
- one executor with the exit wait and the tree order (T4-T7, L5, S8/S9);
- G with the CSS fail-open and the page watchdog, two paint tiers and the boot probe (T8, S1, O3, V19);
- the order contract: `based_on`, the Rust-caused rule, the inbound pump, with carrier 1 as Dioxus's own events (finding 1);
- the focus owner (L17, L18, L20);
- `scrollOwner`'s one-shot kind (W2, S4, V6);
- the native slot owners (rows 468, 469).

Dropped or deferred: findings 1-9. Firm savings are about 22-26 h (phase 0 about 89 to about 82; full port about 355-395 to about 330-370 before the conditional roving row), plus about 30 h conditional on finding 2's witness. About 25 fewer named mechanisms remain to learn.

---

## Attacks that failed

- **Drop the inbound pump now that carrier 1 is native events.** It fails: the single-mode Accordion combines the `beforematch` facts of N panel sessions (V b28 `collapsible.rs:64-86`). Dioxus has no `onbeforematch`, so those facts stay on the bridge, and only the pump orders them by construction.
- **Drop G's page watchdog and keep the CSS fail-open only.** It fails: the theme's own `animation: none !important` under reduced motion matches popups inside a motion-key host (V root :443).
- **Drop the CSS fail-open and keep only the watchdog.** It fails: the bridge needs `unsafe-eval` and loads after boot (root 5.2). Under a strict CSP no page JS runs, and O3/V19's SSR pending states (`li:not([data-sized])`) would stay invisible.
- **Narrow the lease to "prevent and then send".** It fails: `scrollLock` prevents and relies on Rust to unlock, and a wedged runtime would hold the page scroll-locked forever. The lease is also already built (V l30 `session.rs:403-421`).
- **Drop the `stop` effect.** It fails: C4 calls `stopPropagation()` in its capture-phase click suppression (V b28 `carousel/drag.rs:634`).
- **Drop the real-against-real control arm.** It fails: without it, a probe that fires in both arms through animation noise would admit by artefact (root admission point 3).
- **Drop the executor's tree-ordered batch and the layer-tree invariant.** It fails: a closed popover is `display: none`, which removes a descendant top-layer panel's box while a modal descendant stays inert (root :328). The menubar's hover from A to B (`menubar.rs:785-805`) exercises it.
- **Replace epochs by compare-and-set on the level.** It fails for open state: ABA (close, then reopen by the page) lets T12's stale close apply to the reopened panel. It might hold for `value`, but it would add a second fence rule, which is no simplification.
- **Remove `scrollOwner` for the one-shot reveals (make them element-relative `scrollIntoView` verbs).** It fails on the owner's parity bar: a keydown reveal and a user wheel end in different states on web and desktop without the fence (root 1.4, item 8).
- **Delete `labelClick` (L23).** It is blocked: `label-double-click.spec.ts` asserts the unstyled primitive's contract, and `MouseData` has no `detail`. Deleting it is the owner's call (root :415), not a simplification I can make.
