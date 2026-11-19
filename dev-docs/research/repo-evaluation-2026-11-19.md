# Repository evaluation of 2026-11-19: code-analyser, allocation audit, TigerStyle, output and code quality

Five read-only reports answered the owner's four questions (the code-analyser tool, the three defects of
https://aarol.dev/posts/zig-allocator/, TigerStyle, the built output and the code quality). Four verification agents then
re-derived every claim against the tree; each claim is one JSON line with a verdict and receipts (a quote, a ripgrep
count or a logged run), and `scripts/report-receipts.py` checked them. **Every number below comes from those verified
lines; where a claim was only partly right, the verified figure is used, and where it was refuted the note says so.**
All measurements were taken on commit `0df1782` (main at that time). Grades, estimates and severity calls are marked as
opinion. **Nothing was profiled**: no figure here is a measured frequency or a measured cost of an allocation.

## 1. The code-analyser tool

MentalGear/code-analyser at `374bf4a`, run by `scripts/codegraph.sh`. The pipeline is `rust-analyzer scip` (`index.scip`),
`build_graph.py` (a hand-written protobuf reader that writes `graph.json`), a comment-metrics pass (`comments.json`) and a
self-contained `explorer.html` (cytoscape 3.34.3). A graph node is a file carrying only `{id, comp, items}`.

**Measured run.** 1,556 nodes, 264 components, 13,388 edges (rs 7,077; ext 4,673; asset 205; ts 1,275; test 158); 0
unresolved workspace references (9,545 resolved by owner, 2,160 by module); 20 duplicate-symbol warnings in the SCIP log;
`scripts/codegraph.sh smoke` passes 40 of 40. Output: `index.scip` 28,465,582 B, `graph.json` 407,128 B,
`comments.json` 639,074 B, `explorer.html` 1,527,346 B.

**Accuracy of the edges.** The report's spot check found 12 of 13 right; the wrong one, `public-api-snapshot.rs` to
`src/main.rs`, comes from two targets of one package sharing the symbol `crate/`. The verifier's own sample of 15 Rust
edges found none provably wrong: 10 file-precise, 5 pointing at `lib.rs`, `mod.rs` or `main.rs` (coarse, not false). One
real dependency is missing: `primitives/src/sleep.rs` to `gloo_timers` (`sleep.rs:15`, behind the cfg at `:14`).

**Accuracy of each comment metric.**

| Metric | Result |
|---|---|
| Undocumented public items | genuine in 5 of 5 sampled. "83.9 % documented" counts items and public fields; items alone are 2,001 of 2,638 (75.9 %), 637 undocumented; 803 is items plus fields |
| Duplicated comment text | genuine in 6 of 6 sampled; 322 runs covering 1,404 lines |
| TODO | 2 findings, 1 real (`chrome.rs:174`); `block_demo.rs:16` quotes another project's TODO |
| Stale names (N1) | 6 findings, 1 real (`top_layer/component.rs:45` `DocsSidebar` exists only in comments); the other five name external types or a deliberately mentioned removed dependency (`WebDocument`, `poll_vdom`, `GenerationalBox`, `next_element_sibling`, `dioxus_sdk_time`) |

**Blind spots.** The index is built for the host cfg, so code behind `cfg(target_family = "wasm")` is absent. SCIP
occurrence ranges are discarded (`build_graph.py:41-45` keeps only symbol and role), so a node carries no line and no
per-function data. Multi-target packages collapse onto one symbol.

**Setup gap.** `rust-toolchain.toml` lists only `clippy` and `rustfmt`, and `scripts/session-setup.sh` never mentions
`rust-analyzer`; only the tool's own README says `rustup component add rust-analyzer`. The report reproduced the failure
("Unknown binary 'rust-analyzer'"); the verifier did not re-run it.

