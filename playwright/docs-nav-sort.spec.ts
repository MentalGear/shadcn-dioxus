import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

// The docs sidebar lists the components A-Z by default and by category on request (docs-site lane, backlog
// row 234; `preview/src/docs_nav.rs`). The SSG emits the default, so the remembered choice is applied before
// first paint from `localStorage` onto `<html data-nav-sort>`, and CSS shows one of the two lists.
// Run against the SSG lane (the prerendered HTML is half of the claim).

const PAGE = `${BASE_URL}/component/button/`;
const KEY = "dx-nav-sort";

const az = (page: Page) => page.locator(".dx-nav-az");
const category = (page: Page) => page.locator(".dx-nav-category");
const option = (page: Page, which: "az" | "category") =>
  page.locator(`.dx-nav-sort-option[data-nav-sort-option="${which}"]`);

/** The display names of the visible list, in order. */
async function visibleItems(page: Page, list: "az" | "category"): Promise<string[]> {
  const scope = list === "az" ? ".dx-nav-az" : ".dx-nav-category";
  return page.locator(`${scope} .dx-sidebar-menu-item a`).evaluateAll((els) =>
    els.map((el) => (el.firstChild?.textContent ?? "").trim()),
  );
}

/** Remembers `choice` for the next navigation, the way a previous visit's click would have. */
async function remember(page: Page, choice: string) {
  await page.addInitScript(
    ([key, value]) => {
      try {
        localStorage.setItem(key, value);
      } catch {
        /* storage blocked: the page must still be correct */
      }
    },
    [KEY, choice],
  );
}

test("default: one flat A-Z list, no category headings, the toggle says A-Z", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  await expect(az(page)).toBeVisible();
  await expect(category(page).first()).toBeHidden();
  expect(await page.evaluate(() => document.documentElement.getAttribute("data-nav-sort"))).toBeNull();

  const names = await visibleItems(page, "az");
  expect(names.length).toBeGreaterThan(60);
  const sorted = [...names].sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));
  expect(names).toEqual(sorted);
  // One group, one heading: "Components" (the fixed "Start" group has its own).
  await expect(az(page).locator(".dx-sidebar-group-label")).toHaveText("Components");

  await expect(option(page, "az")).toHaveAttribute("aria-pressed", "true");
  await expect(option(page, "category")).toHaveAttribute("aria-pressed", "false");
});

test("the prerendered HTML is the default and carries both lists", async ({ request }) => {
  const html = await (await request.get(PAGE)).text();
  expect(html).toContain("dx-nav-az");
  expect(html).toContain("dx-nav-category");
  // No attribute on <html> in the static document: the choice is applied by script.
  expect(html).not.toMatch(/<html[^>]*data-nav-sort/);
});

test("toggle to Category: seven groups, A-Z list hidden, choice stored; back to A-Z", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  await option(page, "category").click();

  await expect(option(page, "category")).toHaveAttribute("aria-pressed", "true");
  await expect(option(page, "az")).toHaveAttribute("aria-pressed", "false");
  await expect(az(page)).toBeHidden();
  await expect(category(page)).toHaveCount(7);
  for (const group of await category(page).all()) await expect(group).toBeVisible();
  const labels = await category(page).locator(".dx-sidebar-group-label").allTextContents();
  expect(labels.map((l) => l.trim())).toEqual([
    "Forms",
    "Navigation",
    "Overlays",
    "Feedback",
    "Disclosure",
    "Data display",
    "Charts",
  ]);
  expect(await page.evaluate((k) => localStorage.getItem(k), KEY)).toBe("category");
  expect(await page.evaluate(() => document.documentElement.getAttribute("data-nav-sort"))).toBe("category");

  await option(page, "az").click();
  await expect(az(page)).toBeVisible();
  await expect(category(page).first()).toBeHidden();
  expect(await page.evaluate((k) => localStorage.getItem(k), KEY)).toBe("az");
});

test("only the visible list's links are reachable: the hidden list is out of the accessibility tree", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  const sidebar = page.locator('[data-slot="sidebar-inner"]');
  await expect(sidebar.getByRole("link", { name: "tooltip", exact: true })).toHaveCount(1);
  await option(page, "category").click();
  await expect(sidebar.getByRole("link", { name: "tooltip", exact: true })).toHaveCount(1);
});

