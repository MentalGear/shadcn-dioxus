/**
 * Smooth scrolling is opt-in per interaction, never ambient (dev-docs/backlog.md row 228; gate
 * `scripts/check-scroll-behavior.sh`).
 *
 * The failure it guards: with an ambient `html { scroll-behavior: smooth }` the router's `scrollTo(0, 0)` after a push is a SMOOTH
 * scroll started on the long overview page; the short component page replaces it mid-animation, the browser
 * clamps the offset to the new page's bottom, and the visitor sees the bottom and then watches it glide up.
 * Back is the mirror (the overview renders at 0 and glides to the saved offset).
 *
 * This spec records `scrollY` and `location.pathname` on EVERY animation frame across the navigation and
 * asserts what the visitor can see:
 *   1. push from the last gallery card: every frame after the pathname changes is at scrollY 0 -- never the
 *      destination's bottom, never a glide (at 1280 and at an iPhone viewport);
 *   2. Back: no frame between the restore's start and its end -- the saved offset is reached in one step;
 *   3. a same-page anchor click on /docs still scrolls SMOOTHLY (>= 2 intermediate frames), and the opt-in
 *      attribute is gone once the jump has settled (so the next router scroll is instant again).
 *
 * The shared `fixtures.ts` overrides no `scroll-behavior` (row 228), so the page's own CSS decides; a forced
 * `html { scroll-behavior: auto !important }` would hide exactly the defect this spec exists for and would
 * turn the anchor jump instant, and `unmask()` removes any such inline property after load. Functional (not timing-sensitive), so it is fine on a debug build.
 *
 *   PLAYWRIGHT_BASE_URL=http://127.0.0.1:8147 npx playwright test --config=ssg.local.config.ts nav-scroll-instant.spec.ts
 */
import { test, expect, devices } from "./fixtures";
import type { Page } from "@playwright/test";
import { baseUrlOr } from "./base-url";
import { gotoHydrated } from "./hydration";
import { pageMetrics } from "./viewport-box";

const BASE = baseUrlOr("http://127.0.0.1:8080");

type Frame = { y: number; path: string };

/** Block everything that is not the local server (web fonts etc. would keep `networkidle` waiting). */
async function localOnly(page: Page) {
  await page.route(
    (url) => !["127.0.0.1", "localhost"].includes(url.hostname),
    (route) => route.abort(),
  );
}

async function open(page: Page, path: string) {
  await localOnly(page);
  await gotoHydrated(page, `${BASE}${path}`);
  await unmask(page);
}

/** Remove any inline `scroll-behavior` so the page's own CSS decides (see the header). */
async function unmask(page: Page) {
  await page.evaluate(() => document.documentElement.style.removeProperty("scroll-behavior"));
  expect(await page.evaluate(() => document.documentElement.style.getPropertyValue("scroll-behavior"))).toBe("");
}

async function instantScroll(page: Page, top: number) {
  await page.evaluate((t) => window.scrollTo({ top: t, behavior: "instant" }), top);
}

