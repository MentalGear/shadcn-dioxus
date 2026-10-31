# Dependency evaluation: keep, feature-gate, replace, lift or drop (row 371)

Date 2026-10-23, lane `depeval`, base `queue/batch-10` (400a048b). Read-only audit: no code changed; the one added file outside docs is
`scripts/dep-report.sh`. Builds on, and does not redo, row 217 and `dependency-audit-2026-10-14.md` (palette/time gated, `build.rs` and
`lazy-js-bundle` out of primitives, `dirs`/`num-integer`/`strum`/`dioxus-sdk-time` replaced, `dioxus-i18n` from crates.io, version gate).
**Outcomes are proposals; they await owner approval.**

## 1. Method (the rule, owner-approved by Tom 2026-10-23, with the coordinator's refinement pending concurrence)

A dependency **qualifies** for removal or lifting (copying the slice we use into our tree) when ANY one holds:

1. **Security.** Dropping or lifting shrinks attack surface, weighed by how far the crate reaches into our code and our users' data against how
   far its maintainers can be trusted. *Reach* = code run on the user's machine at compile time (`build.rs`, proc-macros), `unsafe`, parsing
   untrusted input, network / filesystem / user-data access. Any crate whose reach is out of proportion to its trust is marked separately
   (section 4).
   Refinement: split into **code risk** (unsafe, untrusted input, fs/network/user data) and **channel risk** (how many maintainers, release from
   CI or by hand, ownership changes). Lifting does not reduce code risk (we would own the same code): code risk is fixed by drop, replace or
   feature-gate. Channel risk is fixed by exact pinning or lifting. Both are recorded per crate and the outcome is routed accordingly.
2. **Cost.** Lifting or dropping removes a cost users notice: compile time, transitive crate count, shipped wasm bytes.
3. **Small slice.** What we use is small, stable and low in domain risk.

**Veto.** Hard-to-get-right domains, or ones likely to need updates (time zones, colour science, parsers, the framework itself), stay a library or
are feature-gated, even if 1 or 2 holds.

