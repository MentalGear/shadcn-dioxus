---
id: 371
title: "Dependency evaluation: keep, feature-gate, replace, lift or drop (rule of 2026-10-23)"
status: "BACKLOG: unparked by the owner 2026-10-24, stage 4 of the adopted order (after the structure rows…"
size: "S each outcome"
area: build
updated: 2026-10-24
---

# Row 371: Dependency evaluation: keep, feature-gate, replace, lift or drop

## Item

Apply the owner's dependency rule (2026-10-23) to every dependency a user's `dx components add` compiles, and decide per dependency: keep, feature-gate, replace, lift or drop.

## Status / trigger

**BACKLOG: unparked by the owner 2026-10-24, stage 4 of the adopted order (after the structure rows, before the review pass and row 365).** Inventory done, no outcome approved; each outcome goes to the owner before it is applied.

Owner notes for when it is picked up:

- If `dioxus-icons` is replaced by the lifted glyphs, the full `dioxus-icons` set stays available to developers as an opt-in.
- Scope: SECURITY covers everything that builds or touches the project (the whole workspace lock, the docs site, build scripts, dev tooling, the CLI); only the PERFORMANCE budgets (crate count, compile time, wasm bytes) are measured on the closure users compile.
- Vendoring (`cargo vendor`) was assessed and is not adopted: vendored code would need the same security checks, and `Cargo.lock` checksums with crates.io's immutable versions already pin the exact bytes, `cargo vet diff` reviews each update without copying source, a yanked version still builds from the lock, and crates.io deletes only malware (which a vendored copy would keep). Lift only where it protects users, and scan lifted code like a dependency (`lifted-from: crate@version`, advisory match, `unsafe` and licence checks).
- Documented: the repository's whole supply-chain approach (scope, threat model, the controls table with every item of this row's construction half marked `planned`, how to adopt it elsewhere) is [`dev-docs/supply-chain/README.md`](../supply-chain/README.md); its drift gate is `scripts/check-supply-chain-doc.sh`. When a planned control from this row lands, flip its row there to `enforced` (the gate turns red until you do).
- Construction half: the scanner research and an independent review of these criteria in [`dev-docs/research/dependency-scanners.md`](../research/dependency-scanners.md) (cargo-deny now, cargo-vet second, a pinned advisory db, gate the whole closure, shrink-only ratchets, `lifted-from:` headers, a schema-checked register).

Research and every measured fact: [`dev-docs/research/dependency-evaluation.md`](../research/dependency-evaluation.md); re-runnable with `scripts/dep-report.sh [--network]`. Follows row 217 (does not redo it).

## Size

S each outcome

## Detail

The rule: a dependency qualifies when (1) security (code risk and channel risk, weighed by reach against trust), (2) cost (compile time, crate count, wasm bytes) or (3) small stable low-risk slice; veto for time zones, colour science, parsers and the framework. Outcome ladder, cheapest and most reversible first: drop > feature-gate > pin exactly > replace (std / web-sys / browser API) > lift > keep. Blast radius is counted per crate in the research doc.

**Proposed outcomes (none applied):**

1. `palette`: `default-features = false, features = ["std"]`. The default `named_from_str` pulls phf, phf_generator, phf_macros, phf_shared, rand, rand_core and siphasher (7 crates, 1 proc-macro) for string colour names we never parse. Drop rung. Unbuilt estimate for ColorPicker users: 12 -> about 5 extra crates.
2. `time`: keep (veto), but `macros` is used only for 10 non-test `date!` defaults; use const `Date::from_calendar_date`, make `macros` a dev-dependency, drop `time-macros` (1 proc-macro) from users' trees.
3. `dioxus-icons` (30 of 83 components): its `build.rs` runs `lazy-js-bundle` in the user's build and compiles 25 MB of icon source for 12 glyphs. Replace rung: lift the 12 Lucide SVGs (ISC) into our own `icons.rs`. Reach out of proportion to trust.
4. `gloo-timers`: optional replace by a ~15-line `setTimeout` future (-1 crate on wasm); low value, owner's call.
5. Keep: dioxus, dioxus-attributes, tracing, serde, tokio (native), time, pulldown-cmark (parser veto, exact pin, re-open on each 0.13.x), the JS tooling.

**Not measured (see the doc, section 6):** wasm bytes per crate (the batch-10 build is debug and stripped, no sizing tool installed), compile-time deltas, complete RustSec advisory lists (`cargo-audit` absent, rustsec.org blocked), real `unsafe` counts (`cargo-geiger` absent). The trims are read from manifests and must be built and gated before anyone promises them.

**Proposed construction (not built):** a dependency register, `dev-docs/dependency-register.tsv`, one entry per direct dependency: outcome, reach class (compile-time code / unsafe / untrusted input / fs-network / none), evidence date, pinned version. Plus `scripts/check-dependency-register.sh` (a `check-*.sh` gate in `run-gates.sh`, no network by default) that fails on: a new direct dependency not in the register; a new `build.rs` or proc-macro anywhere in a user's closure (`cargo tree` per feature set, as `scripts/dep-report.sh` does); a new RustSec advisory or licence outside the allowlist (when `cargo-deny` or an advisory-db checkout is available); a transitive-crate or wasm-size budget breach (a budget file like `playwright/durations.json`'s `budget_seconds`, raised only by a stated one-number diff); and that re-opens an entry when its pinned version changes (the entry's evidence date goes stale). This turns the audit from a document into a class-closing gate, as the owner principle asks.
