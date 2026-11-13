---
id: 511
title: "The browser-engine arm is the primitives' default; Blitz is the `native` opt-in, held to dioxus's own renderer by a compile-time assert"
status: "LANDED on lane/web-contract-511, 2026-11-13, awaiting the merge into the batch. The construction…"
size: "M (inferred)"
area: primitives
updated: 2026-11-12
---

# Row 511: the browser-engine arm is the default, Blitz is the `native` opt-in, and an assert ties it to dioxus

## Item

The primitives' `web` Cargo feature is the browser-engine switch. Every `#[cfg(feature = "web")]` / `#[cfg(not(feature = "web"))]` pair under `primitives/src/` chooses between the browser arm (`<dialog>` + `showModal()`, the `popover` attribute, the top-layer helpers) and the Blitz arm (a plain `div`, listener dismissal, an eval-installed focus trap). It is off by default (`primitives/Cargo.toml:45-47`, `default = ["router"]`) and wired by hand in `preview/Cargo.toml:38,39,41` (web, desktop, server) and `templates/starter/Cargo.toml.liquid:18-20` (web, desktop, mobile), plus `playwright/oracle/attr-synth/Cargo.toml:26` and `labs/carousel-lab/Cargo.toml:15`. Other paths miss it:

- the CLI adds the dependency with `cargo add --no-default-features` plus the component's own features (`cli/src/manifest.rs:113-116`) and never names `web`;
- `scripts/registry-install-smoke.sh:157-167` deletes the pre-declared primitives dependency (to prove the install adds every crate) and with it the `"dioxus-primitives/web"` forward, so it builds the Blitz arm;
- `test-harness/Cargo.toml:14-17` has `default = ["web"]` and `web = ["dioxus/web"]` with no `dioxus-primitives/web`.

The same wiring was missed twice before: in the 2026-09-01 server incident the SSG server built without `web` and hydration broke page-wide (`dev-docs/archive/research/recommended-implementations.md:52`), and [row 23](./023-preview-s-desktop-feature-lacks-dioxus.md) is the same miss for desktop. It is a repeated problem, so it is fixed by construction.

What a wrongly-armed app gets (read in source, not observed in a browser): modal dialogs without top layer or background inertness (`dialog.rs:481`, `alert_dialog.rs:234`); in-flow popovers, menus and select lists (`popover.rs:479`, `:820`, `select/components/list.rs:494`, `menu_root.rs:294-318`); and the bundled `focus-trap.js` asset (`lib.rs:129-130`). Inferred, not run: on `wasm32-unknown-unknown` without `web`, `time` lacks `wasm-bindgen` (`primitives/Cargo.toml:47`), so a Calendar's `UtcDateTime::now()` (`calendar/context.rs:216-218`) panics after mount.

## Status / trigger

LANDED on `lane/web-contract-511`, 2026-11-13, awaiting the merge into the batch. The construction below is built as written, in its first variant (`native = ["dioxus/native"]`), because a dioxus-native check fits the gates. [Row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md)'s focus-trap deletion no longer waits for it, and [row 438](./438-blitz-native-renderer.md) inherits the opt-in, the assert and `NATIVE_ARM`. What the build found, measured on the lane's tree, is under "What landed" at the end.

## Size

M (inferred)

## Detail

### The axis as it is (read in source)

