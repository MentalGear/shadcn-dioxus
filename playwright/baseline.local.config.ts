import { defineConfig, devices } from "@playwright/test";
import { perTestTimeoutMs, timingSpecIgnore } from "./base-url";
import { sandboxLaunchOptions } from "./sandbox-chromium.cjs";
// Local-only: use the image's preinstalled Chromium and the already-running dx server.
// Modeled on oracle.local.config.ts, but parallelized for a full-suite baseline run.
export default defineConfig({
  testDir: ".",
  fullyParallel: true,
  workers: 4,
  reporter: "list",
  timeout: perTestTimeoutMs(90 * 1000), // override: PW_TEST_TIMEOUT=<ms>
  testIgnore: timingSpecIgnore(), // main-thread.spec.ts is skipped on a known-debug build (base-url.ts)
  projects: [{
    name: "chromium",
    // Mirrors playwright.config.ts's own CI project: the `... (mobile)` tests of navbar.spec.ts,
    // select.spec.ts and sidebar.spec.ts need `.tap()`, which needs `hasTouch`, and this launch has
    // no mobile emulation (dev-docs/backlog.md row 105).
    grepInvert: /mobile/,
    use: {
      ...devices["Desktop Chrome"],
      launchOptions: { ...sandboxLaunchOptions(), args: ["--no-sandbox"] }, // sandbox-chromium.cjs (row 374)
    },
  }],
});