/** Record scrollY + pathname on every animation frame until `stop()`. */
async function startTrace(page: Page) {
  await page.evaluate(() => {
    const w = window as unknown as { __trace: Frame[]; __traceOn: boolean };
    w.__trace = [];
    w.__traceOn = true;
    const tick = () => {
      if (!w.__traceOn) return;
      w.__trace.push({ y: Math.round(window.scrollY), path: location.pathname });
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
}

async function stopTrace(page: Page): Promise<Frame[]> {
  return page.evaluate(() => {
    const w = window as unknown as { __trace: Frame[]; __traceOn: boolean };
    w.__traceOn = false;
    return w.__trace;
  });
}

/** The last card's title link on the overview, scrolled to the middle of the viewport (instantly). */
async function lastCardLink(page: Page) {
  const link = page.locator("a.dx-component-card-title-link").last();
  await expect(link).toBeAttached();
  // Cards below are `content-visibility: auto`: bring it in, let the layout settle, bring it in again.
  for (let i = 0; i < 3; i++) {
    const vh = (await pageMetrics(page)).innerHeight;
    const top = await link.evaluate((el, vh) => el.getBoundingClientRect().top + window.scrollY - vh / 2, vh);
    await instantScroll(page, top);
    await page.waitForTimeout(400);
  }
  const viewH = (await pageMetrics(page)).innerHeight;
  const inView = await link.evaluate((el, vh) => {
    const r = el.getBoundingClientRect();
    return r.top >= 0 && r.bottom <= vh;
  }, viewH);
  expect(inView, "the last card's link is inside the viewport before the click").toBe(true);
  return link;
}

const { defaultBrowserType: _ignored, ...iphone } = devices["iPhone 13"];

for (const [label, use] of [
  ["1280x800", { viewport: { width: 1280, height: 800 } }],
  ["iPhone", iphone],
] as const) {
  test.describe(`router scrolls are instant (${label})`, () => {
    test.use(use);

    test("push from the last gallery card: every frame after the pathname change is at the top", async ({ page }) => {
      await open(page, "/");
      const link = await lastCardLink(page);
      const startY = await page.evaluate(() => Math.round(window.scrollY));
      expect(startY, "the last card is far down the overview").toBeGreaterThan(5000);

      await startTrace(page);
      await link.click();
      await page.waitForFunction(() => location.pathname.startsWith("/component/"));
      await page.waitForTimeout(2500); // long enough for any glide (~0.7-1.5 s on a debug build)
      const frames = await stopTrace(page);

      const after = frames.filter((f) => f.path !== "/");
      expect(after.length, "frames recorded after the navigation").toBeGreaterThan(10);
      const bad = after.filter((f) => f.y !== 0);
      expect(
        bad.map((f) => f.y),
        `scrollY must be 0 on every frame after the route change (saw ${[...new Set(after.map((f) => f.y))].slice(0, 8)})`,
      ).toEqual([]);
    });

    test("Back restores the saved offset in one step", async ({ page }) => {
      await open(page, "/");
      const link = await lastCardLink(page);
      await link.click();
      await page.waitForFunction(() => location.pathname.startsWith("/component/"));
      await page.waitForTimeout(600);

      await startTrace(page);
      await page.goBack();
      await page.waitForFunction(() => location.pathname === "/");
      await page.waitForTimeout(2500);
      const frames = await stopTrace(page);

      const home = frames.filter((f) => f.path === "/");
      const finalY = home[home.length - 1].y;
      expect(finalY, "Back restores the scroll position").toBeGreaterThan(5000);
      // A glide shows a run of distinct offsets (dozens of frames over the ~500 ms a smooth scroll takes). The
      // overview's `content-visibility: auto` cards are laid out at their `contain-intrinsic-block-size` estimate until
      // they are rendered, so on a phone viewport the restored offset can be corrected ONCE (43161 -> 43006, 155 px):
      // that is a layout settle, not an animation, so up to two distinct
      // intermediate offsets are tolerated.
      const between = [...new Set(home.filter((f) => f.y > 2 && Math.abs(f.y - finalY) > 2).map((f) => f.y))];
      expect(
        between.length,
        `no run of frames between the top and the restored offset: the restore is not an animation (saw ${between.slice(0, 8)})`,
      ).toBeLessThanOrEqual(2);
    });
  });
}

test.describe("same-page anchors scroll smoothly (1280x800)", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test("a /docs sidebar anchor glides, then the opt-in clears", async ({ page }) => {
    await open(page, "/docs");
    expect(
      await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior),
      "at rest the document scrolls instantly",
    ).toBe("auto");

    // A sidebar "Start" sub-item whose heading is far enough down to need a visible scroll.
    const href = await page.evaluate(() => {
      const links = [...document.querySelectorAll<HTMLAnchorElement>('a[href^="#"]:not(.dx-skip-link)')];
      const far = links
        .map((a) => ({ a, t: document.getElementById(decodeURIComponent(a.getAttribute("href")!.slice(1))) }))
        .filter((x) => x.t && x.t.getBoundingClientRect().top + window.scrollY > 1500 && x.a.getClientRects().length > 0);
      return far.length ? far[far.length - 1].a.getAttribute("href") : null;
    });
    expect(href, "a visible same-page anchor with a far target exists on /docs").not.toBeNull();

    await startTrace(page);
    await page.locator(`a[href="${href}"]`).first().click();
    await page.waitForTimeout(100);
    // While the jump is in flight (or just settled) the opt-in is what makes it smooth.
    await page.waitForTimeout(1500);
    const frames = await stopTrace(page);

    const ys = frames.map((f) => f.y);
    const finalY = ys[ys.length - 1];
    expect(finalY, "the anchor jump scrolled the page").toBeGreaterThan(1000);
    const intermediate = [...new Set(ys.filter((y) => y > 2 && Math.abs(y - finalY) > 2))];
    expect(intermediate.length, `smooth: intermediate frames between 0 and ${finalY}`).toBeGreaterThanOrEqual(2);

    await expect(page.locator("html")).not.toHaveAttribute("data-anchor-scroll", /.*/, { timeout: 8000 });
    expect(await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior)).toBe("auto");
  });
});
