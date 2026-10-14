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
