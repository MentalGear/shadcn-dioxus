---
id: 359
title: "JS tooling: pnpm in playwright/ and preview/ with minimumReleaseAge, exact pins and install scripts off"
status: "BACKLOG (owner approved 2026-10-22, decision card in the \"Switching npm to Bun\" thread; background…"
size: "S-M, about half a day."
area: tooling
updated: 2026-10-22
---

# Row 359: JS tooling: pnpm in playwright/ and preview/ with minimumReleaseAge and install scripts off

## Item

Harden the supply chain of the two JS folders (`playwright/`, `preview/`) by replacing npm with pnpm. The move should refuse freshly published versions and never run dependency install scripts. Everything stays on one toolchain. Bun was evaluated and rejected: under a second of gain, and Playwright doesn't support running under Bun.

## Status / trigger

**BACKLOG (owner approved 2026-10-22, decision card in the "Switching npm to Bun" thread; background: the Bun evaluation of 2026-10-21 in the project files).** Own merge batch after batch 6, kept apart from other tooling changes because it touches session-setup.sh, run-gates.sh, two workflows and the docs.

## Size

S-M, about half a day.

## Detail

- **Lockfiles and config:** in each folder, replace `package-lock.json` with `pnpm-lock.yaml` (`pnpm import`, then check that versions match). Add `pnpm-workspace.yaml` or `.npmrc` settings:
  - `minimumReleaseAge` of 4320 minutes (3 days). Pick the value and write it down.
  - Install scripts off. pnpm 10 already skips dependency lifecycle scripts unless they're listed in `onlyBuiltDependencies`. Keep that list empty. The only script in today's lockfiles is `fsevents`, which only installs on macOS.
  - `packageManager` pinned in each `package.json` (corepack).
  - **Pin every direct dependency exactly** (owner 2026-10-22 12:58Z). No `^` or `~` in either `package.json`; pin to the versions the lockfile resolves today. Set `save-exact=true` so later adds stay pinned. The frozen lockfile with integrity hashes already pins transitive deps.
  - Drop `create-stylelint` from `preview/` dependencies. It's a scaffolding CLI that nothing runs. Check with grep first.
  - `fsevents` (optional macOS-only dep of `playwright`, used only for watch mode) needs no allowlisting. Its 2.3.2 tarball ships a prebuilt `fsevents.node` and has no install script.
- **Installs:** `pnpm install --frozen-lockfile` replaces `npm ci` everywhere.
- **`scripts/session-setup.sh`:** swap `npm_ci_if_stale` for the pnpm install. Replace the `node_modules/.package-lock.json` freshness marker with pnpm's `node_modules/.modules.yaml`. Make sure pnpm is present (corepack enable, or install pinned). Check that symlinked `node_modules` in worktree lanes still resolves, since pnpm's layout is symlinks into `.pnpm`.
- **Gates:**
  - In `scripts/run-gates.sh`, `npx --no-install stylelint` becomes `pnpm exec stylelint`, still failing loudly when deps are missing.
  - Update the hints in `check-select-css.sh` and the backlog row 72 rule text.
- **Scripts:** `playwright/tools/test-changed.sh` uses `npx playwright test`, which becomes `pnpm exec playwright test`. `run-suite.sh` and any other `npx` calls change the same way.
- **CI:** in `playwright.yml` and `stylelint.yml`, add `pnpm/action-setup` with the pinned version, set `setup-node` `cache: pnpm`, and switch to frozen installs. `stylelint.yml` currently does ad-hoc `npm install -g` / `--save-dev`. Replace that with the locked install. The Pages and build side in `main.yml` doesn't use npm, so it stays as it is.
- **Docs:** update README (the fast test loop section and `npm install` / `npx` lines), CLAUDE.md, `dev-docs/lane-rules.md` and the comments that say `npm run test:changed`.

### Landed (2026-10-22, lane pnpm)

- **Config:** pnpm 10.28.0 (`packageManager` in both `package.json`). Each folder has `pnpm-workspace.yaml` (`minimumReleaseAge: 4320`, `onlyBuiltDependencies: []`) and `.npmrc` (`save-exact=true`; the yaml has no `saveExact`). Direct dependencies pinned exactly (playwright 1.60.0 unchanged, stylelint 16.26.1 = what the old lock resolved). `create-stylelint` dropped (grep: only the audit note named it). `pnpm import` kept every playwright version (16/16 identical); preview's tree lost only create-stylelint's subtree. The root `.gitignore` line `package-lock.json` is gone (the old locks were force-added).
- **Refusal proof** (scratch dir outside the repo, same yaml and npmrc): `pnpm add vite@8.3.5` (32 h old) exits 1 with `ERR_PNPM_NO_MATURE_MATCHING_VERSION ... does not meet the minimumReleaseAge constraint`, package.json untouched; `pnpm add vite@8.0.0` installs and is written exact (`"vite":"8.0.0"`). Missing deps stay loud: `pnpm exec stylelint` with no node_modules fails `ERR_PNPM_RECURSIVE_EXEC_FIRST_FAIL`, no download.
- **How lanes get node_modules now (replaces the row 72 rule):** symlink `playwright/node_modules` and `preview/node_modules` from the main checkout (pnpm's layout is relative symlinks into `.pnpm`; verified from a second dir: `pnpm exec playwright --version` = 1.60.0 and `require.resolve` lands in the main tree), or `pnpm install --frozen-lockfile` in the worktree (hard links from the store). Never `pnpm add`/`update`/lock-rewriting install. After the merge the main loop reinstalls the main checkout once (`scripts/session-setup.sh` or `pnpm install --frozen-lockfile` in both folders); until then lanes' symlinks point at the npm layout, which still works.
- **CI:** `pnpm/action-setup@v4.1.0` (a tag, not a SHA: the lane had no GitHub API access to resolve one; pin the SHA when the workflows are re-enabled), version read from `packageManager`, `setup-node` `cache: pnpm`, frozen installs, `pnpm exec`. stylelint.yml now runs the locked install and the same globs as the gate. Not run (workflow_dispatch freeze).
- **Left:** 50-odd `npx playwright` usage comments in `playwright/*.spec.ts` headers and `base-url.ts`/`*.local.config.ts` (touching them would make `test:changed` select every spec; they still work as typed); historical notes in `dev-docs/research`, `dev-docs/issues`, done rows.

### Acceptance

- Fresh clone + `scripts/session-setup.sh` installs both folders with pnpm and no install scripts run.
- `pnpm run test:changed` and `scripts/run-gates.sh` pass. Playwright and stylelint CI jobs are green.
- A version published less than the delay ago is refused. Show it once with `pnpm add <pkg>@<fresh>` in a scratch dir.
- No `package-lock.json` or `npm ci` / `npx` left outside historical research notes (grep).

### Specs

Use `test:changed` during work. Run the full Playwright suite once in the merge batch, since every spec runs through the new install.
