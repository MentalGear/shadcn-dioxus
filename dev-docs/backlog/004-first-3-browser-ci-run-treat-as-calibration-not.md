---
id: 4
title: "First 3-browser CI run — treat as calibration. Not done — see 2026-09-04 Notes below for what's now known and…"
status: "WAITING ON OWNER: CI reactivation (off until further notice, 2026-08-30). The GitHub-hosted…"
size: "S–M"
area: ci
updated: 2026-10-24
---

# Row 4: First 3-browser CI run — treat as calibration. Not done — see 2026-09-04 Notes below for what's now known and…

## Item

First 3-browser CI run — treat as calibration. **Not done — see 2026-09-04 Notes below for what's now known and what's still open.**

## Status / trigger

**WAITING ON OWNER: CI reactivation (off until further notice, 2026-08-30).** The GitHub-hosted `workflow_dispatch` calibration run of the 3-browser Playwright job has never executed; local calibration is possible (all three engines installed since 2026-09-04). The dispatch-only `main.yml`, `all_components.yml`, `playwright.yml` and `stylelint.yml` were deleted by row 266 (their checks run in `scripts/run-gates.sh`, the Playwright suite and `scripts/registry-install-smoke.sh`); reactivating CI restores them from git history (`git show e5bdf652^:.github/workflows/<name>.yml`).

## Size

S–M

## Detail

Expect engine surprises (spike hazards were Chromium-only findings); budget one fix round. **2026-09-04, first local calibration pass, Chromium half only:** **525 passed, 13 failed.** After triage, zero of those 13 were regressions from this session — 5 were `oracle/hydration-parity.spec.ts` connection-refused failures caused by a dev-server/SSG config mismatch, not a product defect (row 22's 2026-09-04 addendum), and the rest map to already-tracked residual flakes (row 21's addendum, same session). A firefox+webkit run was started in this same sandbox and **deliberately STOPPED partway (178/1094, 14 failures recorded in firefox) as a scope decision** — this is user-gated group-6 work (see "Fresh-session order" item 6) and was blocking the queue, not a crash or an environment failure. Do not imply the cross-engine half was completed; a fresh session picking this back up starts from a clean rerun, not a resume of the stopped one, since nothing about the 178/1094 partial state was preserved. Chromium's own run stands unchanged at 525 passed / 13 failed with zero regressions attributable to this session. **2026-09-14 addendum: the 2026-09-04 "all three engines installed" state does not carry over to a fresh sandbox.** This session's own sandbox (used to build and verify PR https://github.com/MentalGear/dioxus-components/pull/51's changes) had only Chromium under `/opt/pw-browsers` (`chromium`, `chromium-1194`, `chromium-1223`, `chromium_headless_shell-1194`, `chromium_headless_shell-1223`, `ffmpeg-1011`) — no `firefox-*`/`webkit-*` directories at all, confirmed by directory listing, not assumption. This session's own environment instructions explicitly prohibit running `npx playwright install` in this sandbox (a hard rule, not contingent on whether it would technically succeed — chromium's own extra revision, `-1223`, appears to have been fetched successfully by some `npx playwright test` invocation this session despite `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`, so downloads are not categorically blocked, but the instruction to avoid `playwright install` here stands regardless), so this session did not attempt to add Firefox/WebKit and PR #51 ships Chromium-only verification, called out plainly in its own description. **Whether "a working Playwright container with all three engines" is available is apparently per-sandbox, not a standing property of this project's environment** — a fresh session should verify `/opt/pw-browsers`' actual contents itself rather than trust this row's 2026-09-04 note, and should not assume `playwright install` is safe/available without first checking its own environment instructions for an explicit prohibition like this session's
