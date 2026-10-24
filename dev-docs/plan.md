# The plan

The single authoritative sequence. Detail for each item lives in the documents linked from it; where an ordering here
differs from a list in another document, **this one wins**.

**Status:** Phases 0-5 are done and merged; Phase 6 (deferred but real) is done except the open decisions below. What is
true now (round, what runs, what is next) is in [`STATUS.md`](./STATUS.md); the open work is
[`backlog.md`](./backlog.md); every dated landing paragraph this file used to carry (2026-09-02 to 2026-10-15) is in
[`archive/rounds/plan-landing-log.md`](./archive/rounds/plan-landing-log.md).

---

## Definition of done, per item

Every item follows the same loop. An item is not done until all five hold:

1. A conformance test exists, cites its rule source, and **fails first** — per [`conformance-harness.md`](./conformance-harness.md). A port that lands without a red test has proved nothing.
2. The implementation follows [`reference/platform-rationale.md`](./reference/platform-rationale.md), which picks the best source per gap rather than copying one fork.
3. Provenance and licence handled per [`guides/lifting-code.md`](./guides/lifting-code.md) — cherry-picks keep their author; lifted files get a header.
4. The repo's own checks pass: the one entry point is `CARGO_TARGET_DIR=<absolute isolated dir> scripts/run-gates.sh` (fmt, the three clippy arms, tests, stylelint, every `scripts/check-*.sh`), plus the Playwright specs that cover the change ([`lane-rules.md`](./lane-rules.md)). The clippy command this plan first wrote (`--all-features`) is not usable: it needs gtk for preview's `desktop` and hides the `not(web)` arm.
5. The provenance ledger (section 8 of [`archive/fork-research/lifting-from-forks.md`](./archive/fork-research/lifting-from-forks.md)) is updated when code is lifted.

---

## Phase 0 — Foundations

