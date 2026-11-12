# Decision kernel proof (row 508): verdict KERNEL

Date: 2026-11-12. Branch `lane/decision-kernel-proof` (proof code only, not merged); head of the proof code
`0739d919`. This note and the row's Status are the only parts that land on main.

**Verdict: `KERNEL`.** All five pass lines hold on the one behaviour that exists (the test fixture). The pure Rust
`decide`, compiled to a 25,700-byte wasm module, is called synchronously by the bridge's capture-phase, non-passive
listener inside a real keydown's dispatch in Chromium on a release SSG build, cancels the default action from its
answer, and is ready before the eval that carries it returns. Nothing here is proven for a desktop webview or Blitz
(see "Open"). Two findings change what row 509 must plan for: the gate fallout is large and mostly structural
(section "Gates"), and the JS-volume grant text of row 509 does not hold by measurement (section "What row 509
should take").

Measured and inferred are kept apart: every number below was measured on this box unless a sentence says
"inferred".

## The five pass lines

| # | Threshold | Measured | Command |
|---|---|---|---|
| 1 | A capture-phase, non-passive listener calls the kernel synchronously on a release SSG build in Chromium and cancels from its answer; the fixture's existing tests pass unchanged against the kernel arm | Holds. On the release SSG build of this branch the page's own transport loaded the kernel (state `ready`, mode `sync`); a real `keyboard.type("abc")` into an input with the behaviour installed left the value empty and sent 3 messages. `decision-kernel.spec.ts` plus `bridge-dom.spec.ts`: 22 of 22 pass in Chromium 153.0.8010.12. `bun test primitives/ts-test`: 82 pass, of which `behaviours.test.ts` is byte-for-byte unchanged and now runs the shared vectors through the wasm module. | `scripts/lane-lock.sh lane --label ssg-508 -- env CARGO_TARGET_DIR=/home/user/tgt/ssg-508 BUILD_SSG_NETNS=1 scripts/build-ssg.sh release`, then `scripts/emit-404.sh <public>; python3 scripts/serve-spa.py <public> 4852`, then `cd playwright && PLAYWRIGHT_BASE_URL=http://127.0.0.1:4852 flock /home/user/tgt/pw.lock pnpm exec playwright test --config=ssg.local.config.ts decision-kernel.spec.ts bridge-dom.spec.ts --workers=2`; `cd primitives && bun test ts-test` |
| 2 | Kernel absent or its load blocked: the behaviour stands down and no input is trapped | Holds. Never loaded, blocked by `script-src 'unsafe-inline'` (a `CompileError` naming the missing source; state `failed`, one console warning), garbage bytes, a module that is not a kernel, a wrong ABI, a trap in the kernel (the kernel stands down for good), an unknown kind, a malformed request: each answers `{}`; typing into the input is unchanged and no event is cancelled. | same spec (describe "the kernel absent"), `primitives/ts-test/kernel.test.ts` |
| 3 | Two clean builds on one toolchain give identical bytes | Holds. Two clean target directories, built from two checkouts at different paths, both sha256 `a557cfa6ade470a57fb2ebb2202f75bedf32493e03eb17a2ba5f6f987da524ee`, equal to the committed module. Toolchain: `rustc 1.97.0 (2d8144b78 2026-07-07)`, LLVM 22.1.6, host x86_64-unknown-linux-gnu. | `CARGO_TARGET_DIR=<clean dir> scripts/build-kernel.sh --out <dir>` in each checkout (via `scripts/lane-lock.sh`), `sha256sum` |
| 4 | Kernel with the fixture alone under 50,000 bytes uncompressed | 25,700 bytes (gzip -9: 10,637). As the base64 text the transport evaluates: 34,268 bytes (gzip -9: 14,268). | `wc -c primitives/src/kernel/decision-kernel.wasm`; `gzip -9c <file> \| wc -c` |
| 5 | Kernel call p99 under 0.5 ms over 10,000 synthetic events | 0.045 to 0.075 ms in 14 runs of 10,000 cold-start events (0.075 once, 0.045 to 0.060 in the other 13; p50 0.010 ms), 7 to 11 times under the line. The timer is the 5 microsecond one of a cross-origin-isolated page. | `playwright test --config=ssg.local.config.ts decision-kernel.spec.ts -g "10,000" --workers=1` (prints `KERNEL-COST {...}`) |

Pass line 5 holds, but the tail is not clean (see "Reported numbers").

## Reported numbers

**Time from install to ready (measured).** On the served release page the kernel's own clock reads 1.1 ms from the
start of its load to ready (base64 decode, synchronous compile, instantiate), inside the same eval as the bridge. On a
blank page the same step takes 2.0 to 6.9 ms in the cost test's runs (and a median of 4.2 ms, range 2.7 to 18.2, over 15 fresh pages). Because the
compile is synchronous the kernel is ready when the eval returns: no event can precede it.

**How the bytes reach the page and what it costs (measured).** The module is base64 text in the Rust crate
(`primitives/src/kernel/decision-kernel.b64`, `include_str!`) and rides in the same `document::eval` as `bridge.js`
(`transport::ensure_installed`). Against main's release build the client wasm `preview_bg` grows 48,325 bytes raw
(20,141,641 to 20,189,966) and 16,870 bytes gzip (5,595,971 to 5,612,841), about 0.3 percent; the kernel's base64 is
34,268 of that and the rest is the bridge growth and the eval string. `bridge.js` grows from 19,956 to 24,226 bytes
(+4,270; gzip 6,632 to 8,074), all of it the marshalling codec and the loader; the fixture's bundle shrinks from 553
to 538 bytes. There is no extra request.

**CSP (measured).** The kernel needs `'wasm-unsafe-eval'` (or `'unsafe-eval'`, which includes it). Under
`script-src 'unsafe-inline'` the compile throws `CompileError: ... violates the following Content Security policy
directive because 'unsafe-eval' is not an allowed source`; with `'wasm-unsafe-eval'` added the same spec loads and
cancels. On the web the bridge itself is evaluated with `document::eval` (a `new Function`), which already needs
`'unsafe-eval'`, so a page that runs the bridge today is not asked for a new source (inferred from that existing
requirement, not tested on a page under a strict policy).

**Marshalling.** One request array `[config, memory, snapshot, event]` and one answer object cross as bytes in a
JSON-shaped encoding that the crate defines (`decision-kernel/src/value.rs`: a tag byte for null, false, true, f64,
string, array, object; `u32` little-endian lengths and counts; no dependency, decoding never panics). The page writes
the request into the kernel's input buffer (`dx_input`), calls `dx_decide_<kind>(len)`, and decodes the answer from
`dx_output()`. The fixture's request is 175 bytes and its answer 112. In Chromium the whole call (encode, copy, call,
decode) has a median of 10 microseconds. Natively (release profile) the pure `decide` is 126 ns at the median (p99
210 ns) and the byte route 1.4 microseconds (p99 1.7).
- Scaling (inferred): cost is linear in the bytes marshalled. An OTP filter sends a short value and the inserted
  data, a few hundred bytes; roving focus sends item states (one byte each in this encoding, a 1,000-item list is
  about 1 to 2 KB). Both stay in the tens of microseconds. A frame-rate stream (pointermove) calls the kernel once per
  event, so 10 microseconds is far inside a frame. The config is re-sent on every event in this proof; installing it
  once per behaviour and passing a handle would cut a third to a half of the request (inferred, not built).
