#!/usr/bin/env node
// Generates preview/assets/theme-presets.css from scripts/theme-presets.data.json.
//
//   node scripts/gen-theme-presets.mjs [--out FILE]           (default: the stylesheet in preview/assets)
//   node scripts/gen-theme-presets.mjs --print-ownership     (the ownership table as JSON, read by the gate)
//   node scripts/gen-theme-presets.mjs --import <shadcn-ui/ui>/apps/v4/registry --sha <git sha>
//                                                             (refreshes the data file from a checkout of shadcn's source)
//
// The data file is shadcn's own theme data, extracted once and vendored so a regeneration needs no network:
// shadcn-ui/ui@295a1f11, apps/v4/registry/themes.ts (7 base colours and 17 accents) and
// registry/_legacy-base-colors.ts baseColorsV4 (gray and slate). Values are verbatim OKLCH. The docs site's
// own "Docs blue" accent is not shadcn data and is written down in SITE_ACCENTS below.
//
// shadcn composes a theme as `{ ...baseColor.cssVars, ...accent.cssVars }`, so a base carries a full palette and
// an accent overrides part of it. This file does not reproduce that overlap: it applies OWNERSHIP, so the base
// axis and the accent axis write disjoint variables and the cascade order of the two sets of blocks carries no
// meaning. scripts/check-theme-preset-axes.sh enforces the table against the committed stylesheet.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.join(here, "..");

const chartVars = [1, 2, 3, 4, 5].map((n) => `chart-${n}`);

/**
 * Who writes what. One table, read by the generator, the gate and the docs.
 *
 *  base.vars     shadcn's bare variables a `[data-theme-base]` block writes: the neutral palette.
 *  base.slots    shadcn variables that BOTH axes need a value for. The accent owns the bare name; the base's neutral
 *                value is written to a base-owned slot instead, which the theme reads as the fallback under the
 *                bare name (`--dx-primary: var(--primary, var(--dx-base-primary, <ramp>))`). With no accent the
 *                slot shows; an accent's `--primary` wins; a host's own `--primary` wins over both.
 *  base.derived  two variables shadcn's data does not carry, written once for every base.
 *  accent.vars   the bare variables a `[data-theme-accent]` block writes: the hue.
 *  dropped       upstream data of an axis that the other axis owns; it is left out, never written.
 */
export const OWNERSHIP = {
  base: {
    vars: [
      "background", "foreground", "card", "card-foreground", "popover", "popover-foreground",
      "secondary", "secondary-foreground", "muted", "muted-foreground", "accent", "accent-foreground",
      "destructive", "border", "input", "ring",
      "sidebar", "sidebar-foreground", "sidebar-accent", "sidebar-accent-foreground", "sidebar-border", "sidebar-ring",
    ],
    slots: {
      primary: "dx-base-primary",
      "primary-foreground": "dx-base-primary-foreground",
      "sidebar-primary": "dx-base-sidebar-primary",
      "sidebar-primary-foreground": "dx-base-sidebar-primary-foreground",
    },
    derived: ["border-strong", "destructive-foreground"],
  },
  accent: {
    vars: ["primary", "primary-foreground", ...chartVars, "sidebar-primary", "sidebar-primary-foreground"],
  },
  dropped: {
    base: chartVars,
    accent: ["secondary", "secondary-foreground"],
  },
};

if (process.argv.includes("--print-ownership")) {
  process.stdout.write(JSON.stringify(OWNERSHIP, null, 2) + "\n");
  process.exit(0);
}

const args = Object.fromEntries(
  process.argv.slice(2).reduce((acc, a, i, all) => (a.startsWith("--") ? [...acc, [a.slice(2), all[i + 1]]] : acc), []),
);
const dataFile = path.join(here, "theme-presets.data.json");

