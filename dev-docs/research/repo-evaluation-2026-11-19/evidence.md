# Evidence and reproduction recipes: the 2026-11-19 evaluation

The findings are in [`../repo-evaluation-2026-11-19.md`](../repo-evaluation-2026-11-19.md); this page is how to re-check
each headline finding without the session's scratch directory. Everything was measured on commit `0df1782` (main at that
time) unless a section says otherwise. The ledgers behind every number are in [`ledgers/`](./ledgers/), explained in
[`README.md`](./README.md); the logs and small scripts the recipes name are under [`scratch/`](./scratch/), laid out as
the session's scratch directory was (`scratch/verify-e/soak.js` is `$S/verify-e/soak.js`). A log's first line is the
command that produced it, with the session's scratch paths in it; `$REPO` below is a checkout of the repository at
`0df1782` and `$TMP` is any empty scratch directory.

Reading rule: a number in this page is the number in a ledger line; where a report's figure was partly right or refuted,
the ledger's `actual` is used. "Not run" marks what only a read of the code supports.

## 1. The CLI deletes outside the components root (rows 529, ledger D-1, D-2)

Build the CLI once (`cargo build` in `$REPO/cli`, dev profile, 26.87 s; `scratch/verify-cli-build.log`). Four scratch
projects; every command exited 0.

Setup, with `$CLI` the built `shadcn-dioxus` binary:

```sh
# p1 (remove ..) and p2 (remove ../../victim): a project with a components module
mkdir -p $TMP/p1/src/components $TMP/p1/victim && echo keep > $TMP/p1/victim/SENTINEL
printf '[package]\nname="p"\nversion="0.1.0"\nedition="2021"\n' > $TMP/p1/Cargo.toml
echo 'fn main(){}' > $TMP/p1/src/main.rs && echo 'pub mod x;' > $TMP/p1/src/components/mod.rs
# p2 is the same layout; p3 adds $TMP/p3/victim/SENTINEL and $TMP/abs-3/SENTINEL for the absolute case
# reg: a local registry whose member's component.json carries a hostile name
mkdir -p $TMP/reg/evil
echo '{"name":"reg","members":["evil"]}' > $TMP/reg/component.json
echo '{"name":"../../victim"}'            > $TMP/reg/evil/component.json
echo 'pub fn f(){}'                       > $TMP/reg/evil/mod.rs
# p4: a project (as p2) with a directory victim/ holding SENTINEL beside src/
```

| Command (cwd in brackets) | Before | After |
|---|---|---|
| `$CLI remove -- ..` [`$TMP/p1`] | `src/` holds `components` and `main.rs` | exit 0, `Removed component '..'`; `ls` shows `Cargo.toml  src  victim`, and `src/` is empty (`main.rs` and `components/` gone) |
| `$CLI remove ../../victim` [`$TMP/p2`] | `$TMP/victim/SENTINEL` exists | exit 0, `Removed component '../../victim'`; `ls victim`: No such file or directory |
| `$CLI remove $TMP/abs-3` [`$TMP/p3`] | `$TMP/abs-3/SENTINEL` exists | exit 0, `Removed component '$TMP/abs-3'`; `ls $TMP/abs-3`: No such file or directory |
| `$CLI add --path $TMP/reg --force --skip-cargo-add ../../victim` [`$TMP/p4`] | `$TMP/p4/victim/SENTINEL` exists | exit 0, `Added component '../../victim'`; `ls victim` shows `component.json  mod.rs` (the sentinel is gone, the component installed into `victim/` outside `src/components/`) |

Recorded output: `scratch/cli-remove-dotdot.log`, `cli-remove-relative.log`, `cli-remove-absolute.log`,
`cli-add-force.log`. The scratch layout of the first and last case is as above; the per-project files the session left
are not kept (only the logs). The code that does it: `cli/src/add.rs:517-523` (the `remove_dir_all` after the
join), `add.rs:216` (`--force`), `cli/src/manifest.rs` (`Component.name: String`). All 84 `component.json` names of this
registry match `[a-z0-9_]+` (`scratch/names-check.log`).

