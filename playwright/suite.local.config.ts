import { defineConfig, devices } from "@playwright/test";
import { perTestTimeoutMs, timingSpecIgnore } from "./base-url";
import { sandboxLaunchOptions } from "./sandbox-chromium.cjs";
import fs from "node:fs";
import path from "node:path";
// The full-suite config (backlog row 152a): ssg.local.config.ts's project (sandbox Chromium, `... (mobile)` excluded)
// plus the speed constructions, each switchable so a run can be compared with and without it:
//
//   PW_WORKERS=<n>          workers (default 2: the lane-rules cap; measured RSS in dev-docs/lane-rules.md).
//   PW_SPECS_FILE=<file>    run only the spec files listed in <file> (newline-separated, relative to playwright/):
//                           `node playwright/tools/shard.mjs 1/3 --out <file>` writes a duration-balanced shard,
//                           `node playwright/tools/select.mjs --explain` the specs covering a diff.
//
// Point it at a build with PLAYWRIGHT_BASE_URL=http://127.0.0.1:<port> (every spec reads it through base-url.ts) and
// PW_BUILD_PROFILE=debug|release (the timing specs are skipped on debug, base-url.ts).
//
// Not here, on purpose: blocking the external font hosts. Visiting all 95 prerendered routes
// with a fresh Chromium context issues exactly two third-party requests, both from the deliberate avatar "slow" and
// "broken" demos (`httpbin.org/delay/3600`, `invalid-url.example`), and those are what avatar-loading.spec.ts asserts
// on; the fonts are self-hosted. There is nothing to block that a spec does not already own.
const launchOptions = sandboxLaunchOptions(); // the sandbox image's newest Chromium, or Playwright's own (row 374)

const listFile = process.env.PW_SPECS_FILE;
const testMatch = listFile
  ? fs.readFileSync(listFile, "utf8").split("\n").map((s) => s.trim()).filter(Boolean).map((s) => path.resolve(__dirname, s))
  : undefined;

export default defineConfig({
  testDir: ".",
  ...(testMatch ? { testMatch } : {}),
  fullyParallel: true,
  workers: Number(process.env.PW_WORKERS) || 2,
  reporter: "line",
  timeout: perTestTimeoutMs(90 * 1000), // override: PW_TEST_TIMEOUT=<ms>
  testIgnore: timingSpecIgnore(), // main-thread.spec.ts is skipped on a known-debug build (base-url.ts)
  projects: [{
    name: "chromium",
    grepInvert: /mobile/,
    use: { ...devices["Desktop Chrome"], launchOptions },
  }],
});
