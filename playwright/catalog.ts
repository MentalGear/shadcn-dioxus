import * as fs from "fs";
import * as path from "path";

/**
 * The ONE reader of `CATALOG` in `preview/src/components/mod.rs`, for every spec that needs the component list.
 *
 * Each spec used to carry its own regex over the `("name", Origin::...)` rows. When `Origin::Extra` grew a
 * reason payload (`Origin::Extra("...")`), the copies that still said `Origin::(?:Shadcn|Extra)\)` silently
 * stopped matching the Extra rows and the all-routes sweep shrank by 48 tests without a single failure.
 *
 * So the parse is checked against an independent count: the number of `Origin::` occurrences in the CATALOG
 * block (one per row). A row shape the regex does not understand makes the two differ and `readCatalog`
 * throws, instead of quietly returning a shorter list.
 */
export interface CatalogRow {
  name: string;
  origin: "Shadcn" | "Extra";
  /** Why it is an extra (`Origin::Extra("<reason>")`); undefined for a shadcn page. */
  reason?: string;
}

export const MOD_RS_PATH = path.join(__dirname, "..", "preview", "src", "components", "mod.rs");

/** The text of `mod.rs`, for specs that also read other lists from it (`examples!`). */
export function readModRs(): string {
  return fs.readFileSync(MOD_RS_PATH, "utf8");
}

/** The body of `pub const CATALOG ... = &[ ... ];`. */
export function catalogBlock(modRs: string = readModRs()): string {
  const block = modRs.split("pub const CATALOG")[1]?.split("\n];")[0];
  if (!block) throw new Error(`CATALOG not found in ${MOD_RS_PATH}`);
  return block;
}

const ROW = /^\s*\("([a-z0-9_]+)", Origin::(Shadcn|Extra)(?:\("((?:[^"\\]|\\.)*)"\))?\),/gm;

/** Every CATALOG row. Throws when the parsed rows do not account for every `Origin::` in the block. */
export function readCatalog(modRs: string = readModRs()): CatalogRow[] {
  const block = catalogBlock(modRs);
  const rows: CatalogRow[] = [...block.matchAll(ROW)].map((m) => ({
    name: m[1],
    origin: m[2] as "Shadcn" | "Extra",
    reason: m[3],
  }));
  const occurrences = (block.match(/Origin::/g) ?? []).length;
  if (rows.length !== occurrences) {
    throw new Error(
      `catalog.ts parsed ${rows.length} CATALOG rows but the block has ${occurrences} \`Origin::\` occurrences; ` +
        `the row shape in preview/src/components/mod.rs changed -- update ROW in playwright/catalog.ts`,
    );
  }
  for (const r of rows) {
    if ((r.origin === "Extra") !== (r.reason !== undefined)) {
      throw new Error(`CATALOG row \`${r.name}\`: ${r.origin} with${r.reason === undefined ? "out" : ""} a reason`);
    }
  }
  return rows;
}

/** Just the page names, in CATALOG order. */
export const catalogNames = (modRs?: string): string[] => readCatalog(modRs).map((r) => r.name);

const COMPONENTS_DIR = path.join(__dirname, "..", "preview", "src", "components");

/**
 * The ONE reader of the block demos (backlog row 241), from the directories `preview/build.rs`
 * (`discover_components`) itself derives the registry from, so there is no list to drift:
 * a component whose `variants/demo.css` exists is a Block, rendered in an iframe page of its own at
 * `/component/block/<name>/<variant>/`; its variants are `main` first, then the names `variants/order.txt`
 * lists, then every other `variants/<name>/` directory sorted.
 *
 * Replaces a text-scrape of a cfg'd-out `examples!(..)` mirror in `components/mod.rs` that `build.rs`
 * had to police for drift.
 */
export interface BlockDemo {
  name: string;
  variant: string;
  /** `/component/block/<name>/<variant>/` */
  path: string;
}

export function readBlockDemos(componentsDir: string = COMPONENTS_DIR): BlockDemo[] {
  const out: BlockDemo[] = [];
  for (const entry of fs.readdirSync(componentsDir, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : 1))) {
    if (!entry.isDirectory()) continue;
    const dir = path.join(componentsDir, entry.name);
    const variantsDir = path.join(dir, "variants");
    if (!fs.existsSync(path.join(dir, "component.json")) || !fs.existsSync(path.join(variantsDir, "demo.css"))) continue;
    const found = fs
      .readdirSync(variantsDir, { withFileTypes: true })
      .filter((d) => d.isDirectory() && d.name !== "main")
      .map((d) => d.name)
      .sort();
    const ordered: string[] = [];
    const orderFile = path.join(variantsDir, "order.txt");
    if (fs.existsSync(orderFile)) {
      for (const raw of fs.readFileSync(orderFile, "utf8").split("\n")) {
        const line = raw.split("#")[0].trim();
        if (!line) continue;
        const at = found.indexOf(line);
        if (at < 0) throw new Error(`${orderFile} lists \`${line}\`, which has no variants/${line}/ directory`);
        ordered.push(...found.splice(at, 1));
      }
    }
    for (const variant of ["main", ...ordered, ...found]) {
      out.push({ name: entry.name, variant, path: `/component/block/${entry.name}/${variant}/` });
    }
  }
  return out;
}
