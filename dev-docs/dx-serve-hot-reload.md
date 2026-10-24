# Working with `dx serve`'s hot reload (agent notes)

`dx serve` **does** auto-rebuild on file change; a normal edit needs no kill or restart. The traps below are not about
whether hot reload works but about *knowing when it has finished*, and about test mistakes that look exactly like a stale
build. The scripted wait (`scripts/dev-wait.sh`), measured latencies and per-lane servers are in
[`dev-loop.md`](./dev-loop.md); the lane brief is [`lane-rules.md`](./lane-rules.md) ("The fast loop").

## The traps

**Trap 0: there is no clean "rebuild finished" signal, and HTTP 200 proves nothing.** A plain `curl -w '%{http_code}'`
returning `200` says the process is alive, not that the asset pipeline has finished your edit. During a cold start the
server answered 200 with the body `Err 404 - dioxus is not currently serving a web app`. Tail the server's own log for a
completion line instead, and know which one you are waiting for: only a **cold start** logs
`Build completed successfully in <N>s`; a **watch rebuild** logs `Build completed in <N>s` (no "successfully"), so
grepping the cold-start string after an ordinary edit finds nothing even when the rebuild finished. Release and SSG
builds log `Client build completed successfully!` and `Server build completed successfully!`. If the log shows no new
completion line after a reasonable wait, the watcher may not have seen the change (a file outside its globs, or a
`git stash`/`pop` inside its debounce window); force a real rebuild (see "When you are not sure").

**Trap 3: `Hotreloading:` with no rebuild.** A structural Rust logic change can make `dx serve` log only
`Hotreloading: <file>`, with no `Build completed` line, and silently not take effect: hot patching covers simple edits,
not every structural one, and there is no error, only a line that looks like progress. If the file changed and only
`Hotreloading:` follows after a reasonable wait, do not keep polling: kill the server and start `dx serve` again, then
re-check (found in the 2026-09-18 integration round: the old behaviour kept being served until a restart).
`dev-wait.sh --expect rebuild` turns this shape into exit 3.

**Trap 4: a hot-reload-only patch is invisible to any fresh navigation.** RSX and template hot reload is applied over a
websocket, in memory, to whatever tab is **already open and connected**; it does not update the wasm or js the server
serves from disk. A page opened before an RSX text edit showed the new text; two fresh loads afterwards (one at once, one
a minute later) still showed the old text, so a naive "reload and check with Playwright" reads as a false "the fix did not
apply". After a `hotreload`-classified change either trust it (dx's template hot reload is well exercised for markup and
CSS) or verify against a page that was open before the edit, unless you wait for or force a real rebuild. An edit to an
already-wired asset file (CSS) does reach later loads, but not provably at the instant of the log line: re-check before
concluding it failed.

**Trap 5: rapid saves can give a "successful" but incomplete rebuild.** Several saves within milliseconds (a new file, a
`mod` line, a call site) can split into two overlapping build cycles, and the first `Build completed` line does not
guarantee every file is in that binary; a trailing rebuild can follow. `dev-wait.sh` reports the first match by design, so
after a multi-file edit re-run it once with `--since-line` past the first match: a timeout means done, another `READY`
was the real one.

**Trap 6: near-simultaneous saves of two different files can drop the rebuild entirely.** Two files saved a few seconds
apart (found 2026-09-18/19, about 7 s apart): no `Hotreloading:` line, no `Build completed` line, the process alive and
serving at about 2% CPU indefinitely. **Recovery is a trivial re-save of either file**: the next save rebuilds everything
pending. Telling the siblings apart: trap 5 still logs a completion, only early; trap 6 logs none, at idle CPU (check
`ps`); a wedged server also logs none but burns CPU or answers nothing.

**The build.rs trap: a new component directory.** `preview/build.rs` generates the docs from every component's `*.md`
and `component.json`, and the dx watcher reruns it only when its own bytes change: `touch` is the cargo-side remedy for a
stale `OUT_DIR` (row 92) but was not enough for the dx watcher (two lanes hit it 2026-10-04). **Rewrite the file, not
touch it** (change a comment's bytes) and confirm the new component's docs appear before trusting what the server shows.

## Test mistakes that look like a stale build

1. **Grepping built CSS for un-minified text.** Source CSS has spaces (`.foo { flex-direction: column; }`); the built
   output is minified (`.foo{flex-direction:column}`), so a grep for the spaced form finds nothing even when the rule is
   present. Grep the minified form, or better, query `document.styleSheets` / `getComputedStyle` in a real browser.
2. **`elementFromPoint` after `pointer-events: none`.** A fix that makes an overlay `pointer-events: none` also hides it
   from `document.elementFromPoint`, so you only ever see what is underneath. To ask what covers a point, do a manual
   point-in-rect check on each candidate's `getBoundingClientRect()`.
3. **Reading `outline-color` alone.** The default computed `outline-color` is `rgb(0, 0, 0)` whether or not an outline
   is drawn; read `outline-style` (or another property whose default is not a plausible "changed" value) with it.

Both of the first two produced a false "the fix regressed something" alarm before the cause (the test, not the build) was
found: when a regression appears right after a fix that should be unrelated, suspect the test before the build.

## When you are not sure if what you see is real

From cheapest to most certain:

1. **Re-check after a confirmed `Build completed` line**, not an HTTP 200. Often this alone resolves it.
2. **`git stash` the fix, wait for a confirmed rebuild, test again.** If the symptom reproduces on the baseline it is
   pre-existing; `git stash pop` and move on.
3. **Wipe the dev build's output** (`rm -rf` your own target's `dx/preview/debug/web/public`) and restart `dx serve` if the
   incremental build itself may be confused.
4. **Build the production recipe** (`scripts/build-ssg.sh release`, or `scripts/deploy-preview.sh`) and serve that output:
   a fully recompiled, non-incremental artifact, the closest thing to ground truth here and worth a few minutes whenever a
   dev-server result has already cost you one confusing answer. `deploy-preview.sh` runs its own hydration-bootstrap check
   against a separate, documented `dx build --ssg` cache-warm race. To serve a build at its real base path, symlink the
   `public` directory under that prefix in a scratch directory (`ln -sfn <public> <scratch>/shadcn-dioxus`) and run
   `python3 -m http.server` from `<scratch>`.

## Summary

- Rely on hot reload for normal edits; trust a logged completion line (or a stash A/B, or a production build), never "the
  server responds".
- Know what `READY hotreload` proves (trap 4) and that a lone `Hotreloading:` after Rust logic is not a rebuild (trap 3).
- Rule out your own test method before concluding a fix caused a regression.
