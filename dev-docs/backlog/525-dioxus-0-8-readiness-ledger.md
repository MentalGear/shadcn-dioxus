---
id: 525
title: "Dioxus 0.8 readiness: the ledger of what the 0.8 alphas change for this repository, with the row or construction each item lands in"
status: "BACKLOG (checked 2026-11-15). Research-gated: nothing is taken until 0.8 is stable. 0.7.10 is not…"
size: "S (the ledger); the migration is sized…"
area: build
updated: 2026-11-18
---

# Row 525: Dioxus 0.8 readiness ledger

## Item

The repository is on Dioxus 0.7.9 (`Cargo.lock`; `Cargo.toml` asks for 0.7.8). Upstream has 0.7.10 (2026-07-30, one
backport: hot-patching when the bin target's crate name differs from the package name) and the 0.8 pre-releases
0.8.0-alpha.0 to alpha.3 (2026-05-19 to 2026-10-16); no release has followed alpha.3 as of 2026-11-15, and no stable
0.8 date is announced (read: the GitHub releases page and docs.rs for `dioxus-html` 0.7.10 and 0.8.0-alpha.3). This
row keeps the list of what 0.8 changes for this repository's constructions, so that the rows building now do not
build against what 0.8 removes, and so that the migration is a known list when 0.8 is stable. The dependency review
(`research/dependency-review-2026-11-06.md`) and the Blitz feasibility study (`research/blitz-feasibility-2026-10-25.md`)
hold the earlier reads; this row is the one place the readiness items live.

## Status / trigger

BACKLOG (checked 2026-11-15). Research-gated: nothing is taken until 0.8 is stable. 0.7.10 is not taken: its one fix
is for a bin whose crate name differs from its package name, which this workspace has no instance of, and a `Cargo.lock`
change rebuilds the warm release template for no gain. Trigger for the migration: a stable 0.8 and a library baseline
that passes its own gates on it ([row 438](./438-blitz-native-renderer.md) is gated the same way).

## Size

S (the ledger); the migration is sized when 0.8 is stable.

## Detail

What the alphas change, read from the release notes and docs.rs, with the consumer here (measured counts are from
`rg` over `primitives/src` and `preview/src` on 2026-11-15; the rest is inferred):

- **`onbeforeinput` (alpha.1), cancellable, `BeforeInputData { input_type(), data(), is_composing(), value() }`.** The
  event [row 448](./done/448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) says is missing in 0.7.9, which is why its
  `otpFilter` witnesses are red by construction and its adapter is bridge-delivered. Row 448's kind takes exactly those
  fields plus the selection, so on 0.8 the rsx handler feeds `decide` directly and the adapter is deleted without a
  kernel change. Row 448's `prevent` witness (`test.fixme`, green on Chromium because a bubble-phase cancel at the host
  works as well) is admitted then: the witness compares the rsx cancel with the bridge's placement. Also the IME half of [row 473](./473-bug-date-picker-segments-let-the-browser-edit-the-vdom-text-node.md).
- **Clipboard events expose `data_transfer()` and `files()`; `onselect` carries the selection (alpha.1).** A Rust path
  for the paste text the OTP and the date-picker segments read today through the bridge.
- **`ToggleData` and `CancelData` stay opaque (alpha.3: `new`, `downcast` only).** [Row 449](./done/449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md)'s
  finding that an rsx `onbeforetoggle` cannot carry the open state or an order number holds on 0.8 too; its per-element
  nonce construction does not wait for upstream. **No upstream request (main loop, 2026-11-18, settled by
  construction).** A `new_state` payload says which state the page reached, not whether the owner decided again since;
  that ordering is what the nonce rule gives, and it is needed on any asynchronous channel (the desktop webview, the
  delayed bridge of `executor-popover.spec.ts`), so a payload would delete nothing here. `CancelData` is empty by the
  platform's own definition: `cancel` is a plain `Event`. The rendered level (`data-dx-level`) is also the form a
  native renderer can apply itself, without the bridge, when Blitz implements the popover. The owner can still file
  one as upstream hygiene.
- **Comment-node placeholders and markers removed at runtime and during hydration (alpha.2).** Consumers: the
  hydration-parity oracle (`playwright/oracle/hydration-parity.spec.ts`) and `chart.spec.ts` read comment nodes; the SSG
  output and the `check-render-keys`/hydration gates may count them. Re-measure when 0.8 is stable.
- **Falsy boolean attributes are removed, not written as `"false"` (alpha.2).** 10 rsx sites write a boolean attribute
  as `false` (`disabled`, `checked`, `hidden`, `open`, ...); `state_attrs` uses presence strings and is not affected.
  Any spec that asserts `="false"` on a boolean attribute changes.
- **HTML elements are trait-based and leave `dioxus_elements` (`html::main`), `rsx!` expands to a const builder,
  `define_elements!` (alpha.2).** 3 `dioxus_elements` references here; `dioxus-attributes` (the proc-macro crate that
  parses `rsx!` for the collision checker) must be re-read against the new expansion.
- **`keyboard-types` 0.8: `Key::Enter` becomes `Key::Named(NamedKey::Enter)` (alpha.3).** 370 match sites in 43 files
  here; a mechanical migration, best done by one script and one commit.
- **`WriteMutations::set_attribute` takes a borrowed value; `linux-libxdo` opt-in (alpha.3).** No consumer here beyond
  the desktop build ([row 23](./023-preview-s-desktop-feature-lacks-dioxus.md)).
- **Blitz (dioxus-native 0.8 on blitz 0.3.0-beta.4): `NodeHandle::set_focus`, `scroll_to`, `get_client_rect`,
  `get_scroll_offset` implemented and no longer panic (alpha.3).** The Rust path for the native slot writes of
  [row 447](./447-phase-0-7-the-order-runtime.md)'s focus owner and [row 462](./462-port-roving-focus-78-set-focus-sites.md),
  and the measurement [row 452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md) places with. The
  overlay tiers of row 438 still wait for upstream.
- **SSR renders `initial_value`/`initial_checked`/`initial_selected` as `value`/`checked`/`selected`; `#[css_module]`
  stylesheets stay in the head after the first request (alpha.1).** Consumer: `default_checked.rs` and the form
  primitives of [row 458](./458-port-3-fc-form-and-input-primitives-and-the-native-slot-kinds.md); the SSG head.
- **Security: an h2 DoS fix for fullstack servers (alpha.3).** No fullstack server here; the SSG is static files.

**Proof when taken.** The gates and the release suite green on the 0.8 baseline; the rows above each name what their
item becomes.