/** The data file's text: one variable per line, so a refresh from upstream diffs value by value. */
function formatData(d) {
  const theme = (e) => {
    const lines = ["  {", `   "name": ${JSON.stringify(e.name)}, "title": ${JSON.stringify(e.title)},`];
    if (e.skipped) lines.push(`   "skipped": ${JSON.stringify(e.skipped)},`);
    for (const mode of ["light", "dark"]) {
      const keys = Object.keys(e[mode]);
      lines.push(`   "${mode}": {`);
      keys.forEach((k, i) => lines.push(`    ${JSON.stringify(k)}: ${JSON.stringify(e[mode][k])}${i < keys.length - 1 ? "," : ""}`));
      lines.push(`   }${mode === "light" ? "," : ""}`);
    }
    lines.push("  }");
    return lines.join("\n");
  };
  const src = JSON.stringify(d.source, null, 1);
  return `{\n"source": ${src},\n"bases": [\n${d.bases.map(theme).join(",\n")}\n],\n"accents": [\n${d.accents.map(theme).join(",\n")}\n]\n}\n`;
}

if (args.import) {
  // Reads shadcn's TypeScript data files without a TypeScript toolchain: the exports are plain object literals.
  const src = args.import;
  const load = (file, strip) => {
    const text = strip(fs.readFileSync(path.join(src, file), "utf8"));
    const m = { exports: {} };
    new Function("module", text)(m);
    return m.exports;
  };
  const THEMES = load("themes.ts", (t) =>
    t
      .replace(/^import .*$/gm, "")
      .replace(/^export type .*$/gm, "")
      .replace("export const THEMES: RegistryItem[] =", "module.exports =")
      .replace(/\]\s*as const satisfies RegistryItem\[\]/, "]"),
  );
  const legacy = load("_legacy-base-colors.ts", (t) => {
    const i = t.indexOf("export const baseColorsV4 =");
    const j = t.indexOf("} as const", i);
    return t.slice(i, j + 1).replace("export const baseColorsV4 =", "module.exports =");
  });
  const baseNames = [...fs.readFileSync(path.join(src, "base-colors.ts"), "utf8").matchAll(/"([a-z]+)"/g)].map((m) => m[1]);
  const title = (s) => s[0].toUpperCase() + s.slice(1);
  const bases = baseNames.map((n) => THEMES.find((t) => t.name === n));
  ["gray", "slate"].forEach((n, i) => {
    bases.splice(3 + i, 0, { name: n, title: title(n), cssVars: legacy[n] }); // neutral stone zinc gray slate mauve ...
  });
  const accents = THEMES.filter((t) => !baseNames.includes(t.name));
  const entry = (t) => {
    const { light, dark } = t.cssVars;
    const keys = [...new Set([...Object.keys(light), ...Object.keys(dark)])].filter((k) => k !== "radius");
    // A variable one mode defines and the other does not cannot be expressed with the space toggle, so it is left out.
    const skipped = keys.filter((k) => light[k] === undefined || dark[k] === undefined);
    const kept = keys.filter((k) => !skipped.includes(k));
    const e = { name: t.name, title: t.title };
    if (skipped.length) e.skipped = skipped.map((k) => `${k} (${light[k] === undefined ? "dark" : "light"} only)`).join(", ");
    e.light = Object.fromEntries(kept.map((k) => [k, light[k]]));
    e.dark = Object.fromEntries(kept.map((k) => [k, dark[k]]));
    return e;
  };
  const prev = JSON.parse(fs.readFileSync(dataFile, "utf8"));
  const next = { source: { ...prev.source, sha: args.sha ?? prev.source.sha }, bases: bases.map(entry), accents: accents.map(entry) };
  fs.writeFileSync(dataFile, formatData(next));
  console.error(`wrote ${dataFile}: ${next.bases.length} bases, ${next.accents.length} accents; now run gen-theme-presets.mjs`);
  process.exit(0);
}

const out = args.out ?? path.join(repo, "preview/assets/theme-presets.css");
const data = JSON.parse(fs.readFileSync(dataFile, "utf8"));
const sha = data.source.sha;

/** The variable name a block of `axis` writes for upstream variable `key` (a slot name for a base's primary), or null when the other axis owns it. */
function route(axis, theme, key) {
  const own = OWNERSHIP[axis];
  if (axis === "base" && OWNERSHIP.base.slots[key]) return OWNERSHIP.base.slots[key];
  if (own.vars.includes(key)) return key;
  if (OWNERSHIP.dropped[axis].includes(key)) return null;
  throw new Error(`${axis} "${theme.name}" carries --${key}, which the ownership table neither assigns nor drops`);
}