- There are 140 `feature = "web"` sites in 27 primitives files (message_scroller.rs 13, lib.rs 12, top_layer/mod.rs 10, popover.rs 10, and so on). 53 read `not(feature = "web")`, 31 are item-level `#[cfg(not(..))]`, and 3 are `cfg!` (for example `command.rs:422`).
- About 14 sites have two real arms. These are the markup leaves: `dialog.rs:481`, `alert_dialog.rs:234`, `popover.rs:479` and `:820`, `tooltip.rs:560`, `hover_card.rs:706`, `navbar.rs:1017`, `navigation_menu.rs:1070`, `toast.rs:959`, `select/components/list.rs:494`, `combobox/components/list.rs:199`, `menu_root.rs:294-318`, `top_layer/attributes.rs:169` and `message_scroller.rs:622`. The rest are one-armed: web-only eval effects, imports, and 10 `expect(dead_code)` on `not(web)`.
- The styled components under `preview/src/components/**` carry no renderer cfg; they use only `cfg(test)`, `debug_assertions` and `dx_repo_lints`. `cli` and `dioxus-attributes` never read `web`. `preview/src/ssr_parity.rs:482` reads preview's own features to split `ROUTE_BASELINE_WEB`.
- dioxus exposes its renderer to dependents only through proc macros. `native!{}`, `web!{}` and the others expand their input only when dioxus-config-macro has that feature (`dioxus-config-macro-0.7.9/src/lib.rs:31-39`). They are re-exported in `dioxus::prelude` under `launch` (`dioxus-0.7.9/src/lib.rs:179-181`), and dioxus's `native` feature sets `dioxus-config-macro/native`. When several renderers are on, `LaunchBuilder::new` picks native > desktop > mobile > web > server (`dioxus-0.7.9/src/launch.rs:24`, `:102-113`).
- On Blitz and on the server, `document::eval` returns `Err(EvalError::Unsupported)` at once (`dioxus-native-0.7.9/src/contexts.rs:19`, `dioxus-server-0.7.9/src/document.rs:84`, `dioxus-document-0.7.9/src/document.rs:130-141`).
- dx 0.7.9 behaviour, from strings in the `dx` binary rather than source: for each renderer it uses the app feature that enables `dioxus/<renderer>`, and otherwise passes `dioxus/<renderer>` itself ("Could not find explicit feature for renderer, passing `fallback`"). It drops default features that point at another renderer, and it emits no renderer cfg.

### The construction

**Cargo.toml shapes.**

- `primitives/Cargo.toml`:
  - `[features] default = ["router"]`, `native = ["dioxus/native"]`, `web = []`. The `web` feature stays, inert, so manifests generated from the old starter keep compiling.
  - `time`'s wasm feature moves to the target table: `[target.'cfg(target_family = "wasm")'.dependencies] time = { version = "0.3.47", optional = true, features = ["wasm-bindgen"] }`, beside the existing `gloo-timers`. `calendar = ["dep:time"]` is unchanged. Resolution was checked with `cargo tree` on a scratch crate of this shape: the wasm target gets `wasm-bindgen`, the host does not, and without `calendar` there is no `time`.
  - `[package.metadata.docs.rs] features = ["router", "calendar", "color-picker"]`, so docs.rs documents the browser arm.
- `preview/Cargo.toml`: `web = ["dioxus/web"]`, `desktop = ["dioxus/desktop"]`, `fullstack = ["dioxus/fullstack"]`, `server = ["dioxus/server", "dioxus/fullstack"]`, `native = ["dioxus/native", "dioxus-primitives/native"]`.
- `templates/starter/Cargo.toml.liquid`: `web = ["dioxus/web"]`, `desktop = ["dioxus/desktop"]`, `mobile = ["dioxus/mobile"]`. If the starter offers a `native` feature, it is `["dioxus/native", "dioxus-primitives/native"]`.
- `playwright/oracle/attr-synth/Cargo.toml` and `labs/carousel-lab/Cargo.toml`: drop `"web"` from the primitives features.
- `test-harness/Cargo.toml`: unchanged; it is correct by default.
- The CLI writes nothing new: `cargo add --no-default-features` plus the component's features is the whole contract.

**The cfg rewrite**, mechanical, at every site under `primitives/src/`:

- `feature = "web"` becomes `not(feature = "native")`, and `not(feature = "web")` becomes `feature = "native"`.
- The compound forms flip the same way:
  - `any(feature = "web", test)` becomes `any(not(feature = "native"), test)`;
  - `all(not(feature = "web"), not(test))` becomes `all(feature = "native", not(test))`;
  - `cfg_attr(not(feature = "web"), expect(dead_code, ..))` becomes `cfg_attr(feature = "native", expect(dead_code, ..))`.