**Detection of the article's three defects.** As shipped, the tool detects none: it has no clone, allocation or thread
logic. The SCIP index it builds does hold what a detector needs, type-resolved with line and column: `format!` 752,
`to_string` 1,514, `vec!` 1,483, `collect` 706, `String::clone` 412, `Vec::clone` 319, `Option::clone` 137, `Rc::clone`
46, `to_owned` 56, `to_vec` 42 (grep finds 758 `format!`, so coverage is about 99 %). Clippy is the right tool for defect
3. The report's example that `chart.rs:700-703` deep-clones a `HitTest` per pointermove is refuted: the sectors are an
`Rc<[Sector]>`, so the clone is a reference-count bump (the `spawn` per event at `pointer.rs:258` is real).

## 2. The three defects in this repository

The defects, as the audit reads the article: (1) clones that appease the borrow checker, (2) allocation inside a hot
loop, (3) an eager allocating argument such as `expect(&format!(..))`.

| Defect | Verdict | Counts |
|---|---|---|
| (1) clones | Present only as one shallow systemic form | 1,032 `.clone()` in 145,365 non-test lines (7.1 per 1,000); 237 `props.attributes.clone()` in 83 files; 88 `redundant_clone` sites (77 production, 11 test); 15 `mem::take`, `swap` or `replace` repo-wide; 0 thread, tokio or channel uses; 74 bare `spawn(` calls |
| (2) hot loops | Present in per-keystroke and per-drag-move paths, not in a game loop | the sites below |
| (3) eager argument | Absent from library code | 0 `.expect(&`; all 195 `.expect(` take literals; 10 allocating `map_or(value.clone(), ..)` hits, all in preview demos (8 combobox variants, `message_scroller/variants/animation/mod.rs:163`, `site/blocks.rs:489`) |

**Hot-path sites (read, not run).**
- `combobox/context.rs:174-176,219` and `command.rs:129-151,743`: with a non-empty query each option evaluation allocates
  five Strings (`query.cloned()`, `query.clone()`, `text_value.clone()`, `normalize_query`, `haystack.to_lowercase()`);
  with an empty query only the `text_value` clone allocates.
- `data_view/derive.rs:75-83`: `filter_text` builds a `Vec<String>` per row on every recompute, empty filter included
  (one Vec plus one String per filterable field; the second String appears only with a non-empty filter).
- `pointer.rs:70-92`: a JSON message per raw pointermove (no rAF coalescing); Rust deserializes a `(String, (i32, f64, f64))`.
- `resizable.rs:700-725`: four certain allocations per effect run (`origin.cloned()`, the panels clone, the collect in
  `constraints_vec`, `sizes.to_vec()` in `resize_pair`); a fifth depends on `set_committed`.
- `slider.rs:638-668,1030`: `granular_thumbs.cloned()` per effect run and one `format!("left: {position}")` per thumb render.
- `drawer.rs:581`: `samples.push` per pointermove, unbounded within one drag.
- `virtual/bridge.rs:263-299`: `set_scroll_top` and `set_scroll` start a new `document::eval` and clone `container_id` per
  call; the script text is a literal. They run when a measured row above the viewport shifts content.
- `SCROLL_BRIDGE_JS` sends one deduplicated JSON message per scroll event; the chart pointermove spawns a task and awaits
  `get_client_rect` per move (`chart.rs:700`, `pointer.rs:258`).

**Severity (main loop, opinion): low.** These paths run only while a user drags or types, and each such event also pays
for a render. **Measured:** the counts above (clippy 0.1.97 on the host target with the perf group and added lints,
exit 0; ripgrep; file reads). **Inferred:** per-event frequency, "N is demo-sized", the cost of a render, the kernel's allocation count.
No counting allocator, `dhat` or `memory.grow` check exists anywhere in the repository; the only heap read is a logged
`JSHeapUsedSize` in `playwright/virtual-list-find.spec.ts`, which asserts node counts. The native and server clippy arms
were not run (about 0.36 % of the lines sit behind those features).

**Lints that could be denied today.** `clippy::perf` fires 0 times; `expect_fun_call` 0 (deny compiles now);
`or_fun_call` 19 (none in tests: 11 preview, 6 primitives, 2 in the `attr-spread-collision-checker` member; 6 do not
allocate and need a rewrite or an `#[expect]`); `redundant_clone` 88 (primitives 71 including
tests, preview 16, kernel 1 at `decision-kernel/tests/vectors.rs:295`; a workspace deny also reaches the scripts member and
the unrun arms, whose counts are unknown). Eight of the sites were traced to the last use of the cloned value and are true.

