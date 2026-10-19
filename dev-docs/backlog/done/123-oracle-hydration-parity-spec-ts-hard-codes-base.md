---
id: 123
title: "oracle/hydration-parity.spec.ts hard-codes BASE = \"http://127.0.0.1:8090\"; 5 cases fail when a base-path…"
status: "LANDED 2026-10-04 (8c99edd) -- was: OPEN -- found 2026-10-03 (chart round): 5 hydration-parity…"
size: "S"
area: tests
updated: 2026-10-04
---

# Row 123: oracle/hydration-parity.spec.ts hard-codes BASE = "http://127.0.0.1:8090"; 5 cases fail when a base-path…

## Item

`oracle/hydration-parity.spec.ts` hard-codes `BASE = "http://127.0.0.1:8090"`; 5 cases fail when a base-path build is served at root

## Status / trigger

**LANDED 2026-10-04** (`8c99edd`) -- was: **OPEN** -- found 2026-10-03 (chart round): 5 hydration-parity failures when the `--base-path dioxus-components` deploy build was served at `/` instead of under the prefix

## Size

S

## Detail

`playwright/oracle/hydration-parity.spec.ts:346` is `const BASE = "http://127.0.0.1:8090"`, with an expected `SSG_SITE_DIR` serving a **no-base-path** build at `/`. Pointing it at the deploy build (which carries `/dioxus-components/` in every URL, row 100) produces 5 failures that are a test-setup mismatch, not hydration regressions -- but they look identical to real ones. **Fix by construction:** derive `BASE` from an env var (`BASE_URL`, as `navigation_menu.spec.ts` already does after commit `5d97f89`) and have the spec assert, once, that the served root has no `/dioxus-components/` references (the row-100 signature) and say so in the failure message, so a mis-served build reads as "wrong build served" rather than five hydration failures. **2026-10-04 addendum -- LANDED (`8c99edd`):** `BASE` derives from `PLAYWRIGHT_BASE_URL` (default `:8090`), and a new Rule 0 asserts the served root carries no `/dioxus-components/` references and names a wrong-base-path build as such. Evidence on release 9f8399d, served at root: `hydration-parity.spec.ts` 41 passed / 1 failed; Rule 0 green; the one failure is the by-design `Rule 4c: selftest:unfixed:id` (`KNOWN_RED`), so every real rule is green (`baseline/RELEASE.md` section 3d).
