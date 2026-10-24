# The fast dev loop: edit → wait → inspect → one spec → suite

The recipes for seeing a change working without a full Playwright pass on a shared dev server, and what each step
costs. Numbers were measured on a 4-core / 15 GB box with `dx` 0.7.9, often under load from other lanes, so read them as
upper bounds. Lanes follow the shorter brief in [`lane-rules.md`](./lane-rules.md) ("The fast loop"); this page is the
detail behind it. **Read [`dx-serve-hot-reload.md`](./dx-serve-hot-reload.md) first**: it holds the traps (no clean
"rebuild finished" signal, a hot-reload-only patch invisible to a fresh load, early and dropped rebuilds) that the
script below turns into one wait.

## TL;DR recommended loop

1. **Start the server once**, log redirected to a file you know the path of, on a warm isolated target:
   ```bash
   eval "$(scripts/lane-target.sh create <lane>)"      # exports an absolute CARGO_TARGET_DIR and CARGO_INCREMENTAL=0
   cd preview
   CARGO_BUILD_JOBS=2 dx serve --web --port 8083 > /home/user/tgt/dx-serve-<lane>.log 2>&1 &
   ```
2. **Edit.**
3. **Wait for the real signal, not a guess:**
   ```bash
   scripts/dev-wait.sh 8083 --log /home/user/tgt/dx-serve-<lane>.log --timeout 60
   ```
   It prints `READY cold|rebuild|hotreload <Ns>: <log line>` and exits 0, or a timeout/ambiguous exit (`--help`).
4. **If it reports `hotreload`, know what that proves**: the edit reached tabs that were already open, not necessarily a
   fresh navigation (trap 4 in `dx-serve-hot-reload.md`).
5. **Inspect directly, in 1-2 seconds against a warm server:**
   ```bash
   node scripts/inspect.mjs "http://127.0.0.1:8083/component/kbd/" --axe
   ```
6. **Run the specs that cover your diff** on a prerendered build, never the whole suite (next section).
7. **The full suite runs once per merge batch** (chunked, `playwright/tools/run-suite.sh`) and once per deploy on a release
   build ([`lane-rules.md`](./lane-rules.md), "Running the suite"); a lane never runs it.

A plain `curl` proves nothing more than "the process is alive": during a cold start a dev server answered HTTP 200 with
the body `Err 404 - dioxus is not currently serving a web app`.

## Running tests: the specs that cover your diff

After the edit loop, verify on a prerendered build (first paint and hydration are what the specs test), with the
diff-based run, never a hand-picked list and never the whole suite:

```bash
cd playwright && PLAYWRIGHT_BASE_URL=http://127.0.0.1:<port> pnpm run test:changed -- --workers=2
node tools/select.mjs --explain        # which specs cover the diff, and why
```

