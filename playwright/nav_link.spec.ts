import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

// Backlog row 126. `dioxus_router::Link` computes `aria-current` as
// `href == router.full_route_string()`; our routes carry `?:dark_mode`, so the
// prerendered HTML had `aria-current="page"` and the hydrated DOM lost it
// (and a `Link` given `aria_current` emitted it twice). Every navigation link
// is a `NavLink` (`preview/src/nav_link.rs`) now: exactly one `aria-current`,
// from a path comparison identical on the server and in the browser.
// Run against the SSG lane (the prerendered HTML is half of the claim).

const BRAND = "shadcn-dioxus"; // SITE_NAME: the brand link is the `/` link of the navbar and the footer
const PAGES: { path: string; nav: string[]; footer: string[]; sidebar: string[] | null; tab?: string }[] = [
  { path: "/", nav: [BRAND], footer: [BRAND, "components"], sidebar: ["home"] },
  { path: "/docs", nav: ["docs"], footer: ["docs"], sidebar: ["overview"] },
  { path: "/demos", nav: ["demos"], footer: ["demos"], sidebar: ["demos"] },
  { path: "/charts/", nav: ["charts"], footer: ["charts"], sidebar: ["charts"], tab: "area charts" },
  { path: "/charts/bar/", nav: [], footer: [], sidebar: ["charts"], tab: "bar charts" },
  { path: "/component/button/", nav: [], footer: [], sidebar: ["button"] },
];

/**
 * The text of every `a[aria-current]` inside `scope`. The docs sidebar holds its component list twice (A-Z and by
 * category, `preview/src/docs_nav.rs`) and hides one with CSS, so the sidebar's marks are the VISIBLE ones: exactly
 * one list shows, and it carries exactly one mark.
 */
async function current(page: Page, scope: string): Promise<string[]> {
  return page.locator(`${scope} a[aria-current]:visible`).evaluateAll((els) =>
    els.map((el) => (el.textContent ?? "").trim().toLowerCase()),
  );
}

async function expectMarks(page: Page, p: (typeof PAGES)[number]) {
  expect(await current(page, "nav.dx-preview-navbar"), `${p.path} navbar`).toEqual(p.nav);
  expect(await current(page, "footer.dx-preview-footer"), `${p.path} footer`).toEqual(p.footer);
  if (p.tab) {
    expect(await current(page, "nav.dx-charts-tabs"), `${p.path} tabs`).toEqual([p.tab]);
  }
  if (p.sidebar) {
    expect(await current(page, '[data-slot="sidebar-inner"]'), `${p.path} sidebar`).toEqual(p.sidebar);
  }
}

test("prerendered HTML has at most one aria-current per anchor, and the marks are the right ones", async ({
  browser,
  request,
}) => {
  // JS off: the DOM is the server's HTML, parsed but never hydrated.
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  for (const p of PAGES) {
    const res = await request.get(`${BASE_URL}${p.path}`);
    expect(res.ok(), p.path).toBe(true);
    for (const tag of (await res.text()).match(/<a\b[^>]*>/g) ?? []) {
      expect((tag.match(/\baria-current=/g) ?? []).length, `${p.path}: ${tag.slice(0, 160)}`).toBeLessThanOrEqual(1);
    }
    // goto-ok: JavaScript is disabled in this context: nothing hydrates, the spec reads the prerendered HTML
    await page.goto(`${BASE_URL}${p.path}`);
    await expectMarks(page, p);
  }
  await context.close();
});

test("the current-page marker survives hydration and client-side navigation", async ({ page }) => {
  for (const p of PAGES) {
    await gotoHydrated(page, `${BASE_URL}${p.path}`);
    await expectMarks(page, p);
  }

  // Client-side navigation moves the marker (no reload).
  await gotoHydrated(page, `${BASE_URL}/docs`);
  await page.evaluate(() => ((window as unknown as { __noReload: boolean }).__noReload = true));
  await page.locator("nav.dx-preview-navbar").getByRole("link", { name: "Charts" }).click();
  await expect(page).toHaveURL(/\/charts\/\??$/);
  await expect
    .poll(() => current(page, "nav.dx-preview-navbar"))
    .toEqual(["charts"]);
  expect(await current(page, "footer.dx-preview-footer")).toEqual(["charts"]);
  expect(await page.evaluate(() => (window as unknown as { __noReload?: boolean }).__noReload)).toBe(true);

  // ... and a sidebar link on a docs page moves the sidebar's mark.
  await gotoHydrated(page, `${BASE_URL}/docs`);
  await page.locator('[data-slot="sidebar-inner"]').getByRole("link", { name: "button", exact: true }).click();
  await expect(page).toHaveURL(/\/component\/button\/\??$/);
  await expect.poll(() => current(page, '[data-slot="sidebar-inner"]')).toEqual(["button"]);
  expect(await current(page, "nav.dx-preview-navbar")).toEqual([]);
});

test("modified and middle clicks leave the router alone (new tab), plain click navigates in place", async ({
  page,
  context,
}) => {
  await gotoHydrated(page, `${BASE_URL}/docs`);
  const demos = page.locator("nav.dx-preview-navbar").getByRole("link", { name: "Demos" });

  const popup = context.waitForEvent("page");
  await demos.click({ modifiers: ["ControlOrMeta"] });
  const tab = await popup;
  await tab.waitForLoadState("domcontentloaded");
  expect(tab.url()).toMatch(/\/demos\/?\??$/);
  await tab.close();
  await expect(page).toHaveURL(/\/docs\/?\??$/);

  const middle = context.waitForEvent("page");
  await demos.click({ button: "middle" });
  const tab2 = await middle;
  await tab2.waitForLoadState("domcontentloaded");
  expect(tab2.url()).toMatch(/\/demos\/?\??$/);
  await tab2.close();
  await expect(page).toHaveURL(/\/docs\/?\??$/);

  await demos.click();
  await expect(page).toHaveURL(/\/demos\/?\??$/);
});

// Row 126 library residue: `NavbarItem` (a shipped component) renders the same
// `NavLink` construction, so no anchor of its demo pages carries two `aria-current`.
test("the library NavbarItem emits at most one aria-current per anchor in the prerendered HTML", async ({
  request,
}) => {
  for (const path of ["/", "/component/navbar/"]) {
    const res = await request.get(`${BASE_URL}${path}`);
    expect(res.ok(), path).toBe(true);
    const anchors = (await res.text()).match(/<a\b[^>]*>/g) ?? [];
    expect(anchors.length, path).toBeGreaterThan(0);
    for (const tag of anchors) {
      expect((tag.match(/\baria-current=/g) ?? []).length, `${path}: ${tag.slice(0, 160)}`).toBeLessThanOrEqual(1);
    }
  }
});
