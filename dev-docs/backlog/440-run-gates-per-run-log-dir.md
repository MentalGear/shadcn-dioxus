---
id: 440
title: "run-gates.sh writes logs to a shared directory: two concurrent runs on one target overwrite each other's logs"
status: "BACKLOG. Found by the row 433 lane (2026-10-26)."
size: "S"
area: tooling
updated: 2026-10-26
---

# Row 440: one log directory per run-gates run

## Item

`scripts/run-gates.sh` writes every gate's log to `$CARGO_TARGET_DIR/run-gates/<gate>.log`. Two lanes ran `--fast` on the shared `/home/user/tgt/check` target at the same time. One run overwrote the other's logs, and the second lane read a false `check-backlog-index` red from the first lane's log. The verdict printed in the summary comes from the gate's exit code, but the log a person or agent opens may belong to a different tree.

## Status / trigger

BACKLOG. Found by the row 433 lane (2026-10-26).

## Size

S

## Detail

- Construction: every run writes to its own directory, for example `run-gates/<HEAD short>-<pid>/`, and prints that path. A `latest` symlink is switched atomically at the end of the run. The summary's log paths point into the run's own directory.
- A test in `check-lane-lock.sh` style starts two concurrent `--only` runs on one target and checks that each summary's log paths hold that run's output.