`test:changed` (`playwright/tools/test-changed.sh`) runs what `tools/select.mjs` maps your diff to (default range
`origin/main...HEAD` plus uncommitted files; `CHANGED_RANGE=A..B`, `CHANGED_MATRIX=1` for the route matrices). Build and
serve with `scripts/lane-ssg.sh` (lanes) or `scripts/build-ssg.sh debug` plus `scripts/serve-spa.py` (README, "Testing
changes"). Every gate before every commit is `scripts/run-gates.sh`.

- **Functional specs may run on debug SSG builds; timing specs only on release.** `oracle/tier2-html/main-thread.spec.ts`
  (and any latency threshold) gives false reds on debug: long tasks of 64-153 ms on debug against 0 on release (row 116).
  The Playwright configs ignore main-thread on known-debug builds (`PW_BUILD_PROFILE`, read off the build when unset;
  `PW_FORCE_TIMING_SPECS=1` overrides), so a green run on debug says nothing about timing.
- **One spec by hand.** Playwright's positional file arguments are substring matches against every spec path:
  `pnpm exec playwright test select.spec.ts` also runs `native_select.spec.ts`. Anchor it (`./select.spec.ts`, or a path
  with a `/`) or filter by title with `--grep`/`-g <regex>`; `--repeat-each <N>`, `--reporter=line` and `--grep-invert`
  are the other flags used here.

## Rebuild latency by edit class (plain `dx serve --web`, debug, warm server)

| Edit class | Log signal | Latency | On a **fresh** page load? |
|---|---|---|---|
| RSX text in a demo | `Hotreloading: <file>` only | ~0.5-0.8 s | No: only tabs already open (trap 4) |
| Component `style.css` | `Hotreloading: /src/components/<name>/style.css` | ~0.8 s | Yes (see the CSS section) |
| Rust logic in a primitive | `Build completed in Ns` | ~40 s | Yes |
| New function, module or call site (several saves) | a first `Build completed` that may not yet hold the change, then a trailing one | ~85 s end to end | Only after the second build (trap 5) |
| New asset file plus its `asset!()` link (a `.rs` edit) | full rebuild | ~30 s | Yes |
| Content-only edit of an already linked asset | `Hotreloading: /assets/<file>` only | ~0.9 s | Eventually; budget a little slack |

A cold build into an empty target is ~4 minutes and 2.1 GB (233 s measured), which is why lanes copy a warm target
(`scripts/lane-target.sh create`) instead of building cold. `--web --release` multiplies compile time (`lto = true`,
`codegen-units = 1`, `opt-level = "z"`) and is not an iteration mode.

## CSS root cause, corrected (backlog rows 72/76, 80)

A component stylesheet edit used to cost a full rebuild (~30 s), identical to a Rust change, although every sheet is
linked through `asset!()`. The cause was not the directory but a compile-time embed: `examples!` rendered each Style tab
with `dioxus_code::code!()`, which expands to `include_str!(<the .css>)`, so rustc listed the stylesheet in the
artifact's dep-info and dx's file-change handler (`needs_full_rebuild` when a changed file is in that list) overrode the
asset hot-reload path. `preview/build.rs`'s blanket `rerun-if-changed=src/components` compounded it. The construction: in
debug builds (`#[cfg(debug_assertions)]`, the `dx serve` path, never prerendered) the Style tab fetches the sheet's own
`asset!()` URL at runtime (`LazyCssCodeBlock`) instead of embedding it, and `build.rs` watches only the `*.md` and
`component.json` files it reads; release and SSG output is unchanged. A brand-new `.md` or `component.json` is not watched
until `build.rs` reruns (see "the build.rs trap" in `dx-serve-hot-reload.md`). Full account and the isolation
measurements: [row 80](./backlog/done/080-css-hot-reload-root-cause-a-compile-time-embed.md).

## `dx serve --hot-patch`

Not usable in this workspace. RSX edits behave as in plain mode, but the first Rust logic edit (in `primitives/`, a very
common edit here) fails with `ERROR Build failed: Missing rustc args for replay: 'dioxus_primitives'` and every later
edit, even in `preview/` alone, fails the same way for the rest of that server's life. The failure is loud (no
completion line, `dev-wait.sh` times out), unlike the silent trap 3. Stay on plain `dx serve --web`: template hot-reload
for markup, full rebuilds for logic.

## A trustworthy "my change is live" signal: `scripts/dev-wait.sh`

`dev-wait.sh <port> --log <file>` snapshots a baseline line of the server log, polls for the next matching line with a
bounded timeout and reports `READY cold|rebuild|hotreload`. The log already tells a cold start (`Build completed
successfully in Ns`) from a watch rebuild (`Build completed in Ns`) and a hot-reload-only patch (`Hotreloading: <file>`);
`--expect rebuild` makes a lone `Hotreloading:` after a Rust logic edit exit 3 (trap 3) instead of false-succeeding.
After a **multi-file** edit, run it once more with `--since-line` past the first match: a timeout means the first
`READY` was the real one, another `READY` means a trailing rebuild carried the change (trap 5). Evaluated and not used as
the signal: dx's `/_dioxus/build_status` websocket (`Building` progress, `Ready`), which sends nothing for a second
hot-reload-only edit in a row, and a build-stamp `<meta>`, which `build.rs`'s narrow `rerun-if-changed` would leave stale.

## Direct inspection: `scripts/inspect.mjs` and `scripts/measure-scroll.mjs`

```bash
node scripts/inspect.mjs "http://127.0.0.1:8083/component/kbd/"
node scripts/inspect.mjs "http://127.0.0.1:8083/component/kbd/" --dark --screenshot /tmp/kbd-dark.png
node scripts/inspect.mjs "http://127.0.0.1:8083/component/dialog/" --click 'button:has-text("Open")' --axe
node scripts/inspect.mjs "http://127.0.0.1:8083/" --eval 'return document.querySelectorAll(".dx-component-card").length'
```

It drives `playwright` from `playwright/node_modules` without a test runner (flags: `--variant`, `--dark`, `--viewport`,
`--click`/`--press` in the given order, `--eval`, `--axe`, `--screenshot`, `--root`, `--timeout`; the script's header has
the list). A fresh worktree needs one symlink so Node's ESM resolver finds the packages: `ln -s playwright/node_modules
node_modules` from the worktree root (gitignored). A warm run is ~1-1.5 s for a DOM outline, ~4.7 s with click, press,
axe and screenshot together.

- Playwright 1.60 has no `page.accessibility`; the ARIA tree comes from `locator(root).ariaSnapshot()`.
- `@axe-core/playwright` needs a page from an explicit `browser.newContext()`, so the script always creates one.
- Readiness is `#main` having a child element, not every stylesheet loaded: check CSS after the first navigation with
  care. Reading `outline-color` alone is a false negative (the computed default is `rgb(0, 0, 0)` whether or not an
  outline is drawn); read `outline-style` too.