test("the choice survives a reload, and the FIRST PAINT already shows it (no flash)", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  await option(page, "category").click();
  await expect(az(page)).toBeHidden();

  // Before any page script runs, start recording what each frame would paint: the computed display of both
  // lists and the background of the selected option, from the first frame in which the sidebar exists.
  await page.addInitScript(() => {
    const frames: Array<{ az: string; cat: string; pressedBg: string; azBg: string }> = [];
    (window as unknown as { __navFrames: typeof frames }).__navFrames = frames;
    const tick = () => {
      const a = document.querySelector(".dx-nav-az");
      const c = document.querySelector(".dx-nav-category");
      const oc = document.querySelector('.dx-nav-sort-option[data-nav-sort-option="category"]');
      const oa = document.querySelector('.dx-nav-sort-option[data-nav-sort-option="az"]');
      if (a && c && oc && oa) {
        frames.push({
          az: getComputedStyle(a).display,
          cat: getComputedStyle(c).display,
          pressedBg: getComputedStyle(oc).backgroundColor,
          azBg: getComputedStyle(oa).backgroundColor,
        });
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  await gotoHydrated(page, PAGE);

  await expect(az(page)).toBeHidden();
  await expect(option(page, "category")).toHaveAttribute("aria-pressed", "true");
  const frames = await page.evaluate(
    () => (window as unknown as { __navFrames: Array<{ az: string; cat: string; pressedBg: string; azBg: string }> }).__navFrames,
  );
  expect(frames.length, "the recorder saw frames with the sidebar present").toBeGreaterThan(0);
  const wrong = frames.filter(
    (f) => f.az !== "none" || f.cat === "none" || f.pressedBg === "rgba(0, 0, 0, 0)" || f.azBg !== "rgba(0, 0, 0, 0)",
  );
  expect(wrong, "no frame showed the A-Z list or the A-Z option as selected").toEqual([]);
});

test("a blocked localStorage leaves the default, and nothing throws", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.addInitScript(() => {
    Object.defineProperty(window, "localStorage", {
      get() {
        throw new DOMException("blocked", "SecurityError");
      },
    });
  });
  await gotoHydrated(page, PAGE);
  await expect(az(page)).toBeVisible();
  await option(page, "category").click(); // the click handler also guards its storage write
  await expect(category(page).first()).toBeVisible();
  expect(errors).toEqual([]);
});

for (const mode of ["az", "category"] as const) {
  test(`${mode}: the current page is marked once and scrolled into view inside the sidebar`, async ({ page }) => {
    await remember(page, mode);
    await page.setViewportSize({ width: 1280, height: 720 });
    await gotoHydrated(page, `${BASE_URL}/component/tooltip/`);

    const marked = page.locator(`.dx-nav-${mode} a[aria-current="page"]`);
    await expect(marked).toHaveCount(1);
    await expect(marked).toHaveText(/tooltip/);

    // Scrolled into view: fully inside the sidebar's own scroller (poll: it runs after hydration).
    await expect
      .poll(async () =>
        page.evaluate((m) => {
          const a = document.querySelector(`.dx-nav-${m} a[aria-current="page"]`)!.getBoundingClientRect();
          const box = document.querySelector('[data-slot="sidebar-content"]')!.getBoundingClientRect();
          return a.top >= box.top - 1 && a.bottom <= box.bottom + 1;
        }, mode),
      )
      .toBe(true);
    // ... without scrolling the page itself.
    expect(await page.evaluate(() => window.scrollY)).toBe(0);
  });
}

test("clicking a link in the A-Z list navigates and keeps the chosen order", async ({ page }) => {
  await remember(page, "category");
  await gotoHydrated(page, PAGE);
  await option(page, "az").click();
  await page.locator(".dx-nav-az").getByRole("link", { name: "badge", exact: true }).click();
  await expect(page).toHaveURL(/\/component\/badge\/\??$/);
  await expect(az(page)).toBeVisible();
  await expect(page.locator('.dx-nav-az a[aria-current="page"]')).toHaveText(/badge/);
});
