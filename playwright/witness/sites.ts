// The generated site list: every `site!` name the library builds, written by `scripts/check-bridge-behaviours.sh
// --update-baseline` from the literals (`primitives/src/ts/behaviours/sites.gen.json`, a JSON array of names). A witness
// for a site that is not in the list is `test.fixme`: flipping it to live needs no edit and no hash change, because the
// list, not the witness, changes when the site is built. A list that does not exist is an empty one.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

export const SITE_LIST = join(__dirname, "..", "..", "primitives", "src", "ts", "behaviours", "sites.gen.json");

export function siteList(file: string = SITE_LIST): string[] {
  if (!existsSync(file)) return [];
  const sites: unknown = JSON.parse(readFileSync(file, "utf8"));
  if (!Array.isArray(sites) || !sites.every((s) => typeof s === "string")) throw new Error(`${file} must be a JSON array of site names`);
  return sites;
}
