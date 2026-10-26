# Adversarial board round 3: synthesis (2026-10-26)

Four fresh members attacked the plan as amended after round 2. Reports in this folder: `blank-slate.md`, `amended-rule.md`, `runtime.md`, `coherence.md`.

## Trajectory

| round | members | verdicts | DESIGN findings | what moved in the core |
|---|---|---|---|---|
| 1 | 5 | 5 x holds with changes | (not classified) | the B layer's form: a delegated predicate interpreter became named scoped behaviours |
| 2 | 5 | 5 x holds with changes | 3, one point | the placement rule's index: by deadline, plus a command kind |
| 3 | 4 | 4 x holds with changes | 4 from two members (coherence 3, amended-rule 1); blank-slate 0, runtime 0 | the admission rule's grounds (declared, so not closed); one scope question |

The blank-slate member, designing from the audit before reading the plan, converged on the same core for the second round running, now including placement by deadline and the command reconciler. The runtime member verified every placement row against the Dioxus 0.7.9 renderers and found only details. Every DESIGN change in three rounds sits inside the B layer (its form, then its index, now its admission grounds); the root rule, the one eval sink, A/G/R and enforcement by sinks plus ratchet held in all three. Round 3 is **not converged** by the owner's bar, so round 4 attacks the amended admission rule; the amended-rule member predicts only details once it is folded in.

## The amendment (DESIGN): admission by witness, not declaration

Admission to the behaviour registry said "a commit-sensitive effect, read from its generated effect set", but `sameFrameWrite` and `focus` are declared by the behaviour, so any program that writes the DOM qualifies (`settle` and `scrollStep` were admitted "provisionally"). It also contradicted the table's next-frame row: a next-frame fallback has no commit-sensitive effect, so it could never pass, which is why C5's fallback became an "allowlisted TypeScript module" outside the registry. Round 2 had proposed the fix and the fold dropped it.