- The three `cfg!(feature = "web")` sites become `!cfg!(feature = "native")`.
- `preview/src/ssr_parity.rs:482`'s `cfg!(any(feature = "web", feature = "desktop", feature = "server"))` becomes `!cfg!(feature = "native")`, so the plain `cargo test` arm takes the browser baseline.

**The assert**, in `primitives/src/lib.rs`:

```rust
/// Whether dioxus is built with its native (Blitz) renderer, read from dioxus's own `native!` config macro.
const DIOXUS_NATIVE: bool = {
    #[allow(unused_mut, unused_assignments, reason = "assigned only when dioxus has its native renderer")]
    let mut on = false;
    dioxus::prelude::native! { on = true; }
    on
};
const _: () = assert!(
    DIOXUS_NATIVE == cfg!(feature = "native"),
    "dioxus is built with its native (Blitz) renderer and dioxus-primitives is not, or the other way round: \
     enable \"dioxus-primitives/native\" in the same feature that enables \"dioxus/native\""
);
/// True when the primitives render their Blitz arm. Styled components branch on this in expression position
/// and never cfg on a feature name, which after `dx components add` would name the user's crate.
pub const NATIVE_ARM: bool = cfg!(feature = "native");
```

**What each build gets.**

| Build | primitives `native` | dioxus `native` | Arm |
| --- | --- | --- | --- |
| Web client (wasm) | off | off | browser |
| SSG / fullstack server | off | off | browser, the same arm the client hydrates |
| Fullstack (dx builds client and server separately) | off on both | off on both | browser on both |
| Desktop webview, mobile webview, liveview | off | off | browser |
| CLI-installed app with no wiring | off | off | browser |
| Blitz with the opt-in | on | on | Blitz |
| Blitz without the opt-in, or dx's `dioxus/native` fallback | off | on | compile error with the assert's message |
| `--all-features`, or a workspace that unifies desktop and native | on (forward) | on | Blitz, which is what `LaunchBuilder` launches |
| A Blitz app's server binary | off | off | browser (inferred harmless: the native client does not hydrate SSR HTML) |

### What becomes impossible, and the residue

These states become impossible:

- a browser-engine build in the Blitz arm, since there is no wiring left to forget;
- a server/client arm mismatch caused by wiring;
- the Blitz arm and `focus-trap.js` in a web bundle;
- primitives in the Blitz arm while dioxus launches anything else, or the reverse. The forward makes "primitives on, dioxus off" unbuildable, and the assert makes "dioxus on, primitives off" a compile error;
- `time` without `wasm-bindgen` on wasm.

The residue, and what covers each part:

- **A `feature = "web"` cfg reintroduced in primitives.** `scripts/check-cfg-axis.sh` bans `feature = "web"` under `primitives/src/`.
- **A renderer feature cfg in a styled component.** After install it would name the user's crate. `check-cfg-axis.sh` bans `feature = "native"` / `feature = "web"` cfgs under `preview/src/components/**` and points at `NATIVE_ARM`.
- **An app-facing manifest whose feature enables `dioxus/native` without `dioxus-primitives/native`.** The assert fires only when someone builds that feature, and nobody builds native here yet. A TOML assertion in `check-cfg-axis.sh` covers `preview/Cargo.toml` and `templates/starter/Cargo.toml.liquid`.
- **The install path picking up `native` by accident.** `scripts/registry-install-smoke.sh` asserts that `cargo tree -e features -i dioxus-primitives` for the installed app shows no `native`.
- **A user's Blitz build without the opt-in.** The compile-time assert fails, and its message names the fix.
- **A build-dependency that turns on `dioxus/native` in the host graph.** Proc-macro features unify across the host graph: a `cargo tree` probe showed a build-dependency's `ssr` on the normal graph's config macro. The assert then fails loudly; this never yields a silent wrong arm.

