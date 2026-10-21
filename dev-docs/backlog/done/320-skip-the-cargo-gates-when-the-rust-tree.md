---
id: 320
title: "Skip the cargo gates when the Rust tree is unchanged since the last green full gate"
status: "Landed 2026-10-21 (lane/gate-speed 0f857cad, owner approved 2026-10-21, condition: build only if a…"
size: "S-M"
area: process
updated: 2026-10-21
---

# Row 320: Skip the cargo gates when the Rust tree is unchanged since the last green full gate

## Item

Skip the cargo gates when the Rust tree is unchanged since the last green full gate.

## Status / trigger

**Landed 2026-10-21 (lane/gate-speed `0f857cad`, owner approved 2026-10-21, condition: build only if a measurement holds up -- it does)**

## Size

S-M

## Detail

fmt, the three clippy arms, `cargo test` and `check-primitives-features` are ~20 of the full gate's ~22 min. A CSS-, spec- or docs-only merge batch (most of batch A's fixes, the triage merge) re-ran them for nothing: about 2 of the 4 full gates on 2026-10-20. Construction: `run-gates.sh` records, per green run, a hash of the cargo inputs (every `*.rs`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `*.ts` bundled into primitives, the toolchain) and reuses the cargo results for an identical hash, printing that it did so; any other change re-runs them. The summary must still say which tree each result came from, so `ALL GREEN` keeps meaning the quoted tree.


## Measurement (2026-10-21)

- **What the cargo gates cost.** Gate log of `f127233a` (1,338.6 s, all green): `check-primitives-features` 130.2 s,
  `clippy-default` 256.1 s, `clippy-web` 227.3 s, `clippy-server` 270.7 s, `test` 310.1 s = 1,194 s (89 %), plus
  `check-attr-spread-collision` 21 s (a cargo build too) = 1,215 s (91 %). fmt is 0.8 s and stays always-run.
- **How often a gated tree changes no cargo input.** The 9 full-gate logs of 2026-10-19/20 (`/home/user/tgt/gates-*.log`),
  hashed with the new input set at each logged commit: 2 of the 8 consecutive pairs have an identical hash --
  `8a7b75a9` -> `e145e393`, and the red fast-gate run `076f9781` -> its rerun `f299fde9` (that rerun re-ran ~21 min of
  cargo for a docs/spec fix). That is ~42 min of ~200 min of gate time (~20 %), and the second pair only helps because
  a PASS is stored even from a run that was red elsewhere (a deliberate widening of "a fully green run": a cargo gate's
  result does not depend on the fast gates). First-parent merges of main since 10-19: 12 of 25 left the hash unchanged
  (deploy refreshes, backlog and CSS/doc PRs), 13 changed it.
- **Not free of Rust-adjacent inputs.** Tests read `preview/assets/main.css` and `theme-presets.css`
  (`preview/src/theme.rs` `include_str!`s them and asserts on them), `dev-docs/motion.md` (`motion_docs.rs`), every
  component `docs.md` (`main.rs` tests and `preview/build.rs`) and `preview/tests/shadcn/*.json`; so those are inputs
  and a change to one reruns the gates. Per-component `style.css`, spec, docs-in-`dev-docs`, backlog and deploy-only
  changes are not.

## Design

`scripts/cargo-inputs-hash.sh` (the input set, derived from the working tree on every run), `scripts/gate-reuse.sh`
(key + store), `run-gates.sh` guard 7 (lookup before each cargo gate, record after, summary), `scripts/check-gate-reuse.sh`
(gate; pins the promises). Input set:

1. Content-hashed: every file under `primitives cli preview test-harness dioxus-attributes
   scripts/attr-spread-collision-checker templates` EXCEPT `css svg png jpg jpeg gif webp ico woff woff2`; the root
   `Cargo.toml`/`Cargo.lock`, `rust-toolchain[.toml]`, `rustfmt.toml`, `clippy.toml` (and dotted forms), `.cargo/*`, the root
   `component.json`; `scripts/check-primitives-features.sh`, `scripts/check-attr-spread-collision.sh` and its two
   baselines; and every file a literal `include_str!("..")`/`include_bytes!("..")` in those roots resolves to (so the CSS
   the tests assert on, `dev-docs/motion.md` and `primitives/README.md` are in, and a new include is covered the day it is written).
2. Path-only: the asset kinds above under the roots that no include pulls in -- reached only through `asset!`, which needs
   the file to exist but whose bytes cannot change a compile, clippy or test result; add, delete and rename still change the hash.
3. The key also holds the format version, `rustc -Vv`/`cargo -V`/`cargo clippy -V`, `RUSTFLAGS`/`RUSTDOCFLAGS`/
   `CARGO_ENCODED_RUSTFLAGS`/`CARGO_BUILD_TARGET`, the exact command line of every cargo gate and the bytes of the two helper scripts.
4. Reuse rules: only PASS is stored (one file per key and gate, written by rename), a FAIL deletes the stored PASS, results are
   stored only if the key recomputed after the gates equals the one taken before, `fmt` and every fast gate always run,
   `--no-reuse`/`GATE_REUSE=0` force a full run, the store is `$LANE_TGT/gate-reuse` (`GATE_REUSE_DIR`), keys older than 21 days are pruned.
5. By construction guards (`check-gate-reuse.sh`): every gate that runs cargo is in `CARGO_GATES` or reviewed as never-reused,
   and any Rust file that reads at run time through `CARGO_MANIFEST_DIR` must be in the reviewed list -- a new reader is
   red until someone shows its input is inside the set. Known limit: a reader that builds its path some other way is not seen.

## Proof

- Full gate on the lane tip `0f857cad` (cold store): `/home/user/tgt/gates-speed-P1-full.log`, 66 gates, ALL GREEN, 1,208.7 s,
  stored 6 PASS under key `f4ba7d34...`.
- Full gate on a scratch commit `0ab03e36` (CSS-only `button/style.css` + a `STATUS.md` line + the row-324 tree, never pushed):
  `/home/user/tgt/gates-speed-P2-css.log`, 67 gates, ALL GREEN, **127.2 s**, "6 cargo gate(s) reused from tree 0f857cad".
- A Rust-touching scratch commit `e4621a44` (a comment in `primitives/src/lib.rs`): `/home/user/tgt/gates-speed-P3-rust.log`,
  a different key (`1fe99eda...`), nothing reused, `check-primitives-features` and `check-attr-spread-collision` really ran.
- `--no-reuse`: "reuse off ... every cargo gate runs".
