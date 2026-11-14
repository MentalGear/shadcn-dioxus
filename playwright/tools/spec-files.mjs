// Which files are spec files: the one definition shared by shard.mjs (what a run executes) and suite-key.mjs (what a tree is
// accepted on). A path segment in SKIP_DIRS keeps everything below it out; `*.spec.ts` is the only spec name.
import fs from "node:fs";
import path from "node:path";

export const SKIP_DIRS = ["node_modules", "fixtures", "reference", "tools", "test-results", "playwright-report"];

/** `rel` is posix and relative to playwright/. */
export const isSpecPath = (rel) => rel.endsWith(".spec.ts") && !rel.split("/").slice(0, -1).some((d) => SKIP_DIRS.includes(d));

/** The spec files on disk below `pw` (the playwright/ directory), relative and posix. */
export function specFilesOnDisk(pw) {
  const out = [];
  const walk = (dir) => {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      if (SKIP_DIRS.includes(e.name)) continue;
      const p = path.join(dir, e.name);
      if (e.isDirectory()) walk(p); else if (e.name.endsWith(".spec.ts")) out.push(path.relative(pw, p).replace(/\\/g, "/"));
    }
  };
  walk(pw);
  return out;
}