### Why not the others

- **The CLI (and a doctor command) writes the wiring.** It has to guess which of an existing app's features are browser-engine ones. It misses the server feature, any manifest the user edits later, and dx's `dioxus/<renderer>` fallback. It helps only CLI users, and the state stays forgettable.
- **`web` as a default feature.** The starter and the CLI both use `--no-default-features` (`cli/src/manifest.rs:115`), which turns it off again silently.
- **A target cfg (`target_arch = "wasm32"`).** The SSG server and the desktop webview are native code. That split is the hydration break `check-cfg-axis.sh` exists to ban.
- **A build-script cfg.** A build script sees only its own crate's features, and dioxus has no `links` key to publish its renderer through.
- **The arm derived from `native!{}` with no primitives feature.** A proc macro expands only in expression position, so it yields a const, not an item cfg. Both arms would then compile everywhere, and about 140 sites would need a non-mechanical rewrite into `if CONST`. Host-graph unification of proc-macro features would pick the wrong arm silently.
- **The arm chosen at run time from the renderer's capability.** The server and Blitz both answer `Unsupported` to eval, so the server cannot tell which arm the client will hydrate, and the web bundle would carry the Blitz arm.
- **`compile_error!` requiring exactly one renderer marker.** Every browser app would have to declare a marker it does not need, and the common case would stay one forgotten line from wrong. The assert above keeps the useful half: the check against dioxus.
- **Shrinking the axis.** The one-armed eval effects can lose their cfg, since eval is inert on Blitz; that is [row 438](./438-blitz-native-renderer.md)'s business. The ~14 markup leaves still need a selector, so shrinking does not replace this row.

### The strongest argument against

- **A Blitz app still has to remember one line, while a pure `native!{}` derivation would need none.** This does not win. Stable Rust gives no item-level cfg from a dependency's feature, so the derivation compiles both arms everywhere and rewrites about 140 sites by hand. Its one failure mode, host-graph unification, is silent. In this construction the same mechanism only checks, so the same failure becomes a compile error that names the fix.
- **`native` is a non-additive feature.** So is dioxus's own renderer feature, with the same priority (`launch.rs:102`). The forward and the assert keep the two identical, so feature unification moves both together.

### Open before building

Two experiments decide the exact shape. The first is measured (2026-11-12, scratch crates outside the repo); the
second ran once disk was free.

**Result of experiment 1 (measured).** The form works with one more lint allowed. With the macro's `native` feature
on, `let mut on = false; native! { on = true; }` fails under `-D warnings` on `unused_assignments`; with
`#[allow(unused_mut, unused_assignments, reason = ...)]` both arms check and pass `cargo clippy --all-targets -- -D
warnings`. The statement-position macro, the hygiene of `on` and the assert in a const all hold. A broken tie fails at
compile time in both directions with `error[E0080]: evaluation panicked: <the assert's message>`. Through
`dioxus::prelude::native!` (dioxus `["lib", "launch"]`; the re-export is gated by `launch` only) the no-feature arm
compiles clean; a cold `cargo check` of that crate took 1 min 20 s and 320 MB. The `+native` arm through dioxus
itself is experiment 2.

**Result of experiment 2 (measured).** A dioxus `["lib", "launch", "native"]` crate checks in this sandbox: no system
library is missing and the network policy refuses nothing. Cold `cargo check` with two jobs and no incremental cache:
3 min 27 s, about 480 crates, a 1.2 GB target. With the Blitz tree built, a recheck of the scratch crate takes under a
second; a recheck of the primitives on top of it is not measured. Through the real path, dioxus `native` on with the
crate's own `native` off fails to compile with the assert's message. Peak memory is not measured.

So the build fits the gates: the first variant below is the one to build (`native = ["dioxus/native"]`), at a one-time
cost of about 3.5 minutes and 1.2 GB on the gate target.