| # | Item | Status |
|---|---|---|
| 0.1 | **Green baseline** of the workspace and the full Playwright suite | **Done** (2026-08-29): [`archive/rounds/baseline.md`](./archive/rounds/baseline.md), 119/127 passing |
| 0.2 | **Oracle structure** `playwright/oracle/{tier1-apg,tier2-html,tier3-radix,reference,subjects}`, so rule tiers do not blur into "matches Radix" | **Done**, as specified in `conformance-harness.md` |
| 0.3 | **Vendor APG reference pages**, pinned by commit (tier 1 calibrates against W3C's own reference) | **Done**: `playwright/oracle/reference/7e4034b/` |
| 0.4 | **Form fixture in `preview/`**: a real `<form>` with a native control beside each component | **Done**: `preview/src/components/form/component.rs` (`FormFixture`), exercised by `oracle/tier2-html/form-participation.spec.ts` |

## Phase 1 — Form controls that lie *(highest severity)*

`RadioGroup` and `Select` declared `name`/`required`, documented them as being for form submission, and never referenced
them: a form following the documented API silently omitted the field.

| # | Item | Source | Status |
|---|---|---|---|
| 1.1 | `Switch` — forward `required` to the hidden input | `dignifiedquire@switch.rs:128` | **Done** 2026-08-29 |
| 1.2 | `RadioGroup` — per-item hidden `<input type="radio">` | `dq radio_group.rs` / `sr`, independently identical | **Done** 2026-08-29 |
| 1.3 | `Select` — hidden native `<select>` with mirrored options (new `required` prop) | `dq select/components/select.rs` | **Done** 2026-08-29 |

Not new design: the pattern is in-tree in `Checkbox`. Render unconditionally, matching `Checkbox`; the reasoning is in
[`reference/platform-rationale.md`](./reference/platform-rationale.md) §1, and it is a framework limitation, not a
preference ([`issues/drafts/dioxus-issue-nudges.md`](./issues/drafts/dioxus-issue-nudges.md)).

## Phase 2 — The verified mined fixes

| # | Item | Source | Status |
|---|---|---|---|
| 2.1 | `RangeSlider` thumb identity at collision | `sarendipitee@42b56dd3` | **Done** 2026-08-29 |
| 2.2 | `VirtualList` borrow held across the call that reads it | `sarendipitee@799a4ff3` | **Done** 2026-08-29 |
| 2.3 | Popover self-dismiss on internal click (shared `use_outside_dismiss`) | `sarendipitee@f63ee07e` | **Done** 2026-08-29 |
| 2.4 | `use_animated_open` unmount race, plus a generation counter neither fork has | `jcgruenhage@6f0a69f0` | **Done** 2026-08-29/30; the follow-on tooltip fade landed 2026-09-18 (row 7) |

All four shipped with a regression test per the definition of done.

## Phase 3 — Accessibility behaviour

| # | Item | Status |
|---|---|---|
| 3.1 | **Focus restore, menu family** (`use_previous` + `use_refocus_on_close_unless`, wired into DropdownMenu/ContextMenu/Select/Menubar; also moves focus *off* the closed item, which our measurement found and no source handled) | **Done** 2026-08-30; `oracle-focus-restore.spec.ts` 4 red -> 5/5 green |
| 3.2 | **Body scroll lock** (`dq scroll_lock.rs` + `sr`'s unlock-flash guard) | **Done** 2026-08-30, hardened since (scrollbar-gutter baseline PR #10, event-interception rewrite PR #16); native `<dialog>` does not scroll-lock on its own, so it stayed needed after Phase 4 |

## Phase 4 — The native-platform decision

Native `<dialog>` + `showModal()` supplies focus trap, focus restore, inert background and top layer as browser behaviour,
subsuming a 743-line `focus_scope.rs` and a 91-line `aria_hidden.rs` port.

| # | Item | Status |
|---|---|---|
| 4.1 | **Ask upstream why `797b343e` dropped `<dialog>`** | **Not started**, a courtesy question owned by a person (row 6), not a blocker |
| 4.2 | Native `<dialog>`, with `open` bound declaratively as the floor (Blitz styles `dialog:not([open])` as `display:none`) | **Done** 2026-08-31: modal `Dialog`/`AlertDialog` web arm renders a real `<dialog>` + `showModal()`; oracle `oracle/tier2-html/native-dialog.spec.ts` |
| 4.3 | If blocked, port `aria_hidden.rs` and finish the Dialog/AlertDialog wiring | **N/A**: 4.2 was taken; Caveat 1 found not to block |
| 4.4 | Non-modal overlays on the `popover=` top layer (fixes clipping inside `overflow:hidden`/transformed ancestors) | **Done** 2026-08-31, then extended to every overlay: Migration A (DropdownMenu, ContextMenu, Menubar, Select, Combobox, Toast) and Migration B (modal Popover onto the native-dialog engine), complete as of PR #23 |

## Phase 5 — Collision detection

Placement was static CSS keyed off `data-side`, so anything near a viewport edge rendered off-screen, and `ContextMenu`
opened at raw click coordinates unclamped.

| # | Item | Status |
|---|---|---|
| 5.1 | Flip, shift and size clamp. The dependency decision (`sr floating.rs` vs `dq`'s vendored port) was **superseded**: CSS Anchor Positioning (`position-try-fallbacks: flip-block, flip-inline`, PR #17) covers FLIP natively and `use_anchor_position_fallback` makes the same decision from viewport math on engines without it | **Done** (FLIP PR #17; clamp: row 10) |
| 5.2 | `ContextMenu` viewport clamping (click coordinates, not solvable by CSS anchors) | **Done** 2026-09-14: `top_layer::use_point_anchor_clamp`, a small parallel hook (row 10's 2026-09-14 addendum) |
| 5.3 | Keep the CSS clamp on `fix/preview-a11y-ux` as defence-in-depth | **Dropped** 2026-09-18; its two `max-width` viewport caps are on `main` (row 10) |

## Phase 6 — Deferred but real

| # | Item | Status |
|---|---|---|
| 6.1 | Typeahead for menus (prefix matching; **do not touch `select/`**, whose matcher beats both alternatives) | **Done** 2026-09-18, row 11: `primitives/src/typeahead.rs` |
| 6.2 | `pub mod portal`, `Toggle`'s `class` prop, the public `CalendarDayState` API (upstream issue #199) | **Done** 2026-09-18, row 12 |
| 6.3 | RTL: `direction.rs` (`Direction::resolve_horizontal`, the one key-resolution site) and the per-family key/CSS flips | **Done** 2026-09-19, row 13: 13 components, oracle `playwright/oracle/tier3-radix/rtl.spec.ts` (25 rules); `navigation_menu`/`drawer`/`command`/`date_picker` had no RTL support at that date (see row 13 for the current state) |

---

## Open decisions

Owned by a person, not by this document.

1. **Where fixes land** — upstream PRs, this fork, or both. **Still open** (row 6). Upstream `main` moved again on 2026-09-07/08 (row 66), so anything carried locally is a permanent rebase cost on files upstream changes.
2. **Phase 4's native question** (4.1). **Resolved by taking 4.2** without waiting for an upstream answer; asking why `797b343e` dropped `<dialog>` stays a courtesy question.
3. **Phase 5's dependency question** (5.1). **Reframed, not answered as posed**: FLIP is solved CSS-natively; the choice only resurfaces if shift/size clamping needs a JS collision library.
4. **Whether to talk to the fork authors at all.** `dignifiedquire` and `sarendipitee` independently built scroll lock, focus coordination and collision detection while upstream sat still; neither is an upstream contributor. **Still open.**
5. **Whether native (Blitz) is a target.** If yes, several gaps are worse there, because `eval` is a no-op on that renderer. **Still open**; `preview`'s `desktop` feature lacks `dioxus-primitives/web` (row 23), untested because no desktop build has been exercised.
6. **Chart engine upstreaming. RESOLVED by the owner, 2026-10-03:** the repo has its own charts (rows 88, 90, 117-119), so nothing is offered to `dioxus-community/dioxus-charts`. The `primitives/src/chart/engine/` seam (no `dioxus` types, no `use crate::` of other modules) stays as a design property.
7. **Theme migration to shadcn's system** (owner decision 2026-09-26, row 111). Phases A (role tokens) and B (Nova foundations, global border-box) landed 2026-10-04; phase C (the remaining components' Nova pass, the axe colour-contrast re-run) and the chart palette question are tracked in the row.

## What is already done

- Fork network mined (81 forks, 111 novel refs: 4 adoptable fixes, ~11 conditional, ~45 rejected); all 4 adoptable fixes landed (Phase 2). The research is archived in [`archive/fork-research/`](./archive/fork-research/).
- Phases 0-5 built and merged, plus the 2026-09-01/02 hardening round that followed: the cfg-axis production incident and its guard (`scripts/check-cfg-axis.sh`, `oracle/hydration-parity.spec.ts`), the keyboard-open contract, the menu role contract (`menu_semantics.rs`), anchored-overlay visualViewport tracking, the touch text-entry zoom floor and the global-stylesheet fix, all in [`reference/platform-rationale.md`](./reference/platform-rationale.md).
- Oracles well beyond the original one (focus restore, keyboard matrix, menu roles, top layer, native dialog, touch focus-zoom, global stylesheet, form participation, scroll lock, hydration parity): [`conformance-harness.md`](./conformance-harness.md).
