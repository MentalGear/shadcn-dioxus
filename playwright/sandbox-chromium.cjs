// The one place that decides which Chromium binary a local Playwright run launches (backlog row 374).
//
// A fresh sandbox image may ship a different Chromium revision (and a `chrome-linux64/` directory) than the
// last one, so no config, spec or probe names a revision path (row 374 has the incident). The binary is chosen here:
//
//   1. `PW_CHROMIUM=<path>` wins. A path that does not exist throws: an explicit override that is wrong must fail
//      loudly, not fall back to another browser.
//   2. Otherwise the newest `chromium-<revision>/chrome-linux*/chrome` under `PLAYWRIGHT_BROWSERS_PATH` (the
//      preinstalled browsers of this sandbox image, `/opt/pw-browsers`), by numeric revision. Always the full
//      Chrome, never `chromium_headless_shell-*`: with `executablePath` set Playwright runs it in new headless mode,
//      which is what every config here has always run.
//   3. Otherwise `undefined`: no `executablePath`, Playwright resolves its own browser (a CI runner after
//      `npx playwright install --with-deps chromium`).
//
// `scripts/check-chromium-path.sh` keeps a hard-coded `chromium-NNNN` path out of playwright/ and scripts/.
//
// CommonJS on purpose: the TypeScript configs and specs (transpiled to CommonJS by Playwright) and the ESM scripts
// (`import { sandboxChromium } from "../playwright/sandbox-chromium.cjs"`) can both load it with no build step.
"use strict";
const fs = require("node:fs");
const path = require("node:path");

/** @returns {string | undefined} the Chromium executable to launch, or undefined for Playwright's default. */
function sandboxChromium(env = process.env) {
  const override = env.PW_CHROMIUM;
  if (override) {
    if (!fs.existsSync(override)) throw new Error(`PW_CHROMIUM=${override} does not exist`);
    return override;
  }
  const root = env.PLAYWRIGHT_BROWSERS_PATH;
  if (!root || root === "0" || !fs.existsSync(root)) return undefined;
  const found = [];
  for (const dir of fs.readdirSync(root)) {
    const m = /^chromium-(\d+)$/.exec(dir);
    if (!m) continue;
    for (const sub of ["chrome-linux64", "chrome-linux"]) {
      const exe = path.join(root, dir, sub, "chrome");
      if (fs.existsSync(exe)) {
        found.push({ revision: Number(m[1]), exe });
        break;
      }
    }
  }
  found.sort((a, b) => b.revision - a.revision);
  return found[0]?.exe;
}

/** `launchOptions` fragment: `{ executablePath }` when a sandbox Chromium was found, `{}` otherwise. */
function sandboxLaunchOptions(env = process.env) {
  const executablePath = sandboxChromium(env);
  return executablePath ? { executablePath } : {};
}

exports.sandboxChromium = sandboxChromium;
exports.sandboxLaunchOptions = sandboxLaunchOptions;
