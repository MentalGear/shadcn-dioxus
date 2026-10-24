---
id: 359
title: "JS tooling: pnpm in playwright/ and preview/ with minimumReleaseAge, exact pins and install scripts off"
status: "LANDED 2026-10-22 (batch 7, PR #134); owner raised minimumReleaseAge to 30 days in batch 9 (PR…"
size: "S-M, about half a day."
area: tooling
updated: 2026-10-23
---

# Row 359: JS tooling: pnpm in playwright/ and preview/ with minimumReleaseAge and install scripts off

## Item

Harden the supply chain of the two JS folders (`playwright/`, `preview/`) by replacing npm with pnpm. The move should refuse freshly published versions and never run dependency install scripts. Everything stays on one toolchain. Bun was evaluated and rejected: under a second of gain, and Playwright doesn't support running under Bun.

## Status / trigger

**LANDED 2026-10-22 (batch 7, PR #134); owner raised minimumReleaseAge to 30 days in batch 9 (PR #136).** **BACKLOG (owner approved 2026-10-22, decision card in the "Switching npm to Bun" thread; background: the Bun evaluation of 2026-10-21 in the project files).** Own merge batch after batch 6, kept apart from other tooling changes because it touches session-setup.sh, run-gates.sh, two workflows and the docs.

## Size

S-M, about half a day.

## Detail

Owner approved 2026-10-22 (decision card in the "Switching npm to Bun" thread; Bun was evaluated 2026-10-21 and rejected: under a
second of gain, and Playwright does not support running under Bun). Landed 2026-10-22 in batch 7 (PR #134) by lane pnpm, kept in
its own merge batch because it touches `session-setup.sh`, `run-gates.sh`, two workflows and the docs.

**What landed.**

- **Config.** pnpm 10.28.0 (`packageManager` in both `package.json`). Each folder has `pnpm-workspace.yaml`
  (`minimumReleaseAge`, `onlyBuiltDependencies: []`: no dependency install script runs) and `.npmrc` (`save-exact=true`). Every
  direct dependency is pinned exactly (playwright 1.60.0 unchanged, stylelint 16.26.1 as the old lock resolved); `create-stylelint`
  dropped (nothing ran it). `pnpm import` kept all 16 playwright versions identical. `package-lock.json` is gone from both folders
  and the root `.gitignore` line for it was removed (the old locks had been force-added). `fsevents` needs no allowlisting.
- **Release age.** First 4320 minutes (3 days); the owner raised it to **30 days (43200)** in batch 9 (PR #136). An urgent fix may
  be let through deliberately with `minimumReleaseAgeExclude`, one package per entry, each with its reason in a comment. On
  2026-10-22 playwright's 16 packages were all at least 30 days old; preview had three younger transitive packages locked under the
  3-day policy (`@csstools/css-syntax-patches-for-csstree@1.1.15`, `ignore@7.0.12`, `source-map-js@1.2.2`), left until the owner
  decides on an exclude or a bump.
- **Refusal proof** (scratch dir, same yaml and npmrc): `pnpm add vite@8.3.5` (32 h old) exits 1 with
  `ERR_PNPM_NO_MATURE_MATCHING_VERSION ... does not meet the minimumReleaseAge constraint` and leaves `package.json` untouched;
  `pnpm add vite@8.0.0` installs and is written exact. Missing deps stay loud: `pnpm exec stylelint` without `node_modules` fails
  `ERR_PNPM_RECURSIVE_EXEC_FIRST_FAIL`, no download.
- **Installs and scripts.** `pnpm install --frozen-lockfile` replaces `npm ci` everywhere; `scripts/session-setup.sh` uses it with
  pnpm's `node_modules/.modules.yaml` as the freshness marker; `run-gates.sh` runs `pnpm exec stylelint`;
  `playwright/tools/test-changed.sh` and `run-suite.sh` use `pnpm exec playwright test`.
- **How lanes get `node_modules`** (replaces the row 72 rule): symlink `playwright/node_modules` and `preview/node_modules` from the
  main checkout (pnpm's layout is relative symlinks into `.pnpm`; verified from a second directory), or
  `pnpm install --frozen-lockfile` in the worktree. Never `pnpm add`/`update` or a lock-rewriting install.
- **CI.** `pnpm/action-setup@v4.1.0` (a tag, not a SHA: the lane had no GitHub API access; pin the SHA when workflows are
  re-enabled), version read from `packageManager`, `setup-node` `cache: pnpm`, frozen installs. Not run (workflow_dispatch freeze).
  Those workflow files were later deleted by row 266.
- **Left.** About 50 `npx playwright` usage comments in `playwright/*.spec.ts` headers and the `*.local.config.ts` files (touching
  them would make `test:changed` select every spec; they still work as typed), and historical notes.

**Acceptance.** Fresh clone plus `scripts/session-setup.sh` installs both folders with pnpm and no install scripts; `test:changed`
and `run-gates.sh` pass; a version younger than the delay is refused; no `package-lock.json`, `npm ci` or `npx` outside historical
notes (grep).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/359-js-tooling-pnpm-minimum-release-age.md`.*
