---
id: 33
title: "3-browser Playwright on every push/PR, sharded — dignifiedquire/dx-components runs chromium+firefox+webkit…"
status: "CI reactivation (user: off until further notice, 2026-08-30) — then it is a workflow-trigger…"
size: "S"
area: ci
updated: 2026-08-30
---

# Row 33: 3-browser Playwright on every push/PR, sharded — dignifiedquire/dx-components runs chromium+firefox+webkit…

## Item

3-browser Playwright on every push/PR, sharded — `dignifiedquire/dx-components` runs chromium+firefox+webkit 4-way on every push; ours is `workflow_dispatch`-only and has never executed a non-Chromium run (row 4)

## Status / trigger

**CI reactivation** (user: off until further notice, 2026-08-30) — then it is a workflow-trigger change, not new capability. The dispatch-only `main.yml`, `all_components.yml`, `playwright.yml` and `stylelint.yml` were deleted by row 266 (their checks run in `scripts/run-gates.sh`, the Playwright suite and `scripts/registry-install-smoke.sh`); reactivating CI restores them from git history (`git show e5bdf652^:.github/workflows/<name>.yml`).

## Size

S

## Detail

From the 2026-09-03 fork comparison