**Amended admission.** A named behaviour is admitted on one of two grounds, both checked:
- **structural effect**: `preventDefault`, `stopPropagation`, `dataTransfer`, pointer capture, or an activation-gated API, detected from the code by the gate (tsc API), not declared; or
- **late-arm witness**: the adapter's test mode delays the behaviour's effects by the deadline (one event task, one frame, or the owner's next-frame budget), and the behaviour's spec must go red under it. For the next-frame row the witness is the owner's measurement stamp (row 443).

The allowlisted-module category is dropped: C5's fallback, if Rust loses the measurement, is a named behaviour admitted by its witness. `scrollStep` becomes a `transaction` unless its witness is red; `settle` must show its witness.

## Contracts that two or more members flagged (DETAIL, folded in)

- **Command reconciler.** One document-scoped observer installed with the bridge at boot (`subtree`, `childList`, attribute filter), not one per element: 8 of 11 popover consumers mount their panel in the render that opens it, so a per-element observer attached by an effect cannot save the frame (amended-rule, runtime, blank-slate). Command attributes are rewritten every render (`volatile`) and carry a Rust-minted generation; reports carry it back, so a browser close that Rust vetoes still converges and a dismissed panel is not resurrected at attach (amended-rule, runtime, blank-slate). Several commands on one element run through one reconciler in a fixed order (closes before opens; `showModal` throws on a showing popover). On gated surfaces the reconciler's open step reads and reports the rects itself with an `applied{gen, rects}` ack: one show writer, which resolves the command-versus-G contradiction (coherence, blank-slate, runtime).
- **Page-owned transitions.** A behaviour applies the transitions its own events cause at once and accepts Rust's value only with a newer epoch: `stickToEnd` detaches on the user's wheel-up in the page, Rust mirrors (runtime, blank-slate).
- **Order, not only latency.** Rust over the bridge only for commutative decisions (absolute sets, never step or toggle from a current value the page also changes); on desktop a Dioxus event is handled at once while a bridge fact waits behind any unacknowledged render (`webview.rs:191` vs `:315,564`), e.g. carousel "next" from a scroll-tracked `selected` (blank-slate).
- **Roving focus.** The "unless key repeat" condition holds for every roving widget (78 `set_focus` sites in 32 files; `tabs.rs:389` loses key-repeat steps on desktop): composite widgets move to `aria-activedescendant` (Rust only) where the pattern allows, else one `roving` behaviour (blank-slate). L20's carousel focus hand-off needs a home now that the `focus` verb is banned and `set_focus` scrolls the page 157 px (amended-rule).
- **Android.** rsx `prevent_default` silently fails for payload events (submit, input, paste) whose event travels in a ~12-16 KB header; the first table row carries that condition (runtime).
- **Tier predicate.** `data-dx-anchored` is an unconditional vdom attribute today, so it cannot be the tier predicate; the macOS WKWebView UA is frozen, so only the boot probe classifies; G on the Apple-desktop fallback tier costs 2-3 frames plus IPC against today's in-turn placement (runtime).
- **Closure rule for two-deadline gestures.** A decision includes computing its inputs; its outputs are separate decisions (C4's recogniser stays on the page because the cancel reads it; its `scrollBy` is next-frame) (amended-rule).
- **Unplaced blocks.** T1, T2, L17's Select arm, L18, L20, L22, S4, V6, W2 (kept in JS by step 2) get placements and lanes (amended-rule, coherence).
- **Timing contract.** Corrected citation (the ack gate is `poll_vdom`); add: events route to the vdom ahead of the DOM, Rust-to-page is one `evaluate_script` per message, bridge flushes are ack-gated, and a rAF exception means no ack and a permanent stall (runtime).
- **Bridge as built (lane 3.0):** only element ops share the pump (each Session its own eval); streams latest-wins per frame rather than one message in flight; loads by eval, not `asset!` plus `Script`; docs still say capture and focus "are rsx handlers"; the fail-open is an optional lease, the plan makes it a mandatory ack (runtime).
- **Coherence.** `js-interop.md` presents the grant rule and `check-hot-events`/`check-bridge-behaviours` as landed (they are not); legacy removal keyed to `<port>.pass` would delete C5's fallback after a failed measurement (key it to the measurement's verdict); 3.FZ needs a `Program` entry in `bridge/*`; lane file conflicts (`lib.rs` 3.OV/3.DJ, `context_menu.rs` a fourth owner, `transport.rs` 3.0b/kit/3.0, 3.PL's G pieces without a TS owner); `FORBIDDEN_VERBS` bans `focus` outright and strands L20's `preventScroll` need; the row 443 kit is counted both in and out of the estimate; stale `3.QH`; `use_resize` vs `use_resize_observer` (coherence).
- **Measurement.** One person cannot run the plan as written (iOS 26 and 27, Windows WebView2, Android, 120 Hz; `__dxPerf` relies on Chromium-only entries; pass criteria need a budget that only exists after three lanes are built on it). Measure one real desktop latency distribution first, and size the owner's protocol to the devices the owner has (coherence).

## The scope question (for the owner)

Coherence rated this DESIGN; it does not change the construct, it changes how much is ported now. The construct itself (sinks, ratchet, placement rule, admission, a legacy-literal freeze, and two proof ports: 3.DJ with `hotkey` for L1/L2, and `reconcile` for T4-T7) is about 35-40 agent-hours. The full port is about 235-270 hours for about 4,127 JS lines (about 15 lines per hour; step 2 measured about 3); lanes that remove almost no JS (3.0b, 3.FZ, the kit, row 442, 3.HV, 3.SL) are about 72 hours, and the three feel-critical ports with their support about 88 hours for 1,567 lines. 3.PL (28 hours) cannot be proven in this container (Chromium only), and 3.HV reverses Rust handlers lane 2C just landed on unmeasured desktop cost. Options: port everything now; build the construct first and port the rest as ordinary backlog rows in order of JS removed per hour; or build the construct and port only on touch.
