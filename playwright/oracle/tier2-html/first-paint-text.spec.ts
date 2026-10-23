/**
 * First-paint TEXT parity -- tier 2 (the server HTML is the first thing a visitor, a crawler and a screen reader get).
 *
 * `first-paint-styles.spec.ts` compares computed STYLES of the JS-off page with the hydrated one; it cannot see that a
 * preselected Select shows its placeholder, or a preselected Combobox an empty input, until hydration (backlog row 306:
 * the label came from the option registry, which fills in a client-only effect). This spec closes the class by property,
 * over every component route (a new component is covered the day it lands): for every element that carries the text of a
 * VALUE -- a text input / textarea `value`, a `role=combobox` element's text and accessible name, a `[data-placeholder]`
 * value span, a chip -- the JS-disabled page and the hydrated page must hold the same `textContent`, `value` and
 * `aria-label`.
 *
 * Only elements present in both documents under the same path are compared (content a client render adds is the style
 * oracle's business). Differences that are the demo's own client state, not delivery, are listed in `ALLOW` with a reason;
 * a stale entry fails.
 *
 * NARROWED (row 329): the every-route sweep moved into `cargo test`. `preview/src/ssr_parity.rs` renders every demo and
 * every component page natively, once as the server frame and once after the first client tick (effects run), and fails on
 * any difference in text, `value`, `aria-label`, `class`, `style` or `data-*`; it is a superset of what this spec read,
 * found a case this spec missed (tag_group, row 332) and reproduced row 306 at `fb8c0b65^`. What it cannot see is a real
 * browser hydrating the shipped wasm, so this spec keeps that half on the routes the class has hit (`ROUTES`):
 * Select and Combobox (row 306) and Tag Group (row 332). `FIRST_PAINT_ROUTES=all` runs every component route again,
 * `FIRST_PAINT_ROUTES=select,field` any list.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8090 npx playwright test --config=ssg.local.config.ts oracle/tier2-html/first-paint-text.spec.ts
 * Functional, not timing: valid on a debug build.
 */
import { test, expect } from "../../fixtures";
import type { Page } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { baseUrlOr } from "../../base-url";

const BASE = baseUrlOr("http://127.0.0.1:8090");
const PREVIEW = path.join(__dirname, "..", "..", "..", "preview");

const ALL_COMPONENTS: string[] = fs
  .readdirSync(path.join(PREVIEW, "src", "components"), { withFileTypes: true })
  .filter((d) => d.isDirectory() && fs.existsSync(path.join(PREVIEW, "src", "components", d.name, "docs.md")))
  .map((d) => d.name)
  .sort();
/** The routes the class has hit; every other route is the native parity test's (`preview/src/ssr_parity.rs`, row 329). */
const ROUTES = ["select", "combobox", "tag_group"];
const COMPONENTS: string[] = (() => {
  const only = (process.env.FIRST_PAINT_ROUTES ?? "").split(",").map((s) => s.trim()).filter(Boolean);
  if (only.includes("all")) return ALL_COMPONENTS;
  return ALL_COMPONENTS.filter((c) => (only.length ? only : ROUTES).includes(c));
})();

/** `<slug>`: reason the route's value text legitimately differs between the JS-off page and the hydrated one. */
const ALLOW: Record<string, string> = {};

type Texts = Record<string, { text?: string; value?: string; aria?: string }>;

/** The value-bearing elements of `main`, keyed by their position path (ancestors' tag.class + nth of that kind). */
async function readTexts(page: Page): Promise<Texts> {
  return page.evaluate(() => {
    const out: Texts = {};
    const root = document.querySelector("main");
    if (!root) return out;
    const label = (el: Element) => {
      const cls = typeof el.className === "string" ? el.className.trim().split(/\s+/).slice(0, 2).join(".") : "";
      return `${el.tagName.toLowerCase()}${cls ? "." + cls : ""}`;
    };
    const carries = (el: Element) =>
      el.matches('input:not([type=hidden]):not([type=checkbox]):not([type=radio]), textarea, [role=combobox], [data-placeholder], [data-slot*=chip]');
    const walk = (el: Element, at: string) => {
      if (carries(el)) {
        const rec: Texts[string] = {};
        if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) rec.value = el.value;
        else rec.text = (el.textContent ?? "").replace(/\s+/g, " ").trim();
        const aria = el.getAttribute("aria-label");
        if (aria !== null) rec.aria = aria;
        out[at] = rec;
      }
      const seen: Record<string, number> = {};
      for (const c of el.children) {
        if (c.tagName === "SCRIPT" || c.tagName === "STYLE") continue;
        const l = label(c);
        walk(c, `${at}/${l}#${(seen[l] = (seen[l] ?? 0) + 1)}`);
      }
    };
    walk(root, label(root));
    return out;
  });
}

function diffTexts(a: Texts, b: Texts): string[] {
  const out: string[] = [];
  for (const k of Object.keys(a)) {
    if (!b[k]) continue;
    for (const f of ["text", "value", "aria"] as const) {
      if (a[k][f] !== b[k][f]) out.push(`${k} ${f}: ${JSON.stringify(a[k][f])} (JS off) -> ${JSON.stringify(b[k][f])} (hydrated)`);
    }
  }
  return out;
}

test.describe("first-paint text: the static page carries every value's text", () => {
  test.describe.configure({ mode: "parallel" });
  for (const slug of COMPONENTS) {
    test(`/component/${slug}/: JavaScript off and hydrated show the same value text`, async ({ browser }) => {
      const read = async (js: boolean) => {
        const ctx = await browser.newContext({ viewport: { width: 1280, height: 900 }, javaScriptEnabled: js, colorScheme: "light" });
        const page = await ctx.newPage();
        // goto-ok: the JavaScript-off variant never hydrates; the hydrated one waits for it below
        await page.goto(`${BASE}/component/${slug}/`, { waitUntil: "load" });
        if (js) {
          await page.waitForSelector('html[data-hydrated="true"]', { state: "attached", timeout: 90_000 });
          await page.waitForTimeout(500);
        }
        const texts = await readTexts(page);
        await ctx.close();
        return texts;
      };
      const off = await read(false);
      const on = await read(true);
      const d = diffTexts(off, on);
      if (ALLOW[slug]) {
        expect(d.length, `${slug} is allowlisted (${ALLOW[slug]}) but has no difference: remove the entry`).toBeGreaterThan(0);
        return;
      }
      expect(d, `${d.length} difference(s): ${d.slice(0, 6).join(" | ")}`).toEqual([]);
    });
  }

  test("the route list is real: every kept route is a component page", () => {
    expect(ROUTES.filter((r) => !ALL_COMPONENTS.includes(r)), "a kept route without a component page would be skipped silently").toEqual([]);
  });

  test("the detector is not blind: a Select's value span and a Combobox input are inspected, with their text", async ({ browser }) => {
    const ctx = await browser.newContext({ javaScriptEnabled: false });
    const page = await ctx.newPage();
    // goto-ok: JavaScript is off by design
    await page.goto(`${BASE}/component/select/`, { waitUntil: "load" });
    const selectTexts = Object.values(await readTexts(page));
    expect(selectTexts.some((t) => (t.text ?? "") !== "" && t.aria !== undefined), "a select trigger with text and a name").toBe(true);
    // goto-ok: JavaScript is off by design
    await page.goto(`${BASE}/component/combobox/`, { waitUntil: "load" });
    const combo = Object.values(await readTexts(page));
    expect(combo.some((t) => (t.value ?? "") !== ""), "a preselected combobox input has its label in the server HTML").toBe(true);
    await ctx.close();
  });
});