- Size per behaviour (measured by a scratch experiment, not committed): adding an OTP-shaped filter and a
  roving-shaped function to a copy of the crate took the module from 25,700 to 29,411 bytes (+3,711, gzip +1,468),
  about 1.9 KB per behaviour. Most of the 25.7 KB floor is Rust's std (the allocator and the panic and format
  paths, visible as strings in the module). Inferred: the 50,000-byte line leaves room for about twelve behaviours of
  this size, and a `no_std` build with its own allocator would lower the floor a lot (not tried).

**The tail (measured, not attributed).** The machine was heavily loaded throughout (load average 5 to 10 on 4 cores,
other lanes building). The kernel call's p999 was 0.4 to 15 ms and its maximum 4 to 40 ms; the whole listener
dispatch's p999 was 0.2 to 10 ms. A control loop of plain arithmetic with the same iteration count showed single
stalls of 7 to 16 ms in the same runs, so the machine alone produces stalls of that size; but the kernel call's p999
is far above that control's p999 (0.03 to 0.1 ms), so some of the tail belongs to the call. Candidates (inferred, not
isolated): young-generation garbage collection from the per-call allocations (the stalls recur at the same call
indices from run to run), or wasm tier-up. A first attempt (reading the answer through cached views, no closures)
did not change it. Row 509 should re-measure on a quiet box before it treats the p999 as a number.

