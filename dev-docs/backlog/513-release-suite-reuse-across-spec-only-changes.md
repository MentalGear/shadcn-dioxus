---
id: 513
title: "A spec-only fix after a red release suite re-runs the whole suite: reuse the results the changed files cannot affect"
status: "BUILT 2026-11-14 (approved by the owner the same day). It serves the owner's standing goal…"
size: "M (inferred): the key derivation, the…"
area: tests
updated: 2026-11-14
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

BUILT 2026-11-14 (approved by the owner the same day). It serves the owner's standing goal "evaluate how the testing
could be made quicker without losing quality". It changes what the deploy accepts as proof: three independent review
passes attacked it, and the construction below is what held. Open: one real batch where a second run executes only
the changed files.

## Size

M (inferred): the key derivation, the per-file record, the deploy's acceptance rule, a self-test.

## Detail

- **The construction.** A suite result is keyed by what it depends on, per spec file: (the site key, the spec key).
  The site key is the hash of every input of the release build: the tree without `playwright/`, `dev-docs/`, `docs/`
  and `primitives/ts-test/`, plus every excluded file a Rust source includes (`dev-docs/motion.md`), plus the build
  tools' versions. The spec key is the hash of the spec file, the files it imports (transitively, including
  `primitives/ts-test/`), one shared hash of the tools, the config's and fixtures' imports and every file under
  `playwright/` that no spec imports (data, vendored scripts, the lock file; `durations.json` excluded because only the
  chunk cutter reads it), the environment variables that code reads, the browser and the installed Playwright. A
  change to a helper re-runs the specs that import it; a change to the site, the config, a tool or a data file re-runs
  everything. What the key cannot classify (a `process` that could read the environment unseen, a test that names or
  enumerates spec files or runs a program, a missing parser, an installed Playwright that differs from the lock)
  leaves the file or the suite without a key, so it runs.
- **What must hold.** A record is green only by proof: the report's tests equal an independent `--list` of the same
  files, each expected or flaky; a file with a skipped, interrupted or unrun test never gets a record and runs every
  time. The built `public/` directory is kept with its hash (about 55 MB, the two newest kept) and served again; a
  record names that hash. The suite builds and runs in a private worktree of the planned tree
  (`scripts/lib/pinned-worktree.sh`), and the deploy refuses to build `docs/` from a checkout that is no longer that
  tree. A flaky pass is recorded as green and listed.
- **Residue.** A spec that reads, by path, a helper another spec imports is reused when that helper changes. The site
  key bounds the cost: a reused result was earned on byte-identical site bytes, so the miss can hide only a test-side
  regression, never an untested site, and it surfaces at the next run that executes the file (the next change to the
  site, to a helper in its closure, to the tools or to the config). The self-test and the end-to-end script each pin
  this case. The threat model is an honest mistake in a spec; forms nobody writes by accident (`Reflect`, `Function`,
  `eval`, a split file name) are outside it and named in the header of `playwright/tools/suite-key.mjs`.
- **Files owned:** `scripts/deploy-preview.sh` (the stamp), `playwright/tools/run-suite.sh`,
  `playwright/tools/suite-key.mjs`, `import-graph.mjs`, `spec-files.mjs`, `suite-reuse-e2e.sh`, their self-tests,
  `scripts/check-suite-reuse.sh`, `scripts/check-suite-reuse-e2e.sh`, `scripts/lib/pinned-worktree.sh`,
  `scripts/gate-at.sh`, `dev-docs/lane-rules.md` ("One verifier per batch").
- **Proof:** the self-test (a spec-only change runs only the changed file; a helper change re-runs its importers only;
  a site, tool, config or data change runs everything) with its mutations; the end-to-end script through the real
  `run-suite.sh`; on real history the fix `8cbb1a04` to `c7e66b74` re-runs 2 of 287 spec files and `0905d5e4` to
  `383371e8` re-runs 3 of 291; one real batch where a second run executes only the changed files and the deploy
  accepts the tree.
- **Measured cost of the skip rule (2026-11-14).** The 22 skipped tests sit in 8 spec files, about 4% of the suite's
  time; those files run on every deploy.
- **Related evidence for row 510.** The red test was in `playwright/virtual-engine.spec.ts`: row 454 changed the
  scroll fade's fallback mechanism in `primitives/src/scroll_area.rs`, and the lane's by-name selection (the
  scroll-area specs) did not include the virtual grid's spec, which reaches the same hook through
  `virtual_grid.rs`. A selection narrow enough for a lane and following primitive-to-primitive use would have named
  it ([row 510](./510-spec-selector-component-edges.md), open item).
