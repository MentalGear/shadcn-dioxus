---
id: 374
title: "One Chromium path resolver for every local Playwright config, spec and script"
status: "LANDED 2026-10-23 (branch claude/playwright-chromium-resolver). Fresh containers ship Chromium…"
size: "S"
area: tooling
updated: 2026-10-23
---

# Row 374: Playwright Chromium path resolver

## Item

`baseline`, `oracle`, `ssg`, `suite` and `xvfb.local.config.ts`, `scroll-fade-scrollbar.spec.ts` and `scripts/{inspect,measure-scroll,compare-ssg-styles}.mjs` each hard-coded the sandbox image's Chromium as revision 1194's `chrome-linux/chrome` under `/opt/pw-browsers`. Fresh containers now ship revision 1243, laid out as `chrome-linux64/chrome`. The configs without an exists-check (`baseline`, `oracle`, `xvfb`) failed to launch; the ones with it (`ssg`, `suite`) fell through to Playwright's own resolution, which wants the revision pinned by `@playwright/test` 1.60.0 (1223, headless shell) and found nothing either. Several lanes failed at browser launch, or ran only after a hand-made symlink to the old path.

## Status / trigger

LANDED 2026-10-23 (branch claude/playwright-chromium-resolver). Fresh containers ship Chromium 1243; every hard-coded 1194 path broke Playwright launches.

## Size

S

## Detail

Construction:

- [`playwright/sandbox-chromium.cjs`](../../../playwright/sandbox-chromium.cjs) is the one resolver: `PW_CHROMIUM` first (a missing path throws, never falls back), else the newest `chromium-<rev>/chrome-linux64|chrome-linux/chrome` under `PLAYWRIGHT_BROWSERS_PATH` by numeric revision, else `undefined` (no `executablePath`, Playwright's own resolution: a CI runner). Always the full Chrome, never `chromium_headless_shell-*`, which is what every config ran before. CommonJS so the TypeScript configs and specs and the ESM scripts load the same file with no build step.
- All five local configs, `scroll-fade-scrollbar.spec.ts` (its `test.use({ launchOptions })` replaces the config's) and the three scripts import it.
- [`scripts/check-chromium-path.sh`](../../../scripts/check-chromium-path.sh), in `run-gates.sh` and the CLAUDE.md gate sentence: fails on any `chromium-<digits>/` or `chromium_headless_shell-<digits>/` path or a string-literal `executablePath:` under `playwright/` or `scripts/`, and on a `playwright/*.config.ts` that sets `launchOptions`/`executablePath` without importing the resolver. Proven red on the old `inspect.mjs` and on a config with a literal path.

Evidence (fresh container, `PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers` holding only `chromium-1243`, `chromium_headless_shell-1243`, `ffmpeg-1011` and the image's own `chromium` link, which the resolver does not use; no symlink added):

- Resolver: default `/opt/pw-browsers/chromium-1243/chrome-linux64/chrome`; `PW_CHROMIUM` override honoured; empty env `undefined`; a missing override throws; same result from ESM `import`.
- A one-test launch smoke (`page.setContent`, not committed) under each config: `baseline`, `oracle`, `ssg`, `suite` and `xvfb` (under `xvfb-run`, headed) all pass on Chromium 153.0.8010.12 via the 1243 path. The same smoke on the pre-change `suite.local.config.ts` fails: `Executable doesn't exist at /opt/pw-browsers/chromium_headless_shell-1223/...`.