## The design

**Crate.** `decision-kernel/`, a workspace member with no dependencies, library kinds `cdylib` and `rlib`. No `dioxus`
or bridge type appears in any signature: `fixture::decide(&Config, &Value, &Snapshot, &Event) -> Decision`, typed
inputs read from the wire `Value`. `wire::decide_fixture(&[u8]) -> Option<Vec<u8>>` is the byte route both the wasm
export and `cargo test` call. `ffi.rs` (wasm only) holds the plain `extern "C"` exports (`dx_abi`, `dx_input`,
`dx_output`, `dx_decide_<kind>`) over two thread-local byte buffers; its `#[no_mangle]` needs a module-level
`#![allow(unsafe_code)]` with a reason, and no `unsafe` block exists. A malformed request is `u32::MAX` (the page
stands down) and never a panic.

**Native.** `cargo test -p decision-kernel` runs the same vectors (`primitives/conformance/behaviours.json`) against
the Rust function and against the byte route, plus malformed-input, round-trip and nesting tests (5 tests; no
dependency, the vectors are read by a 60-line JSON reader in the test). That is the Blitz arm's call (a direct call of
`fixture::decide`); `dioxus-primitives` does not depend on the crate in this proof.

**Page half.** `ts/lib/kernel.ts` (bundled into `bridge.js`, exposed as `window.__dxBridge.kernel`): `load(source)`,
`decide(kind, behaviour, event, snapshot, memory)`, `state`, `stats`. `lib/register.ts` adds `kernelDecide(kind)`;
`ts/behaviours/fixture.ts` is now its fields, its reads and `decide: kernelDecide("fixture")`: it holds no decision.
The bridge's handler is unchanged: it picks the event fields and the attribute reads, calls `impl.decide`, applies
`prevent` only when Rust declared the behaviour non-passive.

**Load path (the crux).** `ensure_installed` evaluates `bridge.js` followed by `kernel.load("<base64>")` in one eval.
`load` decodes the base64 and calls `new WebAssembly.Module(bytes)` synchronously, then instantiates; so the kernel is
ready before the eval returns, before any component can install a behaviour, and before any event. If the engine
refuses a synchronous compile (a `RangeError`, the main-thread size cap of some engines) the asynchronous compile takes
over and every behaviour stands down until it finishes (`kernel.state` is `loading`). Chromium 153 accepted a
synchronous compile of a module of 8 MB, so the cap is not a concern here (measured); other engines are open. Any
other failure (blocked by CSP, not a module, wrong ABI) is `failed`, one console warning, no retry.

