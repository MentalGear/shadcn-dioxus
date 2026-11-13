// The witness rule of the spec selector (backlog row 446): a witness spec (`playwright/witness/**.spec.ts`) is mapped
// to its behaviour's own files, so a change to a behaviour runs that behaviour's witnesses instead of falling into
// shared plumbing (the broad set, once more than 30 components use the bridge).
//
// A spec names its sites through `witness("<probe>", "<site>", ...)` and `binding("<site>")`; the site's binding
// (`witness/bindings/<site>.json`, or `_self/` for the harness's own fixtures) lists the behaviour's `files`.
//   - a changed behaviour file selects the specs of every site whose binding lists it;
//   - a changed binding selects the specs that name its site;
//   - a changed harness file (the page side `primitives/ts-test/witness.ts`, the adapter `ts/bridge.ts` and its bundle, the behaviour
//     registry, the generated site list, the probes, the runner and the wrapper) selects EVERY witness spec, and only
//     them, never the broad set.
import fs from "node:fs";
import path from "node:path";

export const WITNESS_DIR = "witness/";

// the files whose change can move any witness
export const HARNESS = [
  /^primitives\/ts-test\/witness\.ts$/,
  /^primitives\/src\/ts\/bridge\.ts$/,
  /^primitives\/src\/js\/(?:bridge|behaviours\/[\w-]+)\.js$/,
  /^primitives\/src\/bridge\/behaviours/,
  /^primitives\/conformance\/behaviours/,
  /^primitives\/src\/ts\/behaviours\/(?:behaviours|sites)\.gen\.(?:ts|json)$/,
  /^playwright\/witness\/(?!bindings\/)[\w./-]+\.ts$/,
];

const SITE_RE = /\b(?:binding|witness)\(\s*(?:["'][A-Za-z]+["']\s*,\s*)?["']([\w.]+)["']/g;

/** Site names a spec's text names. */
export const sitesOf = (text) => [...text.matchAll(SITE_RE)].map((m) => m[1]);

/** The binding of `site` (its `files`), or null when no file holds it. */
export function bindingFiles(pw, site) {
  for (const dir of ["bindings", "bindings/_self"]) {
    const file = path.join(pw, WITNESS_DIR, dir, `${site}.json`);
    if (fs.existsSync(file)) return JSON.parse(fs.readFileSync(file, "utf8")).files ?? [];
  }
  return null;
}

/** `[{ spec, why }]` for one changed repository path. `specs` are paths relative to playwright/, `src` their texts. */
export function witnessRule(pw, changed, specs, src) {
  const witnessSpecs = specs.filter((s) => s.startsWith(WITNESS_DIR));
  if (HARNESS.some((re) => re.test(changed))) return witnessSpecs.map((spec) => ({ spec, why: `${changed} (witness harness)` }));
  const out = [];
  const bound = /^playwright\/witness\/bindings\/(?:_self\/)?([\w.]+)\.json$/.exec(changed);
  for (const spec of witnessSpecs) {
    for (const site of new Set(sitesOf(src[spec] ?? ""))) {
      if (bound && bound[1] === site) out.push({ spec, why: `${changed} (binding of ${site})` });
      else if ((bindingFiles(pw, site) ?? []).includes(changed)) out.push({ spec, why: `${changed} (behaviour file of ${site})` });
    }
  }
  return out;
}