1. Does the const-block form `native! { on = true; }` compile, resolve `on` hygienically, and stay clean under `-D warnings`, both with dioxus `["lib", "launch"]` and with `+native` added?
2. Does a `dioxus/native` build compile in this sandbox, and what does it cost in time and disk?

The second result decides how the gates compile and lint the Blitz arm:

- **If a dioxus-native build fits the gates:** keep `native = ["dioxus/native"]`. `clippy-native`, the `native` run of `check-primitives-features` and the `native` arm of `check-rustdoc` build against the real renderer.
- **If it does not fit (too slow, too large, or it needs something the sandbox lacks):**
  - Use `native = []` without the forward, and compile the assert under `#[cfg(not(dx_repo_lints))]`.
  - The three gates pass `--cfg dx_repo_lints` (through `RUSTFLAGS` for the `cargo check` and `cargo doc` runs), so they compile the Blitz arm against plain dioxus.
  - For users nothing weakens. The assert is still compiled, and it now catches both directions loudly where the forward made one of them impossible.
  - What weakens is that the repo never compiles the Blitz arm against dioxus-native's own types. That is inferred to cost nothing today, because the arm uses only dioxus core and html APIs.
  - `--all-features` on primitives alone then fails the assert loudly. docs.rs is unaffected, because its feature list is explicit.

Chosen: the forward when experiment 2 fits; otherwise `native = []` with the assert outside `dx_repo_lints`.

### Gates after the change

- `clippy-default` lints the browser arm, which is now the shipped one.
- `clippy-web` is redundant and becomes `clippy-native` (`cargo clippy -p dioxus-primitives --features native`). `clippy-server` and `test-server` are unchanged.
- `check-primitives-features.sh` loops over `native` instead of `web`, and leaves out `web` because it is inert.
- `check-rustdoc.sh` arms become `default` and `native`, and `dep-report.sh` drops `--features web`.
- `check-cfg-axis.sh` gains the two bans and the TOML assertion above.
- `registry-install-smoke.sh` drops the `"dioxus-primitives/web"` replace (`:167`) and adds the `cargo tree` assertion.

### Order of work

1. One commit covers:
   - `primitives/Cargo.toml` and the assert;
   - the mechanical flip: about 140 sites in 27 files, a scripted rewrite, then a read of every `any`/`all` combination;
   - the 58 doc-comment mentions of the web arm;
   - the gates above.
2. The manifests (preview, starter, attr-synth, carousel-lab), `ssr_parity.rs`, the install smoke, the primitives README feature table and the install docs. The docs need one sentence: browser engines need nothing, and a Blitz app enables `dioxus-primitives/native` alongside `dioxus/native`.
3. [Row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md)'s focus-trap deletion. The `not(web)` trap is then compiled only where dioxus launches Blitz and eval is `Unsupported`, so the deletion removes dead code by construction.
4. [Row 438](./438-blitz-native-renderer.md) inherits the single `native` opt-in named after dioxus's own feature, the assert, `NATIVE_ARM`, and `#[cfg(feature = "native")]` as the spelling of the Blitz arm.

Size: M (inferred). About 140 mechanical cfg edits, about 10 manifest and gate edits, one assert, and prose.

## Proof when closed

- `scripts/registry-install-smoke.sh` installs into an app with no primitives wiring, builds it for `wasm32-unknown-unknown`, and asserts the browser arm: no `native` in the feature tree, and `<dialog` in the rendered dialog.
- `scripts/run-gates.sh` is green with the renamed arms.
- The release Playwright suite is unchanged, because the shipped arm of the preview does not change.
- A scratch Blitz manifest with `dioxus/native` and without `dioxus-primitives/native` fails to compile with the assert's message.

## What landed (2026-11-13, measured on the lane's tree)

