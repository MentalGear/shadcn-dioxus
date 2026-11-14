// The properties the host component's own sheet declares, added to the wide projection (`witness.ts`, `project`).
// Read from the component's `style.css`, so a property a component paints through is never missing from the record.

import { readFileSync } from "node:fs";
import { join } from "node:path";

/** Every property name a stylesheet declares, custom properties included, in first-seen order. */
export function sheetProps(css: string): string[] {
  const text = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const names = new Set<string>();
  for (const m of text.matchAll(/[{;]\s*(-{0,2}[a-z][a-z0-9-]*)\s*:/g)) names.add(m[1]);
  return [...names];
}

export function propsOfSheet(sheet: string | undefined): string[] {
  return sheet ? sheetProps(readFileSync(join(__dirname, "..", "..", sheet), "utf8")) : [];
}