**Memory growth per overlay cycle (own soak, local `docs/` and deployed site, identical slopes).** Wasm linear memory
grows about 1.75 KB per dialog open/close cycle (20.25 to 22.94 MB over 1,500 cycles; 1.82 KB in the first half, 1.77 in
the second), 2.9 KB per combobox cycle (26.69 to 30.75 MB; 2.80 in both halves) and 3.0 KB per dropdown-menu cycle (21.38
to 22.81 MB over 500). Popover is flat for about 300 cycles, then slow. The growth is continuous, not a one-time
high-water mark; the cause is not identified. The preview's `logger` feature (`preview/Cargo.toml:8`) makes tracing-wasm
record about 48 performance marks and 48 measures per dialog cycle, which raises the JS heap (3.91 to 6.86 MB after forced
GC; 5.98 MB with the calls stubbed) and leaves the wasm slope unchanged. It affects the preview site only: no other
manifest lists `logger` (`rg` over every `Cargo.toml`). A reported combobox DOM-node leak is Chromium's editing undo
stack (the node counter rises by one per cycle for about 330 cycles, then holds at 10,936; a plain `<input>` page with no
Dioxus grows 1.01 per cycle). **Owner and main loop judge it negligible at this size** (1,000 opens cost about 2 to 3
MB). The open question is whether the slope scales with an overlay's content. The built-output report's 5.7 KB and 33 KB
per cycle were not reproduced (3 to 11 times too high).

## 3. TigerStyle

Source: TIGER_STYLE.md (27,383 B) and tigerstyle.dev. The report classified 73 rules (opinion; the verifier checked that
31 of the rules exist in the source as stated, and corrected the evidence figures, not the classes): 17 already covered
(HAVE), 9 partly (PARTIAL), 12 applicable with nothing enforcing them (GAP), 21 not applicable (NA), 7 in conflict with a
repo rule or idiom (CONFLICT), 7 not mechanically checkable (PRINCIPLE). The table is not complete: six rules of
TIGER_STYLE.md and four of tigerstyle.dev have no row.

