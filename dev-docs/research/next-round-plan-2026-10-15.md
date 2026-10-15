# Next round plan (2026-10-15): dependency follow-ups, dev constructions, visible defects

Status: **SET UP, waiting for the owner's go** (owner 2026-10-15: "do 1 and 2 and stop the round then before 3";
"setup all and document and before you are ready to start, stop and ask for my go"). Main at the time of writing:
`3ba879c` (PR #100). Read first in a new chat: `CLAUDE.md`, `dev-docs/lane-rules.md`, this file, then the rows.

## 0. Owner decisions already taken (2026-10-15)

| # | Decision | Where |
|---|---|---|
| E1 | Order: (1) row 217 with the dev constructions 219, 220, 222; (2) the visible defects 214, 215, 223; STOP before (3) row 213's Message Scroller API gaps and the markdown component | this file |
| E2 | Markdown (the nine questions of the dependency audit artifact, https://claude.ai/artifact/GpaybnGgtbQTtYkebbUDwp): the recommendations, i.e. pulldown-cmark `=0.13.4` (default features off) in a separate optional `markdown` registry component; exact pin; model-output images OFF by default; links limited to http/https/mailto/tel/relative, external ones in a new tab with `noopener noreferrer nofollow`; code as plain `<pre><code>` with a highlighter slot; the streaming heal pass inside the component; bare-URL autolinks YES (own small pass), math NO; both new gates (only `markdown` may list the parser; declared versions match the lock) added; the audit follow-ups scheduled (= row 217) | row 213 (built in round 3, not this one) |
| E3 | Row 136 Questionnaire stays ON HOLD | row 136 |
| E4 | New backlog item: the email-client demo resets scroll to the top on a view change, or restores the user's position if they were already in that view | row 225 (backlog, not this round) |

## 1. Baselines (captured before the batch; they cannot be recovered afterwards)

- Code baseline = `83758d5` (main `3ba879c` differs only in `docs/` and `dev-docs/`).
- Full suite on a debug SSG of `83758d5`: 3254 passed / 17 failed, titles in `$LANE_TGT/pw-n4.fails` (14 known, 4 debug
  timing flakes that pass alone, 1 flaky `dialog.spec.ts:188` geometry-mid-transition). Report: `verify3-report.md`.
- Computed-style snapshot N5: two captures of the same debug build (`$LANE_TGT/snap/n5-a.json`, `n5-b.json`). A/A is
  byte-identical on every page EXCEPT two, both known and both owned by this round: (a) `hover_card`, one open-state
  record (the harness's open pass reached the same card by focus in one run and by click in the other, timing-dependent
  since row 166's 600 ms delay; coverage `via` 8 click + 1 focus vs 9 click) -> lane **specs** makes the harness opener
  deterministic (wait out the configured delay, one opener per candidate); (b) `route:/charts/` in light mode, 976 hunks
  (page and section heights such as 2157.28 vs 2151.14 px) -- row 215's hydration re-layout captured at a different
  moment each run -> lane **charts**. Compare N5 vs the merged tree with those two excluded; **proof for both lanes: an
  A/A of the merged tree is byte-identical on every page, those two included.**
- Registry install smoke: 75/75 on `c3fafe0` (component manifests unchanged since).
- Debug build kept at `$LANE_TGT/pub-verify3` (served on 8130 for the captures).

## 2. Lanes (ONE batch; disjoint file sets; builds serialize on the global lock)

Every lane gets `dev-docs/lane-rules.md` plus its prompt. Model per `dev-docs/agent-model-fit.md`: all Sonnet (each
touches shared code, a gate, or needs root-causing); no Haiku lane this round.

| Lane | Rows | Owns | Must not touch | Proof |
|---|---|---|---|---|
| **deps** | 217 | `primitives/Cargo.toml`, `primitives/build.rs` (delete) + the committed `src/js` bundle and a new repo gate that checks it against `src/ts`, the `cfg` lines in `primitives/src/lib.rs` for gated modules, `preview/Cargo.toml`, `cli/Cargo.toml` + `cli/src/registry.rs`, `templates/starter`, every `component.json` (features, versions), `Cargo.lock`, new `scripts/check-cargo-dep-versions.sh` + a release-time `rev`-stamping script, `run-gates.sh` + the CLAUDE.md gate sentence (its own clause only) | `use_dialog_open_driver` in lib.rs (lane focus), playwright | install smoke 75/75 on the default AND on `--target wasm32-unknown-unknown`; a Button-only app's crate count before/after; `cargo tree` per target before/after; full gates |
| **registry** | 219 | `preview/src/components/mod.rs`, `preview/build.rs` (variant discovery from `variants/*/` dirs or a per-component list file), the docs build that reads it | component sources, specs | every demo still renders (component-catalog spec + check-component-catalog), docs pages unchanged (diff the SSG HTML of all component pages before/after, normalised), a deliberately added variant shows up with no `mod.rs` edit |
| **specs** | 220, 222 (+ the snapshot harness's hover-card opener race) | `playwright/**` locator migration (shared oracle specs: role/position inside the demo frame, never demo copy), a `measureAtRest()` helper in `playwright/animations.ts`, new lint gate(s) under `scripts/` for text locators on demo controls and for `boundingBox()` without the helper on animated surfaces, `run-gates.sh` + CLAUDE.md (own clause) | product code | the migrated specs green on the baseline build (no behaviour change), gate red on a seeded bad spec, `dialog.spec.ts:188` stable (20 repeats alone) |
| **masonry** | 214 | the home page masonry in `preview/src/main.rs` (home route only) + `preview/assets/main.css` (masonry rules only), `playwright/layout-stability.spec.ts` | components, the theme | reproduce first: throttled network/CPU (CDP `Network.emulateNetworkConditions` + `Emulation.setCPUThrottlingRate`) recording of cards changing columns during progressive paint at iPad widths; construction: arrival-independent column assignment (explicit column containers decided at SSG time, or CSS grid without multicol balancing); layout-stability spec extended from first paint, not DOMContentLoaded |
| **charts** | 215 | `primitives/src/chart/**`, `preview/src/charts_gallery.rs`, chart component files | the theme | `/charts/` boxes identical SSG vs hydrated at every width (layout-stability spec covers `/charts/`), chart parity golden tests green |
| **focus** | 223 | `use_dialog_open_driver` in `primitives/src/lib.rs` and the dialog primitives' close path | the `cfg` lines of lib.rs (lane deps), `merge_attributes` | spec: Enter / Escape / Enter at 0 ms reopens (Dialog, AlertDialog, Sheet, Drawer, Command, modal Popover); focus restore and modal-initial-focus specs unchanged; axe |

Named overlaps (resolved at merge by the main loop, not by serialising): `primitives/src/lib.rs` (deps: `cfg` module
lines; focus: the dialog driver), `scripts/run-gates.sh` and the CLAUDE.md gate sentence (deps and specs each add one
clause), `preview/build.rs` (registry only; deps deletes `primitives/build.rs`, a different file).

## 3. Validation protocol (mandatory, row 224)

1. Each lane: lane-rules "Before handback" (merge the session branch, full `run-gates.sh`, own + consumer specs).
2. Main loop: merge lanes, `scripts/gate-at.sh` on every pushed head.
3. After the last merge: a validation agent runs the FULL suite on a debug SSG of the merged tree in two shards and
   classifies against `pw-n4.fails`; computed-style snapshot of the merged tree vs N5 with every difference bucketed to
   a lane (UNEXPLAINED must be 0); install smoke 75/75. Only then: PR, merge, release deploy, link posted.
4. Stop. Round 3 (row 213 API gaps + the markdown component per E2) waits for the owner.