function declarations(axis, theme) {
  const lines = [];
  const written = new Set();
  for (const key of Object.keys(theme.light)) {
    const l = theme.light[key];
    const d = theme.dark[key];
    if (d === undefined) throw new Error(`${theme.name}: --${key} has no dark value`);
    const name = route(axis, theme, key);
    if (name === null) continue;
    written.add(key);
    lines.push(`  --${name}: ${l === d ? l : `var(--dark, ${d}) var(--light, ${l})`};`);
  }
  for (const key of OWNERSHIP[axis].vars.concat(axis === "base" ? Object.keys(OWNERSHIP.base.slots) : [])) {
    if (!written.has(key)) throw new Error(`${axis} "${theme.name}" is missing --${key}`);
  }
  return lines;
}

function block(axis, theme) {
  const note = [theme.title, theme.skipped ? `skipped, defined in one mode only upstream: ${theme.skipped}` : ""].filter(Boolean).join(" -- ");
  return `/* ${note} */\n[data-theme-${axis}="${theme.name}"] {\n${declarations(axis, theme).join("\n")}\n}\n`;
}

const RADII = ["0", "0.3", "0.5", "0.625", "0.75", "1"];

// Not shadcn data: the docs site's own blue as a swatch. Charts are shadcn's blue ramp.
const SITE_ACCENT = {
  name: "docs",
  vars: {
    primary: "oklch(0.562 0.242 260)",
    "primary-foreground": "oklch(0.985 0 0)",
    "chart-1": "oklch(0.809 0.105 251.813)",
    "chart-2": "oklch(0.623 0.214 259.815)",
    "chart-3": "oklch(0.546 0.245 262.881)",
    "chart-4": "oklch(0.488 0.243 264.376)",
    "chart-5": "oklch(0.424 0.199 265.638)",
    "sidebar-primary": "oklch(0.562 0.242 260)",
    "sidebar-primary-foreground": "oklch(0.985 0 0)",
  },
};

const wrap = (label, text) => {
  const lines = [];
  let line = ` *   ${label}`;
  for (const word of text.split(" ")) {
    if (line.length + word.length + 1 > 112) {
      lines.push(line);
      line = ` *       ${word}`;
    } else {
      line += ` ${word}`;
    }
  }
  return [...lines, line].join("\n");
};
const names = (list) => list.map((k) => `--${k}`).join(" ");
const tableText = [
  wrap("base, the neutral palette:", names(OWNERSHIP.base.vars)),
  wrap("base, slots for the no-accent primary:", names(Object.values(OWNERSHIP.base.slots))),
  wrap("base, derived for every base:", names(OWNERSHIP.base.derived)),
  wrap("accent, the hue:", names(OWNERSHIP.accent.vars)),
].join("\n");

