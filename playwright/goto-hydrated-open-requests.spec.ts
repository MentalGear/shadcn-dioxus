/**
 * A `gotoHydrated` that times out names the requests that were still open (backlog row 257).
 *
 * Row 257: one `page.goto(..., { waitUntil: "networkidle" })` on /component/sidebar/ hung to the 90 s test timeout in a
 * 575-test release chunk and never reproduced (17 of 17 green since). The error said only "Timeout 90000ms exceeded",
 * so nobody could tell which request kept the network busy: a font, a block-demo iframe, a wasm chunk, the static
 * server dropping a connection. The fix is not a retry (that would hide the next one) but a better error, by
 * construction for every spec: `gotoHydrated` records the page's requests (every frame) while it navigates and, when
 * the navigation or the hydration wait fails, appends the ones that never finished -- method, resource type, frame,
 * age and URL -- to the thrown error, so the one-off is diagnosable from the report the first time it recurs.
 *
 * Hermetic: the page and its never-answered image are fulfilled by `page.route`, so no server is needed.
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const ORIGIN = new URL(BASE_URL).origin;
const PAGE = `${ORIGIN}/__row-257__/stalled/`;
const STALLED = `${ORIGIN}/__row-257__/never-answers.png`;
const FINE = `${ORIGIN}/__row-257__/answers.png`;

test.beforeEach(async ({ page }) => {
  await page.route(PAGE, (r) =>
    r.fulfill({
      contentType: "text/html",
      body: `<!doctype html><html><body><img src="${FINE}"><img src="${STALLED}"></body></html>`,
    }),
  );
  await page.route(FINE, (r) => r.fulfill({ contentType: "image/png", body: Buffer.alloc(0) }));
  // Never answered: the request stays open, so `networkidle` never arrives.
  await page.route(STALLED, () => {});
});

test("a networkidle timeout names the request that never finished, and only that one", async ({ page }) => {
  const error = await gotoHydrated(page, PAGE, { timeout: 2_000 }).then(
    () => null,
    (e: unknown) => e as Error,
  );
  expect(error, "the navigation cannot reach networkidle").not.toBeNull();
  expect(error!.message, "Playwright's own timeout message is kept").toContain("Timeout 2000ms exceeded");
  expect(error!.message).toContain("requests still open");
  expect(error!.message, "the stalled image is named with its method and type").toMatch(
    new RegExp(`GET image .*${STALLED.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`),
  );
  expect(error!.message, "a finished request is not listed").not.toContain(FINE);
});

test("a hydration-signal timeout after a quiet network says no request was open", async ({ page }) => {
  // `load` returns, the page never sets data-hydrated: the error says the network was not the cause.
  await page.unroute(STALLED);
  await page.route(STALLED, (r) => r.fulfill({ contentType: "image/png", body: Buffer.alloc(0) }));
  const error = await gotoHydrated(page, PAGE, { timeout: 2_000, hydratedTimeout: 1_000 }).then(
    () => null,
    (e: unknown) => e as Error,
  );
  expect(error, "the page never hydrates").not.toBeNull();
  expect(error!.message).toContain('html[data-hydrated="true"]');
  expect(error!.message).toContain("no request was open");
});
