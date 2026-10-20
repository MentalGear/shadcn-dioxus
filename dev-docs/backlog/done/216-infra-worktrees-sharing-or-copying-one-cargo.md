---
id: 216
title: "Infra: worktrees sharing or copying one CARGO_TARGET_DIR build and lint another tree's code (re-confirms row…"
status: "DONE 2026-10-19 (lane lock): scripts/lane-touch.sh with a .tree-owner stamp, called by build-ssg…"
size: "S-M"
area: process
updated: 2026-10-19
---

# Row 216: Infra: worktrees sharing or copying one CARGO_TARGET_DIR build and lint another tree's code (re-confirms row…

## Item

Infra: worktrees sharing or copying one `CARGO_TARGET_DIR` build and lint another tree's code (re-confirms row 68); cold-build lost log

## Status / trigger

**DONE 2026-10-19 (lane lock): scripts/lane-touch.sh with a .tree-owner stamp, called by build-ssg, run-gates, lane-cargo, lane-ssg and gate-at; lane-target.sh create clears it. The cold-build lost-dx-log observation is closed: a tee/dx race on a not-yet-created target root, fixed by creating the root before dx**

## Size

S-M

## Detail

**Hazard.** Cargo gives a workspace crate the same unit hash in every worktree and judges freshness by mtime, so a worktree whose files are older than another tree's artifact silently builds or lints ANOTHER tree's code. Measured by the charts, focus, date and layout lanes: SSG output without their change, clippy/test runs that skipped their new tests, and 3 of 5 "stylesheet not in dx's asset manifest" build failures. Re-confirms row 68 (cross-worktree cargo build-directory hazard). **Workaround used:** touch the worktree's sources before every cargo/dx run, and a fresh per-build copy of the template under one lock. **By-construction candidate:** `scripts/lane-target.sh` (and `build-ssg.sh` whenever the target is not fresh) touches the tree itself, so the rule needs no lane to remember it; also have the lane verify its change is in the built output. **Second observation (not root-caused):** a cold first `build-ssg.sh` run lost its own dx log mid-run (`ssg-verify`: log not found); the warm rebuild was fine; record it here if seen again. Environment note for lanes: `dev-docs/agent-model-fit.md`.
