/**
 * Global stylesheet application — tier 2 (WHATWG HTML / CSSOM).
 *
 * Rule source: HTML Living Standard, "the link element", 'rel="stylesheet"'
 * and CSSOM's "document.styleSheets" — a style sheet participates in the
 * cascade only once it is *loaded*, and a sheet with a pending `@import`
 * (CSS Cascading and Inheritance / css-cascade "@import") is not loaded
 * until the imported sheet is. Browsers therefore keep such a sheet out of
 * `document.styleSheets` and out of the cascade even though `link.sheet`
 * already exposes its parsed rules.
 *
 * The hazard: a `preview/assets/main.css` that begins with an `@import` of
 * the Geist fonts from fonts.googleapis.com. In an environment where that
 * request hangs, the ENTIRE app stylesheet is inert on every route (body
 * computes to the UA default serif, every navbar/layout rule dead) while
 * every other stylesheet applies -- and nothing else in the harness notices,
 * because component behaviour lives in per-component sheets. This rule makes
 * that class of silent failure visible: the app's
 * global stylesheet must be in the applied set on every route, judged by a
 * property only it sets.
 */
import { test, expect } from "../../fixtures";
import { BASE_URL } from "../../base-url";
import { gotoPrerendered } from "../../page-kinds";

const ROUTES = [
  `${BASE_URL}/`,
  `${BASE_URL}/component/select/`,
  `${BASE_URL}/dashboard/email-client`,
];

for (const url of ROUTES) {
  test(`global stylesheet (main.css) is applied on ${new URL(url).pathname}${new URL(url).search}`, async ({ page }) => {
    await gotoPrerendered(page, url, { waitUntil: "domcontentloaded", timeout: 60_000 });
    await page.waitForTimeout(3000);
    const state = await page.evaluate(() => {
      const link = document.querySelector('link[rel="stylesheet"][href*="main-"]') as HTMLLinkElement | null;
      return {
        linked: !!link,
        applied: !!link && [...document.styleSheets].some((s) => s.ownerNode === link),
        bodyFont: getComputedStyle(document.body).fontFamily,
      };
    });
    expect(state.linked, "main.css must be linked").toBe(true);
    expect(state.applied, "main.css must be in document.styleSheets (loaded and applied)").toBe(true);
    // `body { font-family: Geist, ... }` is set only by main.css.
    expect(state.bodyFont.startsWith("Geist"), `body font-family should come from main.css, got: ${state.bodyFont}`).toBe(true);
  });
}