**Adopted slice** (one construction, kernel-centred; row 533):
- Kernel: `too-many-lines-threshold = 70` plus a deny (0 violations; the kernel's own `clippy.toml` is the one clippy reads
  there); deny `unwrap_used`, `expect_used` and `panic` (2 hits under `--all-targets`: `decision.rs:111` and `:129`, both
  in tests, because the kernel's `clippy.toml` lacks the allow-in-tests switches; non-test code has 0); `debug_assert!`
  pre- and postconditions on its 20 public functions (the report counted 15 and missed the five `extern "C"` exports);
  const assertions over its constants (the only const assert in the tree, `primitives/src/lib.rs:27`, checks a cfg, not
  constants); a named cap on `dx_input` (`ffi.rs:45-48` resizes to the caller's `u32`; `kernel.ts` catches the resulting
  trap and stands the kind down after three). The cap changes the committed wasm bytes, which
  `scripts/check-decision-kernel.sh` rebuilds and compares, so it needs the owner's ratchet grant. `[profile.kernel]`
  (`Cargo.toml:80-91`) sets `debug-assertions = false` and `panic = "abort"`, so `debug_assert!` costs nothing there.
- Function length outside components (physical lines, non-test code): primitives 17, cli 5, kernel 0 (preview 15; 37 over
  eight crates). Components are out of scope: 87 of 364 in primitives and 83 in preview exceed 70 (maxima 513 and 962).
  Clippy's `too_many_lines` cannot skip components, so a script holds the NUMBER per crate and lets it only shrink.
- Residue: 13 `loop` and 21 `while` in primitives have no bound check; outside the kernel 1 assert-family macro exists in
  non-test primitives code and 0 of 946 public non-component functions assert.

**Not adopted** (reasons checked against the verified lines):

| Rule | Reason |
|---|---|
| 14, static allocation | The Dioxus VDOM, signals, the DOM and serde allocate on every render; the kernel itself returns `Option<Vec<u8>>` (`kinds.rs:39`) |
| 6 and 5, at least two assertions per function, crash on failure (outside the kernel) | `wasm32-unknown-unknown` aborts on panic (inferred; the committed kernel profile sets `panic = "abort"`), so an assertion failure takes down the user's page. That contradicts the fail-open design (`js_listener.rs:146-148` skips a malformed message; `js-interop.md:190`: a wedged Rust side must never trap the keyboard). It would take 2 x 1,917 = 3,834 assertions in primitives |
| 4, no `usize` | wasm32 `usize` is 32 bits; std indexing needs it; the kernel ABI (`ffi.rs`) is `u32` only |
| 37, qualifier-last naming | Against Rust API convention and the HTML and shadcn prop names (`max_length`); `primitives/public-api.txt` and the public API snapshot gate make a rename a deliberate break. The report counted 42 `max_`/`min_` prefix names against 4 suffix names (not re-counted by the verifier) |
| 16, 70 lines for components | `rsx!` markup; 87 of 364 primitives components and 83 preview components exceed it |
| 66, zero dependencies | The product is built on Dioxus; the control is the dependency register and the supply-chain gates (`check-dependency-register.sh`, `check-cargo-dep-versions.sh`); the kernel has none. Direct normal dependencies: primitives 8 (plus `gloo-timers` and `tokio` under target tables), cli 5, preview 8 (plus 5 build), attributes 4, kernel 0 |
| 20, split compound conditions, no `else if` | Clippy `collapsible_if` and `collapsible_else_if` under `-D warnings` (the gates run `--all-targets -D warnings`; nothing allows them) require the opposite. The report's let-chain argument is false: every crate is edition 2021 and the tree has 0 let chains; there are 65 `else if` in `primitives/src/*.rs` |
| 68, zero technical debt | The shrink-only ratchet model grandfathers by design: 219 lines over 9 baseline files and 57 over 4 allow files in `scripts/`, plus `scripts/lib/ratchet.py` and its grant rule; 62 `#[allow]` and 84 `#[expect]` |
| 67, tooling in the main language | 223 tracked script files under `scripts/` (110 `check-*.sh`), mostly shell and Python; 71 scripts embed Python in a heredoc and 3 compile Rust |
| 24, explicit defaults | `#[props(default)]` is the component idiom (140 `#[props(default = ReadSignal::new(Signal::new(..)))]`) |
| 2, no recursion | 9 candidates by the report's heuristic (not re-counted), all bounded tree walks (cli `lock.rs:92`, `add.rs:224`, `preview/build.rs`); no lint exists |
| 35, 39, 40, 42, acronym case, equal-length names, callback prefixes, item order | Rust convention (the report marks the API-guideline claim "inferred, not fetched"; clippy `upper_case_acronyms`) or cosmetic |

## 4. Output and code quality

**Built output (the deployed wasm is byte-identical to `docs/`).**

| Item | Value | Method |
|---|---|---|
| Main wasm | 20,635,690 B raw; code section 9,999,268 B, data section 10,514,719 B | file size; section walker |
| gzip | 5,734,749 B at `gzip -9`; Pages sends 5,846,847 B | local gzip; request with `Accept-Encoding: gzip` |
| brotli q11 | 3,477,596 B (39.3 % below gzip -9) | `brotli -q 11` |
| JS and CSS | JS 59,139 B; CSS 453,264 B in 5 sheets, identical on 97 of 101 pages | file sizes; `index.html` links |
| Cold load | home 23 requests, 5,972 KiB; dialog page 13 requests, 5,898 KiB | CDP `loadingFinished`, deployed site |
| Unused CSS at load | 88.1 to 92.2 % on component pages; home 78.6 % | `page.coverage`, deployed and local |
| Hydration (navigation start to `data-hydrated`) | deployed 1,078 to 1,574 ms on component pages, home 2,270 and 3,136 ms in two runs; local 401 to 1,222 ms | 6 pages |

**Embedded text.** 649 Rust demo files (`component.rs` and `variants/*/mod.rs`) sit verbatim in the data section, one copy
each, 2,049,994 B; 85 CSS files, 763,439 B (`code!` in `preview/src/components/mod.rs`). Zeroing the embedded source lowers
gzip by 663,325 B (about 0.66 MB, 11.6 %) and brotli by 386,706 B. Another 2.1 MB of the data section is rendered docs
HTML. The report's "4.22 MB of Rust source" is refuted (2.05 MB). `Cargo.toml:47` says `opt-level = "z"` for
`[profile.release]`; the shipped wasm is built at `s` by the profile dx synthesises (static evidence: the literal in the
dx 0.7.9 binary; no live rustc command line was captured).

**No budgets.** The only budgets are the kernel gzip (12,914 of 12,914), `bridge.ts` bytes (61,792 of 61,792), JS volume
(487,932 of 487,932) and the slow-spec total (12,998 s of 17,000 s, a recorded figure of 2026-10-21). Nothing budgets the
wasm size, CSS size, hydration time or memory; the hydration timing specs are deactivated (row 494); row 371 lists
wasm-size budgets as still to build.

**Installed output.** 83 components. The CLI installs each member directory except `["variants","docs.md","component.json"]`:
256 files (173 `.rs`, 83 `.css`), 1,340,450 B; per component min 758 B, median 11,992 B, max 99,477 B (`data_table`).
All 83 `component.json` carry `dioxus-primitives` as a git dependency with no rev, tag, branch or version (the
development form; `scripts/stamp-release-rev.sh` writes the rev at release, row 217). In the 256 installed files (34,666
lines): `row NNN` on 136 lines in 50 components; `dev-docs` on 85 lines in 40 components; `backlog` on 86 in 38;
`scripts/` on 61 in 37; 13 lines mention `dx_repo_lints` in 3 components; 19.8 % of lines are comments.

**Code-quality findings, ranked** (one Opus reviewer; its grades of C+ for architecture down to D for agent-written-code
signs are opinion):
1. **CLI path traversal (high), reproduced.** `cli/src/add.rs:517-523` joins a typed name onto the components root and
   calls `remove_dir_all`; `--force` does the same with the registry's `component.name` (`add.rs:216`); `cli/src/manifest.rs`
   (`Component.name: String`) validates nothing. In scratch projects with a built CLI, all exiting 0: `shadcn-dioxus remove -- ..`
   emptied `src/`; `remove ../../victim` and `remove <absolute path>` deleted sentinel directories; `add --path <registry>
   --force --skip-cargo-add ../..` against a local registry whose `component.json` name is `../../victim` deleted
   `victim/SENTINEL` and installed into `victim/`. All 84 `component.json` names of this registry match `[a-z0-9_]+`.
2. **Non-transactional install.** Order: `ensure_components_module` (writes `mod.rs`), `cargo add` (`add.rs:397`), then per
   component copy files, copy global assets, edit `mod.rs`; `lock.save` last (`add.rs:447`). A failure in component N leaves
   `Cargo.toml` edited, earlier components copied and declared, N copied and undeclared, no lock entry; an overwrite runs
   `remove_dir_all(dest)` before the copy.
3. **A 250 ms close hold for tests.** `use_animated_open` holds every closing overlay in the DOM about 250 ms "so external
   observers (e.g. Playwright polling) reliably see the data-closed element" (`primitives/src/lib.rs:936-943`).
4. **Runtime type panics (read, not run).** `unwrap_or_else(|| panic!(..))` on a value-type mismatch at `combobox.rs:188`,
   `select.rs:202` and `:525`, `selectable.rs:269`, `tag_group.rs:504`; an option's `T` is independent of the root's, so a
   mismatched pair compiles and reaches the panic.
5. **Three gate bypasses (reproduced in a scratch worktree) and a frozen CI.** `check-self-subscribing-effects` exits 0 on
   `use_effect(move || { let v = count.cloned(); count.set(v + 1); })` (the `count()` control exits 1);
   `check-hooks-in-closures` exits 0 for a hook in a helper function, an `if` or a `for`; `check-history-comments` is a
   phrase list ("We switched this from a script to the bridge" and "Earlier this returned None" exit 0). `check-js-listeners`
   matches only `addEventListener\s*\(` (`:85`; not run). `run-gates.sh:3-4` says workflows are `workflow_dispatch`-only
   and `mutants.yml` is the only workflow, so gates run when a person or lane runs them.
6. **Spec timing.** 444 `waitForTimeout` calls in 109 spec files; 494 `20 * 60 * 1000` navigation timeouts; the `app-clock`
   helper is imported by 6 files while 11 files use `page.clock`.

**Corrected or refuted.** The Badge `"padding": true` attribute is read by CSS (`badge/style.css:43`, `.dx-badge[padding="true"]`).
`#[props(default = ReadSignal::new(Signal::new(..)))]` allocates one signal per props construction; `ReadSignal` fields are
excluded from `PartialEq` and compared by their peeked values, so it costs one allocation per render and does not defeat
memoization. `use_previous` writes signals inside a memo (`lib.rs:397-398`), and the memo re-runs once and stops: no loop
(five call sites). The chart pointermove clone is an `Rc` bump.

**`unsafe`.** `Cargo.toml:24` sets `unsafe_code = "deny"` for the workspace. `rg` finds no `unsafe` block, fn, impl or trait
in any tracked `.rs` file. Two files allow the lint only to export symbols: `preview/src/wasm_libc_shim.rs:25-28`
(`#[expect(unsafe_code, ..)]` over `#[unsafe(no_mangle)] pub static mut stderr`) and `decision-kernel/src/ffi.rs:14-17`
(`#![allow(unsafe_code, ..)]` over five `#[no_mangle] pub extern "C"` exports; `dx_input` returns `*mut u8` at `:45` and
`dx_output` `*const u8` at `:56`, raw pointers into the kernel's buffers). The comment at `Cargo.toml:23` names only the shim.

**Worth protecting.** The typed bridge's closed verb tables (`bridge.ts:4-8`); the kernel's bounded decoder (`MAX_DEPTH`
at `value.rs:35`, count checks at `:138` and `:149`, wire keys declared once by `wire_input!`); the two disjoint SSR and
client id namespaces (`dxc-N` and `dxc-cN`, `lib.rs:163-169`); the shrink-only ratchets and their grant rule.

## Not verified

The code-analyser run's wall time (about 10.5 minutes, 331.7 s of it SCIP) is a report figure, not in a ledger; the three
layers it could add (about 50, 120 and 60 lines) are unbuilt estimates; the classes of the 73 rules are opinion. The report
counted 119 gates; the verified figures are 117 entries in `run-gates.sh` and 110 check scripts at `0df1782`. Not run: the
`rust-analyzer` failure, the `check-js-listeners` bypass, the 83-component install smoke, the slider and carousel soak, the
type panics, `use_previous`. Unverifiable: unwrap, expect and panic counts outside tests, and route splitting saving 30 to
50 %. The ledger attributes the 0.66 MB to the Rust files in one line and to Rust plus CSS in another.

## Rows filed

| Row | Title | Existing rows named, not duplicated |
|---|---|---|
| [529](../backlog/529-cli-component-names-validated-no-delete-outside-root.md) | The CLI validates component names and never deletes outside the components root (first to do) | none |
| [530](../backlog/530-add-installs-transactionally.md) | `add` installs transactionally | none |
| [531](../backlog/531-installed-files-carry-no-internal-references.md) | Installed files carry no internal references | pinning: row 217 (done) and `stamp-release-rev.sh` |
| [532](../backlog/532-output-budgets-css-hydration-demo-source.md) | Output budgets for CSS and hydration; optional demo-source step | wasm-size budget: row [371](../backlog/371-dependency-evaluation-keep-lift-drop.md); timing specs: row 494 |
| [533](../backlog/533-tigerstyle-slice-kernel-and-function-length.md) | The TigerStyle slice: kernel assertions and the function-length count | none |
| [534](../backlog/534-overlay-memory-growth-per-cycle.md) | Memory growth per overlay cycle (not scheduled) | none |

Findings without a row, left to the main loop: the 250 ms close hold, the runtime type panics, the three gate bypasses,
the spec sleeps (rows 152, 493 and 495 cover other timing work), the `props.attributes.clone()` form and the lints of
section 2.
