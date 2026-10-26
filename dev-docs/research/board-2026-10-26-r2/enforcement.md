# Board round 2, lens `enforcement` (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 0, DETAIL findings: 10**

Summary:
- The enforcement design (closed sinks, one transport seam, a shrink-only ratchet) is sound. Every hole I found is a gate rule, a parameter or an implementation of it, not the construct.
- The headline answer to Q1: a well-meaning agent can still add bespoke JS with no gate going red only through the seams listed in findings 1, 4, 6 and 10. Everything else turns red, and the red message itself prints the escape (`--allow-raise "row N reason"`), which finding 2 treats.
- The one item I weighed as DESIGN is the authority of a ratchet raise (finding 2). I kept it DETAIL because the fix leaves the construct alone: it changes where a raise is validated (a grant in the merge base), not what is enforced. The owner may overrule.
- The ratchet's base compare works in this repo's flow for drops and raises (nothing is silently lost or double counted), but it is inactive today and renames read as raises (finding 5).
- "A new behaviour needs a backlog row" is not enforceable as written: the planned check is "the row exists", and the umbrella row 434 satisfies it (finding 7).
- Gate set: the three legacy listener/interval/blocking gates cannot retire on "allowlists reach zero", because the end state keeps legitimate listeners (finding 8).

Method: read `scripts/check-js-volume.sh`, `scripts/lib/ratchet.py`, `dev-docs/js-interop.md`, the root doc sections 2.2, 5.2, 5.3, 5.4, and the in-flight lane branch `lane/js-bridge-3-0` (read through `git show`, not part of the plan, labelled "lane"). Ran `scripts/check-js-volume.sh --self-test` (29 scenarios OK, 2.2 s). Every bypass below was tried on a throwaway clone in the scratchpad (`git clone --local`, nothing in `/home/user/wt/b28` was touched), with `RATCHET_BASE_REF` pinned to HEAD where the base compare mattered.

---

## 1. DETAIL, major: the transport seam is typed by `&str`, so the sink is renamed, not closed

