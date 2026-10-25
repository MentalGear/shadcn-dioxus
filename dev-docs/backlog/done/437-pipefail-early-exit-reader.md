---
id: 437
title: "No early-exit reader on the right of a pipe in a pipefail script (grep -q, head, sed q, awk exit)"
status: "DONE. Reported by the main loop from a run-gates.sh --fast run; seen twice, so a class (CLAUDE.md…"
size: "S"
area: tooling
updated: 2026-10-25
---

# Row 437: No early-exit reader on the right of a pipe in a pipefail script

## Item

`scripts/check-css-delivery.sh` went red on `queue/batch-27` with "preview/src/eager_head.rs no longer reads data-covers" although the file has four `data-covers` hits and the gate passes alone. The script runs under `set -euo pipefail` and did `printf '%s\n' "$code" | grep -q 'data-covers' || fail ...`. `grep -q` exits at its first match, `printf` can then die of SIGPIPE (141), and pipefail makes the pipeline fail, so a match became a failure. It depends on timing and load, so a gate goes red on a correct tree, and the inverted shape (`if ! cmd | grep -q x`, `if cmd | grep -qE forbidden; then fail`) can go green on a broken one.

## Status / trigger

DONE. Reported by the main loop from a `run-gates.sh --fast` run; seen twice, so a class (CLAUDE.md "same problem more than once").

## Size

S

## Detail

- **The class:** a reader that stops before end of input (`grep -q`/`-qE`/`-qF`/`--quiet`, `grep -m N`, `grep -l` on stdin, `head`, `sed ... q`, `awk ... exit`) on the right of an unquoted pipe, in a script that sets `pipefail`. The left side may be killed by SIGPIPE and pipefail reports 141 for the pipeline. It also bites when `head` ends a `$(...)` that is a condition (`if hit="$(grep -rl ... | head -n 3)" && [ -n "$hit" ]` in `build-ssg.sh`: more than three hits and the base-path leak check passed silently).
- **33 sites in 13 scripts** (108 of the repo's shell scripts set pipefail): `check-css-delivery` (8), `check-eager-head-document` (8: five `grep -q` on `printf "$code"`, one `grep -B1 | grep -q`, two `grep -n | head -1` in `$(...)`, where SIGPIPE under `set -e` would also kill the script), `session-setup` (3), `dev-wait` (3, `grep -m1`), `build-ssg` (3, `head -n`), `check-demo-forms`, `check-direction-keys`, `check-lane-lock`, `cargo-inputs-hash`, `dep-report`, `registry-install-smoke` (`sort | head -40` before an `exit 1`), `ssg-verify`, and `playwright/tools/test-changed.sh` (a miss there would skip building attr-synth before `hydration-parity`). Not affected: `check-supply-chain-doc.sh` and `gate-reuse.sh` (no pipefail).
- **Construction:** every site now reads a variable (`grep -q PAT <<<"$code"`), a file, or takes a prefix with `sed -n '1,Np'` (reads to the end, so the writer never sees SIGPIPE); `grep -m1` stays only on the LEFT of a pipe or alone. Each check keeps its meaning (`if ! grep -q ... <<<"$(cmd)"` keeps "command failed" = "not found").
- **Gate:** `scripts/check-gate-kit.sh` rule 7 scans every `*.sh` in the repo (node_modules, target and dist skipped) that sets pipefail with a quote-aware scanner (a stack through `'...'`, `"..."` and `$(...)`, so `x="$(cmd | head -n 3)"` is seen; joins continuation lines; skips comments and heredoc bodies). A `# pipe-ok: <reason>` marker on the line or the one above exempts a site whose left side is known to be tiny; a marker without a reason or without a site fails. No marker is used today.
- **Self-test:** 32 red fixtures (every reader spelling, `$(...)`, continuation lines, a nested substitution, `|&`), 28 green ones (here-strings, file arguments, `grep -c/-v`, `sed -n 'a,bp'`, `||`, `>|`, comments, heredoc bodies, quoted pipes, markers), the scope fixtures (a script without pipefail, a commented-out `set -o pipefail`), and 29 scanner mutations that each must turn a fixture red.
- **Does not subsume:** a reader that exits early through other means (`read` after a pipe, `while read ...; do break`), `xargs grep -q`, and workflow YAML `run:` blocks (none has the shape today). Scripts without pipefail are out of scope: there the pipeline status is the last command's.

## Outcome

- **Proof of the bug:** the old `check-css-delivery.sh` run 400 times (two batches of 4 parallel workers x 50 under 6 `yes > /dev/null` busy loops on 4 cores) failed 4 times on the correct tree, each time with a different "no longer ..." message (create_link, the font-face check); the fixed one failed 0 of 400 under the same load.
- **Fix:** 33 sites rewritten in 13 scripts; `cargo-inputs-hash.sh` output (hash and `--list`) is byte-identical before and after on the same tree.
- **Gate:** `check-gate-kit` is red with 33 problems on the pre-fix scripts and green on the fixed ones.
