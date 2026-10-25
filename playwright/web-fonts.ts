import { expect } from "@playwright/test";
import type { BrowserContext, Page } from "@playwright/test";

/**
 * ONE FONT STATE FOR A TWO-LOAD COMPARISON (backlog rows 439, 124).
 *
 * Geist is self-hosted with `font-display: optional` (`preview/src/site/head.rs`): a face is used only if it lands inside
 * the ~100 ms block window, otherwise NOT AT ALL for that document (measured: a woff2 delayed 600 ms ends `loaded` in
 * `document.fonts` and the text is still laid out in the fallback face; waiting for `document.fonts.ready` cannot select
 * the state). Two page loads therefore race the font independently, and under load one of them gets Geist's metrics and
 * the other the fallback's: a description wraps to two lines on one side (+20 px of card header on `/charts/`, +14 px of
 * footer on `/charts/pie/`), a tab is ~3 px wider (198 boxes on `/component/tabs/`).
 *
 * That is font timing, not a hydration difference, and it cannot be told apart from one by a tolerance. A spec that
 * compares one load with another (a JavaScript-off page against the hydrated one, two viewports, two schemes) must put
 * both loads in the SAME font state by construction. The state it picks is the fallback face: it needs no race to win
 * and is the same on every machine. (Within one document the state never changes, so a real hydration shift is still
 * seen.) `fonts-self-hosted.spec.ts` owns "the font really loads". `scripts/check-spec-font-state.sh` fails a spec that
 * opens a JavaScript-off context and reads geometry or styles without calling this.
 */
export const WEB_FONT_REQUESTS = /https:\/\/fonts\.(googleapis|gstatic)\.com\/|\.woff2(\?|$)/;

/** Abort every web-font request of this page or context (call it before the first navigation). */
export async function noWebFonts(target: Page | BrowserContext) {
  await target.route(WEB_FONT_REQUESTS, (r) => r.abort());
}

/**
 * The pin held: Geist is not available to this document. Fails with the font state named, rather than as a pile of
 * 14 px or 20 px box differences, when the route above stops matching (a renamed font file, a new CDN).
 */
export async function expectNoWebFont(page: Page, label: string) {
  const geist = await page.evaluate(() => document.fonts.check("16px Geist"));
  expect(geist, `${label}: Geist is available although the web fonts were aborted (noWebFonts no longer matches the font request)`).toBe(false);
}
