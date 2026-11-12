---
id: 440
title: "run-gates.sh writes logs to a shared directory: two concurrent runs on one target overwrite each other's logs"
status: "DONE 2026-11-05 (lane gate-logs-440), landed in PR #165 (merge d040554a); the automated…"
size: "S"
area: tooling
updated: 2026-11-12
---

# Row 440: one log directory per run-gates run

## Item

`scripts/run-gates.sh` writes every gate's log to `$CARGO_TARGET_DIR/run-gates/<gate>.log`. Two lanes ran `--fast` on the shared `/home/user/tgt/check` target at the same time. One run overwrote the other's logs, and the second lane read a false `check-backlog-index` red from the first lane's log. The verdict printed in the summary comes from the gate's exit code, but the log a person or agent opens may belong to a different tree.

## Status / trigger

DONE 2026-11-05 (lane gate-logs-440), landed in PR #165 (merge `d040554a`); the automated concurrent-run test landed 2026-11-07 (`scripts/check-gate-logs.sh`). Nothing is open. Found by the row 433 lane (2026-10-26).

Landed as constructed: each `run-gates.sh` run writes to `$CARGO_TARGET_DIR/run-gates/<UTC time>-<pid>/` (guard 7), the summary prints `logs: <dir>`, `run-gates/latest` is switched by an atomic rename at the end, the newest `RUN_GATES_KEEP` (20) run dirs are kept, `--log-dir` still pins a directory (no `latest`, no pruning). The reuse store (format 2) records the log path of the run that really ran a gate and a reused line prints it (`(pruned)` when retention removed it). `gate-at.sh` prints the `logs:` line. Proof: two concurrent `--fast --only` runs on one target got `...-3541` and `...-3542`, each holding only its own gates' logs; a second `--only check-attr-spread-collision` run printed `reused ... log <first run's dir>/check-attr-spread-collision.log`; full `--fast` 95 gates green. `check-gate-logs.sh` runs two concurrent `--fast --only` runs on a throwaway target and pins separate run dirs, own `logs:` lines, `latest` -> a finished run, and retention. Writing it found that the retention `find -regex` (emacs syntax, `\{8\}`) matched nothing, so nothing was ever pruned; it now uses `-regextype posix-extended`, and a run writes a `finished` marker so retention deletes only finished runs (an unfinished one only after `RUN_GATES_LIVE_MIN`, default 120, idle minutes).

## Size

S

## Detail

- Construction: every run writes to its own directory, for example `run-gates/<HEAD short>-<pid>/`, and prints that path. A `latest` symlink is switched atomically at the end of the run. The summary's log paths point into the run's own directory.
- A test in `check-lane-lock.sh` style starts two concurrent `--only` runs on one target and checks that each summary's log paths hold that run's output.