**Outcome ladder** (cheapest and most reversible first; take the first rung that resolves the qualifying reason):
drop (unused) > feature-gate (only that component's users pay) > pin exactly > replace with std / web-sys / a browser API > lift > keep.
"feature-trim" below means rung 1 or 2 applied to a *feature of* a dependency (turn the unused feature off).

**Blast radius** = how many of the 83 `component.json` closures pull the crate in (from `component.json` `cargoDependencies`, and for
`dioxus-primitives` internals, every component, since all 83 depend on it).

These criteria apply to removing or lifting. The test for adding a new dependency is the playbook's
[Choosing a new dependency](../supply-chain/README.md#choosing-a-new-dependency-owner-2026-10-31) section (C23).

## 2. Re-running the inventory

```
scripts/dep-report.sh              # offline: lock version, `unsafe` lines, build.rs / proc-macro, wasm crate counts per feature set, blast radius
scripts/dep-report.sh --network    # adds crates.io owners, last release, downloads (generic User-Agent, no account data)
DEP_CRATES="time palette" scripts/dep-report.sh
```

Counts below are `cargo tree --offline -e normal --target wasm32-unknown-unknown --prefix none -p dioxus-primitives <features> | sort -u`
(name + version deduplicated; `(*)` and path suffixes stripped), run through `scripts/lane-cargo.sh`. `unsafe` is a raw `\bunsafe\b` line count in
the crate's `src/` (comments and docs included, so an upper bound; `#![forbid(unsafe_code)]` noted where present). Owners, dates and licences come
from `https://crates.io/api/v1/crates/<name>[/owners]`. Advisories: see section 5 (what could and could not be read).

## 3. Measured baseline

Unique crates in the primitives wasm tree (dioxus included): **111** with no feature (`--no-default-features --features web`), **117** with
`calendar` (+6: time, time-core, time-macros (proc-macro), deranged, num-conv, powerfmt), **123** with `color-picker` (+12: palette,
palette_derive (proc-macro), approx, by_address, fast-srgb8, phf, phf_generator, phf_macros (proc-macro), phf_shared, rand, rand_core, siphasher),
**151** with the default `router` (+40, icu/idna/url/sha2). Of the 111, 21 are proc-macro crates (the whole tree, dioxus's own macros dominate).
Our own direct, non-dioxus crates cost **0** extra crates for `tracing`/`serde` (dioxus brings both), 1 for `gloo-timers` (wasm; `tokio` on native),
and 0 for the always-present `dioxus-attributes` (our own proc macro, 8 host crates shared with dioxus's macros).
`dioxus-icons` adds exactly 1 crate (itself) but see its `build.rs` below. `pulldown-cmark` (no default features) adds 4 (`bitflags`, `memchr`,
`unicase` shared or tiny; net +1 in a Dioxus tree, +`pulldown-cmark-escape` only with `html`, build-time in preview). A fresh preview wasm tree is
222 crates; the `cli` host tree is 34.

## 4. Per-dependency table

Columns: use = items we call and call sites; slice = rough LOC if lifted; code = code-risk facts; channel = owners, last release, cadence;
domain = domain risk and update likelihood; blast = components pulling it in. Outcome: ladder rung plus one-line reason.

| Dependency (lock) | Use / call sites / slice | Cost (wasm crates, proc-macro, build.rs) | Code risk | Channel risk | Domain / updates | Blast | Outcome |
|---|---|---|---|---|---|---|---|
| **dioxus** 0.7.9 (framework) | everything | 111-crate base | not evaluated (the framework) | jkelleyrtp + the `dioxuslabs:publish` team, 16 releases/12 mo, MIT/Apache | the framework itself: veto | 83 | **keep** (veto) |
| **dioxus-attributes** (ours, path) | attr-spread proc macro, 277 lines, ~64 files | 1 proc-macro, 8 host crates shared with dioxus macros | proc-macro runs at compile time, our code | ours | ours | 83 | **keep** (ours; the register still lists it as a proc-macro) |
| **time** 0.3.47 (`std`,`local-offset`; `macros` dev-only) | `Date`, `Month`, `Weekday`, `OffsetDateTime::now_local`, `NumericalDuration`, `date!`; ~204 call sites in calendar/date_picker/lib; slice is the date math of two components, far over any small slice | +6 crates behind `calendar` (already gated); proc-macro `time-macros`; no build.rs | 105 `unsafe` lines in `time` (most in `local-offset`, native only; one 2020 advisory on `localtime_r`, patched in 0.2.23); `time-macros` 4; parses no untrusted input (we do not call `parse`) | 1 owner (jhpratt) across time, time-core, time-macros, deranged, num-conv, powerfmt; 11 releases/12 mo; 1.0 B downloads | calendar arithmetic, public `Date` type in our props: veto; high update likelihood | 2 (calendar, date_picker) | **keep, already feature-gated**, plus a **feature-trim**: `macros` is used for 10 non-test `date!` defaults only (`calendar/single.rs` and `calendar/range.rs` (were `calendar.rs` 618...), `date_picker/picker.rs`, `date_picker/popover.rs`); `Date::from_calendar_date` is const, so `macros` can become a dev-dependency and drop `time-macros` (1 proc-macro, one compile) from users' trees. **Applied 2026-10-25 (row 371)**: the 10 defaults are const dates, `macros` is a primitives dev-dependency; measured wasm tree with `calendar` 152 -> 151 crates (`time-macros` gone) |
| **palette** 0.7.6 (`default-features = false`, `std`) | `Hsv`, `Srgb`, `RgbHue`, `Oklch`, `IntoColor`, `FromColor`; 34 refs in `color_picker.rs` (832 lines) and `color_naming.rs` (123); `Hsv`/`Srgb<u8>` are in ColorPicker's public props | +12 crates behind `color-picker` (gated); proc-macro `palette_derive`; no build.rs | 115 `unsafe` lines (mostly bytemuck/SIMD helpers in conversions); no untrusted input; **default features pull `named_from_str` -> phf + phf_macros + rand + siphasher, which we do not use** (no `from_str`, no `named::` in our two files) | 1 owner (Ogeon); 1 release/12 mo (0.7.6 in 2024, 0.7.7 on 2026-08-02); `palette_derive` same owner | colour science: veto on lifting. Rewrite would break the public types | 1 (color_picker) | **feature-gate (done) + feature-trim: `default-features = false, features = ["std"]`** drops `named_from_str` and with it phf, phf_generator, phf_macros, phf_shared, rand, rand_core, siphasher (7 crates, 1 proc-macro) for ColorPicker users, 12 -> about 5 (palette, palette_derive, fast-srgb8, by_address, approx). **Applied 2026-10-25 (row 371)**; measured wasm tree with `color-picker` 160 -> 150 crates (unique `cargo tree` lines), `check-primitives-features.sh` OK. Rung 1 (drop an unused feature) |
| **phf** 0.11.3 / phf_generator / phf_macros / phf_shared / **rand** 0.8.6 / rand_core 0.6.4 / **siphasher** 1.0.3 (all transitive of palette) | none directly | 7 crates, 1 proc-macro, only via `color-picker` | rand 14 `unsafe` lines; RUSTSEC-2026-0097 (unsound with a custom logger) is patched in 0.8.6 = the lock; siphasher 12; phf 0 | phf: 3 owners; rand: dhardy + `rust-random:maintainers` team, 16 releases/12 mo; siphasher: 1 owner (jedisct1) | n/a | 1 | **drop** with the palette feature-trim above (unused) |
| **tracing** 0.1.44 | 7 `tracing::warn!` calls (questionnaire, selection, carousel) | 0 extra (dioxus brings it); tracing-attributes is a proc-macro already in dioxus's tree | 7 `unsafe` lines (callsite registry); no untrusted input | carllerche, hawkw + `tokio-rs:publish-tracing` team, 3 releases/12 mo, MIT | stable | 83 | **keep** (0 marginal cost, the framework's own logging facade) |
| **serde** 1.0.228 (`derive`) | 10 derive sites, JSON to and from JS in `js_listener`, `virtual/types`, `message_scroller` | 0 extra (dioxus brings serde, serde_derive proc-macro); `serde` itself has a `build.rs` that runs `rustc --version` only | 2 `unsafe` lines; parses JSON from our own JS bundles (not user input) | dtolnay + `serde-rs:publish` team, MIT/Apache | stable | 83 | **keep** (0 marginal cost; the framework needs it) |
| **tokio** 1.52.3 (`time`, native only) | `tokio::time::sleep`, 1 site in `src/sleep.rs` (16 lines) | 0 on wasm (target-specific); native/SSR: tokio, already in dioxus's native tree | 1058 `unsafe` lines (an async runtime; only the timer is compiled); network/fs not used through us | carllerche, Darksonn + `tokio-rs:core` team, 24 releases/12 mo | stable | native/SSR builds of all 83; 0 for wasm users | **keep** (already in the tree through dioxus; reach is the timer only) |
| **gloo-timers** 0.3.0 (`futures`, wasm only) | `gloo_timers::future::sleep`, 1 site in `src/sleep.rs`; slice would be a `setTimeout` promise, ~15 LOC with `web-sys`/`js-sys` | 1 crate on wasm (`gloo-timers`, over `js-sys`/`wasm-bindgen` already present) | 0 `unsafe` lines; no input | 2 owners (ranile, Madoshakalaka); 0.4.0 released 2026-03-25 (1 release/12 mo); the `gloo` project | low domain risk but 0.3 -> 0.4 bump pending | 83 (wasm) | **replace** is possible (a ~15-line `setTimeout` future over `web-sys`, -1 crate on wasm for every user) but qualifies only on rule 3 with a small saving; recommend **keep unless the owner wants the last crate** (the tests `interval.rs` pin its semantics) |
| **dioxus-icons** =0.1.0 | 12 distinct Lucide icons in shipped `component.rs`; 196 preview files use it; slice is 12 SVG templates (~12 x 10 lines) | +1 crate; 25 MB of icon source parsed on a cold compile (1705 `rsx` icon functions) [cost unmeasured here]; **a `build.rs` in every user build that runs `lazy-js-bundle`** (hashes `./picker` with `DefaultHasher`, rewrites `src/js/` on mismatch, shells out to `bun`) and writes README parts to `OUT_DIR` | 10 `unsafe` lines; build.rs runs code on the user's machine at compile time; no untrusted input | ealmloff + `dioxuslabs:publish` team (the dioxus org), 3 releases/12 mo, 0.1.0 only 2026-05, 34k downloads (young, pinned exact); MIT AND ISC (Lucide is ISC) | stable (icons), low update need | 30 of 83 | **replace for the 12 icons we use, feature-gated off by default as an optional `dioxus-icons` component dep: lift the 12 SVGs into a small `icons.rs` of our own** (qualifies on rule 1: a build script plus 25 MB of generated source in the user's build for 12 glyphs; rule 2: cold compile time; rule 3: tiny, stable, low risk; Lucide is ISC, the notice travels). No veto. Not measured: the real compile-time delta (needs a cold build of a user app, ~6 GB) |
| **pulldown-cmark** =0.13.4 (`default-features = false`) | `Parser` + `Event` in the `markdown` component (parses model/user text); 1 import | +4 crates, ~net +1 (`unicase`) in a Dioxus tree; the `html` feature (+`pulldown-cmark-escape`) is preview build-time only; build.rs only for the CLI/simd tables (generated code committed) | **parses untrusted input** (model output); `#![forbid(unsafe_code)]` unless `simd` (off); 5 `unsafe` lines all behind `simd` | marcusklaas, raphlinus, Martin1887; 5 releases/12 mo; 0.13.5 on 2026-10-21 (we pin 0.13.4); MIT | parser: veto on replace or lift; update likelihood for CommonMark and security fixes is high | 1 (markdown) | **keep, feature-gated by being its own component; pinned exactly** (already). Re-open on every 0.13.x release (one parser panic fix already needed). Reach (untrusted input) is proportionate to trust: 3 owners, forbid-unsafe, fuzzed upstream |
| **pulldown-cmark-escape** 0.11.0 | build-time only (docs html) | not in users' trees | 6 `unsafe` lines (SIMD escape paths); no input | 1 owner (Martin1887), 0 releases/12 mo | n/a | 0 users | **keep** (preview build-time only) |
| **unicase** 2.9.0 (via pulldown-cmark) | none directly | 1 crate | 0 `unsafe` | 1 owner (seanmonstar) | n/a | 1 | **keep** |
| **approx** 0.5.1, **by_address** 1.2.1, **fast-srgb8** 1.0.0 (palette's) | none directly | 3 crates, `color-picker` only | approx 0, by_address 2, fast-srgb8 8 `unsafe` lines; by_address and fast-srgb8 have not released in 2.5 years (single owners mbrubeck, thomcc) | single owners, 0 releases/12 mo | stable | 1 | **keep** (inherent to palette; `approx` could go with `default-features = false` minus `approx` if the owner wants the last one) |
| **dioxus-i18n** 0.6.0, **dioxus-code** 0.1.2, **unic-langid** 0.9.6, **time/palette** in `preview` | preview/docs site only, never in a user's `dx components add` | +9 / +21 / 0 | not shipped | dioxus-i18n: 1 owner (marc2332); dioxus-code: dioxus org; unic-langid: 2 owners (Manishearth, zbraniecki), 0 releases/12 mo | n/a | 0 users | **keep** (outside scope: the docs site only; recorded for completeness) |
| **lazy-js-bundle** 0.7.9 (transitive of dioxus's own crates and of dioxus-icons) | none by us (row 217 removed ours) | build-dependency | `build.rs` helper that spawns `bun` if its `DefaultHasher` source hash differs | jkelleyrtp + dioxus publish team | n/a | 83 (through dioxus) | **keep (not ours to remove)**; becomes irrelevant to *our* closure once dioxus-icons is replaced, except through dioxus itself |

Dioxus owner note: `dioxus-primitives` (the crate name a user's `cargo add` writes) is a **git dependency** in every component closure, not a
registry crate: there is no crates.io release channel for it (crates.io lists one owner, DogeDark, for the upstream `dioxus-primitives` name).
Pinning is by `rev` stamped at release time (`scripts/stamp-release-rev.sh`), which is the channel-risk fix already in place.

### JS dev tooling (security axis only; never shipped, never in a user's build)

`playwright/`: 4 direct devDependencies (`@axe-core/playwright` 4.10.2, `@playwright/test` 1.60.0, `axe-playwright` 2.1.0, `playwright` 1.60.0), 16
lock entries. `preview/`: 4 direct (stylelint 16.26.1, config-standard, config-idiomatic-order, stylelint-order), 122 lock entries. Both pin
exactly, enforce `minimumReleaseAge: 43200` (30 days) and `onlyBuiltDependencies: []` (no install scripts run), so the install-time code-execution
reach is already zero; the remaining reach is `playwright` downloading browsers (a Microsoft-owned channel) and stylelint's 122 parse-only packages.
Outcome: **keep, no change** (reach proportionate, mitigations in place); a `pnpm audit` was not run (needs a registry advisory endpoint the proxy
does not allow).

## 5. Reach out of proportion to trust

Marked separately, as the rule asks:

1. **`dioxus-icons` `build.rs` (30 of 83 components).** It runs `lazy-js-bundle` in the user's build: hash with `DefaultHasher`, rewrite
   `src/js/` and shell out to `bun` on a mismatch, plus a README split through `OUT_DIR`. For the sake of 12 icons the user compiles 25 MB of
   generated icon source and runs a third party's build script. The publisher is the dioxus org (same trust as `dioxus`), but the crate is five
   months old with 34k downloads and one human owner besides the team. Reach (compile-time code execution in a user's tree) exceeds the benefit.
   Fix: lift the 12 glyphs (replace rung).
2. **`palette`'s default features (ColorPicker users).** Seven crates incl. a proc-macro (`phf_macros`) and `rand`, for named-colour string
   parsing nobody calls. Single owner, one release a year. Fix: `default-features = false` (drop rung).
3. **`time` (calendar, date picker users).** One owner for six crates (jhpratt), a proc-macro and `unsafe` in `local-offset`; reach is real but
   the domain veto holds and the owner is the long-standing maintainer with a steady release cadence. Fix: trim `macros` only; otherwise keep.
4. **`pulldown-cmark` (markdown users).** Parses untrusted model output, but the three owners, `forbid(unsafe_code)` and exact pin make trust
   proportionate. Not flagged, listed for the register's re-open-on-bump rule.
5. Single-owner, quiet leaf crates with `unsafe` inside palette's tree (`by_address`, `fast-srgb8`): small reach, acceptable.

## 6. What could not be measured

- **Wasm bytes per crate.** `/home/user/tgt/pub-batch10/wasm/preview_bg.wasm` is a **debug** build (56 MB, `build-profile.txt` = `debug`) with
  stripped names; `llvm-nm` returns no sizes and no installed tool (`twiggy`, `cargo-bloat`, `wasm-objdump`, `wasm-opt`) can attribute bytes. A
  release build with names costs the build lock and ~6 GB; not done. So rule 2's wasm-bytes half is unmeasured; the crate-count half is measured.
- **Compile-time delta** of lifting `dioxus-icons`, of trimming palette/time features: needs a cold user-app build; only crate counts are measured
  (and for the trims, derived from the crates' `Cargo.toml` features, not built).
- **RustSec advisories, completely.** `cargo-audit` is not installed; `rustsec.org` and the advisory-db tarball and GitHub API are blocked by
  the proxy; only `raw.githubusercontent.com` answers, and it cannot list a directory. Verified by single-file fetch: `time` RUSTSEC-2020-0071
  (patched >= 0.2.23, lock 0.3.47), `rand` RUSTSEC-2026-0097 (patched >= 0.8.6, lock 0.8.6), `tokio` RUSTSEC-2021-0124 and 2023-0001 (patched
  below the lock), `tracing-subscriber` RUSTSEC-2025-0055 (patched >= 0.3.20, the lock has 0.3.23). I found none open for the lock but could not list
  every advisory per crate. `cargo-deny`/`cargo-vet` stay the open half of row 217.
- **`unsafe` numbers** are text-greps (comments included), not `cargo-geiger` (not installed); they bound, not count, real `unsafe` blocks.
- **Maintainer trust** is from crates.io owners, dates and cadence only; no check of 2FA, release provenance or ownership-change history (not
  exposed by the API).
- The palette and time feature trims were applied and measured on 2026-10-25 (row 371: `color-picker` 160 -> 150, `calendar` 152 -> 151 wasm crates);
  compile-time deltas stay unmeasured.

## 7. Summary of outcomes (proposals awaiting owner approval)

| Dependency | Outcome | Reason |
|---|---|---|
| dioxus, dioxus-attributes | keep | framework (veto); ours |
| time | keep, gated; trim `macros` | calendar math is a veto domain; `date!` in 10 non-test defaults is replaceable by const `Date::from_calendar_date` |
| palette | gated; trim default features | colour science veto; `named_from_str` pulls 7 unused crates |
| phf, phf_*, rand, rand_core, siphasher | drop (with palette trim) | unused transitive stack, one proc-macro |
| tracing, serde | keep | 0 marginal crates |
| tokio (native), gloo-timers (wasm) | keep (gloo-timers: optional replace) | one sleep; gloo-timers saves 1 crate if replaced |
| dioxus-icons | replace (lift the 12 icons) | build script and 25 MB source in 30 components' builds for 12 glyphs |
| pulldown-cmark | keep, own component, exact pin | parser veto; trust proportionate |
| approx, by_address, fast-srgb8, unicase, pulldown-cmark-escape | keep | inherent to a kept crate |
| playwright / preview JS | keep | exact pins, 30-day age, no install scripts |