`scripts/measure-scroll.mjs` is the scroll-jank tool: for one URL it inventories every scroll, resize, wheel and touch
listener, observer, interval and layout read, measures an idle window and a wheel scroll top to bottom with CDP tracing,
and bisects suspects by switching them off (its header has the usage; it needs the same `node_modules` symlink).

## Per-lane servers

Each lane runs one server of its own, fully isolated:

```bash
eval "$(scripts/lane-target.sh create <name>)"        # a warm copy of the template, absolute CARGO_TARGET_DIR
cd preview && CARGO_BUILD_JOBS=2 dx serve --web --port 808<3+N> > /home/user/tgt/dx-serve-<name>.log 2>&1 &
echo $!                                                # keep the PID
```

- **Port:** 8083 and up, one each; pick one no other lane has claimed.
- **Cost:** a copied warm target is a few GB per lane; check `df -h /` before starting many (`scripts/lane-target.sh
  prune` lists the copies). With little free disk or memory, verify on one `lane-ssg` build instead.
- **Stopping:** kill your own server **by PID**, never `pkill -x dx` (it killed an unrelated lane's server, row 72).
- **Pointing Playwright at a non-default port:** every spec takes its base URL from
  [`playwright/base-url.ts`](../playwright/base-url.ts), `process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080"`;
  set `PLAYWRIGHT_BASE_URL` and use the local configs (`ssg.local.config.ts` for a static server of the SSG output,
  `session.local.config.ts` or `baseline.local.config.ts` for a running `dx serve`).

## Computed-style snapshot: hover and open states

`playwright/computed-style-snapshot.spec.ts` (a tool, skipped unless `DX_SNAPSHOT=1`; usage and env in its header) is the
"did any computed value change at all" proof for value-preserving CSS work; a capture from before and one from after the
change are compared with its `compare` mode. Its states are deltas, implemented in `playwright/computed-style-states.ts`:
settle, record a signature of every in-scope element, put the page in a state with REAL input (`page.mouse` and
`keyboard`, so `:hover` genuinely applies), and record every element that is new or whose computed style changed (plus
`::before`, `::after`, `::placeholder`, `::marker`, `::backdrop`). Nothing knows what a dialog is: a `<dialog>`, a
`[popover]`, content mounted into the page and a class flip all land in the same delta.

- **`open`**: triggers are derived (`aria-haspopup`, collapsed `aria-expanded`, `popovertarget`, `summary`, tooltip
  `aria-describedby`, `*-trigger` classes, the first focusable inside a `data-state=closed` root; on the plain-button
  pages every button); each is tried with click, right-click, hover and focus until the page changes, each distinct item
  inside (at most 8) is hovered, then the overlay is closed and VERIFIED closed. **`hover`**: every distinct interactive
  control of the demo frames (at most 60 per page) and every chart plot. **`focusK`**: the focus pass repeated with
  `<html data-dx-modality="keyboard">` forced; `open:<k>/kbd` applies the on-screen-keyboard contract by hand.
- **Determinism is the acceptance test** (two captures of one build are byte-identical), and it is built in: a timer
  drain instead of guessed delays (`movePointer` wraps `setTimeout` and waits until every timeout scheduled by a pointer
  move fired, so a tooltip with any configured delay opens), `quiesce` before the next baseline (closing is not closed;
  a strategy counts as opening only when something was ADDED), the pointer parked after a click, `content-visibility`
  cards laid out for real, Google Fonts requests aborted, the skip link excluded from the item pool, and the document
  reported hidden so every `use_motion`-gated timer stops.
- **Coverage is reported, not asserted**: `<OUT>.coverage.json` lists per page the triggers tried and opened, the ones
  that did nothing with the reason, items hovered and skipped. Read it before trusting a green diff. Not covered:
  `:active`, `::selection`, scrollbars, keyboard-driven highlight, ordinary toggles, the block-demo iframes.
- **Cost and files**: 3 parallel contexts (`DX_SNAPSHOT_PARALLEL`), ~16 min on an idle 4-core box for both schemes; a
  capture of both schemes is written as one file per scheme (`CSS_SNAPSHOT_OUT=/x/N.json` gives `N.light.json` and
  `N.dark.json`, Node's string limit forbids one file) and `compare` takes the same prefix on both sides and also writes
  `<B>.diff.json`. Serve the build under test from a path with no prefix (`python3 -m http.server <port> -d .../public`)
  or pass `PLAYWRIGHT_BASE_URL`; capture before and after from the same build profile.
- **Element-level buckets**: `scripts/snapshot-classify.mjs <B>.diff.json --rules rules.json` explains every difference by
  a rule that names an element (`cls` and/or `page`) or an exact value transition (`prop` + `from` + `to`; a pixel
  tolerance `maxPx`, capped at 16, needs `cls` AND `prop`), reports unused rules and groups the unexplained ones by
  element (exit 1); `--self-test` proves a 1 px shift is not hidden by a rule for another class or property.
