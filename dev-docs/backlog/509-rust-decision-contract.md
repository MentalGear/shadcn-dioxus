---
id: 509
title: "The Rust decision contract: one decide per behaviour, the decision kernel as a committed artefact, and the gates"
status: "LANDED on lane/decision-contract-509 (2026-11-13), awaiting the merge into the batch. Items 1 to 12…"
size: "L (inferred: was M; the review adds…"
area: primitives
js-grant: js_file_bytes=4500 (measured basis: row 508's loader and codec cost 4,270 bytes of `bridge.js`, 19,956 to 24,226, and the deleted TypeScript `decide` was 15 bytes; the in-place codec of item 7 should come in under the proof's 4,270, and the grant is the ceiling, not a target)
adds: []
updated: 2026-11-13
---

# Row 509: The Rust decision contract: one decide per behaviour, the decision kernel as a committed artefact, and the gates

## Item

The rules every named behaviour follows from here on, built on the one behaviour that exists (the test fixture) so
that the first real behaviour (`otpFilter`, row 448) is written to them. Why now: no behaviour but the fixture exists,
so the contract costs one migration today and one per behaviour later. The starting point is the proof code on
`lane/decision-kernel-proof` (head `0739d919`); row 508's adversarial review
([research/decision-kernel-proof.md](../research/decision-kernel-proof.md), "Adversarial review, 2026-11-12") is
where every condition below comes from. Build in this order:

