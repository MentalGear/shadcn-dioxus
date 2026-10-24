/**
 * Shared base URL for specs, so a suite (or a single spec) can be pointed at
 * a non-default dev server port -- e.g. a per-worktree lane running its own
 * `dx serve` on 8083+ instead of the shared default :8080 -- without editing
 * every spec file. See dev-docs/dev-loop.md's "Per-lane servers" section.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8083 \
 *     npx playwright test --config=session.local.config.ts avatar.spec.ts
 *
 * The default is `http://127.0.0.1:8080`, so a spec that does
 * not import this -- some still hardcode the literal inline -- works
 * unchanged.
 *
 * This consolidates three ad hoc variants of the same idea (each works, they
 * are just three names for one thing):
 *   - `drag_and_drop_list.spec.ts`: `const BASE = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080"`
 *   - `sidebar.spec.ts`: `const BASE_URL = "http://127.0.0.1:8080"` (no env override)
 *   - every other spec: the literal inline, repeated per `page.goto(...)` call
 * A spec can migrate by importing `BASE_URL` from here (aliasing to `BASE`
 * where that's the file's existing local name) instead of declaring its own.
 */
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";

/**
 * Base URL for a spec whose conventional server is NOT the :8080 dev server
 * (e.g. `oracle/hydration-parity.spec.ts`, which defaults to the :8090 static
 * SSG server). Same `PLAYWRIGHT_BASE_URL` override as `BASE_URL`, only the
 * fallback differs, so one env var points any spec at any server -- including
 * one that serves under a path prefix (`http://127.0.0.1:8090/shadcn-dioxus`,
 * a `--base-path` build). The trailing slash is stripped so specs can keep
 * writing `${base}/component/x/`.
 */
export function baseUrlOr(fallback: string): string {
  return (process.env.PLAYWRIGHT_BASE_URL ?? fallback).replace(/\/+$/, "");
}

/**
 * Per-test timeout (ms) shared by every config in this directory, so a lane
 * can shorten (or lengthen) it without editing a config:
 *
 *   PW_TEST_TIMEOUT=45000 npx playwright test --config=ssg.local.config.ts ...
 *
 * Only the config-level DEFAULT moves. A spec that calls `test.setTimeout(...)`
 * / `test.slow()` (charts_gallery 240s, carousel-virtual-wheel 120-180s,
 * computed-style-snapshot 20min, hydration-parity per-component 5min) keeps its
 * own absolute budget -- that is intended, those are the genuinely long tests.
 * A non-numeric or non-positive value is ignored (falls back to `fallbackMs`).
 */
export function perTestTimeoutMs(fallbackMs: number): number {
  const n = Number(process.env.PW_TEST_TIMEOUT);
  return Number.isFinite(n) && n > 0 ? n : fallbackMs;
}

/**
 * Which cargo profile the build under test came from: `"debug"`, `"release"`,
 * or `"unknown"`. Explicit `PW_BUILD_PROFILE` wins; otherwise it is read off
 * `SSG_SITE_DIR`'s `dx/<crate>/<profile>/web/public` path. `"unknown"` (no hint,
 * e.g. a `dx serve` dev server) is treated as "do not interfere".
 */
export function buildProfile(): "debug" | "release" | "unknown" {
  const explicit = process.env.PW_BUILD_PROFILE?.toLowerCase();
  if (explicit === "debug" || explicit === "release") return explicit;
  const dir = (process.env.SSG_SITE_DIR ?? "").replace(/\\/g, "/");
  if (/\/dx\/[^/]+\/debug\/web\/public\/?$/.test(dir)) return "debug";
  if (/\/dx\/[^/]+\/release\/web\/public\/?$/.test(dir)) return "release";
  // A lane build is copied to `pub-<lane>` (scripts/lane-ssg.sh), so its path names nothing (row 339: 11 false reds
  // of main-thread.spec through `test:changed`). `scripts/build-ssg.sh` therefore stamps a DEBUG build's public dir
  // with `build-profile.txt`; read it from the site dir, else from the served site itself.
  return profileFromMarker(dir);
}

const PROFILE_MARKER = "build-profile.txt";
let markerProfile: "debug" | "release" | "unknown" | undefined;

function profileFromMarker(siteDir: string): "debug" | "release" | "unknown" {
  if (markerProfile) return markerProfile;
  const parse = (t: string | undefined) => {
    const v = (t ?? "").trim().toLowerCase();
    return v === "debug" || v === "release" ? v : undefined;
  };
  let found: "debug" | "release" | undefined;
  try {
    if (siteDir) found = parse(fs.readFileSync(path.join(siteDir, PROFILE_MARKER), "utf8"));
  } catch {
    /* no marker in the site dir */
  }
  const base = process.env.PLAYWRIGHT_BASE_URL;
  if (!found && base) {
    try {
      const out = execFileSync(
        "curl",
        ["-fsS", "--max-time", "3", `${base.replace(/\/+$/, "")}/${PROFILE_MARKER}`],
        { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
      );
      // A SPA-fallback or 404 body is not a marker: parse() only accepts the one word.
      found = parse(out);
    } catch {
      /* server down, or a release/deployed build without a marker */
    }
  }
  markerProfile = found ?? "unknown";
  return markerProfile;
}

/**
 * `testIgnore` for configs: timing-threshold specs (long-task / INP budgets in
 * `oracle/tier2-html/main-thread.spec.ts`) measure 64-153 ms long tasks on a
 * debug build that are 0 on release (backlog row 116),
 * so they are skipped -- loudly, once -- when the build is known to be debug.
 * `PW_FORCE_TIMING_SPECS=1` runs them anyway (e.g. to confirm the false red).
 */
export function timingSpecIgnore(): RegExp[] {
  if (buildProfile() !== "debug" || process.env.PW_FORCE_TIMING_SPECS === "1") return [];
  if (!process.env.PW_TIMING_SKIP_WARNED) {
    // Set in the main process before workers spawn, so workers inherit it and stay quiet.
    process.env.PW_TIMING_SKIP_WARNED = "1";
    console.warn(
      "[playwright] debug build detected: skipping oracle/tier2-html/main-thread.spec.ts " +
        "(timing thresholds give false reds on debug; use a release build, or PW_FORCE_TIMING_SPECS=1 to run anyway).",
    );
  }
  return [/main-thread\.spec\.ts$/];
}