Non-transactional install (D-3, read, not run): `ensure_components_module` writes `mod.rs`, then `cargo add`
(`add.rs:397`), then per component copy files, copy global assets and edit `mod.rs`; `lock.save` is last (`add.rs:447`).

## 2. Three gate bypasses (row 537, ledger D-48 to D-50)

Each gate was run on a control that it must reject and on a hostile form that it passes. The recorded runs are
`scratch/gate-*.log` (the first line of each names the scratch worktree). The probe files were not kept; the snippets
below are the minimal forms, and **this lane re-ran them against copies of the three gates (the gates are shell and
Python, no build) with the same output lines as the logs**. The first two gates read `primitives/src/**/*.rs` and
`preview/src/**/*.rs`, tracked or not; the history gate reads tracked files (the session used `git add`), here run
through its checker `python3 -I scripts/comment-scan.py history <file>`.

**`scripts/check-self-subscribing-effects.sh`.** Put the snippet in `primitives/src/probe.rs` and run the gate.

```rust
// control: exits 1
use_effect(move || { let v = count(); count.set(v + 1); });
// hostile: exits 0 (Signal::cloned is a tracked read, so the effect re-triggers itself)
use_effect(move || { let v = count.cloned(); count.set(v + 1); });
```

Control output: ``check-self-subscribing-effects: found signal(s) read with tracked syntax and written in the same
use_effect closure:`` then ``primitives/src/probe.rs:4: `count` is read with tracked syntax here and written at line 4 ...``
(exit 1). Hostile output: ``check-self-subscribing-effects: OK -- no signal read with tracked syntax is written in the
same use_effect closure, and no use_effect calls a method that reads its own field with tracked syntax.`` (exit 0).

**`scripts/check-hooks-in-closures.sh`.**

```rust
// control: exits 1
let m = use_memo(move || { let s = use_signal(|| 0); s() });
// hostile, all exit 0: a hook in a helper the closure calls, a hook under `if`, a hook under `for`
fn make() -> i32 { let s = use_signal(|| 0); s() }
let m = use_memo(move || make());
if flag { let s = use_signal(|| 0); }
for _ in 0..2 { let s = use_signal(|| 0); }
```

Control: ``primitives/src/probe.rs:3: `use_signal(` is called inside the closure passed to `use_memo(` -- move it into
the component body instead (dev-docs/backlog.md row 74)`` (exit 1). Hostile: `check-hooks-in-closures: OK -- no hook
called inside another hook's closure.` (exit 0). The tree itself calls a hook in a loop at `input_otp.rs:403` (D-56).

**`scripts/check-history-comments.sh`.** A phrase list.

```rust
// Previously it returned nil when the id was missing.   <- control: exits 1 ([previously], 0 -> 1, new file)
// We switched this from a script to the bridge.         <- hostile: exits 0
// Earlier this returned None.                           <- hostile: exits 0
```

Hostile output: `check-history-comments: OK (0 baselined history lines in 0 files, none new)`.

Not run: `scripts/check-js-listeners.sh` matches only `addEventListener\s*\(` (`ADD_RE`, D-51). The workflows are
`workflow_dispatch`-only (`scripts/run-gates.sh:3-4`, D-47), so none of the gates runs unless a person or lane runs it
(CI freeze rows 4 and 33).

## 3. Clippy: the allocation lints (row 538, ledger B-24, B-13)

Host target (the wasm arm differs by one `cfg(target_arch = "wasm32")` site in primitives and preview; the `native` and
`server` arms were not run). Clippy 0.1.97 on the pinned 1.97.0 toolchain, the repository's own `clippy-default`
invocation plus 26 `-W` flags (the `perf` group and 25 named lints, in `scratch/wflags`). Exit 0; 7 min 22 s is the
report's figure.

```sh
cd $REPO
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=<absolute isolated dir> scripts/lane-cargo.sh -- \
  cargo clippy --workspace --all-targets --message-format=json -- --cfg dx_repo_lints $(cat dev-docs/research/repo-evaluation-2026-11-19/scratch/wflags) > clippy.json
python3 -I dev-docs/research/repo-evaluation-2026-11-19/scratch/parse_clippy.py clippy.json > parsed.txt
python3 -I dev-docs/research/repo-evaluation-2026-11-19/scratch/verify-b/p.py clippy.json rows.json   # the three counts the verifier used
```