let css = `/* GENERATED by scripts/gen-theme-presets.mjs from scripts/theme-presets.data.json -- do not edit by hand.
 * Regenerate: node scripts/gen-theme-presets.mjs. scripts/check-theme-preset-axes.sh compares this file with that output.
 *
 * shadcn's theme customizer, as DATA. Source: shadcn-ui/ui@${sha}
 * (apps/v4/registry/themes.ts for ${data.bases.length - 2} base colours and ${data.accents.length} accents; registry/_legacy-base-colors.ts
 * baseColorsV4 for gray and slate). OKLCH values are verbatim.
 *
 * How it applies. Each block sets shadcn's BARE variables (--background, --muted, ...) on <html>.
 * dx-components-theme.css reads those first (--dx-muted: var(--muted, <ramp>)), so every component
 * recolours with no component edit. No attribute set = no preset = the library's own look.
 *
 *   <html data-theme-base="zinc" data-theme-accent="blue" data-theme-radius="0.5">
 *
 * Two axes with DISJOINT variables. The base is the neutral palette; the accent is the hue. Neither writes a
 * variable the other writes, so picking one never changes what the other owns, and the order of the blocks
 * carries no meaning:
 *
${tableText}
 *
 * A variable both axes need is a slot. The accent owns the bare --primary; the base writes its neutral primary
 * to --dx-base-primary, which dx-components-theme.css reads under the bare name:
 * --dx-primary: var(--primary, var(--dx-base-primary, <ramp>)). With no accent the base's primary shows, an
 * accent's --primary wins, and a host's own --primary wins over both. The same holds for --primary-foreground,
 * --sidebar-primary and --sidebar-primary-foreground.
 *
 * Charts belong to the accent: with no accent the library's own palette (--dx-chart-N in dx-components-theme.css)
 * shows under every base, and an accent swaps in its hue ramp. Secondary belongs to the base: an accent leaves it.
 * shadcn's own data repeats these in the other axis (a base carries charts, an accent carries secondary); the
 * generator drops them (the dropped lists of its OWNERSHIP table).
 *
 * Each variable carries both modes with the same space toggle the theme file uses
 * (var(--dark, <dark>) var(--light, <light>)), so the light/dark switch works with any preset.
 *
 * Attributes live on <html> because the --dx-* role tokens are declared on :root and resolve the bare
 * variables there. Set by the pre-paint snippet in preview/index.html (cookies dx_theme_base/_accent/_radius).
 * The picker's swatches reuse these same blocks (preview/assets/main.css .dx-theme-swatch), so the colour
 * data exists in exactly one place.
 */

/* stylelint-disable lightness-notation, hue-degree-notation -- values are shadcn's, byte for byte (oklch(0.145 0 0), not 14.5% 0deg) */

/* Radius: shadcn's RADII, as a list. --dx-radius reads --radius; the whole --dx-radius-* scale derives from it. */
`;
for (const r of RADII) {
  css += `[data-theme-radius="${r}"] {\n  --radius: ${r}rem;\n}\n\n`;
}

css += `/* Base colours: the neutral palette. */

/* Two variables shadcn's v4 theme data does not carry, which the library's role tokens read (see the comments in
 * dx-components-theme.css), set under any base:
 *  - --border-strong: no shadcn counterpart. Derived from the base's own ink and page (about 26% ink: the theme's
 *    own default derivation, #b6b6b6 / #3e3e3e on neutral), so hover borders keep the tint.
 *  - --destructive-foreground: v3 shipped it, v4 components hard-code \`text-white\` on destructive instead. The
 *    theme's \`--dx-destructive-foreground\` falls back to the same near-white (oklch(0.985)), which is what v4 paints,
 *    so a preset and no preset agree on the ink. */
[data-theme-base] {
  --border-strong: color-mix(in oklab, var(--foreground) 26%, var(--background));
  --destructive-foreground: oklch(0.985 0 0);
}

`;
for (const t of data.bases) {
  css += block("base", t) + "\n";
}
css += `/* Accents: the hue. Primary, charts and sidebar-primary; the base owns everything else. */\n\n`;
for (const t of data.accents) {
  css += block("accent", t) + "\n";
}

const siteDecls = OWNERSHIP.accent.vars.map((k) => {
  if (!(k in SITE_ACCENT.vars)) throw new Error(`the docs accent is missing --${k}`);
  return `  --${k}: ${SITE_ACCENT.vars[k]};`;
});
css += `/* Docs blue -- NOT shadcn data: the docs site's own link blue (\`--highlight-color-tertiary\`, #0065ff in light, #4d94ff
 * in dark; main.css), offered as a swatch so the colour the site shows by default can be picked back. The fill is
 * the light-mode blue in both modes because white ink on #4d94ff is 2.9:1; the ink for TEXT in this accent is the
 * site's pair itself (main.css \`:root[data-theme-accent="docs"]\`). Charts are shadcn's blue ramp. Emitted by
 * scripts/gen-theme-presets.mjs (SITE_ACCENT), after shadcn's accents. */
[data-theme-accent="docs"] {
${siteDecls.join("\n")}
}
`;

fs.writeFileSync(out, css.replace(/\n+$/, "\n"));
console.error(`wrote ${out}: ${data.bases.length} bases, ${data.accents.length + 1} accents, ${css.split("\n").length} lines`);