- **Premise, counted on the lane's tree before the first edit.** 140 `feature = "web"` sites in 27 `primitives/src` files, 53 of them `not(feature = "web")`, 3 `cfg!`; no renderer cfg under `preview/src/components/**`; the manifests that forwarded `dioxus-primitives/web` were preview (3 features), the starter (3), `playwright/oracle/attr-synth` and `labs/carousel-lab`; `test-harness` did not. Nothing differed from the row.
- **The rewrite.** 140 sites flipped by script (`not(feature = "web")` to `feature = "native"` first, then `feature = "web"` to `not(feature = "native")`), then read by hand: 10 `any`/`all` expressions, the 3 `cfg!` sites (written `!cfg!(feature = "native")`; `message_scroller.rs:403`'s negated one became `cfg!(feature = "native")`) and the `cfg_attr(.., expect(..))` pairs, whose `expect` reasons hold on both arms (clippy `-D warnings` is clean on both). The doc comments name the browser and native arms.
- **`dialog_renders_open_as_presence`.** It failed with the lib tests on the old `web` arm because it was written for the Blitz arm's markup: it expected one `data-open` (the root's), while the browser arm's `<dialog>` carries its own (its `[open]` stays set for the exit, so a stylesheet keys the exit on the `<dialog>`'s `data-closed`). The browser arm is correct; the test now counts one on the native arm and two on the browser arm and checks each. The lib tests pass on both arms (1121 default, 1109 `native`), so no other test sat in that position.
- **The native arm's cost.** A cold `cargo clippy -p dioxus-primitives --features native --all-targets` took 3 min 0 s (about 480 crates) and grew the shared check target by 558 MB (561 MB to 1,119 MB); warm it is 26 s.
- **The assert, through scratch crates outside the repo** (`/home/user/tgt/wc511-scratch`): dioxus `native` on with the primitives' `native` off fails with `error[E0080]: evaluation panicked: dioxus is built with its native (Blitz) renderer and dioxus-primitives is not, or the other way round: enable "dioxus-primitives/native" in the same feature that enables "dioxus/native"`. The primitives' `native` on with dioxus left default compiles, because the feature forwards `dioxus/native`. A manifest that still forwards `dioxus-primitives/web` (with `dioxus/web`) compiles; nothing reads `web`, and `check-cfg-axis` bans reading it.
- **`time` and wasm.** `cargo tree -e features -i time` on a crate using `calendar`: `time feature "wasm-bindgen"` on `--target wasm32-unknown-unknown`, absent on the host; without `calendar` there is no `time` at all.
- **Shipped arm unchanged.** The preview's debug SSG before and after: 102 HTML files, 90 identical once the server-allocated `dxc-N` ids are renumbered by first appearance (page text and the decoded hydration blob) and `file.rs:line:col` debug locations are normalised, and the other 12 differ only by the doc-comment prose the docs pages render ("Web arm" to "Browser arm", "On the web" to "In the browser arm"). The 326 asset file names, content-hashed, are identical. The id counter differs between two builds (a page's first popover is `dxc-1293` in one and `dxc-44` in the other): inferred, not run, to be the server's global counter taking route requests in a different order.
- **The gates.** `check-cfg-axis` has the two bans and the manifest assertion, each with a mutation in `--self-test`. `clippy-web` is `clippy-native`; `check-primitives-features` builds `native` alone and with every feature; `check-rustdoc` builds `default` and `native`; `dep-report.sh` drops `--features web`. `dependency-register.py` computes the install closure over every feature except `native` and `web`, and the closure baseline did not change, which shows the default install closure is the one it was.
- **Install path.** `scripts/registry-install-smoke.sh` no longer strips a feature that does not exist, and asserts with `cargo tree -e features -i dioxus-primitives` that the installed app has no `native`. `--only dialog,popover,alert_dialog,calendar,select` on the host: `OK -- 5/5 components installed`. The `--target wasm32-unknown-unknown` run was not made: the lane had under 6 GB free.