1. **One Rust `decide` per behaviour, and its inputs are defined once.** `decide(config, memory, snapshot, event) ->
   Decision`, pure, in the `decision-kernel` crate (no dependencies, no `dioxus` or bridge types). The crate is used
   three ways: by `dioxus-primitives` as a direct call (the Blitz arm and every decision that may answer late), by the
   kernel build, and by `cargo test`. The `Config`, `Snapshot` and `Event` input types of each kind are the one
   definition of their wire keys: the hand-written keys of `from_value` (`fixture.rs` in the proof) and the hand lists
   `fields` / `reads` of `ts/behaviours/<kind>.ts` are replaced by code generated from those types (the same
   `cargo test` that writes `behaviours.gen.ts` writes the adapter's field and read lists and the kind table). This is
   the blocking item (Status).
2. **A fixed export set.** The module exports exactly `dx_abi`, `dx_input`, `dx_output`, `dx_kinds` (a hash of the
   generated kind table) and `dx_decide(kind_index, len)`; a kind is an index into the generated table, dispatched by
   a generated `match`. The bridge refuses a module whose `dx_kinds` differs from the hash in its own generated file
   (state `failed`, one warning). The export count never grows with a behaviour, so the sink count of `ffi.rs` is
   fixed (Detail, "Gate ownership"); the proof's per-kind `dx_decide_<kind>` exports are replaced.
3. **A kind that cannot decide is loud.** A kind missing from the kernel, a request the kernel cannot read (answer
   `u32::MAX`) and a trap each `warnOnce` in the page and send one report to Rust over the scope's channel (`{kernel:
   "stand-down", kind, reason}`), so Rust tears down or resyncs the install (a gesture half done, attributes set by
   earlier decisions). No such case answers `{}` silently.
4. **A trap is contained to its kind.** The kernel keeps no state between calls (only its two I/O buffers), so after a
   trap the bridge re-instantiates from the cached `WebAssembly.Module`; a kind that traps 3 times in one page stands
   down alone, the other kinds keep deciding. The proof's page-wide "failed for good" is replaced.
5. **The decision kernel is a committed artefact, built on a pinned toolchain.** `scripts/build-kernel.sh` builds the
   crate for `wasm32-unknown-unknown` under its own `[profile.kernel]` (inherits nothing from `[profile.release]`,
   `panic = "abort"`) and writes the module, the base64 text and `kernel.manifest` beside the JS bundles;
   `dioxus-primitives` keeps having no build script. A root `rust-toolchain.toml` pins `1.97.0` with the
   `wasm32-unknown-unknown` target (the toolchain of the committed bytes). The manifest hashes the crate's sources,
   the root `Cargo.toml` (the profile) and `rust-toolchain.toml`, and records `rustc -Vv` without the `host:` line.
   `.gitattributes` gives `primitives/src/kernel/*` a merge driver that rebuilds the artefact from the merged
   sources (registered by `scripts/setup-merge-drivers.sh`, kept in step by `check-merge-drivers.sh`), so two lanes
   that each add a behaviour never conflict on the binary.
6. **Kernel delivery is a transport-seam operation.** `transport.rs` loads the kernel through one function of the
   seam (today: the base64 in the bridge's eval, compiled synchronously); a later non-eval backend (Dioxus PR #5194)
   replaces that one function, so the kernel never ties the web build to `'unsafe-eval'`. The kernel's own CSP need
   is `'wasm-unsafe-eval'`, which the web app's own wasm already needs.
7. **An in-place codec.** `ts/lib/kernel.ts` encodes the request straight into the buffer `dx_input` returns (no
   staging buffer, no `Object.entries`, no `subarray` per string, no per-call array) and decodes the answer without a
   `pairs` array per object; the bytes of `bridge.js` it adds stay under this row's `js_file_bytes` grant. Where
   `check-ts-scan` forbids the shape (computed keys, computed `defineProperty`), the codec gets that gate's reasoned
   exception for `ts/lib/kernel.ts` from its owner, not a slower shape.
8. **Snapshots are packed.** A snapshot or memory that carries a list sends arrays of scalars (one array per
   property, or one string), never an object per item: 1,000 `{id, disabled}` objects are a 37,049-byte request at
   p50 0.41 ms and p99 1.6 ms, 1,000 booleans are 1,158 bytes at p50 0.02 ms (measured in the review, JavaScriptCore).
   Each kind's vectors include one at its largest planned snapshot.
9. **Floats behave the same on every renderer.** The crate disallows the transcendental float methods (`exp`, `ln`,
   `powf`, `powi`, `sin`, `cos`, `tan`, `atan2`, `hypot`, `cbrt` and their `f32` forms) through a crate-level
   `clippy::disallowed_methods` list: they come from Rust's bundled libm on wasm32 and from the system libm natively,
   so a threshold could flip by one ulp between web and Blitz. Arithmetic, `sqrt`, `abs`, `floor`, `round` and
   comparisons stay allowed (IEEE-exact on both).
10. **TypeScript is the adapter.** `ts/behaviours/<kind>.ts` holds the generated field and read lists of item 1 and
    applies the effects the decision returns. It holds no `decide`.
11. **One set of vectors, two routes.** `primitives/conformance/behaviours.json` runs against the Rust function under
    `cargo test` and against the committed kernel through the bridge's loader under `bun test`, so the wasm route is
    proven without a browser.
12. **The Blitz arm is not forbidden.** `check-native-slots` and `check-hot-events` exempt items under
    `#[cfg(not(feature = "web"))]`: an rsx slot write and an rsx scroll or pointer handler are the only routes on a
    renderer with no page.

Admission is unchanged: an effect is still admitted by its counterfactual witness or its platform rule row (row 446).

Rejected, one line each. `MIRROR` (a TypeScript copy held equal by vectors): row 508's five pass lines hold, and a
second implementation in TypeScript is the opposite of the owner's "ts only in the bridge part". Deciding in the app's
own wasm on web: the bridge's window, document and capture-phase listener can reach the app's wasm only through
`wasm-bindgen` exports, which the bridge forbids, and the webview needs the kernel anyway, so it would be a second
route on one renderer.

Part of [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md).

## Status / trigger

LANDED on `lane/decision-contract-509` (2026-11-13), awaiting the merge into the batch. Items 1 to 12 are built on the
one behaviour that exists (the test fixture); `otpFilter` (row 448) is written to them.

- **One Rust `decide` per kind, its inputs defined once.** `decision-kernel/` (no dependencies) holds `decide(config,
  memory, snapshot, event) -> Decision`; `Config`, `Snapshot` and `Event` are declared by `wire_input!` with the wire key
  of each field, every key required (a request that lacks one is unreadable, never a decision on an absent value).
  `kinds!` (`decision-kernel/src/kinds.rs`) is the one registration of a kind: the kind table, its hash (a compile-time
  FNV-1a of the table text) and the dispatch by index come from it. `cargo test -p dioxus-primitives behaviours` writes
  `behaviours.gen.ts` with `KINDS`, `KERNEL_HASH`, `KERNEL_KINDS` (config, reads and fields of each kind) and `ADAPTER`;
  a behaviour module registers `kernelBehaviour(kind, ADAPTER.<kind>)` and holds no list and no `decide`.
- **Five fixed exports** (`dx_abi`, `dx_kinds`, `dx_input`, `dx_decide(kind_index, len)`, `dx_output`); the page refuses
  a module whose `dx_kinds` differs from its generated hash.
- **A kind that cannot decide throws a stand-down report** (`missing-kind`, `unreadable-request`, `trap`,
  `kind-stood-down`, `kernel-failed`, `kernel-absent`): one warning, no decision (the browser keeps its action), one
  `{kernel: "stand-down", kind, reason}` message per install. `use_behaviour` (`bridge/session.rs`) reads
  `Inbound<T>`: a decision's message goes to the handler, a report tears the install down. **Tear down, not resync**:
  a resync would meet the same kernel and report again, and a torn-down install clears its memory (a gesture half
  done) and decides nothing more, so no later decision can add attributes to what earlier ones set. Attributes
  already set stay on the element (the page does not remove them); the owner holds the state they mirror. The
  install is not made again while the component lives. A VirtualDom test with a scripted `Document`
  (`use_behaviour_tears_the_install_down_on_a_stand_down_and_does_not_reinstall`) drives the hook: a decision
  reaches the handler and changes nothing, a report sends the teardown once, stops the lease renewal and installs
  nothing, and re-renders do not reinstall.
- **The kernel loads synchronously or not at all.** The module is compiled in the bridge's first eval (`kernel_load`, the
  one seam function); an engine that refuses a synchronous compile leaves the kernel `failed` (one warning, every
  decision reports `kernel-failed`). There is no asynchronous fallback, so no window exists in which a cancelling
  behaviour is silently off.
- **The in-place codec** (`ts/lib/kernel.ts`): the request is written into the buffer `dx_input` returns and the answer is
  read out of `dx_output` through one `DataView`; a call allocates no staging buffer, no `subarray`, no entries array,
  no pairs array and no view (`kernel.test.ts` counts them through spies). `bridge.js` grows by 4,491 bytes
  (22,558 to 27,049) and `behaviours/fixture.js` shrinks by 33 (553 to 520): +4,458 against the grant of 4,500.
- **The committed artefact**: `scripts/build-kernel.sh` (`[profile.kernel]`, `rust-toolchain.toml` pinning 1.97.0 with
  the wasm32 target) writes the module (26,222 bytes, 10,815 gzip), its base64 and `kernel.manifest`;
  `scripts/check-decision-kernel.sh` holds them (manifest, exports, kind table, rebuild on the pinned toolchain, a gzip
  ratchet starting at 10,815 (`gzip -9` on stdin; the proof's 10,637 was `gzip -9c <file>`, which also writes the
  21-byte file name, so its comparable figure is 10,616: the contract's fixed exports, the shared `Decision` and the
  generic wire layer cost 199 bytes), and the float ban list of
  `decision-kernel/clippy.toml`); `scripts/merge-driver-kernel.py` merges two lanes' artefacts by rebuilding the merged
  sources (it reads the three versions of every input out of the manifests' git blob ids and takes the other side from
  `GITHEAD_<commit>`, because git 2.43 has no `%X`/`%Y`).
- **The gates**: `check-bridge-behaviours` (a `decide(` or a hand-written list in a module is red, a variant with no
  kernel `decide` is red, vectors on both routes with one `largest` vector per kind, 104 self-test mutations),
  `check-native-slots` and `check-hot-events` exempt `#[cfg(feature = "native")]` items (`scripts/lib/cfg_arms.py`),
  `check-merge-drivers` holds the kernel driver.

Proof, in order: 1 passed (a renamed `Snapshot` key fails `the_generated_ts_is_current`; a kind commented out of
`kinds.rs` fails `check-decision-kernel`); 2 and 3 passed (`kernel.test.ts`); 4 passed (103 `bun test`, vectors red under
`cargo test` and under `bun test` against a stale kernel, the `decide(` mutation, the stale-source and profile mutations,
a native `decide` call in `vectors.rs`); 5 passed on a real merge of two lanes with the real builder (identical bytes to
a rebuild of the merged sources); 6 passed (fixture call p99 10 to 33 us, best of three runs, on a loaded box; the spy
test); 7 passed (p99 21 to 52 us at the largest vector, 61 to 98 us for 1,000 booleans); 8 **open** (a warm call inside `load` would cut the first call and costs about 40 bytes of `bridge.js`, which has 9 spare under the grant: the owner's question): the reference (a
release SSG build, a quiet box) was not run, and on a loaded debug build the numbers are over both bounds (compile
16 to 32 ms and first call 7 to 14 ms at 4x, 5 to 14 ms and 1.2 to 1.6 ms unthrottled); 9 passed (34 and 32 self-test
scenarios); 10 passed (`scripts/registry-install-smoke.sh --target wasm32-unknown-unknown`: 83 of 83 components installed and the app compiles; the path dependency on `decision-kernel` resolves in a git install); 11, 12 and 13 stay open as written.

## Size

L (inferred: was M; the review adds generation of the adapter's inputs, the fixed export set, trap containment, the
pinned toolchain and merge driver, the in-place codec and the budget gates)

## Detail

- **Gates, in build order.**
  - `check-bridge-behaviours`: the proof's two inverted rules flip: a `decide(` in `ts/behaviours/<kind>.ts` is red,
    a `Behaviour` variant with no Rust `decide` in `decision-kernel/src/<kind>.rs` is red; a hand-written `fields` or
    `reads` list in a behaviour module is red (they are generated, item 1); every kind has vectors on both routes; the
    byte ratchets of `bridge.ts`, `lib/register.ts`, `types/bridge.d.ts` and `fixture.ts` move to the measured values
    at landing (the proof's numbers: 42,784, 1,262, 6,940, 656, re-measured after item 7).
  - New `check-decision-kernel` (header, registration in `run-gates.sh`, `scripts/gates-doc.py --write`):
    1. the manifest matches the sources, the root `Cargo.toml`, `rust-toolchain.toml` and the module;
    2. the module's `dx_kinds` hash equals the hash of the generated kind table, and its exports are exactly the five
       of item 2;
    3. on the pinned toolchain it rebuilds and requires identical bytes; on any other it reports the rebuild half as
       skipped (as `check-js-bundle` does for `bun`), and the pin makes that the exception;
    4. **size budget:** a gzip-size ratchet on `decision-kernel.wasm` (`gzip -9`) that starts at the proof's measured
       **10,637 bytes** and rises only by a `js-grant`-style row at the merge base naming the behaviour that pays for
       it; the raw size is reported, not gated. This replaces row 508's fixed 50,000-byte line, which had no
       derivation and which the planned registry (9 firm kinds, up to 13 with `hotkey`, `roving`, `scrollStep`,
       `wheelBand`, at about 1.9 KB each) would cross.
  - `check-native-slots`, `check-hot-events`: the `not(web)` exemption, each with a self-test case.
  - `check-merge-drivers`: the kernel artefact driver of item 5 is in step.
- **Gate ownership (resolves the conflict row 508 found).** This row edits neither `scripts/check-js-volume.sh` nor
  `scripts/check-js-volume.allow.tsv` (row 445's, and an allow row is the owner's decision). The fixed five `extern
  "C"` exports of `decision-kernel/src/ffi.rs` (item 2) are excused by one `allow` row (`sink=5` plus the `js_lit`
  count the gate reports for that file, anchor `dx_abi`, reason "the decision kernel's fixed wasm exports, row 509")
  that the main loop asks the owner for and adds at the merge base before this row lands; because the export set is
  fixed, no behaviour row ever raises it. The kernel's base64 is not JavaScript and is not counted by
  `check-js-volume`: its bytes are held by `check-decision-kernel`'s gzip ratchet (this row owns that rule).
  `transport.rs`'s `js_lit` raise (6 to 8, the kernel load line) is paid inside this row's grant.
- **Files owned:** the `decision-kernel` crate and its `Cargo.toml` entry; the root `rust-toolchain.toml`; the root
  `Cargo.toml`'s `[profile.kernel]`; `.gitattributes` (the kernel line) and `scripts/setup-merge-drivers.sh` plus its
  driver script; `scripts/build-kernel.sh`, `scripts/check-decision-kernel.sh`; `primitives/src/kernel/**`;
  `primitives/src/bridge/behaviours.rs`, `primitives/src/bridge/transport.rs` (the kernel load function only);
  `primitives/src/ts/behaviours/**`, `primitives/src/ts/lib/**` and the kernel loader in `ts/bridge.ts`;
  `primitives/ts-test/behaviours.test.ts`, `primitives/ts-test/kernel.test.ts`, `primitives/ts-test/fake-dom.ts`;
  `playwright/decision-kernel.spec.ts` and its `playwright/durations.json` entry; `scripts/check-bridge-behaviours.*`,
  `scripts/check-native-slots.*`, `scripts/check-hot-events.*`, `scripts/check-merge-drivers.*`;
  `dev-docs/js-interop.md`; `dev-docs/dependency-register.tsv` if the register lists workspace crates.
  **Must not touch:** any component, `scripts/check-js-volume.*` (row 445's; see "Gate ownership").
- **Depends on:** row 508 (verdict `KERNEL`, review 2026-11-12),
  [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (landed), the owner's `allow` row for
  `ffi.rs` at the merge base. Shares `ts/bridge.ts` with rows 446, 447 and 449: it lands before them or after the
  `ts/lib` split, never beside them. **Blocks:** the `otpFilter` half of
  [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) and every row whose `adds:` names a behaviour.
- **Proof, in build order (each line is checkable in the lane unless marked open):**
  1. a `Snapshot` field renamed in Rust and not regenerated is red under `cargo test` (drift of the generated adapter
     lists), and a kind removed from the kernel but left in `KINDS` is red in `check-decision-kernel`;
  2. a kind the kernel cannot read, under `bun test`, warns once and sends one stand-down report to Rust, and answers
     no decision; the same for a trap;
  3. a trap in one kind (a trapping test module, as in the proof) leaves another kind deciding; three traps stand that
     kind down alone;
  4. the fixture's tests pass with its decision in the kernel; a vector the Rust function fails is red under
     `cargo test`, and the same vector is red under `bun test` against a stale kernel; a `decide(` added to
     `fixture.ts` is red; an edited source without a rebuild is red; a changed `[profile.kernel]` without a rebuild is
     red; a unit test calls the fixture's `decide` natively with no bridge;
  5. two lane branches that each change `decision-kernel/src` merge with the driver and give the module a rebuild of
     the merged sources produces;
  6. `bun test` times the in-place codec: the fixture call's p99 under 0.05 ms, and the codec allocates no array and
     no view per call (a test that counts `subarray` and `Object.entries` calls through a spy);
  7. a vector at each kind's largest planned snapshot (packed) is timed under `bun test`: p99 under 0.5 ms;
  8. **first call and compile, on a stated reference:** in Chromium on the release SSG build, the first kernel call
     on a fresh page (cold) under 2 ms and the synchronous compile plus instantiate under 8 ms, measured on the lane
     container with CPU throttling 4x (Playwright CDP `Emulation.setCPUThrottlingRate`, the stand-in for a low-end
     phone); the review measured the cold first call at 1.3 to 11.6 ms under JavaScriptCore on a loaded box, and row
     508 measured 1.1 to 6.9 ms for the compile;
  9. each exempted gate passes a `not(web)` item and still fails the same item under `web`;
  10. `scripts/registry-install-smoke.sh` is green (the decision crate changes what an installed component depends
      on);
  11. **open, waits on a quiet box (the owner's machine or a deploy-free slot the main loop schedules):** the call's
      p999 and maximum over 10,000 events with the tail attributed (Chromium `--js-flags=--trace-gc`, the stall
      indices matched to scavenges, against an allocation-free control loop); the review found the full call's p999
      about ten times the raw wasm boundary's under the same load, so the codec is the suspect;
  12. **open, waits on row [23](./023-preview-s-desktop-feature-lacks-dioxus.md) (a desktop build):** after the
      bridge's eval, `window.__dxBridge.kernel.state` is `"ready"` and `stats.mode` is `"sync"` in WKWebView, WebView2
      and WebKitGTK, through `evaluate_script` with the 34 KB base64, under the webview's own Content-Security-Policy;
  13. **open, waits on the deploy's Firefox and WebKit runs (they cannot execute in a lane):** proof lines 4 and 8 in
      Gecko and WebKit; the review confirmed only that JavaScriptCore (bun) compiles the module synchronously.
- **Hours (inferred):** about 14 to 22 for one Opus lane: generation of the adapter inputs and the fixed export set 4
  to 6, trap containment and the report to Rust 2 to 3, the pinned toolchain, profile and merge driver 2 to 3, the
  in-place codec and its timing tests 2 to 4, the gate half 4 to 6.

## Open after landing

- `decision-kernel` is `publish = false` and a path dependency of `dioxus-primitives`, so publishing primitives to
  crates.io fails until the kernel is published too or its module is the only thing primitives ships (untested).
- `rust-toolchain.toml` pins 1.97.0: a fresh container downloads that toolchain on the first cargo call unless it is
  already installed under that name.