The 26 flags: `perf`, `expect_fun_call`, `or_fun_call`, `redundant_clone`, `needless_collect`, `inefficient_to_string`,
`unnecessary_to_owned`, `format_push_string`, `format_collect`, `manual_string_new`, `implicit_clone`,
`clone_on_ref_ptr`, `large_enum_variant`, `needless_pass_by_value`, `mem_forget`, `str_to_string`,
`trivially_copy_pass_by_ref`, `assigning_clones`, `unnecessary_box_returns`, `rc_buffer`, `box_collection`,
`vec_init_then_push`, `iter_overeager_cloned`, `redundant_allocation`, `useless_format`, `to_string_in_format_args`.

Unique primary sites, all targets including tests (1,529 in total; `parse_clippy.py` output, first lines). Lints that
fired 0 times and are not in the table: the `perf` group members, `expect_fun_call`, `mem_forget`, `assigning_clones`,
`unnecessary_box_returns`.

| lint | primitives | preview | cli | decision-kernel | dioxus-attributes | test-harness | scripts | other | total |
|---|---|---|---|---|---|---|---|---|---|
| `str_to_string` | 407 | 788 | 0 | 0 | 0 | 0 | 19 | 0 | 1,214 |
| `redundant_clone` | 71 | 16 | 0 | 1 | 0 | 0 | 0 | 0 | 88 |
| `trivially_copy_pass_by_ref` | 12 | 45 | 0 | 0 | 0 | 0 | 0 | 0 | 57 |
| `clone_on_ref_ptr` | 39 | 5 | 2 | 0 | 0 | 0 | 0 | 0 | 46 |
| `format_push_string` | 23 | 17 | 2 | 0 | 0 | 0 | 2 | 0 | 44 |
| `needless_pass_by_value` | 34 | 6 | 0 | 0 | 0 | 0 | 1 | 0 | 41 |
| `or_fun_call` | 6 | 11 | 0 | 0 | 0 | 0 | 2 | 0 | 19 |
| `needless_collect` | 3 | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 11 |
| `inefficient_to_string` | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 |
| `format_collect` | 2 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| `manual_string_new` | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| `implicit_clone` | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |

Test versus production split (`scratch/verify-b/log-split.txt`): `redundant_clone` 88 = 11 test + 77 production
(primitives 62, preview 15 in production; the kernel's one is `decision-kernel/tests/vectors.rs:295`); `or_fun_call`
19, none in a `#[cfg(test)]` module (the split counts a test-only file such as `polar_parity.rs` as production). The 19 `or_fun_call` sites: eight combobox variants
(`preview/src/components/combobox/variants/{clear_form:23, controlled:42, custom_items:64, disabled:20, main:23,
multiple:22, multiple_clear:20, multiple_form:22}/mod.rs`, `map_or`), `message_scroller/variants/animation/mod.rs:163`,
`preview/src/site/blocks.rs:489`, `preview/src/polar_parity.rs:559`, `primitives/src/calendar/select.rs:156,354`,
`chart/components/chart.rs:1008`, `toolbar.rs:332`, `virtual/virtualizer.rs:171,225`, and two in
`scripts/attr-spread-collision-checker/src/bin/public-api-snapshot.rs:684,729`. Six do not allocate (for example
`unwrap_or(view_date.month() as u8)`). The eight sites of `redundant_clone` traced by hand to the last use of the cloned
value were true. Counted by grep, not clippy: 237 `props.attributes.clone()` in 83 files, 1,032 `.clone()` in 145,365
non-test lines of the five crates (`scratch/verify-b/log-clone.txt`), 15 `mem::take`/`swap`/`replace`
(`log-memtake.txt`), 0 `.expect(&`, all 195 `.expect(` take literals (B-13: one drift, see the README).

Lines behind `feature = "native"` or `"server"` (heuristic brace matching, `scratch/verify-b/cfg.py`, `log-cfg.txt`): 576
under `native`, 88 under `server` (and 1,851 under `not(native)`, which the default arm compiles). The 664 lines are about
0.36 % of the roughly 184.6 k Rust lines (D-61; that total was not recounted). Whether a workspace deny of the three lints
is feasible: B-24 (the scripts member and the unrun arms add unknown counts).

## 4. The wasm data section (rows 532, ledger E-4, E-8, E-9, E-33)

```sh
cd $REPO/docs
W=assets/preview_bg-dxh92ee26ea6afb8ff3.wasm            # the deployed build; sha256 b41c4b21ef4a32ca...
python3 -I $REPO/dev-docs/research/repo-evaluation-2026-11-19/scratch/verify-e/wasmdata.py $W $TMP     # sections, text runs
python3 -I $REPO/dev-docs/research/repo-evaluation-2026-11-19/scratch/verify-e/wasmmatch.py $W          # verbatim search
python3 -I $REPO/dev-docs/research/repo-evaluation-2026-11-19/scratch/verify-e/marginal.py              # zero them, recompress
```

The scripts hard-code `/home/user/shadcn-dioxus` as the repository path and `/tmp/...` for outputs; edit the constants
at their tops.

- `wasmdata.py` walks the sections (LEB128): code 9,999,268 B, data 10,514,719 B (10,221 segments, 10,431,617 bytes of
  segment data), gzip of the code section 2,973,256 B and of the data section 2,695,882 B. Its classifier by text-run
  density (rust 229,010, css 2,926,235, html 2,108,977, other 427,819 bytes) is a heuristic that folded rendered docs
  HTML into other classes; the matching below replaces it.
- `wasmmatch.py` searches every file `dioxus_code::code!` and `css_highlight!` embed, byte for byte, in the joined data
  segments: 649 Rust files (`component.rs` plus `variants/*/mod.rs`) found verbatim, one copy each, 2,049,994 B, gzip of
  their concatenation 367,758 B; 85 CSS files (`style.css`, `demo.css`, `dx-components-theme.css`) found verbatim,
  763,439 B, gzip 200,977 B. No other `.rs` file of primitives or preview is embedded (226 files, 4,187,008 B, none
  found). The earlier "4.22 MB of Rust source" came from a printable-run scan; the verbatim search gives 2.05 MB.
- `marginal.py` zeroes the matched bytes (the Rust and the CSS files together, same length) and recompresses: zlib-9
  5,731,288 to 5,067,963 (delta 663,325 B, 11.6 %), brotli q11 3,477,596 to 3,090,890 (delta 386,706 B). So the
  "0.66 MB" is the Rust and CSS sources together. Another 2.1 MB of the data section is rendered docs HTML.
- `Cargo.toml:47` sets `opt-level = "z"` for `[profile.release]` (E-10); the wasm is built at `s` by the profile dx
  synthesises (static evidence: the literal in the dx 0.7.9 binary; no live rustc command line was captured).

## 5. Sizes, cold load, hydration, unused CSS (ledger E-1 to E-7, E-17 to E-20, E-32)

```sh
bash scratch/verify-e/sizes.sh     # raw, gzip -9, brotli q11 (node zlib, lgwin 24), JS/CSS per index.html, page sizes
bash scratch/verify-e/sizes2.sh    # zlib levels, the 88 nav-linked pages
bash scratch/verify-e/deployed.sh  # curl the deployed wasm: sha256, content-length, Accept-Encoding: gzip size, 88 pages
node scratch/verify-e/e5.js <base> # six pages: hydration, CLS, console errors, CSS coverage, axe, requests, transfer
```

Results (`scratch/verify-e/sizes.log`, `sizes2.log`, `deployed.log`, `e5-deployed.log`, `e5-local.log`):

| Item | Value |
|---|---|
| wasm | raw 20,635,690 B; `gzip -9` 5,734,749 B; zlib 1/6/9 6,677,767 / 5,761,946 / 5,731,288 B; brotli q11 3,477,596 B |
| deployed copy | HTTP 200, `content-length` 20,635,690, identical sha256 to `docs/`; with `Accept-Encoding: gzip` 5,846,847 B; `cache-control: max-age=600` |
| JS and CSS | `preview-*.js` 59,139 B; CSS 294,339 + 51,868 + 17,197 + 37,080 + 52,780 = 453,264 B; 97 of 101 `index.html` link the identical set |
| HTML | 88 nav-linked deployed pages all 200: min / median / max 121,320 / 292,590 / 1,403,364 B (gzip 16,957 / 38,965 / 110,529 locally); 102 `.html` files, 33,244,043 B |
| cold load (CDP `loadingFinished`, deployed) | home 23 requests, 5,972 KiB; dialog 13 requests, 5,898 KiB; button 5,888; combobox 5,915; data_table 5,930; sidebar 46 requests, 5,974 |
| hydration, nav start to `data-hydrated` | deployed: button 1,107, dialog 1,078, combobox 1,378, sidebar 1,574, data_table 1,372, home 2,270 ms (a second home run 3,136); local 401 to 1,222 ms |
| unused CSS at load | button 89.2, dialog 89.8, combobox 88.8, sidebar 92.2, data_table 88.1, home 78.6 % |
| console errors and CLS | 0 and 0 on all 12 runs (6 pages, deployed and local) |
| axe 4.10.3 | sidebar: `heading-order` (1) and `landmark-unique` (3 nodes) with AxeBuilder over all frames; the main-frame-only injection finds none (`axe2.log`) |

Budgets that exist (E-28 to E-32, `scratch/verify-e/e6b.log`, `jsvolume.log`): kernel gzip 12,914 of 12,914
(`scripts/check-decision-kernel.budgets.tsv:4`), `bridge.ts` 61,792 of 61,792, JS volume 487,932 of 487,932
(`check-js-volume.sh`, run from the repo root, `git status` clean before and after), slow-spec 12,998 s of 17,000 s
(`playwright/durations.json`, 297 entries). Nothing budgets the wasm, the CSS, hydration or memory.

## 6. The soak: memory per overlay open/close cycle (row 534, ledger E-11 to E-16)

Method. A built `docs/` served at `http://127.0.0.1:8741/shadcn-dioxus` by a static server (the server command was not
recorded; its log line is `Serving HTTP on 127.0.0.1 port 8741`; `docs/` must answer under `/shadcn-dioxus/`), or the
deployed site. Headless Chromium through Playwright, one page, one overlay component, CDP session.

```sh
node scratch/verify-e/soak.js <base> <component> <cycles> [every=100] [diff=0]
# component: dialog | popover | dropdown_menu | combobox; the run logs are scratch/verify-e/soak-*.log
bash scratch/verify-e/runsoak.sh <base> <component> <cycles> <tag> [diff]    # writes soak-<tag>-<component>.log
python3 -I scratch/verify-e/soaktable.py                                      # the table and slopes below
```

`soak.js` wraps `WebAssembly.instantiate` and `instantiateStreaming` before the page loads to keep every
`exports.memory`; waits for `html[data-hydrated=true]`; runs three warm-up cycles that are not counted; then a cycle is
a trigger click (combobox: click the input, type `sv`, wait for the listbox, `Control+a`, `Backspace`), `Escape`, wait
until the overlay is not visible. Every 100 cycles it forces garbage collection three times (`HeapProfiler.collectGarbage`,
250 ms apart) and reads: the summed byte length of the wasm memories, `JSHeapUsedSize`, `Memory.getDOMCounters` (nodes,
`jsEventListeners`) and `document.querySelectorAll('*').length`. No open or close wait timed out and no console error
appeared in any run. `scratch/verify-e/soak-noperf.js` is the same with `performance.mark` and `performance.measure`
replaced by no-ops.

Results (`scratch/verify-e/soaktable.log`; wasm MB, JS heap MB after GC, CDP nodes, listeners):

| Run | cycles | wasm MB first to last | slope KB per cycle (all, first half, second half) | JS heap MB | CDP nodes | listeners |
|---|---|---|---|---|---|---|
| dialog, local | 1,500 | 20.25 to 22.94 | 1.75 (1.82, 1.77) | 3.91 to 6.86 | 6,531 flat | 107 flat |
| dialog, deployed | 600 | 20.25 to 21.38 | 1.84 (2.07, 1.67) | 3.97 to 5.41 | 6,531 | 107 |
| dialog, `performance` stubbed | 1,500 | 20.25 to 22.94 | 1.75 (1.78, 1.80) | 3.91 to 5.98 | 6,531 | 107 |
| combobox, local | 1,500 | 26.69 to 30.75 | 2.96 (2.80, 2.80) | 4.23 to 6.91 | 10,605 to 10,936 | 150 flat |
| combobox, deployed | 400 | 26.69 to 27.56 | 2.43 (1.59, 2.87) | 4.15 to 5.18 | 10,605 to 10,936 | 150 |
| dropdown menu, local | 500 | 21.38 to 22.81 | 2.99 (2.63, 3.48) | 4.06 to 4.81 | 9,067 flat | 128 flat |
| popover, local | 500 | 19.31 to 19.44 | 0.25 (0.00, 0.67): flat to about cycle 300 | 3.87 to 5.55 | 5,232 flat | 119 flat |

The report's 5.7 KB (dialog) and 33 KB (combobox) per cycle were not reproduced (3 to 11 times too high); the shape,
continuous growth with the same slope in both halves, was. Slider and carousel were not soaked (E-14).

Control for the node counter (`scratch/verify-e/ctrl.js`, `ctrl.log`): a plain `<input>` page with no Dioxus, same
keystrokes, grows 1.01 CDP nodes per cycle; the combobox page 0.39 per cycle for click only, 1.31 to 1.39 with typing.
The combobox counter rises one per cycle for about 330 cycles, then holds at 10,936 while `querySelectorAll('*')` stays
5,213 (`soak-local-combobox.log`). A 200-cycle diff of every attached node (`cbdiff.js`, `cbdiff.log`) finds only the
option ids `dxc-c0..c6` replaced by `dxc-c14..c20`, net zero. A heap-snapshot diff over 200 cycles
(`heapdiff.js`, `heapdiff-combobox-full.log`) attributes the growth to `blink::UndoStep` (+600),
`InsertIntoTextNodeCommand` (+400), `Text` (+200), `RemoveNodeCommand`, `DeleteFromTextNodeCommand`,
`AppendNodeCommand` (+200 each): Chromium's editing undo stack. The same log shows 37,906 `PerformanceMark` and
`PerformanceMeasure` objects over 200 combobox cycles (9,600 each over 200 dialog cycles, `heapdiff-dialog.log`).

The `logger` feature: `preview/Cargo.toml:8` lists it; `tracing-wasm` 0.2.1 (`src/lib.rs:346` in the cargo registry)
writes `performance.mark` and `performance.measure` per span. `scratch/verify-e/perfmarks.js` counts 677 marks and 677
measures after load, 1,637 and 1,637 after 20 dialog cycles (48 each per cycle); the span names are
`new_maybe_sync` of `dioxus_signals::signal` and `recompute` of `dioxus_signals::memo`. `rg logger` over every
`Cargo.toml` finds only that line.

## 7. Installed output (row 531, ledger E-21 to E-27, D-27, D-29)

```sh
bash scratch/verify-e/installed.sh     # run from a checkout; counts over exactly the files the CLI installs
bash scratch/verify-e/e6b.sh           # per-component installed sizes, component.json pins
```

The installed set is every member directory except `["variants","docs.md","component.json"]` (`cli/src/add.rs:219`; each
`component.json`, for example `preview/src/components/button/component.json:5`), excluding `top_layer/` and `mod.rs`:
83 components, 256 files (173 `.rs`, 83 `.css`), 1,340,450 B, 34,666 lines; per component min 758, median 11,992, max
99,477 B (`data_table`). Over those files: `row NNN` on 136 lines in 50 components; `dev-docs` 85 lines in 40; `backlog`
86 in 38; `scripts/` 61 in 37; `dx_repo_lints` 13 lines in 3 components; `playwright` or `.spec.ts` 50 lines in 19;
`primitives/src` 75 in 31; `preview/src` 18 in 13; comments 3,534 Rust `//` lines plus 3,345 CSS comment lines, 19.8 %
of the lines. All 83 `component.json` carry `dioxus-primitives` as a git dependency with no `rev`, `tag`, `branch` or
`version` (`e6b.log`); that is the development form and `scripts/stamp-release-rev.sh` pins it at release (row 217).
The 83-component install smoke (`scripts/registry-install-smoke.sh`) was not run (E-22).

## 8. Function length, assertions, panics, unsafe (row 533, ledger C-9, C-10, C-11, D-37, D-59)

```sh
cd $REPO && python3 -I dev-docs/research/repo-evaluation-2026-11-19/scratch/verify-ac/scan2.py x   # edit R at the top
```

`scan2.py` is an independent scanner: a real tokenizer (strings, raw strings, lifetimes, nested comments), token-level
`fn` detection, exclusion of `#[cfg(test)]` modules and of the files such a `mod` declares (12 files named in its log:
`calendar/tests.rs`, `test_support.rs`, `chart_parity.rs`, ...), a function counted from its `fn` line to the closing
brace in physical lines, and a function is a component when it carries `#[component]` or returns `Element`. Result
(`scratch/verify-ac/scan2.log`): over 70 lines outside components: primitives 17 (of 1,553; max 203), preview 15 (of
615; max 160), cli 5 (of 97; max 168), kernel 0 (of 39; max 38), `scripts/attr-spread-collision-checker` 2,
`scripts/wrapper-props` 3, `playwright/oracle` 1; 43 in all. Components over 70: primitives 87 of 364 (max 513, 54
over 100), preview 83 of 1,203 (max 962, 30 over 100). Assert-family macros outside tests: primitives 1 (the const
assert at `lib.rs:27`), preview 8; public non-component functions: primitives 946 with 0 assertions, cli 59 with 0,
kernel 20 with 0; two assertions per function would be 2 x 1,917 = 3,834 in primitives. Non-test unwraps in preview 26
(the 140 others sit in test-only parity files), primitives 18 `expect` and 11 `panic`.

The kernel's own lint arm (`scratch/verify-ac/kclippy.log`): a scratch copy of `decision-kernel` with
`too-many-lines-threshold = 70` added to its `clippy.toml` and `#![deny(clippy::too_many_lines, unwrap_used,
expect_used, panic)]` added to `lib.rs`, `cargo clippy --all-targets --message-format=short`: `src/decision.rs:111`
(`unwrap`) and `:129` (`panic!`), both in tests, because the kernel's `clippy.toml` lacks the allow-in-tests switches;
`tests/vectors.rs` could not be built in the copy (`behaviours.json` path), so its 23 `unwrap`/`panic` calls are counted
by reading. Other counts: `let` chains 0 (`letchains.log`); `rustfmt.toml` none (`rustfmt.log`); `pedantic` is not
enabled anywhere (`pedantic.log`); 62 `#[allow]` and 84 `#[expect]` in tracked Rust (`debt.log`, whole tree; per crate
counts in D-37); 223 tracked files under `scripts/`, 110 `check-*.sh` (`scripts_count.log`); `unsafe`: no block, fn, impl
or trait in any tracked `.rs`, two allow sites (`preview/src/wasm_libc_shim.rs:25-28`, `decision-kernel/src/ffi.rs:14-17`).

## 9. The code-analyser run (row 486, row 507, ledger A-1 to A-6)

MentalGear/code-analyser at `374bf4a`, checked out at `/home/user/code-analyser`, run by `scripts/codegraph.sh`.

```sh
rustup component add rust-analyzer --toolchain 1.97.0      # the step it needed; the first two runs failed with
                                                           # "error: Unknown binary 'rust-analyzer' in official toolchain"
cd $REPO && CARGO_TARGET_DIR=<absolute isolated dir> LANE_SLOTS=1 scripts/codegraph.sh
scripts/codegraph.sh smoke                                 # SMOKE PASSED 40/40, 20 s (scratch/verify-ac/smoke.log)
```

`rust-toolchain.toml` lists only `clippy` and `rustfmt`, and `scripts/session-setup.sh` never mentions `rust-analyzer`.
Timing: the run took 10 min 30 s wall (user 8 min 42 s), about 5 min of it queued in `scripts/lane-lock.sh`; the tool's
log says "Generating SCIP finished 331.7s"; the smoke's status line says "614 s cold". These two figures are a report's
and the tool's own, not a ledger line; the status line is in `smoke.log`.

Output (`scratch/verify-ac/sizes.log`, `recount.log`, `scratch/verify-ac/recount.py`): `index.scip` 28,465,582 B,
`graph.json` 407,128 B, `comments.json` 639,074 B, `explorer.html` 1,527,346 B; 1,556 nodes, 264 components, 13,388 edges
(rs 7,077; ext 4,673; asset 205; ts 1,275; test 158); 942 SCIP documents, 14,633 items defined, 59,954 std references
dropped; 0 unresolved workspace references (9,545 resolved by owner, 2,160 by module); 20 duplicate-symbol warnings
(`dupwarn.log`). Comment metrics: 1,705 files; code 131,924, comment 40,229, doc 31,752 lines; 2,638 public items of
which 2,001 documented (75.9 %); with fields 2,350 of which 2,184 documented, so 83.9 % and 803 undocumented are items
plus fields; 322 duplicate runs over 1,404 lines; 2 TODO, 6 stale-name findings.

Accuracy: edges, the report's 13 checked, 12 right; the wrong one is `scripts/attr-spread-collision-checker/src/bin/
public-api-snapshot.rs` to `src/main.rs` (`edges2.log`, `nocrate.log`: the file has no `crate::` use outside test
strings; the two binary targets of one package share the symbol `crate/`). The verifier's own sample of 15 Rust edges:
none provably wrong, 10 file-precise, 5 pointing at `lib.rs`, `mod.rs` or `main.rs`. The miss: `primitives/src/sleep.rs`
to `gloo_timers` (the host-cfg index does not see `cfg(target_family = "wasm")` code; only the `tokio` edge exists).
Comment findings: undocumented 5 of 5 genuine, duplicates 6 of 6 genuine (3 of the 6 are rustdoc example boilerplate),
TODO 1 of 2 (`block_demo.rs:16` quotes another project's TODO), stale names 1 of 6 (`top_layer/component.rs:45`).

Allocation call sites in the index (`scratch/verify-ac/probe.py` decodes `index.scip` into a pickle, `counts.py` counts;
`counts.log`): `format!` 752, `to_string` 1,514, `vec!` 1,483, `Iterator::collect` 706, `String::clone` 412, `Vec::clone`
319, `Option::clone` 137, `Rc::clone` 46, `to_owned` 56, `to_vec` 42; grep finds 758 `format!(` and the text at all 752
index positions starts with `format` (0 mismatches). The tool as shipped detects none of the article's three defects.

## 10. Counts that have no recipe above

- 1,032 `.clone()` in 145,365 non-test lines: `scratch/verify-b/log-clone.txt` (text before the first `#[cfg(test)]`, excluding
  `tests/`, `tests.rs` and `build.rs`, crates primitives, preview, cli, decision-kernel, dioxus-attributes).
- JavaScript volume (D-8 to D-10): by `rg` and by summing
  `scripts/check-js-volume.baseline.tsv`; 75 `document::eval` sites in primitives, 30 in preview, 39 `const *_JS: &str` in
  26 files, `message_scroller.js` 1,065 lines, `bridge.ts` 36 `any`.
- Spec timing (D-39 to D-43): `rg -c waitForTimeout playwright` 444 matches in 109 `spec.ts` files (116 files with
  helpers); `20 * 60 * 1000` 494 occurrences, 463 in the exact `timeout: 20 * 60 * 1000` form; 6 files import
  `app-clock`, 11 files call `page.clock` directly or through it; 297 spec files.
- Dependencies per manifest: `scratch/verify-ac/deps.py`, `deps.log`.

## 11. What did not run

The `rust-analyzer` failure was reproduced by the report and not re-run by the verifier; the `check-js-listeners` bypass
and the type panics (`combobox.rs:188`, `select.rs:202`, `:525`, `selectable.rs:269`, `tag_group.rs:504`) and
`use_previous` are read from the source, not run; the 83-component install smoke, the slider and carousel soak, the
`native` and `server` clippy arms, any counting allocator or heap profile of a drag or a keystroke, and
`scripts/run-gates.sh` on the clippy-denied tree did not run. Unverifiable: unwrap, expect and panic counts outside
tests for primitives and preview (D-59), route splitting saving 30 to 50 % (E-34), the slider and carousel numbers
(E-14), the install smoke's 249 warnings (E-22).
