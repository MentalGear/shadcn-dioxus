// Which pages a test title names, for the matrix specs whose tests are one-per-page (backlog row 323).
//
// `makeTitleScopes(comps)` returns { "<spec path under playwright/>": (titlePath) => string[] | null }, where the page
// keys are `c:<component>` / `site:<page>` (css-impact.mjs routeKey) and null means "this title does not follow the
// spec's page-naming shape": the test is then ALWAYS kept. select.mjs drops a test only when it names pages and none is
// reached, so a title shape this table does not know costs run time, never coverage.
import { routeKey } from "./css-impact.mjs";

// Matrix specs whose test titles NAME the page the test visits. Each entry returns the page keys a test title names, or
// null when the title does not follow the spec's page-naming shape (the test is then ALWAYS kept: an unknown title
// shape costs run time, never coverage). A test is dropped only when it names pages and none is reached. The shapes
// below were read against each spec's source (one `goto` per test, on the page the title names); `pattern` specs match
// the exact title shape, `alias` specs (a describe per component, "Dropdown menu", "DropdownMenu") match a component by
// its squashed name anywhere in the title, which only ever names MORE pages.
export function makeTitleScopes(comps) {
const normComps = comps.map((c) => [c, c.replace(/_/g, "")]);
const idKeys = (id) => {
  const m = id.match(/^site-(home|docs|demos|charts)$/);
  if (m) return [`site:${m[1]}`];
  if (comps.includes(id)) return [`c:${id}`];
  const b = id.startsWith("block-") ? comps.find((c) => id.startsWith(`block-${c}-`)) : null;
  return b ? [`c:${b}`] : null;
};
const routeKeys = (route) => { const k = routeKey(route, comps); return k === "site:other" || k === "site:component-index" ? null : [k]; };
const aliasKeys = (t) => { const n = t.toLowerCase().replace(/[^a-z0-9]/g, ""); const ks = normComps.filter(([, x]) => n.includes(x)).map(([c]) => `c:${c}`); return ks.length ? ks : null; };
const first = (t, re, f) => { const m = t.match(re); return m ? f(m) : null; };
const TITLE_SCOPED = {
  "all-routes.spec.ts": (t) => first(t, /^(site-\w+|block-[\w-]+|[a-z_]+): (?:axe|no horizontal overflow|skip link and visible focus|the active option shows)/, (m) => idKeys(m[1])),
  "oracle/tier2-html/first-paint-styles.spec.ts": (t) =>
    first(t, /(\/[\w/-]*?) -> \/component\/(\w+)\/$/, (m) => [...(routeKeys(m[1]) ?? []), `c:${m[2]}`]) ??
    first(t, /\/component\/(\w+)\/: JavaScript off/, (m) => [`c:${m[1]}`]),
  "layout-stability.spec.ts": (t) => first(t, /^layout stability (\/\S*) (?:light|dark)/, (m) => routeKeys(m[1])) ?? first(t, /^progressive paint \(home widget masonry\)/, () => ["site:home"]),
  "theme-picker-phone.spec.ts": (t) => first(t, /\((\/[^)\s]*)\)/, (m) => routeKeys(m[1])),
  "oracle/tier2-html/touch-double-tap.spec.ts": (t) => first(t, /^component "(\w+)":/, (m) => [`c:${m[1]}`]),
  "oracle/tier2-html/touch-focus-zoom.spec.ts": (t) => first(t, /^component "(\w+)":/, (m) => [`c:${m[1]}`]),
  "dx-class-migration.spec.ts": (t) => first(t, /^(\w+): stylesheet delivered/, (m) => [`c:${m[1]}`]),
  "demos-batch2-b.spec.ts": (t) => first(t, /^(\w+)\/\w+ opens/, (m) => [`c:${m[1]}`]),
  "user-select.spec.ts": (t) => first(t, /^every control on (\/\S*) computes/, (m) => routeKeys(m[1])),
  "popup-style-isolation.spec.ts": (t) => first(t, /looks like the same popup alone ([\w-]+)$/, (m) => idKeys(m[1])),
  "oracle/tier1-apg/keyboard-matrix.spec.ts": aliasKeys,
  "oracle/tier3-radix/rtl.spec.ts": aliasKeys,
  "oracle/tier2-html/top-layer-ink.spec.ts": aliasKeys,
  "oracle/tier2-html/listener-inventory.spec.ts": (t) => (/\broute: entering and leaving\b/.test(t) ? null : aliasKeys(t)),
  "dark-surface-contrast.spec.ts": aliasKeys,
  "hover-mix-space.spec.ts": aliasKeys,
  "focus-ring.spec.ts": aliasKeys,
  "theme-preset-contrast.spec.ts": (t) => aliasKeys(t) ?? (/home page|home\b/i.test(t) ? ["site:home"] : null),
};
return TITLE_SCOPED;
}
