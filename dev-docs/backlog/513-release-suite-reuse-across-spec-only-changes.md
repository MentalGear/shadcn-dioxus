---
id: 513
title: "A spec-only fix after a red release suite re-runs the whole suite: reuse the results the changed files cannot affect"
status: "BACKLOG. It serves the owner's standing goal \"evaluate how the testing could be made quicker…"
size: "M (inferred): the key derivation, the…"
area: tests
updated: 2026-11-13
---

# Row 513: the release suite reuses results across a spec-only change

## Item

`scripts/deploy-preview.sh` records one stamp per tree (`$LANE_TGT/release-suite/<tree>.pass`). Any commit after a red
run changes the tree, so the whole suite runs again, including when the commit touches only Playwright files and the
site it tests is byte-identical. Measured on 2026-11-13 (the batch of rows 451, 454 and 511): the suite on `8cbb1a04`
took 3,243 s inside the lock plus the release build and ended 4,107 passed, 1 failed; the fix (`c7e66b74`) changed three
files under `playwright/` and nothing the site is built from; the run on the new tree repeats the 4,107 results the
change cannot affect.

## Status / trigger

BACKLOG. It serves the owner's standing goal "evaluate how the testing could be made quicker without losing quality".
It changes what the deploy accepts as proof, so it needs the owner's approval before it is built.

## Size

M (inferred): the key derivation, the per-file record, the deploy's acceptance rule, a self-test.

## Detail

- **The construction.** A suite result is keyed by what it depends on, per spec file: (the site key, the spec key). The
  site key is the hash of every input of the release build (the tree without `playwright/`, `dev-docs/`, `docs/` and
  the row files). The spec key is the hash of the spec file and of every module it imports, transitively, plus the
  Playwright config, the fixtures and `pnpm-lock.yaml`. A run executes only the spec files whose key has no green
  record and writes one record per file; the deploy accepts a tree when every spec file of that tree has a green
  record under the tree's site key. A change to the site, to the config or to a shared helper invalidates everything
  it reaches, by the hash, with no rule to maintain.
- **What must hold.** The release build of the first run is kept (or rebuilt from the same site key, which is
  deterministic for the content-hashed assets) for the second run; a spec generated from the built site reads the
  same site, so its key is unchanged; a flaky pass is recorded as today (listed, allowed).
- **Residue.** A spec that depends on something outside its imports (another spec's side effect, a file it reads at
  run time by a computed path) would be reused wrongly; the self-test plants one of each known kind, and
  `playwright/tools/component-graph.mjs` already lists the run-time reads the selector knows.
- **Files owned:** `scripts/deploy-preview.sh` (the stamp), `playwright/tools/run-suite.sh`, a new key tool under
  `playwright/tools/`, their self-tests, `dev-docs/lane-rules.md` ("One verifier per batch").
- **Proof:** the self-test (a spec-only change runs only the changed file and its dependants; a helper change runs
  every importer; a site change runs everything; a config change runs everything); one real batch where a second run
  executes only the changed files and the deploy accepts the tree.
- **Related evidence for row 510.** The red test was in `playwright/virtual-engine.spec.ts`: row 454 changed the
  scroll fade's fallback mechanism in `primitives/src/scroll_area.rs`, and the lane's by-name selection (the
  scroll-area specs) did not include the virtual grid's spec, which reaches the same hook through
  `virtual_grid.rs`. A selection narrow enough for a lane and following primitive-to-primitive use would have named
  it ([row 510](./510-spec-selector-component-edges.md), open item).
