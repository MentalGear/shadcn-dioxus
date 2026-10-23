import { defineConfig, devices } from "@playwright/test";
import { perTestTimeoutMs, timingSpecIgnore } from "./base-url";
import { sandboxLaunchOptions } from "./sandbox-chromium.cjs";
// Local-only: use the image's preinstalled Chromium and the already-running dx server.
export default defineConfig({
  testDir: ".",
  fullyParallel: false,
  workers: 1,
  reporter: "list",
  timeout: perTestTimeoutMs(90 * 1000), // override: PW_TEST_TIMEOUT=<ms>
  testIgnore: timingSpecIgnore(), // main-thread.spec.ts is skipped on a known-debug build (base-url.ts)
  projects: [{
    name: "chromium",
    // Same gap as baseline.local.config.ts (dev-docs/backlog.md row 105): no mobile-emulation
    // device here, so the `... (mobile)` `.tap()` specs fail rather than being excluded.
    grepInvert: /mobile/,
    use: {
      ...devices["Desktop Chrome"],
      launchOptions: sandboxLaunchOptions(), // sandbox-chromium.cjs (row 374)
    },
  }],
});
