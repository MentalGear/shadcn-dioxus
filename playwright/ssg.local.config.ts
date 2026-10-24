import { defineConfig, devices } from "@playwright/test";
import { perTestTimeoutMs, timingSpecIgnore } from "./base-url";
import { sandboxLaunchOptions } from "./sandbox-chromium.cjs";
// The SSG lane. Points at a plain static file server serving the
// fullstack-SSG-prerendered site (`dx build --ssg --features fullstack
// --platform web`'s output directory,
// `target/dx/preview/release/web/public` -- `debug` instead of `release` if
// built without `--release`), NOT the `dx serve` dev server
// baseline.local.config.ts/playwright.config.ts depend on -- no webServer
// entry here, so this config never tries to start (or wait on) a dx dev
// server. Start the static server yourself first, e.g.:
//
//   python3 -m http.server 8090 -d <site-dir>
//
// Specs default to `http://127.0.0.1:8080` as their base URL (base-url.ts), so when running any of *those*
// against this lane, also serve the same SSG site dir on port 8080 (two static-server processes over the
// same directory is fine -- it's just files). The oracle spec this config
// serves (`oracle/hydration-parity.spec.ts`) defaults to 8090 (override:
// `PLAYWRIGHT_BASE_URL`, base-url.ts's `baseUrlOr`), and reads `SSG_SITE_DIR` (the same site dir this config's own comment
// above names) to enumerate every prerendered component page for its row
// 46 rules -- see that file's own header. See dev-docs/conformance-
// harness.md, "SSG lane", for the full build+serve recipe this mirrors
// (dev-docs/backlog.md row 22).
//
// `executablePath` comes from `sandbox-chromium.cjs` (row 374): `PW_CHROMIUM`,
// else the newest Chromium preinstalled under `PLAYWRIGHT_BROWSERS_PATH` (this
// agent sandbox's image), else nothing -- a real CI runner (which installs its
// own browser via `npx playwright install --with-deps chromium` into
// Playwright's normal cache dir) gets Playwright's own default resolution,
// same as every other config in this repo that never sets `executablePath`.
const launchOptions = sandboxLaunchOptions();

export default defineConfig({
  testDir: ".",
  fullyParallel: true,
  workers: 4,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  timeout: perTestTimeoutMs(90 * 1000), // override: PW_TEST_TIMEOUT=<ms>
  testIgnore: timingSpecIgnore(), // main-thread.spec.ts is skipped on a known-debug build (base-url.ts)
  projects: [{
    name: "chromium",
    // Same gap as baseline.local.config.ts (dev-docs/backlog.md row 105): no mobile-emulation
    // device here, so the `... (mobile)` `.tap()` specs fail rather than being excluded.
    grepInvert: /mobile/,
    use: {
      ...devices["Desktop Chrome"],
      launchOptions,
    },
  }],
});