**Fail open.** `decide` answers `{}` unless the state is `ready` and the module exports the kind; a trap in a call sets
the state `failed` for good (the module's state after a trap is unknown); an encode error (a value nested past 16)
throws to the bridge's existing handler, which does nothing and lets the event go on. A behaviour's lease and the
bridge's other fail-opens are unchanged.

**Rejected or measured alternatives** (fresh page each, 15 runs, from the start of the call to ready, in-page clock):

| Route | min / median / max (ms) | Verdict |
|---|---|---|
| inline base64, synchronous compile (chosen) | 2.7 / 4.2 / 18.2 | ready in the loading eval; no gap |
| inline base64, `WebAssembly.compile` | 2.2 / 4.7 / 19.5 | about the same time, but an event loop turn later: a gap in which a cancelling behaviour is silently off |
| separate file, `fetch` then synchronous compile | 4.4 / 13.3 / 52.4 | a request per webview, a gap, and the file must be served on every renderer; the numbers are for a route-fulfilled local response, a real network adds its round trip |
| `fetch` + `WebAssembly.instantiateStreaming` | 4.2 / 11.3 / 22.0 | the same request and gap, plus a MIME requirement (`application/wasm`) every static host must meet |

`wasm-opt` is not on the box and was not used (the size above is the plain linker output at the workspace's `opt-level = "z"`, fat LTO, stripped).

Also rejected: `wasm-bindgen` and any generated glue (forbidden by the row; the plain exports need none); JSON across
the boundary (needs a JSON parser in the kernel, bytes and time for nothing); a kernel call per kind through a numeric
id (per-kind exports are simpler and let the linker drop what no kind uses). The loading eval carries the bytes as
base64 because `document::eval` takes text; a binary asset would need the asset pipeline row 434 rejected for the
bridge (a provider in every app, a head effect that can run after the eval).

## Gates

`CARGO_TARGET_DIR=/home/user/tgt/check scripts/run-gates.sh --fast` on the branch before the fixes below: 104 gates,
5 red, `--fast` skips the cargo gates (clippy, test, rustdoc). After the fixes: `gates run: 104   red: 3   total: 221.4s`, `RED: check-bridge-behaviours check-js-volume check-slow-spec-budget`. No gate script, baseline or
allowance was edited; the proof does not need one. Every remaining red is a finding row 509 owns, and each is caused by the proof's shape:

- `check-bridge-behaviours` (7 problems): (1) `ts/behaviours/fixture.ts: no decide(`: the gate requires a `decide` in
  every behaviour module, the opposite of row 509's rule; (2) byte budgets rose above their ratchet: `bridge.ts`
  42,732 to 42,784, `lib/register.ts` 777 to 1,262, `types/bridge.d.ts` 6,111 to 6,940, and `fixture.ts` fell 891 to
  656 with the baseline still at the old number; (3) the `document::eval` allowlist names the argument of the eval in
  `ensure_installed` as exactly `BRIDGE_JS`, and the sink count of `transport.rs` as three: the bridge-plus-kernel
  eval is a new expression.
- `check-js-volume` (4 problems): `decision-kernel/src/ffi.rs` is a new file with sink count 4 and `js_lit` 1 (every
  `extern "C"` is a sink by that gate's rule, so the kernel's exports are counted as JS entering; the crate needs an
  exclusion row or its own rule); `transport.rs` `js_lit` 6 to 8 (the format string with the kernel call);
  `primitives/src/js/bridge.js` bytes 19,956 to 24,226; `fixture.js` fell 553 to 538 with the baseline at the old
  number. The base64 text is not counted (the gate counts `.js`, `.ts` and `.html` includes), so the kernel's 34 KB
  is invisible to it; whether it should count is row 509's decision.
- `check-spec-goto-hydrated`: the spec's isolated probe page used a bare `goto`; fixed here with `gotoReference`.
- `check-slow-spec-budget`: red (not fixed on the branch): a new spec needs a measured entry in
  `playwright/durations.json`. The entry is one command (the gate's header gives it) but it needs the spec run once
  under the shared Playwright lock, which another lane's full-suite run held for the rest of this lane's time.
- `fmt`: the new Rust was unformatted; fixed here.
- `check-ts-scan` passes with no baseline change, but only after the codec was written with `DataView` byte access and
  `Object.fromEntries`: a byte codec indexes a buffer with a computed key (`buf[i]`), which the scan's
  `computed-access` rule rejects, and it rejects `Object.defineProperty` with a computed key. Row 509 should expect
  this for any codec in `ts/lib/`.
- Not run: `scripts/registry-install-smoke.sh` (row 509's proof list names it; the decision crate would change what an
  installed component depends on), `cargo clippy` and `cargo test` of the workspace (skipped by `--fast`). The release
  SSG build compiled `dioxus-primitives` with the new transport.

## Open

- **The desktop webview (row 23).** No desktop build exists, so loading the kernel inside it is not tested. The one check
  a desktop build must make: after the bridge's eval, `window.__dxBridge.kernel.state` is `"ready"` and
  `kernel.stats.mode` is `"sync"` (or `"async"` followed by `"ready"`), in WKWebView, WebView2 and WebKitGTK, with
  the webview's own Content-Security-Policy (the origin of a custom protocol), through the `evaluate_script` path that
  carries a 34 KB string. If an engine only allows the asynchronous compile, cancelling behaviours are off until it
  finishes (fail open, not wrong), and row 509 must say whether that is acceptable for a first keystroke.
- **Blitz** runs no JavaScript: the kernel is not loaded there; the direct call of the same function is the route
  (`fixture::decide`, proven natively). Wiring `dioxus-primitives` to call it is row 509.
- **Other engines on the web** (Safari, Firefox): only Chromium was run (the only browser installed).
- The tail of the per-call time (above) needs a quiet-box measurement.
- Not decided here: whether config should install once per behaviour (a handle) rather than travel with every
  event; whether a `no_std` kernel is worth its extra unsafe code.

## What row 509 should take

1. **The proof's shape is the contract's shape.** `decide(config, memory, snapshot, event) -> Decision` as one crate,
   one wire `Value`, `dx_abi` + `dx_input` + `dx_output` + `dx_decide_<kind>`, the page half in `ts/lib/kernel.ts`,
   `kernelDecide(kind)` in the behaviour module, `kernel.load` in the transport's first eval. Take the
   branch's files as the starting point (they are not merged).
2. **`scripts/build-kernel.sh` and `kernel.manifest`** (toolchain from `rustc -Vv`, sha256 of every source and of the
   module) already do what `check-decision-kernel` needs; the gate's rebuild half should compare bytes only when
   `rustc -Vv` equals the manifest's. The module embeds no checkout path (std's panic locations name
   `/rustc/<commit>/...`), so only the toolchain matters. A freshness gate needs a pin of the exact rustc commit
   `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3` (1.97.0); the repo has no `rust-toolchain` file, and the wasm
   target must be installed. `build-kernel.sh` unsets `RUSTFLAGS`, so a user's flags cannot change the bytes.
3. **The grant in row 509's front matter is wrong by measurement.** It says `js_file=0` because the loader's lines are
   paid for by the fixture's deleted `decide`. The deleted TypeScript `decide` was 15 bytes of bundle (553 to 538);
   the loader and codec are 4,270 bytes of `bridge.js` (19,956 to 24,226) and 9,437 bytes of `ts/lib/kernel.ts`. The row
   needs a `js_file` grant of about 4,300 bytes, or the codec has to be made smaller (the encoder and decoder are
   about 130 lines; a smaller wire format is possible), and the budgets of `bridge.ts`, `lib/register.ts` and
   `types/bridge.d.ts` move too (list above).
4. **Gate work list**: the two inverted rules of `check-bridge-behaviours` (a `decide` in a behaviour module is red,
   a Rust function per kind is required); the eval-site allowlist; the `extern "C"` exemption for the decision crate in
   `check-js-volume` and the decision whether the base64 counts; the byte ratchets.
5. **Size and time budget.** Plan for about 2 KB of module per behaviour and a floor of 25.7 KB until a `no_std` or
   smaller-std build is tried; the 50,000-byte line is a limit for about a dozen behaviours, not a permanent one.
6. **Unproven on the Blitz arm**: the direct call needs `dioxus-primitives` to depend on the crate, which changes what
   an installed component depends on; run `registry-install-smoke.sh` for it.
7. **Fail-open is complete only if the first-keystroke gap is acceptable on every engine**: on the web the compile is
   synchronous and there is no gap; the async fallback is the one place a cancelling behaviour is silently off.

## Reproducing

```
scripts/lane-lock.sh lane --label k -- env CARGO_TARGET_DIR=/home/user/tgt/kernel-508 cargo test --release -p decision-kernel -- --nocapture
scripts/lane-lock.sh lane --label k -- env CARGO_TARGET_DIR=/home/user/tgt/kernel-508 scripts/build-kernel.sh
scripts/build-js-bundle.sh && (cd primitives && bun test ts-test)
```

## Adversarial review, 2026-11-12

A read-only review of the proof branch (`lane/decision-kernel-proof`, head `0739d919`), trying to refute `KERNEL`.
Labels: **[src]** read in source, **[me]** measured by the reviewer (bun 1.4.4, which runs JavaScriptCore, on this box
at load average 6 to 7 on 4 cores; no Playwright, dx or SSG run), **[inf]** inferred.

**Verdict: `KERNEL STANDS WITH A BLOCKING CONDITION`.** No pass line is refuted. The one change that must happen before
any behaviour row builds on the kernel is finding 1; it is carried as the blocking item of
[row 509](../backlog/509-rust-decision-contract.md), with the other findings as its conditions.

Re-checked pass lines. Line 1 holds [src]: `keyboard.type("abc")` leaving the value `""` can only come from a
`preventDefault` inside the keydown's dispatch, so no asynchronous or after-the-fact path passes it. "Fixture tests
unchanged" is true: against main only `fake-dom.ts` (`loadBridge` loads the kernel by default) and one `kernel.load`
line of `bridge-dom.spec.ts` change. `bun test ts-test`: 82 pass [me]; `cargo test -p decision-kernel`: 5 pass [me];
a rebuild with `build-kernel.sh --out` gives sha256 `a557cfa6…`, the committed module [me]. JavaScriptCore compiles
the module synchronously (bun: mode `sync`, 1.5 ms) [me], evidence for WKWebView and WebKitGTK, not a webview run.

Findings, most severe first:

1. **Blocking: a TS and Rust drift silently disables a behaviour on web and webview only** [src]. `kernel.ts`
   `decide` answers `{}` without a warning for a kind the module does not export and for a malformed request
   (`n < 0`). The inputs are three unlinked hand lists: the TS `fields` / `reads` (`fixture.ts`), the Rust keys of
   `Config/Snapshot/Event::from_value` (`fixture.rs`), and the serde camelCase renames of
   `primitives/src/bridge/behaviours.rs`. One typo, and the behaviour never decides in a page while Blitz's direct
   call works. Fix: generate the lists and keys from the Rust input types, a gate holding the kinds equal to the
   exports, `warnOnce` plus a report to Rust on a stand-down.
2. **The tail is the codec, and the first call is cold** [me]. Interleaving the full `kernel.decide` with a raw wasm
   boundary call under the same load: full p999 0.09 to 2.7 ms and maximum 0.55 to 129 ms; raw p999 0.01 to 0.03 ms
   and maximum under 0.8 ms. The first call in a fresh process: 1.3 to 11.6 ms over 5 runs (JIT and Liftoff); the
   10,000-event p99 hides it, and the first keystroke is the one a user meets. Per call the JS side allocates [src]:
   the outer request array, `Object.entries` arrays per object, an `encodeInto` result and a `buf.subarray` view per
   string, the `items` / `pairs` arrays and `Object.fromEntries` per answer object, partly forced by `check-ts-scan`.
   Separate box from design with Chromium `--js-flags=--trace-gc` (stall indices matched to scavenges), an in-place
   encoder as the variant, and a first-call measurement on a page.
3. **Item-shaped snapshots break the 0.5 ms line** [me]. A memory of 1,000 `{id, disabled}` objects is a 37,049-byte
   request at p50 0.41 ms and p99 1.6 ms; the same list as 1,000 booleans is 1,158 bytes at p50 0.02 ms. The note's
   "1 to 2 KB for 1,000 items" holds only for a packed shape: roving focus and a carousel need packed snapshots and a
   timed vector at their largest snapshot.
4. **A trap in one kind stands every kind down for good, and Rust is not told** [src]. The catch in `decide` sets the
   page-wide state `failed`: a panic in `pan` also turns off `scrollLock`, `otpFilter` and the focus behaviours until a
   reload, while Rust keeps a half-done gesture (attributes set, a start `send` with no end). The kernel keeps no state
   between calls, only its I/O buffers, so re-instantiating from the cached module is safe [inf]. No state was found
   in which an event is mis-decided: a bad answer tag throws inside the try, an encode error reaches the bridge's catch.
5. **Reproducibility hangs on an unpinned toolchain** [src]. The bytes embed `/rustc/2d8144b…/library/std/...` and
   `dlmalloc-0.2.13` (std's wasm32 thread-local, RwLock and panic paths) [me, `strings`]. There is no
   `rust-toolchain` file, no CI toolchain step, and `session-setup.sh` installs whatever is current; the manifest's
   `rustc -Vv` comparison includes `host:`, so a macOS host or a newer container skips the rebuild half for good. The
   manifest does not hash the root `[profile.release]` the kernel shares with every crate. Two lanes that add a
   behaviour each conflict on `.wasm`, `.b64` and `kernel.manifest`, which `.gitattributes` has no driver for.
6. **The size line does not match the plan** [src, inf]. The planned registry is 9 firm kinds (`otpFilter`,
   `scrollLock`, `dropAnywhere`, `pan`, `settle`, `openFocus`, `stickToEnd`, `scrollOwner`, `labelClick`), plus
   `hotkey` and `roving` if their witnesses are red and `scrollStep`, `wheelBand` on the exits list (`js-interop.md`,
   the `adds:` of rows 448 to 467). At 1.9 KB each, 11 kinds are about 46.6 KB and 13 about 50.4 KB, and the 1.9 KB is
   toy-shaped. The 50,000 figure appears nowhere but row 508 (no derivation). The real costs are the main-thread
   synchronous compile (1.1 to 6.9 ms at 25.7 KB) and the first call; the kernel is 10,637 bytes gzip against a page
   wasm of 5.6 MB gzip.
7. **Floats can differ by renderer** [inf]. Transcendental functions come from Rust's bundled libm on wasm32 and from
   the system libm natively: a threshold in `pan` or `settle` can flip by one ulp. `usize` is safe, every length
   crosses as `u32` [src].
8. **CSP adds no source on web and does not close the PR #5194 route** [inf, src]. The Dioxus web app compiles its own
   wasm, which under CSP Level 3 already needs `'wasm-unsafe-eval'`. The route is coupled, not closed: `KERNEL_B64` is
   spliced into the eval's format string in `transport.rs`, so the kernel's delivery should be one seam function. On
   desktop a host's `evaluate_script` is not page-CSP-governed but the compile inside it is; no documented size limit
   of wry's `evaluate_script` on WKWebView, WebView2 or WebKitGTK was found (not confirmed either way), and the 34 KB
   rides the same path as the untested 20 KB bridge. Chromium raised its main-thread synchronous-compile cap from 4 KB
   to 8 MB (chromium/src commit d1a1a8fd, "Increase the limit on sync compilation on the main thread"), and Chrome's
   blink-dev Intent to Ship "Deprecate module size limit for WebAssembly.Module()" called the cap Chrome-only (Safari
   and Firefox did not enforce one); the lane's 8 MB test sat at that cap. Gecko not confirmed.
9. **A gate-ownership conflict** [src]. Row 509 must not touch `check-js-volume`, which the lane found red on
   `ffi.rs` (every `extern "C"` counts as a sink); the 34 KB of base64 is invisible to that gate; and row 509's
   `js_file=0` grant was 4,270 bytes short. Resolved in row 509: a fixed export set, one owner `allow` row for
   `ffi.rs`, the base64 held by `check-decision-kernel`'s gzip ratchet.
10. **The design is not a local minimum** [inf]. On web, Rust decides synchronously only through rsx handlers, which
    cannot attach at the window, the document or the capture phase; reaching the app's own wasm from the bridge needs
    `wasm-bindgen` exports, which the bridge forbids; the webview needs the kernel anyway. `MIRROR` is a TypeScript
    copy, against the owner's goal. The cost: under row 509, `decide` and the codec ship twice on web (in the app's
    wasm and in the kernel), about 2 KB per kind plus the base64 overhead. Of the page wasm's 48,325-byte growth,
    34,268 is the base64 and 4,270 the bridge; about 9.8 KB is not attributed. The answer would change only if a
    renderer cannot compile wasm in the page (an asynchronous-only webview) or if packed-snapshot vectors still miss
    the time budget.

Reviewer's scratch scripts (not committed): a bun benchmark of `kernel.decide`, a split into wasm-only and JS-only
cost, an interleaved full-against-raw run, and a fresh-process first-call timer.