- **Claim attacked:** root doc 5.3 item 1, "One eval site ... its argument must be a declared constant plus `serde_json` output", and 2.2/5.2 "closed bridge with one eval sink".
- **Evidence (V, lane branch):** `primitives/src/bridge/transport.rs` has `pub(crate) fn open(body: &str) -> Eval` and `pub(crate) async fn request<T>(expression: &str)`; `bridge/mod.rs:115` declares `pub(crate) mod transport`; `js_listener.rs:122` already calls `transport::open(&script)`; `mod.rs:690,723,734,807` build the request text with `format!`. So any module of `dioxus-primitives` can run an arbitrary script string through `transport::open`/`request` with no `eval` identifier in its file. The landed volume gate does not count `open(` or `request(` (not in `SINK_RX`); a string counts only if it has a `[;{}]` shape plus a JS word (`reads_as_js`). Tried: `crate::bridge_run("document.querySelector('x').showPopover()")` (no shape) gave `OK`, exit 0 (V, scratch T1b). The plan's single-site rule and the planned gate (`check-bridge-behaviours`: "nothing under bridge/ but transport.rs names `document::eval`") are both satisfied by this.
- **Change:** type the seam. `request(program: Program, args: &impl Serialize)` and `open(program: Program)`, where `Program` is an enum whose variants map to `&'static str` constants private to `bridge/`; both functions `pub(in crate::bridge)`. Until `js_listener.rs` retires, a shrink-only callers table in the sink gate (`transport::(open|request)` outside `bridge/`, one reasoned row: `js_listener.rs`). No `format!` into either function except one `encode` fn.
- **Subsumes:** the transport clause of the planned clippy `disallowed_methods` list, the "exact site count" bookkeeping (4 `document::eval` sites in the lane's transport today), and half of plan 5.3 item 8.

## 2. DETAIL, major: a raise is self-service, and the failure message teaches it (answers Q1/Q3)

- **Claim attacked:** `js-interop.md:87` "visibility for the owner, not permission ... wait for the owner"; root doc 5.3 item 5 "visibility, not prevention".
- **Evidence (V, scratch):** `--update-baseline --allow-raise "row 1 x"` on a new file with an `eval` was accepted and printed under `RATCHET RAISES`, exit 0. `RAISE_NOTE` (`ratchet.py:52`) requires only `row` and digits; it never checks that the row exists or is open. A `+1000`-line raise with a hand-edited `#! total-raised: row 7 small thing (total 5518->5519)` also passed (`TOTAL: 5518 -> 6518` printed, exit 0): `check()` step 3 only tests that a new directive exists (`new = [d for d in directives if ... not in bdirs]`), not its number. `ADVICE` prints the exact `--allow-raise "row N reason"` command at every red. The lane's first use proves the path is the normal one: the 3.0 baseline diff self-raises 17 rows and adds `#! total-raised: row 434 lane 3.0 ... (total 7786->7826)`.
- **Change:** a raise is valid only if a grant exists at the merge base: `git show <base>:dev-docs/backlog/N-*.md` exists, is open, and carries front matter `js-grant: <column>=<n>` at least the delta; the note records the remaining grant. Read the base as the nearest of `origin/main` and `origin/queue/*` (the row 418 rule `test:changed` already uses), so a grant commit by the main loop on the batch branch counts and a lane's own diff cannot grant itself. The total directive carries the number and `check()` compares it. The red text stops printing the raise command and says "ask the owner for a grant row". The main loop's full `run-gates.sh` sets `RATCHET_REQUIRE_BASE=1` so "base unavailable" is red there, not a note (the landed gate exits 0 with the note, `ratchet.py` `base()`).
- **Subsumes:** Q3's enforceable half (finding 7), the stale-total-directive hole, and the fail-open base.

## 3. DETAIL, major: the ratchet measures the wrong population (tests, `.d.ts`, generated files, display code)

- **Claim attacked:** root doc 4 and 5.3 item 5 "the number is reported in STATUS.md and held down by the JS-volume ratchet".
- **Evidence (V):** `universe()` takes every tracked `.ts`, so the lane's `ts-test/fake-dom.ts` (274 lines), `bridge.test.ts` (727), `behaviours.test.ts` (316), `types/bridge.d.ts` (89), the generated `behaviours.gen.ts` (16) are all counted as `js_file`, each a raise row in the lane's baseline diff. The plan requires vectors and `bun test` per behaviour, so every behaviour lane (about 12) raises its own test rows: the raise path becomes routine and the signal dies. Display code counts as JS volume: today's baseline carries `code_block/variants/*` snippets (7+4+4+4+9 lines, `--inventory`) and `coarse_docs.rs:31` (a docs sample that shows `document::eval(...)`). False positives that go red on a new file (all V, scratch): a displayed JS snippet in a `&str` (`js_lit 0->2`), a non-JS `Bezier::eval(&self, t)` (`sink 0->2`), an enum variant `Script { name }` (`sink 0->1`), an `onclick` HTML sample (`sink 0->1`). Each trains an agent to use finding 2's escape.
- **Change:** count what ships. `js_file` = `primitives/src/js/**` bundle bytes (manifest-pinned by `check-js-bundle`, so tamper-proof and unaffected by tests or types) plus hand-written JS outside `ts/`; exclude `ts-test/`, `*.d.ts`, `*.gen.ts`. Make `sink:eval` the identifier `eval` called or passed from a `document`-rooted or `Document`-typed path (clippy def-path does this exactly, see 9) rather than any `eval(` token. Declare `js_lit` transitional: it dies with the legacy removal lane, when no string can reach a sink.
- **Subsumes:** the per-lane raise noise, the display false positives, and the STATUS.md number's pollution.

## 4. DETAIL, major: holes in the landed gate's universe and spellings

All V by scratch test unless marked.
- **(a) Excluded paths.** `EXCLUDE_PARTS` drops any path with a `/vendor/` or `/vendored/` segment: `primitives/src/vendor/x.rs` with an `eval` is green. `.min.js` is excluded: a hand-written `ts/zz.min.js` with a non-passive wheel listener is green (its `asset!`/`Script {` costs one `sink` regardless of size).
- **(b) Markdown and SVG.** `.md` and `.svg` are not in the universe, yet `preview/build.rs:343-349,543,613` passes `Event::Html`/`InlineHtml` through `push_html` into `dangerous_inner_html` (`site/component_page.rs:237`, `variant_page.rs:161`), so `<img src=x onerror="...">` in a `docs.md` or `notes.md` (1,047 tracked `.md`) is JS the gate never reads (V by code reading). Same for `<script>`/`on*=` in `.svg`.
- **(c) Untracked files.** `git ls-files` hides a new file until `git add`; the lane's fast-gate run before `git add` is a false green (`zz_untracked.rs` with `eval` gave OK).
- **(d) Spelling.** `use dioxus_document as dd; let f = dd::eval; f("document.activeElement.blur()")` is uncounted (no paren, no `document::` prefix; `use dioxus_document::*` also escapes `document\s*::\s*\*` because `\bdocument` fails after `_`). `gloo_events`, `gloo_file`, `wasm_bindgen_futures` are not in the `ffi` regex and `Cargo.toml` entries are not scanned (the plan says they are).
- **(e) Volume unit.** `js_file` counts physical lines: a 300-statement minified one-line `.ts` is `js_file 1` (a raise, but the amount is hidden from the note and the total).
- **Change:** move the `vendor`/`.min.js` exclusions into a reasoned TSV (stale fails) instead of a regex; scan `.md`/`.svg` for `HANDLER_TAG`, `<script`, `javascript:`; use `git ls-files -co --exclude-standard`; count `\beval\b` bare (the FP is then paid by finding 3's population rule); add a `Cargo.toml` dependency allowlist (`check-cargo-dep-versions` has the machinery); `js_file` = `ceil(len/120)` per line like `js_lit`. Clippy `disallowed_methods` by def path (planned) covers (d) exactly and should be the primary sink gate, with the regex as backstop.
- **Subsumes:** the "NOT SEEN" paragraph of the gate header.

## 5. DETAIL, major: the merge-base compare, in this flow (answers Q2)

- **Works (V, by reading `Ratchet.check`/`update`):** each row is per file and compared three ways. A lane's legitimate drop and another lane's raise on different files are separate TSV lines and merge clean. On the same row they conflict textually and then fail loud: "fell: run --update-baseline" or "rose", never silently green; a measured value below the chosen row is only a write-back, a value above fails. A note already in the base cannot be spent twice (`note != bnotes.get(path)`). A deleted baseline fails when the base has it. So no drop is lost and no raise is double counted unsafely.
- **Does not work, or needs a rule:**
  - **Inactive now.** `--report` on b28: "scripts/check-js-volume.baseline.tsv does not exist at the merge base af63f39312 ... compared with the tree only". Every raise in batch-28, including lane 3.0's, is unaudited, and the first merge to main launders all of it. Process change: the main loop pins `RATCHET_BASE_REF` to the ratchet-landing merge (`1e976f24`) for the rest of batch-28.
  - **Attribution blur.** The b28 lanes merge `origin/queue/batch-28` into themselves (`git log`: "Merge origin/queue/batch-28 into lane/js-bridge-3-0"), so `merge-base HEAD origin/main` is the batch's fork and every earlier lane's raise prints as the lane's own; `lane-rules.md:299` ("quote every `RATCHET RAISES` entry") then misattributes. Print `own` against the nearest of `origin/main` and `origin/queue/*`, `inherited` against `origin/main`.
  - **Renames read as raises (V scratch).** `git mv primitives/src/scroll_lock.rs scroll_lock2.rs` with the row renamed: "the baseline row rises above the merge base (sink 0->4, js_lit 0->90, install 0->1) and it has no `# raised:` note". The plan renames and splits JS-bearing files (3.WB `wheel_band.rs`, 3.CA, `js_macros.rs`, 3.SL). Carry base rows across `git diff -M --name-status <base>`.
  - **Conflict rule.** `lane-rules.md:270` "conflicts keep both sides" produces duplicate TSV rows (a parse error). Say: a baseline conflict is resolved by taking either side and re-running `--update-baseline`, never by hand.
  - **Headroom.** The total is checked against the base, so lane A's drop funds lane B's total rise inside one batch without a `total-raised` line (per-file notes still needed). Minor; solved by finding 2's numeric directive.
- **Subsumes:** nothing else; this is the whole of Q2.

## 6. DETAIL, major: verbs are closures, budgets are quotas, and a forbidden verb already shipped

- **Claim attacked:** root doc 2.2 "a new verb (data, one table line) is not [a behaviour]"; 5.2 "commit-sensitive effects are not verbs"; 5.3 item 4 "per-file line budget (`path<TAB>max_lines<TAB>row`)".
- **Evidence (V, lane `bridge.ts`):** `METHODS` rows are `run: (el, args) => ...` closures and already hold logic (`showPopover` guards on `typeof` and `:popover-open`; `close` branches on an argument). So "verb = data" is a convention and any logic can enter as a row without a row-of-backlog. `check-bridge-behaviours.sh` sets one constant `BEHAVIOUR_BUDGET = 150` for every module (slack = 150 minus the module's real size) and `BUDGETS` has `bridge.ts` at 1100 against 874 real lines (226 lines of free growth); the budgets are constants in the script, not ratcheted, so growth is an unflagged edit. `setPointerCapture`, `releasePointerCapture` and `focus` are `call` verbs in `METHODS`, contradicting 5.2 and 1.4 item 4 (the lane's own `transport.rs` docs say they are not usable at `pointerdown`), and no gate lists forbidden verb names.
- **Change:** budgets equal the current size and move through the same `Ratchet` class (same base compare, finding 2's grant); count bytes or canonical tokens, not physical lines; a `tsc`-API rule caps a `METHODS`/`PROPS`/`FIELD_GETTERS` row at one expression plus at most one guard `if` (anything bigger is a behaviour and needs a row); a `FORBIDDEN_VERBS` list from 5.2 is enforced on both the Rust `Method` enum and the TS keys.
- **Subsumes:** the "verbs collapse" question's gate side, and the registry-creep risk (5.5 item 3) as a mechanical rule.

## 7. DETAIL, major: "a new behaviour needs a backlog row" is not enforceable as written (Q3)

- **Claim attacked:** 2.2 "a new behaviour is a backlog row, a named variant, a vector, a spec tag and a budget row"; 5.5 item 3.
- **Evidence (V, lane):** `check-bridge-behaviours.allow.tsv` says "the row must exist"; its only entry cites row 434, the umbrella, so any behaviour can cite 434. The row, the enum variant, the generated file, the vectors and the budget are all written in the lane's own diff, so a lane can create the row in the same commit. Nothing mechanical separates "the owner decided" from "the lane wrote a row".
- **What is enforceable:** ordering and attribution, not intent. A budget row cites its own row N; the gate requires `dev-docs/backlog/N-*.md` to exist **at the merge base** (not created by the lane), `status` open, and a front matter line `adds: [<kind>]` naming the behaviour; a new `KINDS` entry in `behaviours.gen.ts` against the base with no such row is red; citing 434 is refused. That makes "row first" a distinct earlier act (the main loop or owner lands the row in the batch), which is the best a repo-local gate can do. Intent stays a human review of the `ALLOWLIST ADDED`/`RATCHET RAISES` lines.
- **Subsumes:** the second half of finding 2 for registry growth (one grant mechanism for volume and kinds).

## 8. DETAIL, minor: the gate set overlaps, and the retirement criterion cannot fire (Q4)

- **Claim attacked:** root doc 5.3 item 9 "`check-js-listeners`, `check-blocking-scroll-listeners` and `check-uncleared-intervals` retire when their allowlists reach zero".
- **Evidence (V):** marker counts in `primitives/src`, `preview/src`, `preview/index.html`: `listener-ok` 18, `blocking-ok` 4, `interval-ok` 3 (plus `motion-ok` 1, `focus-ok` 27). The end state keeps legitimate ones: the lane's `bridge.ts:593` carries `// listener-ok: the bridge's own registration`; `preview/index.html` has six page-lifetime `listener-ok` scripts; `modality.rs` and `PREPAINT_JS` (class c) stay. So the allowlists never reach zero. Also the three gates are not all JS: `check-uncleared-intervals` also bans the Rust shapes `Interval::new(`, `spawn_forever(`, `set_interval` (not JS, and `gloo_timers` is not a volume sink), and `check-blocking-scroll-listeners`'s rsx half bans `onwheel:`/`ontouchstart:`/`ontouchmove:`, which the planned `check-hot-events` re-bans (`wheel`, `touchmove`) with a different list and marker (`blocking-ok` vs a reasoned TSV).
- **Overlap today:** for a NEW listener or timer the volume ratchet already goes red (new lines), so the JS halves add value only on net-zero edits of legacy strings (e.g. `passive: true` to `false` inside an existing script).
- **Change (fate table):**

| gate / marker | fate | when |
|---|---|---|
| JS halves of `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners` | retire; their legitimate sites move to a reasoned TSV (reason, row, stale fails) read by `check-bridge-behaviours` (`bridge.ts` adapter: one `addEventListener` site) and by `check-theme-mode-writer`/`check-open-focus` (class c) | when the last legacy JS string is gone (legacy removal lane), not "allowlists at zero" |
| Rust half of `check-uncleared-intervals` | keep, rename `check-rust-timers`, or become clippy `disallowed_methods` on `gloo_timers::callback::Interval::new` and `spawn_forever` | now |
| rsx half of `check-blocking-scroll-listeners` | fold into `check-hot-events` with ONE event table generated from the `hot` flag the lane's `EVENTS` table and `bridge/events.rs` already carry; `blocking-ok` retires | with `check-hot-events` |
| inline markers `listener-ok`, `blocking-ok`, `interval-ok` | no new ones from now; delete with the gates above | same |
| `js_lit`, `comment_eval` columns | transitional, retire with the legacy removal lane | same |

- **Subsumes:** a silent permanent three-gate overlap; a retirement trigger that never fires.

## 9. DETAIL, minor: the planned hot-events, not(web) and clippy gates

- **hot-events as a grep is the wrong tool (V, I).** `dioxus-rsx-0.7.9/src/attribute.rs:271,287` expands every `onX:` to `dioxus_elements::events::onX(...)` and `dioxus-html-0.7.9/src/events/mod.rs:25` defines each as `pub fn $name`, a plain function with a def path. Clippy `disallowed_methods` on `dioxus_html::events::{onscroll, onresize, onpointermove, onmousemove, onwheel, ontouchmove, ondrag, ondragover}` and their `::call_with_explicit_closure` resolves by def path, and `#[expect(clippy::disallowed_methods, reason = "...")]` at the site is a reasoned allowlist whose stale entries fail (`unfulfilled_lint_expectations`). That subsumes a grep and its allowlist by construction. (I: whether clippy fires on proc-macro-expanded calls with user spans needs a ten-minute probe in lane 3.0b; if it does not, keep the grep.) Today's population (V, grep): about 20 sites (`select/components/list.rs` 4, `virtual_grid.rs` 3, `virtual_list.rs:486`, `table_rows.rs:289`, `slider`/`resizable`/`color_picker` `onresize`, `chart` ×4, `drag_and_drop_list` ×2 `ondragover`, `context_menu.rs:531` `onpointermove`, preview `data_table` ×2, `justified` `onresize`, `reorder` `ondragover`). Scope: the plan names `primitives/src`; the shipped components in `preview/src/components/**` must be in scope too.
- **not(web) rule (5.3 item 7).** After the single sink lands, any `eval` or `JsListeners::install` outside `transport.rs`/`js_listener.rs` is already red, so the rule is subsumed; before that, 3.DJ deletes the 16 sites. If written, extend `check-cfg-axis.sh` (the one parser of the `feature = "web"` axis) rather than writing a second; the forms `cfg!(feature = "web")`, `cfg_attr`, and a helper without the cfg called only from the arm need cases. Retire it with 3.DJ plus the typed seam.
- **clippy escapes.** `#[allow(clippy::disallowed_methods)]`, `#![allow(...)]`, editing `clippy.toml` (today only `allow-unwrap-in-tests`, V lane) or a `[lints]` table are all agent edits that the clippy arm cannot see. Add to the sink gate: `allow(clippy::disallowed_` outside `transport.rs` is red, and the clippy.toml `disallowed-methods` list is recorded in the baseline by hash. Clippy covers workspace members only; the detached crates (`labs/carousel-lab`, `templates/starter`, `playwright/oracle/attr-synth`) stay on the volume gate.

## 10. DETAIL, minor: the TS-side scan is a spelling denylist

- **Claim attacked:** root doc 5.3 item 4 (canonical-form scan with `bun build --minify-syntax`).
- **Evidence (V, python run of the lane's own regexes `COMPUTED`, `SINKS`, `FORBIDDEN`):** not caught: `e[name]()`, `evt[name]()` (the receiver list has `ev` but not `e`/`evt`; `bridge.ts` itself names its parameter `e`), `self['ev'+'al']('1')`, `Reflect.get(e, 'prevent'+'Default').call(e)`, `W.setTimeout('document.title=1', 0)`, `(0, W.eval)('1')` (`eval)` is not `eval(`), `(()=>{}).constructor('return 1')()`. The lane did not implement the canonical form; `bridge.ts` types receivers as `any` throughout (`(el: any, args: any[])`), which is what makes computed access legal.
- **Change:** the lane already runs `tsc -p`. Use the compiler: `noImplicitAny` plus typed receivers (`Element`, `Window`, `Event`) makes a computed member access a compile error, and a small `tsc`-API pass bans `ElementAccessExpression` with a non-literal argument outside `own(TABLE, name)`, any call whose symbol resolves to `eval`/`Function`/`setTimeout` with a string argument, and `.constructor`. By construction, with no regex list.
- **Subsumes:** the `COMPUTED` receiver-name list, the `SINKS` regex, and the `FORBIDDEN` token list for everything except the registry's declared files.

---

## Attacks that failed

- **Moving JS between files to dodge the per-file count.** Red: a new file is a new row (`new file with a non-zero count`), and the total `js_lit + js_file` must still drop. A minified one-line `.ts` is still a new row (`js_file 0->1`); only the amount is hidden (finding 4e).
- **Editing the baseline in the same commit.** Red without a `# raised:` note: "the baseline row rises above the merge base ... has no note" (checked with the base pinned, scratch).
- **Reusing a base note.** `note != bnotes.get(path)` refuses the note the base already has; the delta text in each note differs on a second raise.
- **Deleting the baseline.** Red when the base has it (`ratchet.py` `check()` top).
- **The landed self-test.** 29 scenarios (aliased `use ... eval as`, `concat!` split, inline `document::Script`, new `.js`/`.ts`, raise without note, merge-base-only raise, total rise) all turn red; I did not break any of them with a natural spelling. `document::eval`, `.eval(`, `use ... eval as run`, `document::*`, `Script {`, `create_head_element`, `dangerous_inner_html`, `web_sys`, `include_str!` of a `.js`/`.html` are all counted.
- **Growing the inline `<script>` of `preview/index.html`.** Red (`inline_script` bytes, per file).
- **A drop or a raise lost or double counted across a batch merge.** Not found: rows are per file; same-row conflicts fail loud in the safe direction (finding 5).
- **Editing a baseline row to match a measured raise without a note under a pinned base.** Red (same as "editing the baseline").
